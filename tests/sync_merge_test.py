import sqlite3, tempfile
from pathlib import Path

checks=0
def ok(cond,msg):
    global checks
    if not cond: raise AssertionError(msg)
    checks+=1; print(f'OK {checks:02d} - {msg}')

SCHEMA='''
CREATE TABLE transactions(id TEXT PRIMARY KEY, kind TEXT, amount_cents INTEGER, description TEXT, date TEXT, category_id TEXT, account_id TEXT, payment_method TEXT, notes TEXT, archived INTEGER, created_at TEXT, updated_at TEXT);
CREATE TABLE cards(id TEXT PRIMARY KEY, name TEXT, bank TEXT, brand TEXT, last4 TEXT, limit_cents INTEGER, close_day INTEGER, due_day INTEGER, account_id TEXT, color TEXT, icon TEXT, sort_order INTEGER, archived INTEGER, created_at TEXT, updated_at TEXT);
CREATE TABLE budgets(id TEXT PRIMARY KEY, category_id TEXT, month_key TEXT, limit_cents INTEGER, archived INTEGER, created_at TEXT, updated_at TEXT);
CREATE UNIQUE INDEX idx_budget ON budgets(category_id,month_key) WHERE archived=0;
CREATE TABLE sync_tombstones(entity_type TEXT NOT NULL, entity_id TEXT NOT NULL, deleted_at TEXT NOT NULL, PRIMARY KEY(entity_type,entity_id));
'''

TABLE_FOR_ENTITY={
  'transaction':'transactions', 'card':'cards', 'budget':'budgets'
}

def apply_tombstones(c):
    for entity_type, entity_id in c.execute('SELECT entity_type,entity_id FROM sync_tombstones').fetchall():
        table=TABLE_FOR_ENTITY.get(entity_type)
        if table:
            c.execute(f'DELETE FROM {table} WHERE id=?',(entity_id,))

def merge(local,remote):
    c=sqlite3.connect(local)
    c.execute('ATTACH DATABASE ? AS remote',(str(remote),))
    c.execute('PRAGMA foreign_keys=OFF')
    c.execute('BEGIN IMMEDIATE')
    merges=[
      ('transactions','id,kind,amount_cents,description,date,category_id,account_id,payment_method,notes,archived,created_at,updated_at','kind=excluded.kind,amount_cents=excluded.amount_cents,description=excluded.description,date=excluded.date,category_id=excluded.category_id,account_id=excluded.account_id,payment_method=excluded.payment_method,notes=excluded.notes,archived=excluded.archived,created_at=excluded.created_at,updated_at=excluded.updated_at'),
      ('cards','id,name,bank,brand,last4,limit_cents,close_day,due_day,account_id,color,icon,sort_order,archived,created_at,updated_at','name=excluded.name,bank=excluded.bank,brand=excluded.brand,last4=excluded.last4,limit_cents=excluded.limit_cents,close_day=excluded.close_day,due_day=excluded.due_day,account_id=excluded.account_id,color=excluded.color,icon=excluded.icon,sort_order=excluded.sort_order,archived=excluded.archived,created_at=excluded.created_at,updated_at=excluded.updated_at')]
    for table,cols,updates in merges:
        sql=f'INSERT INTO {table} ({cols}) SELECT {cols} FROM remote.{table} WHERE true ON CONFLICT(id) DO UPDATE SET {updates} WHERE excluded.updated_at > {table}.updated_at'
        c.execute(sql)
    c.executescript('''
      INSERT INTO sync_tombstones(entity_type,entity_id,deleted_at)
      SELECT entity_type,entity_id,deleted_at FROM remote.sync_tombstones WHERE true
      ON CONFLICT(entity_type,entity_id) DO UPDATE SET deleted_at=excluded.deleted_at
      WHERE excluded.deleted_at>sync_tombstones.deleted_at;

      DELETE FROM budgets WHERE archived=0 AND EXISTS (SELECT 1 FROM remote.budgets r WHERE r.archived=0 AND r.category_id=budgets.category_id AND r.month_key=budgets.month_key AND r.id<>budgets.id AND r.updated_at>budgets.updated_at);
      INSERT INTO budgets(id,category_id,month_key,limit_cents,archived,created_at,updated_at)
      SELECT r.id,r.category_id,r.month_key,r.limit_cents,r.archived,r.created_at,r.updated_at FROM remote.budgets r
      WHERE r.archived=1 OR NOT EXISTS (SELECT 1 FROM budgets l WHERE l.archived=0 AND l.category_id=r.category_id AND l.month_key=r.month_key AND l.id<>r.id AND l.updated_at>=r.updated_at)
      ON CONFLICT(id) DO UPDATE SET category_id=excluded.category_id,month_key=excluded.month_key,limit_cents=excluded.limit_cents,archived=excluded.archived,created_at=excluded.created_at,updated_at=excluded.updated_at
      WHERE excluded.updated_at>budgets.updated_at;
    ''')
    apply_tombstones(c)
    c.commit(); c.execute('DETACH DATABASE remote'); c.close()

