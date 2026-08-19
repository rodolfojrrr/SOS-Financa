use crate::db;
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Key, Nonce};
use hmac::{Hmac, Mac};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpListener, TcpStream, UdpSocket};
#[cfg(target_os = "windows")]
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

const PROTOCOL: &str = "SOSFINANCA/1";
const FIXED_PORT: u16 = 45454;
const FIXED_TOKEN: &str = "534F5346494E414E";
const MAX_SYNC_BYTES: usize = 128 * 1024 * 1024;
const SESSION_SECONDS: u64 = 600;

#[derive(Clone)]
struct Session {
    id: String,
    stop: Arc<AtomicBool>,
}

#[derive(Clone)]
struct PreparedPayload {
    encrypted: Arc<Vec<u8>>,
    nonce_hex: String,
    hash_hex: String,
    plain_len: usize,
}

static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();

fn session_slot() -> &'static Mutex<Option<Session>> {
    SESSION.get_or_init(|| Mutex::new(None))
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
    let mut out = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let part = std::str::from_utf8(&bytes[i..i + 2]).map_err(|_| "Código hexadecimal inválido")?;
        out.push(u8::from_str_radix(part, 16).map_err(|_| "Código hexadecimal inválido")?);
    }
    Ok(out)
}


fn clean_token(value: &str) -> String {
    value.chars().filter(|c| c.is_ascii_hexdigit()).collect::<String>().to_uppercase()
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

fn clean_error_line(value: &str) -> String {
    value.replace('\r', " ").replace('\n', " ").chars().take(240).collect()
}

fn send_error(stream: &mut TcpStream, message: &str) {
    let _ = writeln!(stream, "ERR {}", clean_error_line(message));
    let _ = stream.flush();
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

fn prepare_payload(plain: Vec<u8>) -> Result<PreparedPayload, String> {
    if plain.is_empty() {
        return Err("O snapshot do banco ficou vazio. Nada foi enviado.".into());
    }
    if plain.len() > MAX_SYNC_BYTES {
        return Err("O banco excede o limite de 128 MB para sincronização.".into());
    }

    let plain_len = plain.len();
    let hash_hex = hex(&Sha256::digest(&plain));
    let nonce_uuid = Uuid::new_v4();
    let nonce_bytes = &nonce_uuid.as_bytes()[..12];
    let nonce_hex = hex(nonce_bytes);
    let key_bytes = crypto_key(FIXED_TOKEN);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let encrypted = cipher
        .encrypt(Nonce::from_slice(nonce_bytes), plain.as_ref())
        .map_err(|_| "Não foi possível preparar a criptografia do banco para o celular.".to_string())?;

    Ok(PreparedPayload {
        encrypted: Arc::new(encrypted),
        nonce_hex,
        hash_hex,
        plain_len,
    })
}

fn read_protocol_line(stream: &mut TcpStream, limit: usize) -> Result<String, String> {
    let mut buf = Vec::with_capacity(128);
    let mut one = [0u8; 1];

    while buf.len() < limit {
        match stream.read(&mut one) {
            Ok(0) => break,
            Ok(_) => {
                if one[0] == b'\n' {
                    break;
                }
                if one[0] != b'\r' {
                    buf.push(one[0]);
                }
            }
            Err(e) => return Err(format!("Falha ao ler resposta do celular: {e}")),
        }
    }

    if buf.len() >= limit {
        return Err("Resposta do celular excedeu o limite permitido.".into());
    }

    String::from_utf8(buf).map_err(|_| "Resposta inválida recebida do celular.".into())
}

fn handle_client(mut stream: TcpStream, payload: &PreparedPayload) -> Result<bool, String> {
    stream.set_read_timeout(Some(Duration::from_secs(120))).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(Duration::from_secs(180))).map_err(|e| e.to_string())?;
    let _ = stream.set_nodelay(true);

    let challenge_uuid = Uuid::new_v4();
    let challenge = challenge_uuid.as_bytes();

    let hello = format!("{} {}\n", PROTOCOL, hex(challenge));
    stream.write_all(hello.as_bytes()).map_err(|e| format!("Falha ao iniciar conversa com o celular: {e}"))?;
    stream.flush().map_err(|e| format!("Falha ao enviar identificação ao celular: {e}"))?;

    let auth_line = read_protocol_line(&mut stream, 512)?;
    if auth_line.is_empty() {
        return Ok(false);
    }

    let Some(auth_hex) = auth_line.trim().strip_prefix("AUTH ") else {
        send_error(&mut stream, "autenticação inválida");
        return Ok(false);
    };

    let received = match from_hex(auth_hex) {
        Ok(value) => value,
        Err(err) => {
            send_error(&mut stream, &err);
            return Ok(false);
        }
    };

    if !auth_matches(FIXED_TOKEN, challenge, &received)? {
        send_error(&mut stream, "código incorreto; use 534F-5346-494E-414E");
        return Ok(false);
    }

    let header = format!(
        "DATA {} {} {}\n",
        payload.encrypted.len(),
        payload.nonce_hex,
        payload.hash_hex
    );

    let total_len = header.len()
        .checked_add(payload.encrypted.len())
        .ok_or_else(|| "Tamanho da transferência inválido.".to_string())?;

    let mut packet = Vec::with_capacity(total_len);
    packet.extend_from_slice(header.as_bytes());
    packet.extend_from_slice(payload.encrypted.as_slice());

    stream
        .write_all(&packet)
        .map_err(|e| format!("Falha ao transmitir o banco para o celular: {e}"))?;
    stream
        .flush()
        .map_err(|e| format!("Falha ao concluir a transmissão para o celular: {e}"))?;

    let _ = stream.shutdown(Shutdown::Write);
    Ok(true)
}

pub fn platform() -> Value {
    json!({
        "platform": if cfg!(target_os = "android") { "android" } else if cfg!(target_os = "windows") { "windows" } else { "desktop" },
        "canSend": !cfg!(target_os = "android"),
        "canReceive": cfg!(target_os = "android"),
        "port": FIXED_PORT
    })
}

pub fn start_server(app: &AppHandle) -> Result<Value, String> {
    if cfg!(target_os = "android") {
        return Err("No celular use Receber do PC.".into());
    }

    stop_server();

    let snapshot = db::create_sync_snapshot(app)?;
    let plain = fs::read(&snapshot).map_err(|e| format!("Não foi possível preparar o banco para sincronização: {e}"))?;
    let _ = fs::remove_file(&snapshot);
    let payload = Arc::new(prepare_payload(plain)?);

    let listener = TcpListener::bind(("0.0.0.0", FIXED_PORT)).map_err(|e| {
        format!("Não foi possível abrir a porta fixa {FIXED_PORT}. Feche outra sessão do SOS Finança e confira o Firewall do Windows. Detalhe: {e}")
    })?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let ip = local_ip();
    if ip == "127.0.0.1" {
        return Err("Não consegui identificar o IP local do PC. Confirme que o PC está conectado ao Wi-Fi e tente novamente.".into());
    }

    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = stop.clone();
    let payload_thread = payload.clone();
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
                    match handle_client(stream, payload_thread.as_ref()) {
                        Ok(true) => break,
                        Ok(false) => {}
                        Err(_) => {}
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(100)),
                Err(_) => break,
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
        "code": format_token(FIXED_TOKEN),
        "expiresInSeconds": SESSION_SECONDS,
        "bytesReady": payload.plain_len,
        "warning": "Porta e código são fixos para compatibilidade com o APK atual. Use somente em uma rede Wi-Fi privada e conhecida."
    }))
}

