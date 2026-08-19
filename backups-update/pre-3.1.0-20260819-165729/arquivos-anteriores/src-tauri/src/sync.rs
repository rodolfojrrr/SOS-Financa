use crate::db;
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Key, Nonce};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const PROTOCOL: &str = "SOSFINANCA/1";
const MAX_SYNC_BYTES: usize = 128 * 1024 * 1024;
const SESSION_SECONDS: u64 = 600;

#[derive(Clone)]
struct Session {
    id: String,
    stop: Arc<AtomicBool>,
}

static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();

fn session_slot() -> &'static Mutex<Option<Session>> {
    SESSION.get_or_init(|| Mutex::new(None))
}

fn clean_token(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_hexdigit()).collect::<String>().to_uppercase()
}

fn make_token() -> String {
    let raw = Uuid::new_v4().simple().to_string().to_uppercase();
    raw[..16].to_string()
}

fn format_token(token: &str) -> String {
    token.as_bytes().chunks(4).map(|c| String::from_utf8_lossy(c).to_string()).collect::<Vec<_>>().join("-")
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn from_hex(value: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 { return Err("Código hexadecimal inválido".into()); }
    let mut out = Vec::with_capacity(value.len()/2);
    let bytes = value.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let part = std::str::from_utf8(&bytes[i..i+2]).map_err(|_| "Código hexadecimal inválido")?;
        out.push(u8::from_str_radix(part, 16).map_err(|_| "Código hexadecimal inválido")?);
    }
    Ok(out)
}

fn auth_digest(token: &str, challenge: &[u8]) -> Result<Vec<u8>, String> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(token.as_bytes()).map_err(|e| e.to_string())?;
    mac.update(challenge);
    Ok(mac.finalize().into_bytes().to_vec())
}

fn auth_matches(token: &str, challenge: &[u8], received: &[u8]) -> Result<bool, String> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(token.as_bytes()).map_err(|e| e.to_string())?;
    mac.update(challenge);
    Ok(mac.verify_slice(received).is_ok())
}

fn crypto_key(token: &str) -> [u8; 32] {
    let digest = Sha256::digest(token.as_bytes());
    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    key
}

fn local_ip() -> String {
    if let Ok(addr) = UdpSocket::bind("0.0.0.0:0")
        .and_then(|socket| { socket.connect("8.8.8.8:80")?; socket.local_addr() }) {
        if !addr.ip().is_loopback() {
            return addr.ip().to_string();
        }
    }
    #[cfg(target_os = "windows")]
    {
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

fn handle_client(mut stream: TcpStream, token: &str, snapshot: &PathBuf) -> Result<bool, String> {
    stream.set_read_timeout(Some(Duration::from_secs(15))).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(30))).map_err(|e| e.to_string())?;

    let challenge_uuid = Uuid::new_v4();
    let challenge = challenge_uuid.as_bytes();
    writeln!(stream, "{} {}", PROTOCOL, hex(challenge)).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut auth_line = String::new();
    reader.read_line(&mut auth_line).map_err(|e| e.to_string())?;
    let auth_line = auth_line.trim();
    let Some(auth_hex) = auth_line.strip_prefix("AUTH ") else {
        let _ = writeln!(stream, "ERR autenticação");
        return Ok(false);
    };
    let received = from_hex(auth_hex)?;
    if !auth_matches(token, challenge, &received)? {
        let _ = writeln!(stream, "ERR código");
        return Ok(false);
    }

    let plain = fs::read(snapshot).map_err(|e| e.to_string())?;
    if plain.len() > MAX_SYNC_BYTES { return Err("O banco excede o limite de 128 MB para sincronização".into()); }
    let plain_hash = Sha256::digest(&plain);

    let nonce_uuid = Uuid::new_v4();
    let nonce_bytes = &nonce_uuid.as_bytes()[..12];
    let key_bytes = crypto_key(token);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let encrypted = cipher.encrypt(Nonce::from_slice(nonce_bytes), plain.as_ref()).map_err(|_| "Falha ao proteger os dados da sincronização".to_string())?;

    writeln!(stream, "DATA {} {} {}", encrypted.len(), hex(nonce_bytes), hex(&plain_hash)).map_err(|e| e.to_string())?;
    stream.write_all(&encrypted).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;
    Ok(true)
}

pub fn platform() -> Value {
    json!({
        "platform": if cfg!(target_os = "android") { "android" } else if cfg!(target_os = "windows") { "windows" } else { "desktop" },
        "canSend": !cfg!(target_os = "android"),
        "canReceive": cfg!(target_os = "android")
    })
}

