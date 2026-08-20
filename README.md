# SOS Finança — V3.2.0

Aplicativo financeiro **local e individual** para Windows e Android.

A V3.1 mantém a interface e a lógica financeira aprovadas e adiciona **sincronização local PC → celular pela mesma rede Wi‑Fi**, sem nuvem e sem compartilhar dados entre pessoas.

## O que entra nesta versão

- Dashboard de situação financeira.
- Receitas e despesas comuns.
- Receita fixa, incluindo salário mensal.
- Contas fixas sem obrigar data inicial/final.
- Contas bancárias e carteira.
- Cartões independentes, compras por cartão, parcelamento e faturas.
- Dívidas, empréstimos e financiamentos com informações opcionais.
- Pagamentos e estornos protegidos.
- Orçamento por categoria.
- Categorias e subcategorias.
- Metas financeiras.
- Relatórios e visão mensal.
- Banco SQLite local por instalação.
- Backup local do banco.
- Tema claro/escuro.
- Interface responsiva para desktop e celular.
- Botão Voltar do Android integrado à navegação/modais.
- Safe areas de celulares respeitadas.
- Instalador Windows gerado pelo GitHub Actions.
- APK Android gerado pelo GitHub Actions.

## Regra de privacidade

Cada instalação possui o próprio banco. O SOS Finança não possui conta online, servidor financeiro ou banco compartilhado entre familiares.

Exemplo:

- PC do pai → banco local do pai.
- Celular do pai → banco local do pai naquele aparelho.
- PC da mãe → banco local da mãe.
- Seu PC/celular → seus bancos locais.

A sincronização Wi‑Fi PC ↔ celular do mesmo usuário **ainda não está ativa nesta versão**. Por segurança, os bancos permanecem independentes até esse mecanismo receber tratamento próprio de conflito e validação.

## Atualização do Windows V2 → V3

O identificador `com.sosfinanca.app` foi preservado e o arquivo de banco continua `sos_financa.db`. Faça um backup dentro do SOS Finança antes de instalar a nova versão e instale a V3 por cima da versão anterior.

## Arquivos principais

- `00_ABRIR_PREVIA.bat` — prévia rápida da interface.
- `01_PREPARAR_WINDOWS.bat` — prepara ambiente Windows.
- `02_EXECUTAR_WINDOWS.bat` — executa em desenvolvimento.
- `03_GERAR_INSTALADOR_WINDOWS.bat` — build local Windows.
- `04_PREPARAR_ANDROID.bat` — prepara projeto Android em PC com SDK configurado.
- `05_GERAR_APK_ANDROID.bat` — gera APK localmente.
- `06_SUBIR_OU_ATUALIZAR_GITHUB.bat` — commit/push para o GitHub.
- `07_TESTAR_REGRAS.bat` — testes de lógica, banco e release.
- `08_CRIAR_CHAVE_ANDROID.bat` — cria a chave privada para APK Release.

## GitHub Actions — Windows

Ao fazer push na branch `main`, o workflow **Build Windows** executa os testes, compila o Tauri e publica o artifact:

`SOS-Financa-Windows`

Dentro dele fica o instalador `.exe` NSIS.

## GitHub Actions — Android

O workflow **Build Android** também roda em cada push na `main`.

### Primeiro teste, sem chave Release

Se os secrets de assinatura ainda não estiverem configurados, o workflow gera automaticamente:

`SOS-Financa-Android-TESTE`

Esse artifact contém um APK debug instalável, indicado apenas para validar o aplicativo no celular. Ele pode usar uma assinatura de teste diferente entre builds; portanto, não coloque dados reais nele. Para migrar ao APK Release, pode ser necessário desinstalar a build de teste, o que remove os dados locais dela.

### APK Release definitivo

Antes de usar o Android como instalação definitiva, crie uma chave de assinatura:

1. Execute `08_CRIAR_CHAVE_ANDROID.bat`.
2. Guarde o arquivo `.jks` e a senha em local seguro, fora do Git.
3. No GitHub, abra `Settings → Secrets and variables → Actions`.
4. Crie os secrets:
   - `ANDROID_KEY_BASE64` — conteúdo completo de `ANDROID_KEY_BASE64.txt`.
   - `ANDROID_KEY_PASSWORD` — senha escolhida ao criar a chave.
   - `ANDROID_KEY_ALIAS` — `sosfinanca`.
5. Rode novamente o workflow **Build Android**.

Com os três secrets disponíveis, o artifact passa a ser:

`SOS-Financa-Android-Release`

**Não apague a chave `.jks`.** A mesma chave deve ser usada nas futuras versões para que o Android aceite uma atualização instalada por cima da anterior.

## Testes

A entrega inclui testes de:

- regras financeiras;
- armazenamento da prévia;
- estrutura/migrações SQLite;
- interface desktop/mobile;
- navegação do botão Voltar no Android;
- configuração Windows/Android;
- preparação da assinatura Android.

Veja `docs/RELATORIO_QA_V3.0.0.md`.

## Antes de colocar dados reais

1. Faça o primeiro build Windows e Android no GitHub.
2. Teste com valores fictícios.
3. No Android, prefira configurar a chave Release antes de começar a usar dados reais.
4. Teste receita fixa, conta fixa, cartão parcelado, fatura, dívida e backup.
5. Só depois migre o uso cotidiano para a V3.


## Sincronização Wi-Fi — V3.1

A sincronização desta versão é propositalmente **PC → celular**. O PC é a origem e o banco do celular é substituído somente depois de confirmação. Antes da troca, o Android cria automaticamente um backup do banco que já estava no aparelho.

1. Conecte PC e celular à mesma rede Wi-Fi privada.
2. No PC: `Configurações → Sincronização → Enviar dados deste PC`.
3. No celular: `Configurações → Sincronização`.
4. Digite IP, porta e chave temporária mostrados no PC.
5. Confirme `Receber do PC`.

A sessão expira em 10 minutos. O pacote usa autenticação HMAC, criptografia ChaCha20-Poly1305 e SHA-256 para verificar integridade. Não use em Wi-Fi público.


## Sincronização local 3.2

A sincronização PC → Android foi reescrita. O protocolo TCP próprio, a chave temporária, HMAC e criptografia do transporte foram removidos do fluxo de pareamento.

Agora o Windows abre temporariamente um pequeno servidor HTTP local na porta 45454. O Android usa um cliente HTTP para testar `/ping` e baixar `/sos-financa.db`. O banco recebido é validado pelo cabeçalho do SOS Finança, limite de tamanho, SHA-256 e `PRAGMA integrity_check` antes de substituir o banco local. Um backup do celular é criado antes da importação.

O servidor fica disponível por cerca de 3 minutos e encerra após um download. Use somente em uma rede Wi-Fi privada/confiável.
