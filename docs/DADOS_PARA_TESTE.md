# Roteiro de teste — SOS Finança V3.0.0

Não comece com dados financeiros reais. Faça este roteiro primeiro no Windows e depois no Android.

## 1. Conta

Cadastre:

- Nome: Conta Teste
- Saldo inicial: R$ 2.000,00

## 2. Receita fixa

Cadastre:

- Nome: Salário
- Valor: R$ 3.000,00
- Sem data inicial/final, se preferir

Marque o mês atual como recebido. Avance a visão para o mês seguinte e confira se o salário continua previsto.

## 3. Conta fixa

Cadastre:

- Nome: Internet
- Valor: R$ 100,00
- Dia: 10
- Sem data final

Pague o mês atual. Confira que a recorrência continua existindo no mês seguinte.

Edite depois para R$ 110,00 e verifique se o mês já quitado não é reaberto.

## 4. Gasto rápido

Registre:

- R$ 4,50
- Descrição: Bombom

Deixe campos opcionais vazios para testar a tolerância do formulário.

## 5. Cartão

Cadastre:

- Nome: Cartão Teste
- Limite: R$ 2.000,00
- Fechamento: 15
- Vencimento: 22

Entre no cartão e adicione:

- Compra: Mercado
- Total: R$ 600,00
- 3 parcelas

Confira as faturas e o limite utilizado.

Registre um pagamento de fatura e confira que a saída acontece na conta somente no pagamento, sem duplicar a compra como despesa bancária.

## 6. Dívida

Cadastre:

- Nome: Financiamento Teste
- Saldo: R$ 10.000,00
- Parcela: R$ 500,00

Você pode deixar data de início, juros e total de parcelas em branco para testar o fluxo opcional.

Registre um pagamento com amortização e confira a redução do saldo.

## 7. Backup

Abra Configurações e crie um backup local. Confira se ele aparece na lista.

## 8. Android

Teste especificamente:

- abrir/fechar teclado;
- navegar pelas abas inferiores;
- abrir um cadastro e usar o botão Voltar físico/gesto;
- abrir um modal e apertar Voltar: o modal deve fechar antes do app;
- rotação não é necessária para uso normal; priorize modo retrato;
- confira se nenhum botão fica atrás das barras do sistema;
- registre gasto rápido com uma mão;
- feche e abra o app novamente para confirmar persistência local.

## 9. Resultado esperado

Se os passos acima passarem, faça o primeiro backup real no Windows e só então comece a cadastrar a situação financeira verdadeira.
