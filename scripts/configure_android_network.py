from pathlib import Path

manifest = Path('src-tauri/gen/android/app/src/main/AndroidManifest.xml')
if not manifest.exists():
    raise SystemExit('AndroidManifest.xml não encontrado após cargo tauri android init')

text = manifest.read_text(encoding='utf-8')
permissions = [
    '<uses-permission android:name="android.permission.INTERNET" />',
    '<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />',
]
missing = [p for p in permissions if p not in text]
if missing:
    marker = '<application'
    pos = text.find(marker)
    if pos < 0:
        raise SystemExit('Tag <application> não encontrada no AndroidManifest.xml')
    text = text[:pos] + ''.join(p + '\n    ' for p in missing) + text[pos:]
    manifest.write_text(text, encoding='utf-8')

print('Permissões de rede local configuradas no AndroidManifest.xml')
