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

ok(main_conf['version'] == '3.1.4', 'configuração Tauri está na versão 3.1.4')
ok(re.search(r'^version = "3\.1\.4"$', cargo, re.M) is not None, 'pacote Rust está na versão 3.1.4')
ok(main_conf['identifier'] == 'com.sosfinanca.app', 'identificador foi preservado para atualizar a instalação existente')
ok(win_conf['bundle']['targets'] == ['nsis'], 'Windows gera instalador NSIS')
ok(win_conf['app']['windows'][0]['minWidth'] == 900, 'restrições de janela ficam somente na configuração Windows')
ok(android_conf['bundle']['android']['minSdkVersion'] == 24, 'Android mantém compatibilidade mínima API 24')
ok('windows_subsystem = "windows"' in main_rs, 'build Release do Windows não abre CMD')
ok('cargo tauri android init --ci --skip-targets-install' in android_yml, 'workflow Android inicializa o projeto sem prompts')
ok('cargo tauri android build --apk --ci' in android_yml, 'workflow Android gera APK Release quando há assinatura')
ok('cargo tauri android build --debug --apk --ci' in android_yml, 'workflow Android gera APK de teste quando não há assinatura')
ok('SOS-Financa-Android-Release' in android_yml and 'SOS-Financa-Android-TESTE' in android_yml, 'workflow Android publica artifacts separados para Release e teste')
ok('SOS-Financa-Windows' in windows_yml and 'bundle/nsis/*.exe' in windows_yml, 'workflow Windows publica o instalador como artifact')
ok('window.addEventListener(\'popstate\'' in app_js and 'pushHistory(true)' in app_js, 'navegação trata o botão Voltar do Android')
ok('env(safe-area-inset-top)' in styles and 'env(safe-area-inset-bottom)' in styles, 'layout respeita áreas seguras de celulares')
ok('*.jks' in gitignore and 'keystore.properties' in gitignore, 'chaves de assinatura Android ficam fora do Git')
ok('start_sync_server' in lib_rs and 'receive_sync_from_pc' in lib_rs, 'comandos de sincronização PC e Android estão registrados no Tauri')
ok('ChaCha20Poly1305' in sync_rs and 'HmacSha256' in sync_rs and 'Sha256' in sync_rs, 'sincronização usa autenticação, criptografia e verificação de integridade')
ok('SESSION_SECONDS: u64 = 600' in sync_rs, 'sessão de sincronização expira automaticamente em 10 minutos')
ok('FIXED_PORT: u16 = 45454' in sync_rs, 'Windows usa sempre a porta fixa 45454')
ok('FIXED_TOKEN: &str = "534F5346494E414E"' in sync_rs, 'Windows usa código fixo compatível com o APK atual')
ok('let plain = fs::read(&snapshot)' in sync_rs and 'let _ = fs::remove_file(&snapshot)' in sync_rs, 'snapshot é carregado em memória antes de abrir a sessão de rede')
ok('prepare_payload' in sync_rs and 'DATA {} {} {}' in sync_rs, 'payload criptografado é preparado antes de atender o celular')
ok('stream.flush().map_err' in sync_rs and 'set_nodelay(true)' in sync_rs, 'cabeçalho é descarregado imediatamente e socket usa baixa latência')
ok("'on':\n  workflow_dispatch:" in android_yml and "paths:\n      - '.github/workflows/build-android.yml'" in android_yml, 'esta atualização dispara um build Android e mantém execução manual disponível')
ok('import_sync_database' in sync_rs and 'create_sync_snapshot' in sync_rs, 'sincronização usa snapshot consistente e importação protegida do SQLite')
ok('startSyncServer' in storage_js and 'receiveSyncFromPc' in storage_js, 'ponte JavaScript expõe envio e recebimento Wi-Fi')
ok('Enviar dados deste PC' in app_js and 'Receber do PC' in app_js, 'interface mostra fluxos específicos para PC e celular')
ok('configure_android_network.py' in android_yml, 'workflow Android injeta permissões de rede após inicialização')
ok('async fn receive_sync_from_pc' in lib_rs and 'spawn_blocking' in lib_rs, 'recebimento Android sai da thread principal da interface')
ok('BufReader' not in sync_rs and 'read_protocol_line(&mut stream, 1024)' in sync_rs, 'Android lê cabeçalho e banco pela mesma conexão sem buffer duplicado')
ok("port: 45454" in storage_js and "code: '534F-5346-494E-414E'" in storage_js, 'porta e código de compatibilidade ficam internos no aplicativo')
ok("S.receiveSyncFromPc(host)" in app_js and 'name="port"' not in app_js[app_js.find("if(sync.canReceive)"):app_js.find("return shell", app_js.find("if(sync.canReceive)"))], 'tela Android pede somente o IP do PC')
ok('A conexão terminou após receber' in sync_rs and 'A transferência parou em' in sync_rs, 'erros de transferência informam quantos bytes chegaram antes da falha')

with tempfile.TemporaryDirectory() as tmp:
    tmp_root = Path(tmp)
    gradle = tmp_root / 'src-tauri' / 'gen' / 'android' / 'app' / 'build.gradle.kts'
    gradle.parent.mkdir(parents=True)
    gradle.write_text(
        '''plugins {\n    id("com.android.application")\n}\n\nandroid {\n    buildTypes {\n        getByName("release") {\n            isMinifyEnabled = false\n        }\n    }\n}\n''',
        encoding='utf-8'
    )
    script = root / 'scripts' / 'configure_android_signing.py'
    result = subprocess.run([sys.executable, str(script)], cwd=tmp_root, capture_output=True, text=True)
    patched = gradle.read_text(encoding='utf-8')
    ok(result.returncode == 0 and 'signingConfigs' in patched and 'signingConfig = signingConfigs.getByName("release")' in patched,
       'script de assinatura consegue preparar um projeto Android gerado')


with tempfile.TemporaryDirectory() as tmp:
    tmp_root = Path(tmp)
    manifest = tmp_root / 'src-tauri' / 'gen' / 'android' / 'app' / 'src' / 'main' / 'AndroidManifest.xml'
    manifest.parent.mkdir(parents=True)
    manifest.write_text('<manifest xmlns:android="http://schemas.android.com/apk/res/android">\n    <application android:label="SOS Finança" />\n</manifest>\n', encoding='utf-8')
    script = root / 'scripts' / 'configure_android_network.py'
    result = subprocess.run([sys.executable, str(script)], cwd=tmp_root, capture_output=True, text=True)
    patched = manifest.read_text(encoding='utf-8')
    ok(result.returncode == 0 and 'android.permission.INTERNET' in patched and 'android.permission.ACCESS_NETWORK_STATE' in patched,
       'script de rede adiciona as permissões Android necessárias para sockets locais')

print(f'\nTodos os {checks} testes de release passaram.')
