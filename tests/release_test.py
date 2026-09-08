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
capability = json.loads((root / 'src-tauri' / 'capabilities' / 'default.json').read_text(encoding='utf-8'))
cargo = (root / 'src-tauri' / 'Cargo.toml').read_text(encoding='utf-8')
main_rs = (root / 'src-tauri' / 'src' / 'main.rs').read_text(encoding='utf-8')
db_rs = (root / 'src-tauri' / 'src' / 'db.rs').read_text(encoding='utf-8')
android_yml = (root / '.github' / 'workflows' / 'build-android.yml').read_text(encoding='utf-8')
windows_yml = (root / '.github' / 'workflows' / 'build-windows.yml').read_text(encoding='utf-8')
app_js = (root / 'app' / 'app.js').read_text(encoding='utf-8')
storage_js = (root / 'app' / 'storage.js').read_text(encoding='utf-8')
sync_rs = (root / 'src-tauri' / 'src' / 'sync.rs').read_text(encoding='utf-8')
lib_rs = (root / 'src-tauri' / 'src' / 'lib.rs').read_text(encoding='utf-8')
styles = (root / 'app' / 'styles.css').read_text(encoding='utf-8')
gitignore = (root / '.gitignore').read_text(encoding='utf-8')

ok(main_conf['version'] == '4.1.0', 'configuração Tauri está na versão 4.1.0')
ok(re.search(r'^version = "4\.1\.0"$', cargo, re.M) is not None, 'pacote Rust está na versão 4.1.0')
ok(main_conf['identifier'] == 'com.sosfinanca.app', 'identificador foi preservado para atualizar instalações existentes')
ok(win_conf['bundle']['targets'] == ['nsis'], 'Windows gera instalador NSIS')
ok(android_conf['bundle']['android']['minSdkVersion'] == 24, 'Android mantém compatibilidade mínima API 24')
ok('windows_subsystem = "windows"' in main_rs, 'build Release do Windows não abre CMD')
ok('cargo tauri android build --apk --ci' in android_yml, 'workflow Android gera APK Release')
ok('cargo tauri android build --debug --apk --ci' in android_yml, 'workflow Android gera APK de teste')
ok('SOS-Financa-Windows' in windows_yml and 'bundle/nsis/*.exe' in windows_yml, 'workflow Windows publica instalador')
ok('*.jks' in gitignore and 'keystore.properties' in gitignore, 'chaves Android ficam fora do Git')
ok('env(safe-area-inset-top)' in styles and 'env(safe-area-inset-bottom)' in styles, 'layout respeita áreas seguras')

# Organização V4
ok('card-folder-grid' in app_js and 'folder-tile' in app_js, 'cartões são renderizados como pastas')
ok('card-status-bar' in app_js and 'Limite' in app_js and 'Restante da fatura' in app_js, 'pasta do cartão exibe barra financeira completa')
ok("data-action=\"toggle-card-edit\"" in app_js and "data-edit=\"card_purchase\"" in app_js, 'edição de compras é controlada por modo Editar')
ok('manage-folder-grid' in app_js and 'MANAGE_FOLDERS' in app_js, 'Gerenciar usa estrutura de pastas')
ok("mobileItem('manage','Gerenciar','folder')" in app_js, 'Gerenciar fica direto na navegação mobile')
ok('folder_prefs' in db_rs and 'sort_order' in db_rs and 'ensure_column(&conn, "cards", "color"' in db_rs, 'personalização de pastas/cartões é persistida no SQLite')
ok("navItem('accounts','Fixos','calendar')" in app_js, 'antiga aba Contas foi substituída por Fixos')
ok('fixed-folder-grid' in app_js and 'fixed_income' in app_js and 'fixed_expense' in app_js, 'Fixos possui duas pastas organizadas')
ok('fixed-item-card' in app_js and 'fixed-paid-badge' in app_js, 'valores fixos têm cartões contornados e indicador de pagamento')
ok('fixed-history-panel' in app_js and 'ui.manageEditMode' in app_js, 'histórico de pagamentos fica condicionado ao modo Editar')
ok('home-attention-empty' in app_js and '.home-attention-empty{margin-top:14px}' in styles, 'aviso sem pendências possui espaçamento superior')

# Lixeira
ok('get_trash' in lib_rs and 'restore_archived' in lib_rs and 'delete_forever' in lib_rs, 'backend expõe lixeira, restauração e exclusão permanente')
ok('pub fn trash' in db_rs and 'pub fn restore_archived' in db_rs and 'pub fn delete_forever' in db_rs, 'SQLite implementa ciclo completo da lixeira')
ok('data-trash-restore' in app_js and 'data-trash-delete' in app_js, 'interface permite restaurar ou excluir permanentemente')
ok('mover este item para a Lixeira' in app_js, 'exclusão normal pede confirmação e envia para lixeira')

