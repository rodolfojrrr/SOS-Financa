# Escopo — SOS Finança V3.0.0

## Objetivo

Fornecer uma aplicação financeira local, individual e simples de usar para organização pessoal ou gestão das contas de uma casa, sem misturar finanças de usuários diferentes.

## Plataformas

- Windows desktop.
- Android.

A mesma base de código Tauri/Rust/HTML/CSS/JavaScript atende as duas plataformas, com configurações específicas por plataforma.

## Armazenamento

- SQLite local por instalação.
- Sem nuvem obrigatória.
- Sem cadastro online.
- Sem banco compartilhado entre familiares.
- Backups locais criados pelo próprio app.

## Área Consultar

- Dashboard.
- Visão do mês.
- Contas e receitas previstas/realizadas.
- Cartões e faturas.
- Dívidas.
- Orçamento.
- Relatórios.

A área de consulta evita controles de edição espalhados pela interface.

## Área Gerenciar

- Gasto rápido.
- Receita/despesa.
- Receita fixa/salário.
- Conta fixa.
- Conta bancária.
- Transferência.
- Cartão.
- Compra no cartão.
- Pagamento de fatura.
- Dívida/financiamento/empréstimo.
- Pagamento de dívida.
- Categoria/subcategoria.
- Orçamento.
- Meta.
- Configurações/backup.

## Regras de usabilidade

- Campos são opcionais sempre que a lógica financeira permitir.
- Recorrência sem data final continua ativa.
- Pagar um mês não encerra uma recorrência.
- Alterar valor futuro não reabre mês já quitado.
- Dívidas podem começar incompletas e ser detalhadas depois.
- Cartões concentram suas próprias compras.
- Gasto pequeno deve ser registrável com poucos toques.
- Histórico financeiro não deve ser apagado silenciosamente.

## Regras financeiras principais

- Compra no cartão não diminui saldo bancário no momento da compra.
- Pagamento da fatura diminui a conta e libera limite.
- Parcelas preservam exatamente o valor total em centavos.
- Transferência entre contas não é receita nem despesa.
- Pagamento de dívida separa pagamento, amortização e juros quando informados.
- Contas atrasadas continuam visíveis ao atravessar meses.
- Meses históricos quitados não são reabertos quando valores recorrentes mudam.

## Mobile

- Navegação inferior responsiva.
- Safe area para barras/recortes do aparelho.
- Alvos de toque maiores.
- Botão/gesto Voltar fecha modal antes de sair e permite voltar de tela.
- Banco local próprio no Android.

## Não incluído na V3

- Sincronização automática PC ↔ Android.
- Open Finance.
- Nuvem.
- Compartilhamento financeiro entre familiares.
- Módulos de treino, estudo ou rotina.
