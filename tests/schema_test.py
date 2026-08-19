from pathlib import Path
import re
import sqlite3

src = Path(__file__).resolve().parents[1] / 'src-tauri' / 'src' / 'db.rs'
text = src.read_text(encoding='utf-8')
match = re.search(r'conn\.execute_batch\(\s*r#"(.*?)"#\s*,?\s*\)', text, re.S)
if not match:
    raise SystemExit('FALHOU - não foi possível localizar o esquema SQLite em db.rs')

schema = match.group(1)
conn = sqlite3.connect(':memory:')
conn.execute('PRAGMA foreign_keys=ON')
conn.executescript(schema)

tables = {row[0] for row in conn.execute("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")}
expected = {
    'profile', 'accounts', 'categories', 'transactions', 'commitments', 'commitment_payments',
    'cards', 'card_purchases', 'card_payments', 'debts', 'debt_payments', 'budgets', 'transfers', 'goals'
}
assert expected <= tables, f'tabelas ausentes: {sorted(expected - tables)}'
print(f'OK 01 - esquema cria as {len(expected)} tabelas principais')

columns = {row[1] for row in conn.execute('PRAGMA table_info(debt_payments)')}
assert 'installment_month' in columns
print('OK 02 - pagamento de dívida possui mês de referência da parcela')

indexes = {row[1] for row in conn.execute("PRAGMA index_list('budgets')")}
assert 'idx_budgets_unique_active' in indexes
print('OK 03 - orçamento possui proteção contra limite duplicado por categoria/mês')

conn.execute("INSERT INTO categories (id,name,kind,parent_id,icon,archived,created_at,updated_at) VALUES ('cat','Feira','expense',NULL,'cart',0,'x','x')")
conn.execute("INSERT INTO budgets (id,category_id,month_key,limit_cents,archived,created_at,updated_at) VALUES ('b1','cat','2026-08',10000,0,'x','x')")
try:
    conn.execute("INSERT INTO budgets (id,category_id,month_key,limit_cents,archived,created_at,updated_at) VALUES ('b2','cat','2026-08',20000,0,'x','x')")
    raise AssertionError('índice único de orçamento não bloqueou duplicata')
except sqlite3.IntegrityError:
    pass
print('OK 04 - SQLite bloqueia dois orçamentos ativos para a mesma categoria no mesmo mês')

conn.execute("INSERT INTO accounts (id,name,institution,account_type,opening_balance_cents,archived,created_at,updated_at) VALUES ('acc','Conta','','Conta corrente',0,0,'x','x')")
try:
    conn.execute("INSERT INTO transfers (id,from_account_id,to_account_id,amount_cents,date,notes,archived,created_at,updated_at) VALUES ('t','acc','missing',100,'2026-08-18','',0,'x','x')")
    raise AssertionError('FK não protegeu referência de conta inexistente')
except sqlite3.IntegrityError:
    pass
print('OK 05 - chaves estrangeiras protegem vínculos financeiros inválidos')

assert 'ALTER TABLE debt_payments ADD COLUMN installment_month TEXT' in text
print('OK 06 - existe migração para mês de referência em instalações antigas')

commitment_columns = {row[1] for row in conn.execute('PRAGMA table_info(commitment_payments)')}
assert 'expected_amount_cents' in commitment_columns
print('OK 07 - pagamentos recorrentes guardam o valor esperado do mês para preservar histórico')

alter_expected = 'ALTER TABLE commitment_payments ADD COLUMN expected_amount_cents INTEGER NOT NULL DEFAULT 0'
update_expected = 'UPDATE commitment_payments SET expected_amount_cents = COALESCE((SELECT amount_cents FROM commitments WHERE commitments.id = commitment_payments.commitment_id), amount_cents) WHERE expected_amount_cents = 0'
assert alter_expected in text
assert update_expected in text
print('OK 08 - existe migração segura do valor histórico para bancos V1/V2 anteriores')

legacy = sqlite3.connect(':memory:')
legacy.execute('CREATE TABLE commitments (id TEXT PRIMARY KEY, amount_cents INTEGER NOT NULL)')
legacy.execute('CREATE TABLE commitment_payments (id TEXT PRIMARY KEY, commitment_id TEXT NOT NULL, month_key TEXT NOT NULL, amount_cents INTEGER NOT NULL)')
legacy.execute("INSERT INTO commitments VALUES ('internet', 10990)")
legacy.execute("INSERT INTO commitment_payments VALUES ('p1','internet','2026-08',9990)")
legacy.execute(alter_expected)
legacy.execute(update_expected)
snapshot = legacy.execute("SELECT expected_amount_cents FROM commitment_payments WHERE id='p1'").fetchone()[0]
assert snapshot == 10990
print('OK 09 - migração de banco antigo preenche o valor esperado usando a recorrência existente')

print('\nTodos os 9 testes estruturais do SQLite passaram.')
