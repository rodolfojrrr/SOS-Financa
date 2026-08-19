# Roteiro de teste — SOS Finança V2.0.0

Use valores fictícios antes de cadastrar as finanças reais.

## 1. Conta e gasto rápido

1. Crie uma conta chamada `Conta principal` com saldo inicial de R$ 2.500.
2. Registre um gasto rápido de R$ 4,50 e deixe descrição, categoria e forma de pagamento vazias.
3. Confira se o app salva normalmente e usa um texto padrão.

## 2. Salário fixo

1. Vá em **Gerenciar → Receitas fixas**.
2. Cadastre `Salário` de R$ 3.000.
3. Deixe início, fim e dia de recebimento vazios.
4. Registre o recebimento do mês atual.
5. Mude para o mês seguinte e confirme que o salário aparece de novo como previsto.
6. Volte ao mês atual, edite o salário para R$ 3.200 e confirme que o mês já recebido continua quitado.
7. Avance novamente e confirme que o novo valor de R$ 3.200 passa a valer dali em diante.

## 3. Conta fixa

1. Vá em **Gerenciar → Contas fixas**.
2. Cadastre `Internet` de R$ 99,90 sem data inicial, data final ou vencimento.
3. Marque o mês atual como pago.
4. Avance um mês e confirme que a conta reaparece.
5. Edite o valor para R$ 109,90.
6. Confira se o histórico já pago não é reaberto.

## 4. Cartão e compras

1. Cadastre um cartão chamado `Nubank teste`.
2. Deixe limite, fechamento e vencimento vazios para validar o cadastro simples.
3. Abra o próprio cartão e use **Adicionar compra**.
4. Cadastre `Geladeira` de R$ 1.200 em 6x.
5. Confira se a compra aparece dentro daquele cartão e se são criadas exatamente seis parcelas.
6. Se quiser, depois informe fechamento/vencimento e faça outro teste próximo da data de fechamento.

## 5. Dívida incompleta

1. Cadastre uma dívida chamada `Empréstimo antigo`.
2. Informe somente a parcela de R$ 450 e deixe data de início, saldo, juros e total de parcelas vazios.
3. Confirme que ela continua visível como `saldo não informado`.
4. Edite depois e complete os dados que descobrir.

## 6. Financiamento conhecido

1. Cadastre um financiamento com saldo devedor de R$ 18.000 e parcela de R$ 900.
2. Se ele já existia antes do app, informe as parcelas históricas que souber; se não souber, deixe em branco.
3. Registre uma parcela de R$ 900 com R$ 750 de principal e R$ 150 de juros.
4. Confira se o saldo cai para R$ 17.250.

## 7. Segurança

1. Teste o estorno de um lançamento errado.
2. Teste o estorno de pagamento de conta fixa, fatura e dívida.
3. Tente pagar uma fatura acima do restante e confirme o bloqueio.
4. Faça um backup local.
5. Só depois comece a cadastrar dados financeiros reais.
