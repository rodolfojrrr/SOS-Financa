# SOS Finança — V1.0.1 (revisada)

Aplicativo local de organização financeira para Windows e Android.

## Ideia central

O SOS Finança foi pensado para responder rapidamente:

- Quanto dinheiro tenho agora?
- Quanto devo no total?
- O que vence neste mês?
- Quanto ainda posso gastar?
- Onde meu dinheiro está indo?
- Minha situação está melhorando ou piorando?

Cada instalação é individual. Não existe banco familiar compartilhado, login online ou nuvem. Uma pessoa pode usar sua instalação para finanças pessoais e outra pode usar a mesma aplicação para administrar as contas da casa inteira.

## O que existe na V1

### Consultar

- Início com saldo disponível, dívida total, despesas do mês e fechamento projetado.
- Mês com receitas/despesas previstas x realizadas e agenda financeira.
- Contas com saldo consolidado e transferências.
- Cartões com limite, fatura, parcelamentos e próximas faturas.
- Dívidas com empréstimos, financiamentos, saldo devedor, parcelas, juros e progresso.
- Orçamento mensal por categoria e metas.
- Relatórios de 6 meses e gastos por categoria.

### Gerenciar

- Lançamentos avulsos de receita/despesa.
- Gasto rápido.
- Contas bancárias/carteira.
- Transferências entre contas sem virar receita/despesa.
- Receitas e despesas recorrentes.
- Registro de pagamento/recebimento de recorrências.
- Cartões.
- Compras parceladas.
- Pagamento de faturas.
- Dívidas, empréstimos e financiamentos.
- Pagamentos de dívida com mês da parcela, data real do pagamento, principal/amortização e juros opcionais.
- Categorias e subcategorias.
- Orçamento por categoria.
- Metas financeiras.
- Backup/restauração local do SQLite no app instalado.

## Regras financeiras importantes

1. **Compra no cartão não baixa a conta bancária.** Ela entra na fatura correspondente. O dinheiro sai da conta somente quando a fatura é paga.
2. **Transferência entre contas não é receita nem despesa.**
3. **Dívida é contrato, não apenas parcela.** O app guarda saldo devedor e pagamentos separadamente.
4. **Valor recorrente previsto não é considerado pago automaticamente.** É necessário registrar pagamento/recebimento.
5. **Consulta e edição ficam separadas.** A área principal serve para entender os números; a área Gerenciar concentra alterações.
6. **Arquivamento é preferido a apagar registros diretamente.**

## Tecnologia

- Tauri 2
- Rust
- SQLite com `rusqlite` e SQLite embutido
- HTML/CSS/JavaScript sem framework e sem CDN
- Banco salvo na pasta de dados local do aplicativo

O frontend não precisa de Node/npm para rodar: o Tauri empacota os arquivos estáticos em `app/`.

## Como testar a interface sem compilar

No Windows, execute:

`00_ABRIR_PREVIA.bat`

Essa prévia usa `localStorage` do navegador apenas para permitir testar telas e fluxo rapidamente. **Ela não é o banco definitivo.** O aplicativo compilado usa SQLite real.

## Como executar no Windows

1. Instale os pré-requisitos do Tauri para Windows: Rust e Microsoft C++ Build Tools.
2. Execute `01_PREPARAR_WINDOWS.bat`.
3. Execute `02_EXECUTAR_WINDOWS.bat`.

## Como gerar o instalador Windows

Execute:

`03_GERAR_INSTALADOR_WINDOWS.bat`

O NSIS será produzido na pasta de bundle do Tauri.

## Como preparar/gerar Android

Com Android Studio, SDK, NDK e JDK configurados:

1. `04_PREPARAR_ANDROID.bat`
2. `05_GERAR_APK_ANDROID.bat`

## Banco de dados

O SQLite cria tabelas separadas para:

- `accounts`
- `categories`
- `transactions`
- `commitments`
- `commitment_payments`
- `cards`
- `card_purchases`
- `card_payments`
- `debts`
- `debt_payments`
- `budgets`
- `transfers`
- `goals`

A separação é proposital para evitar misturar cartões, dívidas, contas e lançamentos em uma estrutura difícil de manter.

## Sincronização

A V1 já usa identificadores UUID nos registros e mantém o banco individual por instalação. A sincronização local PC ↔ Android **ainda não está habilitada nesta entrega**. Ela foi deixada de fora do primeiro núcleo para que cartões, faturas, dívidas e saldos sejam validados antes de permitir mesclagem entre bancos financeiros.

Quando implementada, a regra do projeto é: somente dispositivos da mesma pessoa podem sincronizar entre si.

## Build automático no GitHub

O projeto inclui `.github/workflows/build-windows.yml`. Ao enviar para a branch `main`, o GitHub Actions executa os testes financeiros e tenta gerar o instalador NSIS do Windows como artefato do workflow.

## Teste das regras financeiras

Se tiver Node disponível, execute:

`07_TESTAR_REGRAS.bat`

ou diretamente:

`node tests/finance.test.js`

O script executa os testes financeiros e os testes do armazenamento da prévia. A revisão V1.0.1 também foi validada com testes estruturais do SQLite e smoke tests de interface em desktop e mobile.

## Revisão de QA da V1.0.1

Antes desta entrega, a V1 foi revisada novamente com foco em integridade financeira e correção de uso real. Foram corrigidos, entre outros pontos:

- datas locais para evitar mudança de dia por UTC;
- vencimentos que atravessam a virada do mês;
- financiamentos antigos com parcelas já pagas antes de começar a usar o app;
- pagamento atrasado de dívida com mês de referência separado da data real do pagamento;
- proteção contra pagamento duplicado ou maior que fatura/compromisso/saldo devedor;
- validação de categorias, dias, parcelas e dados de cartão;
- estorno/arquivamento de transferências, pagamentos recorrentes, faturas e dívidas;
- bloqueio do saldo-base da dívida depois que pagamentos já foram registrados;
- comportamento dos modais compatível com a política de segurança do app instalado;
- navegação e responsividade em largura de desktop e celular.

A bateria atual contém 31 testes de regras financeiras, 9 testes do armazenamento da prévia, 6 testes estruturais do SQLite e 15 verificações de interface.

**Importante:** esses testes reduzem bastante o risco de erro, mas não substituem uma rodada de validação do executável compilado no Windows e do APK em aparelho Android real. Comece com valores fictícios antes de cadastrar a situação financeira real.

## Referências de produto usadas na V1

A V1 não copia nenhum aplicativo específico. Ela combina ideias observadas em ferramentas local-first de finanças, materiais de educação financeira sobre fluxo de caixa e redução de dívidas e relatos de usuários que preferem lançamento manual simples sem conexão bancária. As referências serviram para validar decisões de produto como: registro rápido, orçamento por categoria, visão de vencimentos, dívida tratada como saldo devedor e dados sob controle do próprio usuário.
