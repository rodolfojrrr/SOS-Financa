# SOS Finança V1 — decisões funcionais

## 1. Escopo

Aplicação exclusivamente financeira. Não há treino, estudo, agenda de rotina, comunidade ou outros módulos.

## 2. Privacidade e uso

- Uma instalação = uma pessoa.
- Banco local por aparelho.
- Sem nuvem obrigatória.
- Sem compartilhamento de gastos entre familiares.
- O modo “Gestão da casa” apenas muda o contexto do usuário; tecnicamente continua sendo uma instalação individual.

## 3. Navegação

### Área Consultar

Início → Mês → Contas → Cartões → Dívidas → Orçamento → Relatórios.

Nenhuma dessas telas exibe controles de apagar/editar registros diretamente.

### Área Gerenciar

Lançamentos → Fixos → Cartões → Dívidas → Cadastros → Orçamento.

Aqui ficam criação, edição, pagamentos e arquivamento.

## 4. Saldo agora x previsão

- Saldo disponível: saldo contábil das contas locais considerando registros efetivados.
- Resultado previsto do mês: receitas previstas menos despesas previstas.
- No mês atual, a tela inicial combina saldo disponível com valores ainda não realizados para estimar o fechamento.

## 5. Cartões

Compra e pagamento da fatura são entidades diferentes.

A compra é dividida em centavos entre as parcelas. O mês inicial da fatura considera dia de fechamento e dia de vencimento do cartão. O pagamento da fatura é que reduz a conta bancária selecionada.

## 6. Dívidas

O cadastro aceita:

- tipo;
- credor;
- valor original;
- total contratado;
- saldo devedor no início do controle;
- valor da parcela;
- quantidade de parcelas;
- parcelas já pagas;
- dia do vencimento;
- taxa de juros informada pelo usuário;
- prioridade;
- observação.

Os pagamentos guardam o mês da parcela, a data real em que o pagamento aconteceu, o valor pago, principal/amortização e juros opcionais. O saldo estimado diminui pelo principal registrado. Parcelas já pagas antes do início do uso não reaparecem como atrasadas.

## 7. Compromissos recorrentes

Conta ou receita recorrente é previsão. Um registro separado confirma quando o valor foi realmente pago ou recebido.

## 8. Orçamento

Orçamentos são mensais e vinculados a categoria. O consumo da categoria pode receber gastos avulsos, parcelas de cartão e pagamentos recorrentes.

## 9. Segurança operacional

- Foreign keys ativadas no SQLite.
- WAL ativado.
- Backup antes de restauração.
- Nome de arquivo de backup validado.
- Arquivamento lógico nos registros principais.
- IDs UUID nos registros financeiros.


## 10. Correções e integridade da V1.0.1

- Vencimentos atrasados continuam aparecendo mesmo depois da virada do mês.
- Próximos 7 dias podem incluir vencimentos do mês seguinte.
- Pagamentos não podem ultrapassar o restante da fatura, do compromisso ou do saldo devedor.
- O saldo-base de uma dívida não pode ser alterado depois de haver pagamentos registrados, evitando dupla redução do saldo.
- Operações financeiras registradas por engano têm caminho de estorno/arquivamento na área Gerenciar quando seguro.
- Datas do dia atual são calculadas no horário local do aparelho.
