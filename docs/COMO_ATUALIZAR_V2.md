# Como atualizar da V1 para a V2

A V2 mantém o identificador `com.sosfinanca.app` e continua usando o banco local `sos_financa.db`.

## Antes da atualização

1. Abra a V1 instalada.
2. Vá em **Configurações → Backup** e crie um backup.
3. Feche completamente o SOS Finança.

## Atualizar o código no GitHub

Use a mesma pasta local que já está conectada ao repositório `SOS-Financa`.

1. Substitua os arquivos do projeto pelos arquivos da V2, mantendo a pasta oculta `.git` do seu repositório local.
2. Abra o terminal nessa pasta.
3. Execute:

```bash
git add -A
git commit -m "SOS Financa V2.0.0 - usabilidade"
git push
```

O workflow `Build Windows` será iniciado no GitHub.

## Instalar

Quando o workflow terminar com sucesso:

1. Abra a execução do GitHub Actions.
2. Baixe o artifact `SOS-Financa-Windows`.
3. Extraia o ZIP do artifact.
4. Execute o instalador NSIS da V2.
5. Instale por cima da versão atual.

O banco local não deve ser apagado pelo instalador, mas o backup anterior continua obrigatório como precaução.

## Depois de instalar

Valide primeiro:

- salário fixo;
- conta fixa;
- cartão e compras;
- dívida;
- saldo da conta;
- dados antigos da V1.

Se qualquer dado antigo parecer incorreto, não continue cadastrando informações: restaure o backup e revise a atualização.
