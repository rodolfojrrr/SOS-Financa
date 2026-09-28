# Relatório de QA — SOS Finança V4.2.0

## Resultado local

Foram aprovadas **151 verificações automatizadas** no ambiente de preparação:

- 52 regras financeiras;
- 11 cenários de armazenamento e Lixeira;
- 18 verificações de estrutura e migração SQLite;
- 8 cenários de sincronização bidirecional;
- 62 verificações de configuração e release.

Também foram executadas verificações de sintaxe nos arquivos JavaScript alterados.

## Correções cobertas

- primeira fatura sugerida a partir da data da compra e do fechamento do cartão;
- possibilidade de corrigir manualmente a primeira fatura antes de salvar;
- persistência da competência escolhida no SQLite e na sincronização;
- abertura automática do cartão no mês em que a compra foi lançada;
- separação entre compras em andamento e compras concluídas;
- compras à vista encerradas fora da lista principal;
- editar e excluir diretamente em cada compra;
- excluir pagamento de fatura lançado por engano;
- migração automática de bancos antigos sem apagar registros;
- exportação do banco SQLite para um destino escolhido pelo usuário;
- conferência do tamanho do arquivo depois da exportação.

## Validações finais fora deste ambiente

O teste de interface ponta a ponta está preparado em `tests/ui_smoke.py`, mas o navegador Chromium não está disponível neste ambiente de preparação. A compilação nativa Rust/Tauri e os builds `.exe`/`.apk` são validados pelos fluxos do GitHub Actions depois do envio do repositório.

Antes de instalar a atualização, faça um backup pelo aplicativo atual. O identificador continua `com.sosfinanca.app` e o banco continua `sos_financa.db`.
