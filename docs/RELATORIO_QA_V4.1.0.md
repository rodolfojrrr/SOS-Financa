# Relatório de QA — SOS Finança V4.1.0

## Resultado

**177 verificações aprovadas** no ambiente de preparação.

- 45 regras financeiras;
- 9 armazenamento;
- 16 estrutura/migrações SQLite;
- 8 cenários de sincronização bidirecional;
- 55 configuração/release;
- 44 interface desktop/mobile.

## Melhorias V4.1 testadas

- antiga aba `Contas` substituída por `Fixos`;
- página `Receitas e despesas fixas`;
- duas pastas: `Receitas fixas` e `Contas fixas`;
- personalização das duas pastas;
- cartões contornados para cada valor fixo;
- botão `Marcar como pago/recebido` no modo normal;
- indicador de conferido após pagamento/recebimento;
- edição/exclusão ocultas no modo normal;
- histórico de pagamentos oculto no modo normal;
- histórico visível no modo Editar;
- adicionar novo valor fixo somente no modo Editar;
- alerta de compromisso fixo leva para a pasta correta;
- espaçamento do aviso `Nenhuma pendência urgente`;
- layout sem rolagem horizontal em 390 px e 320 px.

## Preservação de dados

Não houve alteração destrutiva no esquema do banco para esta versão.

O identificador continua `com.sosfinanca.app`, e o arquivo local continua `sos_financa.db`.

A atualização do repositório não contém banco financeiro.
