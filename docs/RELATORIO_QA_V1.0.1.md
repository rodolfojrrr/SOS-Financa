# SOS Finança — Relatório de QA V1.0.1

## Resultado

- 31/31 testes de regras financeiras: aprovados.
- 9/9 testes do armazenamento da prévia: aprovados.
- 6/6 testes estruturais do SQLite: aprovados.
- 15/15 smoke tests de interface: aprovados.
- Sintaxe JavaScript (`app.js`, `finance.js`, `storage.js`): aprovada.
- `tauri.conf.json`, capability e estrutura dos ícones: revisados.
- Scripts Windows/Android e workflow de Windows: revisados.

## Cenários cobertos

Datas locais, fevereiro/ano bissexto, dias 29/30/31, virada de mês e ano, parcelamento com centavos, fechamento/vencimento de cartão, pagamento de fatura, saldo bancário, financiamentos antigos, pagamentos atrasados, amortização x juros, contas recorrentes, vencimentos atrasados, próximos 7 dias, integridade de referências, arquivamento/estorno e responsividade mobile/desktop.

## Limitação desta rodada

O ambiente de QA não possui a toolchain nativa completa do Windows/Android para compilar e executar o `.exe`/`.apk` final. Por isso a camada Rust/SQLite foi validada estruturalmente e a interface foi validada em Chromium headless; o executável e o APK ainda precisam de smoke test no hardware real depois do primeiro build.

## Recomendação para o primeiro teste humano

Use apenas valores fictícios. Siga `DADOS_PARA_TESTE.md`, faça um backup, tente também corrigir/estornar lançamentos e só depois comece a cadastrar valores reais.