pub fn start_server(app: &AppHandle) -> Result<Value, String> {
    if cfg!(target_os = "android") {
        return Err("No celular use Receber do PC.".into());
    }

    stop_server();
    let snapshot = db::create_sync_snapshot(app)?;
    let token = make_token();
    let listener = TcpListener::bind(("0.0.0.0", 0)).map_err(|e| format!("Não foi possível abrir a sincronização na rede: {e}"))?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let ip = local_ip();
    if ip == "127.0.0.1" {
        let _ = fs::remove_file(&snapshot);
        return Err("Não consegui identificar o IP local do PC. Confirme que o PC está conectado ao Wi-Fi e tente novamente.".into());
    }
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = stop.clone();
    let token_thread = token.clone();
    let snapshot_thread = snapshot.clone();
    let session_id = Uuid::new_v4().simple().to_string();
    let session_id_thread = session_id.clone();

    if let Ok(mut slot) = session_slot().lock() {
        *slot = Some(Session { id: session_id, stop: stop.clone() });
    }

    thread::spawn(move || {
        let started = Instant::now();
        while !stop_thread.load(Ordering::Relaxed) && started.elapsed() < Duration::from_secs(SESSION_SECONDS) {
            match listener.accept() {
                Ok((stream, _)) => {
                    match handle_client(stream, &token_thread, &snapshot_thread) {
                        Ok(true) => break,
                        Ok(false) => {}
                        Err(_) => {}
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(150)),
                Err(_) => break,
            }
        }
        let _ = fs::remove_file(snapshot_thread);
        if let Ok(mut slot) = session_slot().lock() {
            if slot.as_ref().map(|s| s.id.as_str()) == Some(session_id_thread.as_str()) {
                *slot = None;
            }
        }
    });

    Ok(json!({
        "ip": ip,
        "port": port,
        "code": format_token(&token),
        "expiresInSeconds": SESSION_SECONDS,
        "warning": "Use somente em uma rede Wi-Fi privada e conhecida."
    }))
}

pub fn stop_server() {
    if let Ok(mut slot) = session_slot().lock() {
        if let Some(session) = slot.take() {
            session.stop.store(true, Ordering::Relaxed);
        }
    }
}

pub fn receive_from_pc(app: &AppHandle, host: &str, port: u16, code: &str) -> Result<Value, String> {
    if !cfg!(target_os = "android") {
        return Err("O recebimento foi preparado para o aplicativo Android.".into());
    }
    let host = host.trim();
    if host.is_empty() { return Err("Informe o IP mostrado no PC.".into()); }
    if port == 0 { return Err("Informe a porta mostrada no PC.".into()); }
    let token = clean_token(code);
    if token.len() != 16 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Digite a chave de 16 caracteres mostrada no PC.".into());
    }

    let ip: IpAddr = host.parse().map_err(|_| "Digite um IP válido, como 192.168.0.15.".to_string())?;
    let socket_addr = SocketAddr::new(ip, port);
    let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(10)).map_err(|e| format!("Não consegui conectar ao PC. Confirme o mesmo Wi-Fi e o firewall. Detalhe: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(40))).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(15))).map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut hello = String::new();
    reader.read_line(&mut hello).map_err(|e| e.to_string())?;
    let parts = hello.trim().split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2 || parts[0] != PROTOCOL { return Err("Resposta de sincronização inválida.".into()); }
    let challenge = from_hex(parts[1])?;
    let digest = auth_digest(&token, &challenge)?;
    writeln!(stream, "AUTH {}", hex(&digest)).map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    let mut header = String::new();
    reader.read_line(&mut header).map_err(|e| e.to_string())?;
    let header = header.trim();
    if header.starts_with("ERR ") { return Err(format!("Sincronização recusada: {}", &header[4..])); }
    let parts = header.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 4 || parts[0] != "DATA" { return Err("Cabeçalho de dados inválido.".into()); }
    let size: usize = parts[1].parse().map_err(|_| "Tamanho de sincronização inválido.".to_string())?;
    if size == 0 || size > MAX_SYNC_BYTES { return Err("Tamanho de sincronização recusado.".into()); }
    let nonce_vec = from_hex(parts[2])?;
    if nonce_vec.len() != 12 { return Err("Proteção da sincronização inválida.".into()); }
    let expected_hash = parts[3].to_lowercase();

    let mut encrypted = vec![0u8; size];
    reader.read_exact(&mut encrypted).map_err(|e| format!("A transferência foi interrompida: {e}"))?;
    let key_bytes = crypto_key(&token);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let plain = cipher.decrypt(Nonce::from_slice(&nonce_vec), encrypted.as_ref()).map_err(|_| "Não foi possível descriptografar os dados. Confira a chave.".to_string())?;
    let actual_hash = hex(&Sha256::digest(&plain));
    if actual_hash != expected_hash { return Err("A verificação de integridade falhou. Nada foi alterado no celular.".into()); }

    let imported = db::import_sync_database(app, &plain)?;
    Ok(json!({
        "ok": true,
        "bytes": plain.len(),
        "backup": imported,
        "message": "Dados do PC recebidos e aplicados com sucesso."
    }))
}
