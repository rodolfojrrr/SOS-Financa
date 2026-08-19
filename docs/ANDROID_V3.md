# Android — SOS Finança V3.0.0

## Caminho mais fácil: GitHub Actions

Depois de copiar a V3 para a pasta do repositório e fazer push, abra:

`GitHub → SOS-Financa → Actions → Build Android`

O workflow compila o aplicativo sem você precisar montar o Android Studio manualmente para o primeiro teste.

## APK de teste

Sem secrets de assinatura configurados, o workflow produz:

`SOS-Financa-Android-TESTE`

Baixe o artifact, extraia o ZIP e instale o `.apk` no celular.

A assinatura debug é apenas de teste e pode mudar entre builds do CI. Não coloque dados reais nessa instalação; ao trocar para o APK Release pode ser necessário desinstalar a build debug, apagando os dados locais dela.

Use essa build somente para validar:

- abertura do app;
- navegação inferior;
- botão Voltar;
- cadastros;
- cartões/compras;
- contas fixas;
- dívidas;
- banco local;
- backup interno;
- responsividade.

## Preparar o APK Release

Execute no Windows:

`08_CRIAR_CHAVE_ANDROID.bat`

A chave será criada em uma pasta fora do repositório:

`%USERPROFILE%\SOS-Financa-Chave-Android`

O script gera:

- `sos-financa-release.jks` — chave privada, não enviar ao Git.
- `ANDROID_KEY_BASE64.txt` — texto para o Secret do GitHub.

Durante a criação, quando o `keytool` perguntar a senha específica da chave, pressione ENTER para reutilizar a mesma senha do JKS.

## Secrets no GitHub

No repositório:

`Settings → Secrets and variables → Actions → New repository secret`

Crie:

### ANDROID_KEY_BASE64

Cole todo o conteúdo de `ANDROID_KEY_BASE64.txt`.

### ANDROID_KEY_PASSWORD

Coloque a senha usada na criação da chave.

### ANDROID_KEY_ALIAS

Use:

`sosfinanca`

Depois execute novamente **Build Android**. O artifact final será:

`SOS-Financa-Android-Release`

## Atualizações futuras

Guarde para sempre:

- o arquivo `sos-financa-release.jks`;
- a senha;
- o alias `sosfinanca`.

O Android verifica a assinatura do aplicativo ao atualizar uma instalação existente. Se uma versão futura for assinada com outra chave, ela não poderá substituir normalmente a versão anterior.

## Banco no celular

O banco SQLite fica na área de dados local do próprio aplicativo Android. Ele não é o mesmo banco do PC.

A V3 não faz sincronização Wi‑Fi automática entre os aparelhos. Isso evita que duas cópias divergentes do histórico financeiro sejam mescladas de forma insegura antes de existir um mecanismo específico de conflitos.