pub fn stop_server() {
    if let Ok(mut slot) = session_slot().lock() {
        if let Some(session) = slot.take() {
            session.stop.store(true, Ordering::Relaxed);
        }
    }
}

pub fn receive_from_pc(app: &AppHandle, host: &str, _port: u16, _code: &str) -> Result<Value, String> {
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
    let socket_addr = SocketAddr::new(ip, FIXED_PORT);

    let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_secs(15))
        .map_err(|e| format!(
            "Não consegui conectar ao PC em {host}:{FIXED_PORT}. Confirme o mesmo Wi-Fi e deixe a tela de envio aberta no PC. Detalhe: {e}"
        ))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(180)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(60)))
        .map_err(|e| e.to_string())?;
    let _ = stream.set_nodelay(true);

    let hello = read_protocol_line(&mut stream, 512)
        .map_err(|e| format!("Não consegui receber a identificação do PC: {e}"))?;
    if hello.is_empty() {
        return Err("O PC encerrou a conexão antes de iniciar a sincronização.".into());
    }

    let parts = hello.trim().split_whitespace().collect::<Vec<_>>();
    if parts.len() != 2 || parts[0] != PROTOCOL {
        return Err(format!(
            "O PC respondeu com um protocolo inesperado: {}",
            clean_error_line(&hello)
        ));
    }

    let challenge = from_hex(parts[1])?;
    let digest = auth_digest(FIXED_TOKEN, &challenge)?;
    let auth = format!("AUTH {}\n", hex(&digest));
    stream
        .write_all(auth.as_bytes())
        .map_err(|e| format!("Não consegui confirmar o pareamento com o PC: {e}"))?;
    stream
        .flush()
        .map_err(|e| format!("Não consegui enviar a confirmação ao PC: {e}"))?;

    let header = read_protocol_line(&mut stream, 1024)
        .map_err(|e| format!("Falha ao aguardar o banco enviado pelo PC: {e}"))?;
    if header.is_empty() {
        return Err(
            "O PC encerrou a conexão antes de enviar o banco. Mantenha a janela de sincronização aberta no PC e tente novamente."
                .into(),
        );
    }

    let header = header.trim();
    if let Some(message) = header.strip_prefix("ERR ") {
        return Err(format!("O PC recusou a transferência: {message}"));
    }

    let parts = header.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 4 || parts[0] != "DATA" {
        return Err(format!(
            "O PC respondeu, mas o cabeçalho do banco não foi reconhecido: {}",
            clean_error_line(header)
        ));
    }

    let size: usize = parts[1]
        .parse()
        .map_err(|_| "O PC informou um tamanho de banco inválido.".to_string())?;
    if size == 0 || size > MAX_SYNC_BYTES {
        return Err(format!(
            "O tamanho recebido ({size} bytes) foi recusado por segurança."
        ));
    }

    let nonce_vec = from_hex(parts[2])?;
    if nonce_vec.len() != 12 {
        return Err("A proteção criptográfica recebida do PC é inválida.".into());
    }
    let expected_hash = parts[3].to_lowercase();

    let mut encrypted = vec![0u8; size];
    let mut received = 0usize;
    while received < size {
        match stream.read(&mut encrypted[received..]) {
            Ok(0) => {
                return Err(format!(
                    "A conexão terminou após receber {received} de {size} bytes. Nada foi alterado no celular."
                ));
            }
            Ok(n) => received += n,
            Err(e) => {
                return Err(format!(
                    "A transferência parou em {received} de {size} bytes. Nada foi alterado no celular. Detalhe: {e}"
                ));
            }
        }
    }

    let key_bytes = crypto_key(FIXED_TOKEN);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));
    let plain = cipher
        .decrypt(Nonce::from_slice(&nonce_vec), encrypted.as_ref())
        .map_err(|_| "Os dados chegaram, mas não foi possível validá-los/descriptografá-los. Nada foi alterado.".to_string())?;

    let actual_hash = hex(&Sha256::digest(&plain));
    if actual_hash != expected_hash {
        return Err("A verificação de integridade falhou. Nada foi alterado no celular.".into());
    }

    let imported = db::import_sync_database(app, &plain)?;
    Ok(json!({
        "ok": true,
        "bytes": plain.len(),
        "backup": imported,
        "message": "Dados do PC recebidos e aplicados com sucesso."
    }))
}

