# SOS Finança — V2.0.0

Aplicativo local de organização financeira para Windows e Android.

## O foco da V2

A V2 mantém o visual da V1 e refaz os fluxos de uso para reduzir atrito. A regra é: **pedir somente o que é indispensável e deixar o restante para depois**.

### Principais mudanças

- Receitas fixas e contas fixas agora são áreas separadas.
- Salário pode ser cadastrado como receita fixa e se repete todos os meses.
- Conta fixa não exige data de início, data final nem dia de vencimento.
- Sem data final, a recorrência continua indefinidamente.
- Marcar um mês como pago/recebido não encerra a recorrência.
- Valor de uma recorrência pode ser editado normalmente.
- Meses já pagos/recebidos preservam o valor esperado daquele período mesmo depois de uma alteração no valor mensal.
- Recorrências podem ser pausadas sem apagar o histórico.
- Cartões possuem uma visão própria de compras por cartão.
- É possível abrir um cartão e adicionar compras diretamente nele.
- Cadastro de cartão aceita limite, fechamento e vencimento em branco.
- Dívidas podem ser cadastradas sem data de início, juros, total de parcelas ou saldo conhecido.
- A dívida continua visível como “saldo não informado” até os dados serem completados.
- Formulários de movimentação, cartão e dívida usam valores padrão quando campos não essenciais ficam vazios.
- A área Gerenciar ganhou atalhos para gasto rápido, salário/receita fixa, conta fixa, compra no cartão e nova dívida.
- O executável Release do Windows não abre mais uma janela CMD junto com o app.

## Dados locais

Cada instalação usa seu próprio banco SQLite local. Não existe conta online ou compartilhamento entre usuários.

## Atualização da V1

A V2 usa o mesmo identificador do aplicativo e o mesmo arquivo `sos_financa.db`. O `init` do banco mantém as tabelas existentes, então uma instalação V1 pode ser atualizada preservando o banco. Mesmo assim, faça backup antes de instalar qualquer atualização.

## Teste rápido

- `00_ABRIR_PREVIA.bat`: abre a prévia HTML.
- `07_TESTAR_REGRAS.bat`: executa os testes de regras financeiras e armazenamento.
- `03_GERAR_INSTALADOR_WINDOWS.bat`: gera o instalador Windows em máquina preparada para Tauri.

## Fluxo recomendado de teste da V2

1. Cadastre uma conta.
2. Vá em Gerenciar → Receitas fixas e cadastre um salário sem data inicial/final.
3. Registre o recebimento do salário no mês atual e avance o mês: ele deve continuar previsto.
4. Cadastre uma conta fixa sem data inicial/final, pague o mês atual e avance: ela deve reaparecer.
5. Edite o valor da conta fixa e confira que meses já pagos não são reabertos.
6. Cadastre um cartão sem fechamento/vencimento se quiser, abra o cartão e adicione compras diretamente nele.
7. Cadastre uma dívida sem data de início e sem saldo, apenas com nome/parcela, e complete os dados depois.

A V2 prioriza usabilidade e tolerância a informações incompletas sem esconder a estrutura financeira.
