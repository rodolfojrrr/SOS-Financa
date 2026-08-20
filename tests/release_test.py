from pathlib import Path
import json
import re
import subprocess
import sys
import tempfile

root = Path(__file__).resolve().parents[1]
checks = 0

def ok(condition, message):
    global checks
    if not condition:
        raise AssertionError(message)
    checks += 1
    print(f'OK {checks:02d} - {message}')

main_conf = json.loads((root / 'src-tauri' / 'tauri.conf.json').read_text(encoding='utf-8'))
win_conf = json.loads((root / 'src-tauri' / 'tauri.windows.conf.json').read_text(encoding='utf-8'))
android_conf = json.loads((root / 'src-tauri' / 'tauri.android.conf.json').read_text(encoding='utf-8'))
cargo = (root / 'src-tauri' / 'Cargo.toml').read_text(encoding='utf-8')
main_rs = (root / 'src-tauri' / 'src' / 'main.rs').read_text(encoding='utf-8')
android_yml = (root / '.github' / 'workflows' / 'build-android.yml').read_text(encoding='utf-8')
windows_yml = (root / '.github' / 'workflows' / 'build-windows.yml').read_text(encoding='utf-8')
app_js = (root / 'app' / 'app.js').read_text(encoding='utf-8')
storage_js = (root / 'app' / 'storage.js').read_text(encoding='utf-8')
sync_rs = (root / 'src-tauri' / 'src' / 'sync.rs').read_text(encoding='utf-8')
lib_rs = (root / 'src-tauri' / 'src' / 'lib.rs').read_text(encoding='utf-8')
styles = (root / 'app' / 'styles.css').read_text(encoding='utf-8')
gitignore = (root / '.gitignore').read_text(encoding='utf-8')

ok(main_conf['version'] == '3.2.0', 'configuração Tauri está na versão 3.2.0')
ok(re.search(r'^version = "3\.2\.0"$', cargo, re.M) is not None, 'pacote Rust está na versão 3.2.0')
ok(main_conf['identifier'] == 'com.sosfinanca.app', 'identificador foi preservado para atualizar instalações existentes')
ok(win_conf['bundle']['targets'] == ['nsis'], 'Windows gera instalador NSIS')
ok(android_conf['bundle']['android']['minSdkVersion'] == 24, 'Android mantém compatibilidade mínima API 24')
ok('windows_subsystem = "windows"' in main_rs, 'build Release do Windows não abre CMD')
ok('cargo tauri android build --apk --ci' in android_yml, 'workflow Android gera APK Release')
ok('cargo tauri android build --debug --apk --ci' in android_yml, 'workflow Android gera APK de teste')
ok('SOS-Financa-Windows' in windows_yml and 'bundle/nsis/*.exe' in windows_yml, 'workflow Windows publica instalador')
ok('push:\n    branches: [main]' in android_yml, 'alterações do projeto disparam novo build Android')
ok('window.addEventListener(\'popstate\'' in app_js and 'pushHistory(true)' in app_js, 'navegação trata o botão Voltar do Android')
ok('env(safe-area-inset-top)' in styles and 'env(safe-area-inset-bottom)' in styles, 'layout respeita áreas seguras')
ok('*.jks' in gitignore and 'keystore.properties' in gitignore, 'chaves Android ficam fora do Git')

