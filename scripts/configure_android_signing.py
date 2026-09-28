from pathlib import Path

path = Path('src-tauri/gen/android/app/build.gradle.kts')
if not path.exists():
    raise SystemExit(f'Arquivo Android não encontrado: {path}')

text = path.read_text(encoding='utf-8')

imports = 'import java.io.FileInputStream\nimport java.util.Properties\n\n'
if 'import java.io.FileInputStream' not in text:
    text = imports + text

if 'signingConfigs {' not in text:
    anchor = '    buildTypes {'
    block = '''    signingConfigs {
        create("release") {
            val keystorePropertiesFile = rootProject.file("keystore.properties")
            val keystoreProperties = Properties()
            if (keystorePropertiesFile.exists()) {
                keystoreProperties.load(FileInputStream(keystorePropertiesFile))
            }
            keyAlias = keystoreProperties["keyAlias"] as String
            keyPassword = keystoreProperties["password"] as String
            storeFile = file(keystoreProperties["storeFile"] as String)
            storePassword = keystoreProperties["password"] as String
        }
    }

'''
    if anchor not in text:
        raise SystemExit('Bloco buildTypes não encontrado no Gradle Android.')
    text = text.replace(anchor, block + anchor, 1)

signing_line = 'signingConfig = signingConfigs.getByName("release")'
if signing_line not in text:
    anchors = ['        getByName("release") {', '        release {']
    for anchor in anchors:
        if anchor in text:
            text = text.replace(anchor, anchor + '\n            ' + signing_line, 1)
            break
    else:
        raise SystemExit('Bloco release não encontrado no Gradle Android.')

path.write_text(text, encoding='utf-8')
print('Assinatura Release configurada no Gradle Android.')
