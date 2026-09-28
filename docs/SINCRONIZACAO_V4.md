# Sincronização bidirecional — V4

1. PC abre servidor HTTP local na porta 45454.
2. Celular informa somente o IP.
3. Celular envia snapshot ao PC.
4. PC valida e combina os dois bancos.
5. Mesmo UUID: vence `updated_at` mais recente.
6. Registros exclusivos dos dois aparelhos são preservados.
7. Tombstones de exclusão permanente são aplicados.
8. PC devolve banco combinado.
9. Celular verifica SHA-256, cria backup e aplica.

Sem chave temporária e sem protocolo TCP próprio. Use somente Wi-Fi privado/confiável.