with tempfile.TemporaryDirectory() as tmp:
    a=Path(tmp)/'pc.db'; b=Path(tmp)/'cel.db'
    for path in (a,b):
        c=sqlite3.connect(path); c.executescript(SCHEMA); c.close()
    pc=sqlite3.connect(a); cel=sqlite3.connect(b)
    base=('expense',1000,'Compra antiga','2026-09-01',None,None,'PIX','',0,'2026-09-01T10:00:00')
    pc.execute('INSERT INTO transactions VALUES(?,?,?,?,?,?,?,?,?,?,?,?)',('same',*base,'2026-09-08T10:00:00'))
    cel.execute('INSERT INTO transactions VALUES(?,?,?,?,?,?,?,?,?,?,?,?)',('same','expense',1500,'Compra editada no celular','2026-09-01',None,None,'PIX','',0,'2026-09-01T10:00:00','2026-09-08T11:00:00'))
    pc.execute('INSERT INTO transactions VALUES(?,?,?,?,?,?,?,?,?,?,?,?)',('pc-only','expense',200,'PC only','2026-09-02',None,None,'PIX','',0,'2026-09-02T10:00:00','2026-09-08T12:00:00'))
    cel.execute('INSERT INTO transactions VALUES(?,?,?,?,?,?,?,?,?,?,?,?)',('mobile-only','expense',300,'Mobile only','2026-09-03',None,None,'PIX','',0,'2026-09-03T10:00:00','2026-09-08T12:10:00'))
    pc.execute('INSERT INTO transactions VALUES(?,?,?,?,?,?,?,?,?,?,?,?)',('deleted','expense',999,'Excluir','2026-09-04',None,None,'PIX','',0,'2026-09-04T10:00:00','2026-09-08T09:00:00'))
    cel.execute('INSERT INTO transactions VALUES(?,?,?,?,?,?,?,?,?,?,?,?)',('deleted','expense',999,'Excluir','2026-09-04',None,None,'PIX','',1,'2026-09-04T10:00:00','2026-09-08T13:00:00'))
    pc.execute('INSERT INTO transactions VALUES(?,?,?,?,?,?,?,?,?,?,?,?)',('gone','expense',777,'Nao ressuscitar','2026-09-05',None,None,'PIX','',0,'2026-09-05T10:00:00','2026-09-08T16:00:00'))
    cel.execute('INSERT INTO sync_tombstones VALUES(?,?,?)',('transaction','gone','2026-09-08T17:00:00'))
    pc.execute('INSERT INTO cards VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)',('card1','Nubank','Nubank','','',500000,19,26,None,'#176b52','card',0,0,'2026-09-01','2026-09-08T10:00:00'))
    cel.execute('INSERT INTO cards VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)',('card1','Nubank Roxo','Nubank','','',500000,19,26,None,'#663399','wallet',2,0,'2026-09-01','2026-09-08T14:00:00'))
    pc.execute('INSERT INTO budgets VALUES(?,?,?,?,?,?,?)',('budget-pc','cat1','2026-09',50000,0,'2026-09-01','2026-09-08T10:00:00'))
    cel.execute('INSERT INTO budgets VALUES(?,?,?,?,?,?,?)',('budget-mobile','cat1','2026-09',65000,0,'2026-09-01','2026-09-08T15:00:00'))
    pc.commit(); cel.commit(); pc.close(); cel.close()

    merge(a,b)
    c=sqlite3.connect(a)
    rows={r[0]:r for r in c.execute('SELECT id,amount_cents,description,archived FROM transactions')}
    ok(rows['same'][1]==1500 and rows['same'][2]=='Compra editada no celular','edição mais recente do mesmo UUID vence')
    ok('pc-only' in rows and 'mobile-only' in rows,'registros exclusivos dos dois aparelhos são preservados')
    ok(rows['deleted'][3]==1,'estado da lixeira mais recente é sincronizado')
    ok('gone' not in rows,'exclusão permanente não ressuscita a partir do outro aparelho')
    tomb=c.execute('SELECT deleted_at FROM sync_tombstones WHERE entity_type="transaction" AND entity_id="gone"').fetchone()
    ok(tomb==('2026-09-08T17:00:00',),'tombstone de exclusão permanente é preservado na mesclagem')
    card=c.execute('SELECT name,color,icon,sort_order FROM cards WHERE id="card1"').fetchone()
    ok(card==('Nubank Roxo','#663399','wallet',2),'personalização do cartão sincroniza pelo updated_at')
    budgets=c.execute('SELECT id,limit_cents FROM budgets WHERE archived=0 AND category_id="cat1" AND month_key="2026-09"').fetchall()
    ok(budgets==[('budget-mobile',65000)],'orçamento offline duplicado é resolvido pela versão mais recente')
    c.close()

    # Faz uma segunda ida e volta para garantir que o aparelho remoto também não ressuscite o registro apagado.
    merge(b,a)
    c=sqlite3.connect(b)
    ok(c.execute('SELECT 1 FROM transactions WHERE id="gone"').fetchone() is None,'exclusão permanente propaga para o segundo aparelho na volta')
    c.close()

print(f'\nTodos os {checks} testes de mesclagem bidirecional passaram.')
