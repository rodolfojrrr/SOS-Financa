# SOS Finança — Relatório de QA V2.0.0

## Resultado

A rodada local automatizada da V2 passou em:

- 45 testes de regras financeiras;
- 9 testes do armazenamento da prévia;
- 9 testes estruturais do SQLite;
- 29 verificações de interface desktop/mobile.

**Total: 92 verificações aprovadas.**

Também foram validados:

- sintaxe de `app.js`, `finance.js` e `storage.js`;
- JSON de configuração do Tauri/capabilities;
- TOML do Cargo;
- YAML do workflow do GitHub Actions;
- atributo de subsistema Windows no `main.rs`.

## Cenários V2 cobertos

- salário fixo sem data de início/fim;
- recebimento do salário sem encerrar a recorrência;
- conta fixa paga em um mês reaparecendo no mês seguinte;
- conta fixa sem vencimento não gerando falso atraso;
- edição do valor de conta fixa;
- preservação do valor histórico de uma recorrência já paga/recebida após edição;
- dívida sem data inicial e sem saldo conhecido;
- dívida incompleta permanecendo visível;
- cartão sem limite/fechamento/vencimento;
- compra adicionada diretamente dentro de um cartão;
- parcelamento preservando centavos e número exato de parcelas;
- compra no cartão sem baixar conta bancária antes da fatura;
- pagamento da fatura baixando a conta;
- financiamento antigo com parcelas históricas;
- pagamento atrasado com mês de competência separado;
- virada de mês, fevereiro e ano bissexto;
- layout em 1440 px, 390 px e 320 px sem erro de JavaScript.

## Limitação do ambiente

O ambiente usado para esta revisão não possui Rust/Cargo, portanto o binário Tauri não foi recompilado localmente. O workflow do GitHub continua sendo a validação final do build nativo Windows. Antes de usar dados financeiros reais, faça backup da instalação V1 e teste a V2 com valores fictícios.
