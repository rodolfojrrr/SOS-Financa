# SOS Finança V2.0.0 — foco em usabilidade

## Princípio central

A V2 mantém a separação entre **Consultar** e **Gerenciar**, mas reduz a rigidez dos cadastros. O usuário deve conseguir começar com informação incompleta e completar depois.

## Receitas fixas

- Salário e outras entradas mensais possuem área própria.
- Data de início, data final, dia de recebimento, categoria e conta são opcionais.
- Se a data inicial ficar vazia, o controle passa a valer a partir do mês em que o registro foi cadastrado, sem alterar relatórios de meses anteriores.
- Sem data final, continua indefinidamente.
- Registrar recebimento em um mês não encerra a recorrência.
- O valor mensal pode ser editado.
- A recorrência pode ser pausada.

## Contas fixas

- Separadas das dívidas contratuais.
- Não exigem data de início/fim nem vencimento.
- Sem vencimento, entram no planejamento mensal sem gerar falso atraso.
- Pagamento de um mês não remove os meses seguintes.
- Valor pode ser alterado.

## Cartões

- Cada cartão possui sua própria visão de compras.
- A compra pode ser adicionada já de dentro do cartão.
- Limite, fechamento, vencimento, banco, bandeira e últimos dígitos são opcionais.
- Se fechamento não for informado, a compra usa o mês da compra como fatura de referência.
- Compra e pagamento da fatura continuam separados para não duplicar despesas.

## Dívidas

- Data de início é opcional.
- Saldo, juros, quantidade de parcelas e credor são opcionais.
- É possível cadastrar apenas o nome e a parcela mensal.
- Dívida sem saldo conhecido permanece visível como “saldo não informado”.
- Se não houver data de início, o app passa a projetar a parcela a partir do mês em que a dívida foi cadastrada.
- Dados podem ser completados depois.

## Formulários

Campos não essenciais foram explicitamente marcados como opcionais. Valores monetários que dão sentido ao registro continuam obrigatórios nos fluxos em que são necessários. Descrições vazias recebem nomes padrão para evitar travar o usuário.

## Windows

O executável Release usa `windows_subsystem = "windows"`, eliminando a janela CMD que aparecia junto do aplicativo instalado.

## Preservação do histórico de valores fixos

Ao registrar o pagamento/recebimento de uma recorrência, a V2 guarda também o valor esperado daquele mês. Assim, alterar o salário ou uma conta fixa depois não transforma meses antigos já quitados em pendências falsas.
