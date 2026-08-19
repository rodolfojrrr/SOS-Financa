# Relatório de QA — SOS Finança V3.0.0 FINAL

Data da revisão: 19/08/2026.

## Resultado

Foram executadas **111 verificações automatizadas** na base entregue:

- 45 testes de regras financeiras.
- 9 testes do armazenamento da prévia.
- 9 testes estruturais/migrações SQLite.
- 32 testes de interface desktop/mobile.
- 16 testes de configuração de release Windows/Android.

Resultado: **111/111 aprovados**.

Também foram validados separadamente:

- sintaxe de `app.js`;
- sintaxe de `finance.js`;
- sintaxe de `storage.js`;
- JSON das configurações Tauri geral, Windows e Android;
- YAML dos workflows Windows e Android.

## Cobertura financeira

Entre os casos testados estão:

- datas locais sem deslocamento por UTC;
- vencimento em dia 31 e fevereiro/ano bissexto;
- intervalos atravessando virada de ano;
- distribuição exata de centavos em parcelamentos;
- fatura conforme fechamento/vencimento;
- limite comprometido/liberado;
- compra no cartão sem duplicar saída bancária;
- pagamento da fatura reduzindo a conta;
- amortização separada de juros;
- dívidas antigas e parcelas históricas;
- parcela paga atrasada em outro mês;
- contas atrasadas atravessando meses;
- próximos sete dias atravessando a virada do mês;
- salário fixo sem datas;
- recorrência mantida após pagamento/recebimento;
- preservação do valor histórico de mês quitado;
- edição de valor futuro;
- dívida com informações incompletas;
- cartão com campos opcionais.

## Cobertura de interface

Foram testados:

- configuração inicial;
- cadastro de conta;
- salário fixo;
- recebimento de salário;
- edição de recorrência;
- conta fixa;
- gasto rápido;
- cartão sem campos opcionais;
- compra adicionada de dentro do cartão;
- dívida incompleta;
- layout mobile;
- largura de 390 px;
- largura de 320 px;
- ausência de rolagem horizontal global;
- botão/gesto Voltar do Android;
- fechamento de modal antes de sair da tela;
- ausência de erros JavaScript no console durante o roteiro.

## Cobertura de release

Validado automaticamente:

- versão 3.0.0 no Tauri/Rust;
- identificador `com.sosfinanca.app` preservado;
- Windows configurado para NSIS;
- Release Windows sem janela CMD;
- Android API mínima 24;
- workflow Android sem prompts;
- APK Release quando existem secrets de assinatura;
- APK debug de teste quando os secrets ainda não existem;
- artifacts distintos para Windows/Android;
- safe areas mobile;
- `.jks` e `keystore.properties` ignorados pelo Git;
- script que injeta a configuração de assinatura no projeto Android gerado.

## Limitação da revisão neste ambiente

O ambiente usado para preparar a entrega não possui a cadeia Rust/Tauri/Android instalada para executar um build nativo completo de Windows e Android localmente. Por isso, o primeiro teste de compilação nativa da V3 será realizado pelos workflows do GitHub Actions depois do push.

A lógica JavaScript, banco, migrações, interface e arquivos de configuração foram testados antes da entrega. Se um workflow nativo falhar por incompatibilidade específica do runner/SDK, o log do GitHub indicará a etapa exata a corrigir.

## Procedimento recomendado

1. Fazer backup da V2 no Windows.
2. Subir a V3 para o GitHub.
3. Confirmar `Build Windows` verde.
4. Confirmar `Build Android` verde.
5. Instalar primeiro com dados fictícios.
6. Configurar a chave Android Release antes de usar dados reais no celular.
7. Repetir o roteiro de `DADOS_PARA_TESTE.md` nos dois aparelhos.
