# Sincronização HTTP local — V3.2

A sincronização foi reescrita do zero.

## Fluxo

1. O Windows cria um snapshot consistente do SQLite.
2. O Windows abre temporariamente um servidor HTTP local na porta 45454.
3. O Android testa `GET /ping`.
4. O Android baixa `GET /sos-financa.db`.
5. O Android confere o protocolo, o limite de tamanho e o SHA-256.
6. O arquivo recebido passa pela validação SQLite e `PRAGMA integrity_check`.
7. O Android cria um backup do banco atual.
8. Só depois substitui o banco e executa as migrações.

## O que foi removido

- chave temporária;
- chave fixa;
- HMAC;
- ChaCha20Poly1305;
- cabeçalho TCP personalizado;
- leitura manual de socket;
- pareamento por código.

## Segurança

Sem chave, o conteúdo trafega sem criptografia dentro da rede local. Por isso o servidor:
- só abre quando o usuário toca em Compartilhar;
- expira em cerca de 3 minutos;
- encerra após o primeiro download do banco;
- deve ser usado apenas em Wi-Fi privado/confiável.

O SHA-256 e o integrity_check protegem contra arquivo incompleto/corrompido, não contra espionagem da rede.