# Alertas acionáveis e notificações
ok('data-alert-open-type' in app_js and 'attention-panel' in app_js, 'avisos da visão geral são acionáveis')
ok('tauri-plugin-notification = "2"' in cargo, 'plugin nativo de notificações foi adicionado')
ok('tauri_plugin_notification::init()' in lib_rs, 'plugin de notificações é inicializado')
ok('notification:default' in capability.get('permissions', []), 'capability permite notificações nativas')
ok('notifications_enabled' in db_rs and 'notification_days' in db_rs, 'preferências de notificação são persistidas')
ok('maybeSendDailyNotification' in app_js and 'sendNativeNotification' in app_js, 'frontend gera lembretes financeiros nativos')

# Sync HTTP bidirecional V4
ok('tauri-plugin-http = "2"' in cargo, 'cliente HTTP Tauri continua disponível')
ok('tiny_http = "0.12"' in cargo, 'servidor HTTP local continua no Windows')
ok('tauri_plugin_http::init()' in lib_rs, 'plugin HTTP é inicializado')
ok('ChaCha20Poly1305' not in sync_rs and 'HmacSha256' not in sync_rs and 'FIXED_TOKEN' not in sync_rs, 'sincronização permanece sem chave/protocolo criptográfico caseiro')
ok('TcpListener' not in sync_rs and 'TcpStream' not in sync_rs, 'sincronização não usa protocolo TCP próprio')
ok('Server::http' in sync_rs and 'Method::Post' in sync_rs and 'SYNC_PATH: &str = "/sync"' in sync_rs, 'Windows recebe o banco do celular por POST HTTP')
ok('tauri_plugin_http::reqwest::Client::builder()' in sync_rs and '.post(format!("{base}{SYNC_PATH}"))' in sync_rs, 'Android envia seu banco ao PC por HTTP')
ok('merge_sync_database' in sync_rs and 'merge_sync_database' in db_rs, 'PC mescla os dois bancos em vez de substituir')
ok('excluded.updated_at >' in db_rs and 'strategy":"latest-updated-at' in db_rs, 'conflitos do mesmo UUID usam a edição mais recente')
ok('X-SOS-Financa-SHA256' in sync_rs and 'Sha256::digest' in sync_rs, 'ida e volta do banco têm verificação SHA-256')
ok('PROTOCOL: &str = "SOSFINANCA-HTTP/3"' in sync_rs and 'bidirectional' in sync_rs, 'protocolo V4 declara sincronização bidirecional')
ok('SESSION_SECONDS: u64 = 240' in sync_rs, 'sessão local expira automaticamente')
ok('FIXED_PORT: u16 = 45454' in sync_rs, 'porta local permanece fixa')
ok('PRAGMA integrity_check' in db_rs, 'banco recebido passa por integrity_check')
ok('sync_with_pc' in lib_rs and 'start_sync_server' in lib_rs, 'comandos bidirecionais estão registrados')
ok("invoke('sync_with_pc', { host })" in storage_js, 'frontend Android chama a sincronização bidirecional')
ok('name="port"' not in app_js and 'name="code"' not in app_js, 'porta e chave não voltaram para a interface')
ok('Não substitui mais um banco pelo outro' in app_js, 'interface explica que os bancos são combinados')
ok('sync_tombstones' in db_rs and 'record_tombstone' in db_rs, 'exclusão permanente registra tombstone sincronizável')
ok('apply_sync_tombstones' in db_rs and 'remote.sync_tombstones' in db_rs, 'sincronização aplica exclusões permanentes dos dois aparelhos')

# Configuração Android mantida
ok('configure_android_network.py' in android_yml, 'workflow Android mantém configuração de rede')
with tempfile.TemporaryDirectory() as tmp:
    tmp_root = Path(tmp)
    gradle = tmp_root / 'src-tauri' / 'gen' / 'android' / 'app' / 'build.gradle.kts'
    gradle.parent.mkdir(parents=True)
    gradle.write_text('plugins {\n    id("com.android.application")\n}\n\nandroid {\n    buildTypes {\n        getByName("release") {\n            isMinifyEnabled = false\n        }\n    }\n}\n', encoding='utf-8')
    result = subprocess.run([sys.executable, str(root / 'scripts' / 'configure_android_signing.py')], cwd=tmp_root, capture_output=True, text=True)
    patched = gradle.read_text(encoding='utf-8')
    ok(result.returncode == 0 and 'signingConfigs' in patched and 'signingConfig = signingConfigs.getByName("release")' in patched, 'script de assinatura Android continua válido')

with tempfile.TemporaryDirectory() as tmp:
    tmp_root = Path(tmp)
    manifest = tmp_root / 'src-tauri' / 'gen' / 'android' / 'app' / 'src' / 'main' / 'AndroidManifest.xml'
    manifest.parent.mkdir(parents=True)
    manifest.write_text('<manifest xmlns:android="http://schemas.android.com/apk/res/android">\n <application android:label="SOS Finança" />\n</manifest>\n', encoding='utf-8')
    result = subprocess.run([sys.executable, str(root / 'scripts' / 'configure_android_network.py')], cwd=tmp_root, capture_output=True, text=True)
    patched = manifest.read_text(encoding='utf-8')
    ok(result.returncode == 0 and 'android.permission.INTERNET' in patched and 'android.permission.ACCESS_NETWORK_STATE' in patched, 'script de rede adiciona permissões Android necessárias')

print(f'\nTodos os {checks} testes de release passaram.')
