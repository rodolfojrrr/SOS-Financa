# Atualizar para SOS Finança V3.0.0

## Windows já usando V2

1. Abra a V2 instalada.
2. Entre em Configurações e faça um backup.
3. Feche o aplicativo.
4. Atualize o repositório com os arquivos da V3.
5. Aguarde o workflow **Build Windows** ficar verde.
6. Baixe o artifact `SOS-Financa-Windows`.
7. Extraia e execute o instalador `.exe`.
8. Instale por cima da versão existente.
9. Abra o app e confira contas, cartões, fixos e dívidas antes de lançar novos dados.

O identificador da aplicação não foi alterado e a inicialização SQLite é feita por migrações/`CREATE TABLE IF NOT EXISTS`, preservando a base anterior.

## Android novo

Para o primeiro teste, use o artifact `SOS-Financa-Android-TESTE`.

Para uso definitivo e futuras atualizações, configure a assinatura descrita em `ANDROID_V3.md` e use `SOS-Financa-Android-Release`.

## Repositório Git

Copie **o conteúdo** da pasta V3 para a pasta que já contém `.git`, substituindo os arquivos antigos. Não apague `.git` e não copie nenhum banco `.db` real para o repositório.

Depois execute:

`06_SUBIR_OU_ATUALIZAR_GITHUB.bat`

ou:

```bash
git add -A
git commit -m "SOS Financa V3.0.0 FINAL - Windows e Android"
git push
```

O push dispara os dois workflows: Windows e Android.
