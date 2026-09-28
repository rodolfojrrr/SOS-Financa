use crate::db;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::net::{IpAddr, UdpSocket};
use std::sync::{Arc, Mutex, OnceLock, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use uuid::Uuid;

#[cfg(target_os = "windows")]
use tiny_http::{Header, Method, Response, Server, StatusCode};

const FIXED_PORT: u16 = 45454;
const MAX_SYNC_BYTES: usize = 128 * 1024 * 1024;
const SESSION_SECONDS: u64 = 240;
const PROTOCOL: &str = "SOSFINANCA-HTTP/3";
const PING_PATH: &str = "/ping";
const SYNC_PATH: &str = "/sync";

#[derive(Clone)]
struct Session {
    id: String,
    stop: Arc<AtomicBool>,
}

static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();

fn session_slot() -> &'static Mutex<Option<Session>> {
    SESSION.get_or_init(|| Mutex::new(None))
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn local_ip() -> String {
    if let Ok(addr) = UdpSocket::bind("0.0.0.0:0").and_then(|socket| {
        socket.connect("1.1.1.1:80")?;
        socket.local_addr()
    }) {
        if !addr.ip().is_loopback() {
            return addr.ip().to_string();
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("ipconfig").output() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if line.contains("IPv4") {
                    if let Some((_, value)) = line.rsplit_once(':') {
                        let value = value.trim();
                        if let Ok(ip) = value.parse::<IpAddr>() {
                            if ip.is_ipv4() && !ip.is_loopback() {
                                return ip.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    "127.0.0.1".into()
}

#[cfg(target_os = "windows")]
fn http_header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("cabecalho HTTP interno invalido")
}

#[cfg(target_os = "windows")]
fn text_response(status: u16, body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body.to_string())
        .with_status_code(StatusCode(status))
        .with_header(http_header("Cache-Control", "no-store"))
        .with_header(http_header("X-SOS-Financa-Protocol", PROTOCOL))
}

pub fn platform() -> Value {
    json!({
        "platform": if cfg!(target_os = "android") { "android" } else if cfg!(target_os = "windows") { "windows" } else { "desktop" },
        "canHost": cfg!(target_os = "windows"),
        "canSync": cfg!(target_os = "android"),
        "canSend": cfg!(target_os = "windows"),
        "canReceive": cfg!(target_os = "android"),
        "port": FIXED_PORT,
        "protocol": "http-merge-v3",
        "bidirectional": true
    })
}

pub fn start_server(app: &AppHandle) -> Result<Value, String> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        return Err("O servidor de sincronização é iniciado pelo aplicativo Windows.".into());
    }

    #[cfg(target_os = "windows")]
    {
        stop_server();
        let address = format!("0.0.0.0:{FIXED_PORT}");
        let server = Arc::new(Server::http(&address).map_err(|e| format!("Não foi possível abrir a porta {FIXED_PORT}: {e}"))?);
        let ip = local_ip();
        if ip == "127.0.0.1" {
            return Err("Não consegui identificar o IP local do PC. Confirme que o PC está conectado à rede Wi-Fi.".into());
        }

        let stop = Arc::new(AtomicBool::new(false));
        let session_id = Uuid::new_v4().simple().to_string();
        if let Ok(mut slot) = session_slot().lock() {
            *slot = Some(Session { id: session_id.clone(), stop: stop.clone() });
        }

        let server_thread = server.clone();
        let stop_thread = stop.clone();
        let session_id_thread = session_id.clone();
        let app_thread = app.clone();

        thread::spawn(move || {
            let started = Instant::now();
            while !stop_thread.load(Ordering::Relaxed) && started.elapsed() < Duration::from_secs(SESSION_SECONDS) {
                let mut request = match server_thread.recv_timeout(Duration::from_millis(250)) {
                    Ok(Some(request)) => request,
                    Ok(None) => continue,
                    Err(_) => break,
                };
                let url = request.url().to_string();

                if request.method() == &Method::Get && url == PING_PATH {
                    let _ = request.respond(text_response(200, PROTOCOL));
                    continue;
                }

                if request.method() != &Method::Post || url != SYNC_PATH {
                    let _ = request.respond(text_response(404, "Rota de sincronização não encontrada"));
                    continue;
                }

                if request.body_length().unwrap_or(0) > MAX_SYNC_BYTES {
                    let _ = request.respond(text_response(413, "Banco enviado excede 128 MB"));
                    continue;
                }

                let expected_hash = request.headers().iter()
                    .find(|h| h.field.equiv("X-SOS-Financa-SHA256"))
                    .map(|h| h.value.as_str().to_lowercase())
                    .unwrap_or_default();

                let mut incoming = Vec::new();
                let mut limited = request.as_reader().take((MAX_SYNC_BYTES + 1) as u64);
                if let Err(err) = limited.read_to_end(&mut incoming) {
                    let _ = request.respond(text_response(400, &format!("Falha ao receber banco do celular: {err}")));
                    continue;
                }
                if incoming.is_empty() || incoming.len() > MAX_SYNC_BYTES {
                    let _ = request.respond(text_response(413, "Banco recebido vazio ou acima do limite"));
                    continue;
                }

                let actual_hash = hex(&Sha256::digest(&incoming));
                if expected_hash.is_empty() || expected_hash != actual_hash {
                    let _ = request.respond(text_response(400, "Integridade do banco enviado pelo celular não confere"));
                    continue;
                }

                if let Err(err) = db::merge_sync_database(&app_thread, &incoming) {
                    let _ = request.respond(text_response(500, &format!("Não foi possível combinar os bancos: {err}")));
                    continue;
                }

                let snapshot = match db::create_sync_snapshot(&app_thread) {
                    Ok(path) => path,
                    Err(err) => {
                        let _ = request.respond(text_response(500, &format!("Falha ao preparar banco combinado: {err}")));
                        continue;
                    }
                };
                let merged = match fs::read(&snapshot) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        let _ = fs::remove_file(&snapshot);
                        let _ = request.respond(text_response(500, &format!("Falha ao ler banco combinado: {err}")));
                        continue;
                    }
                };
                let _ = fs::remove_file(&snapshot);
                let merged_hash = hex(&Sha256::digest(&merged));
                let response = Response::from_data(merged)
                    .with_status_code(StatusCode(200))
                    .with_header(http_header("Content-Type", "application/octet-stream"))
                    .with_header(http_header("Cache-Control", "no-store"))
                    .with_header(http_header("X-SOS-Financa-Protocol", PROTOCOL))
                    .with_header(http_header("X-SOS-Financa-SHA256", &merged_hash))
                    .with_header(http_header("X-SOS-Financa-Merge", "two-way"));
                let _ = request.respond(response);
                break;
            }

            if let Ok(mut slot) = session_slot().lock() {
                if slot.as_ref().map(|s| s.id.as_str()) == Some(session_id_thread.as_str()) {
                    *slot = None;
                }
            }
        });

        Ok(json!({
            "ip": ip,
            "port": FIXED_PORT,
            "expiresInSeconds": SESSION_SECONDS,
            "protocol": "HTTP local em duas vias",
            "bidirectional": true,
            "warning": "Durante a sincronização, alterações do PC e do celular são combinadas. O registro com atualização mais recente vence apenas quando o mesmo item foi editado nos dois aparelhos."
        }))
    }
}

pub fn stop_server() {
    if let Ok(mut slot) = session_slot().lock() {
        if let Some(session) = slot.take() {
            session.stop.store(true, Ordering::Relaxed);
        }
    }
}

pub async fn sync_with_pc(app: &AppHandle, host: &str) -> Result<Value, String> {
    if !cfg!(target_os = "android") {
        return Err("A sincronização com o PC é iniciada pelo aplicativo Android.".into());
    }
    let host = host.trim();
    if host.is_empty() { return Err("Informe o IP mostrado no PC.".into()); }
    let ip: IpAddr = host.parse().map_err(|_| "Digite um IP válido, como 192.168.0.15.".to_string())?;
    let base = format!("http://{ip}:{FIXED_PORT}");
    let client = tauri_plugin_http::reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(240))
        .build()
        .map_err(|e| format!("Não foi possível preparar a conexão HTTP: {e}"))?;

    let ping = client.get(format!("{base}{PING_PATH}")).send().await.map_err(|e| format!("Não encontrei o SOS Finança no PC. Confira o mesmo Wi-Fi e deixe a tela de sincronização aberta no PC. Detalhe: {e}"))?;
    if !ping.status().is_success() { return Err(format!("O PC respondeu ao teste com status HTTP {}.", ping.status())); }
    let protocol = ping.headers().get("x-sos-financa-protocol").and_then(|v| v.to_str().ok()).unwrap_or("");
    if protocol != PROTOCOL { return Err("O serviço encontrado no IP informado não é a sincronização V4 do SOS Finança.".into()); }

    let app_snapshot = app.clone();
    let local_bytes = tauri::async_runtime::spawn_blocking(move || -> Result<Vec<u8>, String> {
        let path = db::create_sync_snapshot(&app_snapshot)?;
        let bytes = fs::read(&path).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(path);
        Ok(bytes)
    }).await.map_err(|e| format!("Falha ao preparar banco local: {e}"))??;
    if local_bytes.is_empty() || local_bytes.len() > MAX_SYNC_BYTES { return Err("O banco deste celular não pôde ser preparado para sincronização.".into()); }
    let local_hash = hex(&Sha256::digest(&local_bytes));

    let response = client.post(format!("{base}{SYNC_PATH}"))
        .header("X-SOS-Financa-SHA256", local_hash)
        .header("Content-Type", "application/octet-stream")
        .body(local_bytes)
        .send().await
        .map_err(|e| format!("A conexão com o PC abriu, mas a sincronização falhou: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(format!("O PC recusou a sincronização ({status}). {detail}"));
    }
    let response_protocol = response.headers().get("x-sos-financa-protocol").and_then(|v| v.to_str().ok()).unwrap_or("");
    if response_protocol != PROTOCOL { return Err("A resposta recebida não pertence à sincronização V4 do SOS Finança.".into()); }
    let expected_hash = response.headers().get("x-sos-financa-sha256").and_then(|v| v.to_str().ok()).ok_or_else(|| "O PC não enviou a assinatura de integridade do banco combinado.".to_string())?.to_lowercase();
    let merged = response.bytes().await.map_err(|e| format!("Não consegui baixar o banco combinado: {e}"))?;
    if merged.is_empty() || merged.len() > MAX_SYNC_BYTES { return Err("O banco combinado recebido é inválido.".into()); }
    let actual_hash = hex(&Sha256::digest(merged.as_ref()));
    if actual_hash != expected_hash { return Err("O banco combinado chegou incompleto. Nada foi aplicado no celular.".into()); }

    let app_import = app.clone();
    let data = merged.to_vec();
    let bytes = data.len();
    let backup = tauri::async_runtime::spawn_blocking(move || db::import_sync_database(&app_import, &data))
        .await.map_err(|e| format!("Falha interna ao aplicar sincronização: {e}"))??;

    Ok(json!({
        "ok": true,
        "bytes": bytes,
        "backup": backup,
        "bidirectional": true,
        "message": "Sincronização concluída: alterações do PC e do celular foram combinadas nos dois aparelhos."
    }))
}
