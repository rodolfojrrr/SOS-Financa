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
print('OK 06 - existe migração para instalações criadas antes da revisão')

print('\nTodos os 6 testes estruturais do SQLite passaram.')
