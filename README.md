# SOS Finança — V4.1.0

Aplicativo financeiro **local e individual** para Windows e Android.

A V4 reorganiza o SOS Finança em torno de uma regra simples: **nada solto**. Cartões e áreas de gerenciamento funcionam como pastas; ações de edição ficam escondidas até o usuário pedir; alertas levam diretamente ao item que precisa de atenção; exclusões vão para a Lixeira; e a sincronização PC ↔ celular passa a combinar alterações dos dois aparelhos.


## Novidades da V4.1

- A antiga aba **Contas** foi substituída por **Fixos**.
- A página principal se chama **Receitas e despesas fixas**.
- Duas pastas personalizáveis: **Receitas fixas** e **Contas fixas**.
- Visualização limpa: editar/excluir fica escondido até ativar **Editar**.
- Cada conta fixa recebe um cartão contornado com valor, vencimento e status do mês.
- **Marcar como pago/recebido** fica disponível diretamente no modo de visualização.
- Após pagar/receber, aparece um indicador de conferido.
- Histórico de pagamentos/recebimentos aparece somente durante a edição.
- O aviso de “Nenhuma pendência urgente” ganhou espaçamento correto abaixo dos indicadores da Visão geral.
- Alertas de valores fixos agora levam diretamente para a pasta correta.

## Principais novidades da V4

- Cartões tratados como **pastas personalizáveis**.
- Cor, ícone e ordem por cartão.
- Página própria para cada cartão.
- Barra com limite, utilizado, fatura do mês e disponível.
- Compras organizadas dentro da pasta do cartão.
- Botão **Editar** controla quando aparecem editar/excluir compras.
- **Gerenciar** virou uma área principal no menu lateral e no menu mobile.
- Gerenciar organizado em pastas e subpastas.
- Pastas de Gerenciar personalizáveis.
- **Lixeira** com restaurar e excluir permanentemente.
- Confirmação antes de excluir.
- Exclusão permanente gera tombstone para não ressuscitar após sincronização.
- Alertas da visão geral são clicáveis e levam direto para a ação.
- Preferências de notificações financeiras.
- Sincronização **bidirecional** PC ↔ celular pela rede local.
- Alterações exclusivas de PC e celular são preservadas.
- Conflitos do mesmo UUID usam `updated_at` mais recente.
- Backup automático no celular antes de aplicar o banco combinado.
- Verificação SHA-256 e `PRAGMA integrity_check`.
- Sem conta online, sem nuvem e sem compartilhamento entre familiares.

## Privacidade

Cada instalação continua individual.

- PC do pai ↔ celular do pai.
- PC da mãe ↔ celular da mãe.
- Seu PC ↔ seu celular.

Os bancos de pessoas diferentes nunca são combinados pelo aplicativo.

## Sincronização V4

1. PC e celular precisam estar na mesma rede Wi-Fi privada.
2. No PC: `Configurações → PC ↔ celular → Compartilhar/Sincronizar`.
3. O PC mostra o IP local.
4. No celular: `Configurações → PC ↔ celular`.
5. Digite somente o IP mostrado pelo PC.
6. O celular envia suas alterações ao PC.
7. O PC combina os dois bancos.
8. O PC devolve o banco combinado.
9. O celular cria backup e aplica o resultado.
10. Os dois aparelhos terminam com a mesma base combinada.

A porta local permanece fixa em `45454`. Não há chave temporária.

> A sincronização é feita por HTTP dentro da rede local. Use somente Wi-Fi privado/confiável.

## Notificações

A V4 usa o plugin nativo de notificações do Tauri e permite configurar lembretes financeiros. As notificações funcionam no aplicativo instalado e a permissão é solicitada quando necessário.

## Atualização para V4.1

Faça um backup dentro do SOS Finança antes de instalar os novos builds.

O identificador continua `com.sosfinanca.app` e o banco continua `sos_financa.db`.

As migrações da V4 preservam os dados existentes e adicionam os campos/estruturas necessários para personalização, Lixeira, notificações e sincronização bidirecional.

## GitHub Actions

Após o push na branch `main`, serão executados:

- **Build Windows**
- **Build Android**

Artifacts esperados:

- `SOS-Financa-Windows`
- `SOS-Financa-Android-Release` quando assinatura estiver configurada
- `SOS-Financa-Android-TESTE` caso os Secrets de assinatura não estejam configurados

## Documentação V4

- `docs/COMO_ATUALIZAR_V4.md`
- `docs/ORGANIZACAO_V4.md`
- `docs/SINCRONIZACAO_V4.md`
- `docs/RELATORIO_QA_V4.0.0.md`
- `docs/DADOS_PARA_TESTE.md`

## Testes

A V4 passou por testes de regras financeiras, armazenamento, migrações SQLite, Lixeira, tombstones, personalização, cartões em pastas, interface desktop/mobile, alertas acionáveis, sincronização bidirecional, conflitos por `updated_at` e release Windows/Android.

O ambiente de preparação não possui Rust/Tauri nativo instalado, então a compilação final `.exe/.apk` é validada pelo GitHub Actions após o push.