ok('tauri-plugin-http = "2"' in cargo, 'cliente HTTP oficial do ecossistema Tauri foi adicionado')
ok('tiny_http = "0.12"' in cargo, 'servidor HTTP dedicado foi adicionado ao Windows')
ok('tauri_plugin_http::init()' in lib_rs, 'plugin HTTP é inicializado no Tauri')
ok('ChaCha20Poly1305' not in sync_rs and 'HmacSha256' not in sync_rs and 'FIXED_TOKEN' not in sync_rs, 'protocolo antigo com chave/HMAC/ChaCha foi removido')
ok('TcpListener' not in sync_rs and 'TcpStream' not in sync_rs, 'sincronização não usa mais o protocolo TCP próprio')
ok('Server::http' in sync_rs and 'Response::from_data' in sync_rs, 'Windows serve o snapshot por HTTP padrão')
ok('tauri_plugin_http::reqwest::Client::builder()' in sync_rs, 'Android baixa o snapshot usando cliente HTTP')
ok('PING_PATH: &str = "/ping"' in sync_rs and 'SNAPSHOT_PATH: &str = "/sos-financa.db"' in sync_rs, 'HTTP possui rota de diagnóstico e rota de banco')
ok('X-SOS-Financa-SHA256' in sync_rs and 'Sha256::digest' in sync_rs, 'download é conferido por SHA-256')
ok('SESSION_SECONDS: u64 = 180' in sync_rs, 'servidor fica aberto por apenas três minutos')
ok('FIXED_PORT: u16 = 45454' in sync_rs, 'porta local continua fixa em 45454')
ok('create_sync_snapshot' in sync_rs and 'import_sync_database' in sync_rs, 'snapshot e importação protegida do SQLite foram preservados')
ok('PRAGMA integrity_check' in (root / 'src-tauri' / 'src' / 'db.rs').read_text(encoding='utf-8'), 'banco recebido passa por integrity_check')
ok('start_sync_server' in lib_rs and 'receive_sync_from_pc' in lib_rs, 'comandos de envio e recebimento continuam registrados')
ok("invoke('receive_sync_from_pc', { host })" in storage_js, 'frontend envia somente o IP ao comando Android')
ok("port: 45454" not in storage_js and "534F-5346-494E-414E" not in storage_js, 'frontend não transporta mais porta/chave de pareamento')
ok('Nova sincronização por HTTP local' in app_js, 'interface do PC explica o novo método')
ok('HTTP local na porta 45454' in app_js, 'interface Android informa a transferência automática')
ok('name="host" inputmode="decimal"' in app_js, 'Android pede somente o IP do PC')
ok('name="port"' not in app_js and 'name="code"' not in app_js, 'campos manuais de porta e chave foram removidos')
ok('spawn_blocking' in sync_rs, 'aplicação do SQLite no Android ocorre fora da rotina HTTP assíncrona')
ok('configure_android_network.py' in android_yml, 'workflow Android mantém permissões de rede')

with tempfile.TemporaryDirectory() as tmp:
    tmp_root = Path(tmp)
    gradle = tmp_root / 'src-tauri' / 'gen' / 'android' / 'app' / 'build.gradle.kts'
    gradle.parent.mkdir(parents=True)
    gradle.write_text(
        'plugins {\\n    id("com.android.application")\\n}\\n\\nandroid {\\n    buildTypes {\\n        getByName("release") {\\n            isMinifyEnabled = false\\n        }\\n    }\\n}\\n',
        encoding='utf-8'
    )
    script = root / 'scripts' / 'configure_android_signing.py'
    result = subprocess.run([sys.executable, str(script)], cwd=tmp_root, capture_output=True, text=True)
    patched = gradle.read_text(encoding='utf-8')
    ok(result.returncode == 0 and 'signingConfigs' in patched and 'signingConfig = signingConfigs.getByName("release")' in patched,
       'script de assinatura prepara projeto Android gerado')

with tempfile.TemporaryDirectory() as tmp:
    tmp_root = Path(tmp)
    manifest = tmp_root / 'src-tauri' / 'gen' / 'android' / 'app' / 'src' / 'main' / 'AndroidManifest.xml'
    manifest.parent.mkdir(parents=True)
    manifest.write_text('<manifest xmlns:android="http://schemas.android.com/apk/res/android">\\n    <application android:label="SOS Finança" />\\n</manifest>\\n', encoding='utf-8')
    script = root / 'scripts' / 'configure_android_network.py'
    result = subprocess.run([sys.executable, str(script)], cwd=tmp_root, capture_output=True, text=True)
    patched = manifest.read_text(encoding='utf-8')
    ok(result.returncode == 0 and 'android.permission.INTERNET' in patched and 'android.permission.ACCESS_NETWORK_STATE' in patched,
       'script de rede adiciona permissões Android necessárias')

print(f'\\nTodos os {checks} testes de release passaram.')
