# Sincronização Wi-Fi — SOS Finança V3.1

## Regra da V3.1

A sincronização é **PC → celular**. Ela foi pensada para o cenário em que você organiza os dados no computador e quer levar a mesma situação financeira para o próprio celular.

Ela não mistura usuários e não usa nuvem.

## Como usar

1. PC e celular precisam estar na mesma rede Wi-Fi privada.
2. No PC, abra `Configurações > Sincronização PC ↔ celular`.
3. Clique em `Enviar dados deste PC`.
4. Se o Firewall do Windows perguntar, permita o SOS Finança somente em **redes privadas**.
5. O PC mostrará:
   - IP;
   - porta;
   - chave temporária.
6. No celular, abra `Configurações > Sincronização PC ↔ celular`.
7. Digite os três dados exatamente como aparecem no PC.
8. Toque em `Receber do PC` e confirme.

## O que acontece com o banco do celular

Antes de substituir qualquer dado, o app cria um backup automático do banco atual do celular. Só depois o banco recebido é verificado e aplicado.

A transferência valida:

- autenticação temporária;
- criptografia ChaCha20-Poly1305;
- integridade SHA-256;
- formato SQLite;
- `PRAGMA integrity_check`;
- tabelas essenciais do SOS Finança.

Se alguma validação falhar, o banco do celular não é substituído.

## Importante

- A chave expira em 10 minutos.
- Use somente Wi-Fi privado/conhecido.
- Nesta versão não há mesclagem bidirecional. O PC é a origem.
- Alterações exclusivas feitas no celular após a última sincronização serão substituídas ao receber novamente do PC. O backup automático permite recuperar o banco anterior se necessário.
