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
styles = (root / 'app' / 'styles.css').read_text(encoding='utf-8')
gitignore = (root / '.gitignore').read_text(encoding='utf-8')

ok(main_conf['version'] == '3.0.0', 'configuração Tauri está na versão 3.0.0')
ok(re.search(r'^version = "3\.0\.0"$', cargo, re.M) is not None, 'pacote Rust está na versão 3.0.0')
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

print(f'\nTodos os {checks} testes de release passaram.')
