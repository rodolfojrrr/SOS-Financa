# SOS Finança 3.1.1 — correção do cabeçalho de sincronização

A versão 3.1.1 corrige uma falha do protocolo PC → Android em que a conexão podia ser encerrada após a autenticação e antes de o Android receber a linha `DATA`.

Mudanças:

- Snapshot do SQLite carregado em memória antes de iniciar o servidor.
- O arquivo temporário do snapshot é removido antes da sessão, evitando dependência do arquivo durante a transferência.
- Falhas do servidor depois da autenticação retornam `ERR <mensagem>` ao Android.
- Flush explícito do cabeçalho antes do payload criptografado.
- Timeouts maiores e `TCP_NODELAY`.
- Mensagens de diagnóstico mais específicas no Android.

É necessário atualizar Windows e Android para 3.1.1 antes de testar novamente.
