use crate::db;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
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
const SESSION_SECONDS: u64 = 180;
const PROTOCOL: &str = "SOSFINANCA-HTTP/2";
const PING_PATH: &str = "/ping";
const SNAPSHOT_PATH: &str = "/sos-financa.db";

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
    if let Ok(addr) = UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| {
            socket.connect("1.1.1.1:80")?;
            socket.local_addr()
        })
    {
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
    Header::from_bytes(name.as_bytes(), value.as_bytes())
        .expect("cabecalho HTTP interno invalido")
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
        "canSend": cfg!(target_os = "windows"),
        "canReceive": cfg!(target_os = "android"),
        "port": FIXED_PORT,
        "protocol": "http-local-v2"
    })
}

pub fn start_server(app: &AppHandle) -> Result<Value, String> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        return Err("O envio pela rede local está disponível no aplicativo Windows.".into());
    }

    #[cfg(target_os = "windows")]
    {
        stop_server();

        let snapshot = db::create_sync_snapshot(app)?;
        let plain = fs::read(&snapshot)
            .map_err(|e| format!("Não foi possível preparar o banco para sincronização: {e}"))?;
        let _ = fs::remove_file(&snapshot);

        if plain.is_empty() {
            return Err("O snapshot do banco ficou vazio. Nada foi enviado.".into());
        }
        if plain.len() > MAX_SYNC_BYTES {
            return Err("O banco excede o limite de 128 MB para sincronização.".into());
        }

        let hash = hex(&Sha256::digest(&plain));
        let payload = Arc::new(plain);
        let address = format!("0.0.0.0:{FIXED_PORT}");
        let server = Arc::new(
            Server::http(&address)
                .map_err(|e| format!("Não foi possível abrir a porta {FIXED_PORT}: {e}"))?
        );

        let ip = local_ip();
        if ip == "127.0.0.1" {
            return Err("Não consegui identificar o IP local do PC. Confirme que o PC está no Wi-Fi.".into());
        }

        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let payload_thread = payload.clone();
        let hash_thread = hash.clone();
        let server_thread = server.clone();
        let session_id = Uuid::new_v4().simple().to_string();
        let session_id_thread = session_id.clone();

        if let Ok(mut slot) = session_slot().lock() {
            *slot = Some(Session { id: session_id, stop: stop.clone() });
        }

        thread::spawn(move || {
            let started = Instant::now();

            while !stop_thread.load(Ordering::Relaxed)
                && started.elapsed() < Duration::from_secs(SESSION_SECONDS)
            {
                let request = match server_thread.recv_timeout(Duration::from_millis(250)) {
                    Ok(Some(request)) => request,
                    Ok(None) => continue,
                    Err(_) => break,
                };

                if request.method() != &Method::Get {
                    let _ = request.respond(text_response(405, "Metodo nao permitido"));
                    continue;
                }

                let url = request.url().to_string();
                match url.as_str() {
                    PING_PATH => {
                        let _ = request.respond(text_response(200, PROTOCOL));
                    }
                    SNAPSHOT_PATH => {
                        let response = Response::from_data((*payload_thread).clone())
                            .with_status_code(StatusCode(200))
                            .with_header(http_header("Content-Type", "application/octet-stream"))
                            .with_header(http_header("Cache-Control", "no-store"))
                            .with_header(http_header("X-SOS-Financa-Protocol", PROTOCOL))
                            .with_header(http_header("X-SOS-Financa-SHA256", &hash_thread));
                        let _ = request.respond(response);
                        break;
                    }
                    _ => {
                        let _ = request.respond(text_response(404, "Nao encontrado"));
                    }
                }
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
            "bytesReady": payload.len(),
            "protocol": "HTTP local",
            "warning": "O compartilhamento fica aberto por 3 minutos e encerra apos um download. Use somente em uma rede Wi-Fi privada."
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

pub async fn receive_from_pc(app: &AppHandle, host: &str) -> Result<Value, String> {
    if !cfg!(target_os = "android") {
        return Err("O recebimento foi preparado para o aplicativo Android.".into());
    }

    let host = host.trim();
    if host.is_empty() {
        return Err("Informe o IP mostrado no PC.".into());
    }

    let ip: IpAddr = host
        .parse()
        .map_err(|_| "Digite um IP válido, como 192.168.0.15.".to_string())?;

    let base = format!("http://{ip}:{FIXED_PORT}");
    let client = tauri_plugin_http::reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| format!("Não foi possível preparar a conexão HTTP: {e}"))?;

    let ping = client
        .get(format!("{base}{PING_PATH}"))
        .send()
        .await
        .map_err(|e| format!(
            "Não consegui localizar o SOS Finança no PC. Confirme o mesmo Wi-Fi, deixe 'Enviar para o celular' aberto no PC e tente novamente. Detalhe: {e}"
        ))?;

    if !ping.status().is_success() {
        return Err(format!("O PC respondeu ao teste de conexão com status HTTP {}.", ping.status()));
    }

    let protocol = ping
        .headers()
        .get("x-sos-financa-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if protocol != PROTOCOL {
        return Err("Encontrei um serviço no IP informado, mas ele não é a sincronização do SOS Finança.".into());
    }

    let response = client
        .get(format!("{base}{SNAPSHOT_PATH}"))
        .send()
        .await
        .map_err(|e| format!("O PC respondeu ao teste, mas não consegui baixar o banco: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("O PC não liberou o banco. Status HTTP {}.", response.status()));
    }

    let response_protocol = response
        .headers()
        .get("x-sos-financa-protocol")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if response_protocol != PROTOCOL {
        return Err("A resposta recebida não pertence ao SOS Finança.".into());
    }

    let expected_hash = response
        .headers()
        .get("x-sos-financa-sha256")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "O PC não enviou a assinatura de integridade do banco.".to_string())?
        .to_lowercase();

    if let Some(size) = response.content_length() {
        if size == 0 || size > MAX_SYNC_BYTES as u64 {
            return Err(format!("O tamanho informado pelo PC ({size} bytes) foi recusado."));
        }
    }

    let body = response
        .bytes()
        .await
        .map_err(|e| format!("A conexão HTTP foi aberta, mas o download do banco falhou: {e}"))?;

    if body.is_empty() {
        return Err("O PC enviou um arquivo vazio. Nada foi alterado no celular.".into());
    }
    if body.len() > MAX_SYNC_BYTES {
        return Err("O banco recebido excede o limite de 128 MB. Nada foi alterado.".into());
    }

    let actual_hash = hex(&Sha256::digest(body.as_ref()));
    if actual_hash != expected_hash {
        return Err("O download terminou, mas a verificação de integridade falhou. Nada foi alterado no celular.".into());
    }

    let app_handle = app.clone();
    let plain = body.to_vec();
    let byte_count = plain.len();

    let backup = tauri::async_runtime::spawn_blocking(move || {
        db::import_sync_database(&app_handle, &plain)
    })
    .await
    .map_err(|e| format!("Falha interna ao aplicar o banco recebido: {e}"))??;

    Ok(json!({
        "ok": true,
        "bytes": byte_count,
        "backup": backup,
        "message": "Dados do PC recebidos por HTTP local e aplicados com sucesso."
    }))
}
