use chrono::Local;
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Map, Value};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

fn now_iso() -> String {
    Local::now().to_rfc3339()
}

fn app_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_dir(app)?.join("sos_financa.db"))
}

fn open(app: &AppHandle) -> Result<Connection, String> {
    let path = database_path(app)?;
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;")
        .map_err(|e| e.to_string())?;
    Ok(conn)
}

pub fn init(app: &AppHandle) -> Result<(), String> {
    let conn = open(app)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS profile (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            name TEXT NOT NULL DEFAULT '',
            usage_mode TEXT NOT NULL DEFAULT 'personal',
            theme TEXT NOT NULL DEFAULT 'system',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            institution TEXT NOT NULL DEFAULT '',
            account_type TEXT NOT NULL DEFAULT 'Conta corrente',
            opening_balance_cents INTEGER NOT NULL DEFAULT 0,
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL CHECK(kind IN ('income', 'expense')),
            parent_id TEXT,
            icon TEXT NOT NULL DEFAULT 'tag',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(parent_id) REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS transactions (
            id TEXT PRIMARY KEY,
            kind TEXT NOT NULL CHECK(kind IN ('income', 'expense')),
            amount_cents INTEGER NOT NULL,
            description TEXT NOT NULL,
            date TEXT NOT NULL,
            category_id TEXT,
            account_id TEXT,
            payment_method TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(category_id) REFERENCES categories(id),
            FOREIGN KEY(account_id) REFERENCES accounts(id)
        );

        CREATE TABLE IF NOT EXISTS commitments (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL CHECK(kind IN ('income', 'expense')),
            amount_cents INTEGER NOT NULL,
            due_day INTEGER NOT NULL DEFAULT 1,
            category_id TEXT,
            account_id TEXT,
            start_month TEXT NOT NULL,
            end_month TEXT,
            active INTEGER NOT NULL DEFAULT 1,
            notes TEXT NOT NULL DEFAULT '',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(category_id) REFERENCES categories(id),
            FOREIGN KEY(account_id) REFERENCES accounts(id)
        );

        CREATE TABLE IF NOT EXISTS commitment_payments (
            id TEXT PRIMARY KEY,
            commitment_id TEXT NOT NULL,
            month_key TEXT NOT NULL,
            amount_cents INTEGER NOT NULL,
            expected_amount_cents INTEGER NOT NULL DEFAULT 0,
            date TEXT NOT NULL,
            account_id TEXT,
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(commitment_id) REFERENCES commitments(id),
            FOREIGN KEY(account_id) REFERENCES accounts(id)
        );

        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            bank TEXT NOT NULL DEFAULT '',
            brand TEXT NOT NULL DEFAULT '',
            last4 TEXT NOT NULL DEFAULT '',
            limit_cents INTEGER NOT NULL DEFAULT 0,
            close_day INTEGER NOT NULL DEFAULT 1,
            due_day INTEGER NOT NULL DEFAULT 10,
            account_id TEXT,
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(account_id) REFERENCES accounts(id)
        );

        CREATE TABLE IF NOT EXISTS card_purchases (
            id TEXT PRIMARY KEY,
            card_id TEXT NOT NULL,
            description TEXT NOT NULL,
            total_cents INTEGER NOT NULL,
            purchase_date TEXT NOT NULL,
            installments INTEGER NOT NULL DEFAULT 1,
            category_id TEXT,
            notes TEXT NOT NULL DEFAULT '',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(card_id) REFERENCES cards(id),
            FOREIGN KEY(category_id) REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS card_payments (
            id TEXT PRIMARY KEY,
            card_id TEXT NOT NULL,
            invoice_month TEXT NOT NULL,
            amount_cents INTEGER NOT NULL,
            date TEXT NOT NULL,
            account_id TEXT,
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(card_id) REFERENCES cards(id),
            FOREIGN KEY(account_id) REFERENCES accounts(id)
        );

        CREATE TABLE IF NOT EXISTS debts (
            id TEXT PRIMARY KEY,
            debt_type TEXT NOT NULL DEFAULT 'other',
            name TEXT NOT NULL,
            creditor TEXT NOT NULL DEFAULT '',
            original_cents INTEGER NOT NULL DEFAULT 0,
            total_contract_cents INTEGER NOT NULL DEFAULT 0,
            balance_cents INTEGER NOT NULL DEFAULT 0,
            installment_cents INTEGER NOT NULL DEFAULT 0,
            installments_total INTEGER NOT NULL DEFAULT 0,
            installments_paid INTEGER NOT NULL DEFAULT 0,
            due_day INTEGER NOT NULL DEFAULT 1,
            interest_rate REAL NOT NULL DEFAULT 0,
            start_date TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            priority INTEGER NOT NULL DEFAULT 3,
            notes TEXT NOT NULL DEFAULT '',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS debt_payments (
            id TEXT PRIMARY KEY,
            debt_id TEXT NOT NULL,
            amount_cents INTEGER NOT NULL,
            principal_cents INTEGER NOT NULL DEFAULT 0,
            interest_cents INTEGER NOT NULL DEFAULT 0,
            date TEXT NOT NULL,
            installment_month TEXT,
            account_id TEXT,
            notes TEXT NOT NULL DEFAULT '',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(debt_id) REFERENCES debts(id),
            FOREIGN KEY(account_id) REFERENCES accounts(id)
        );

        CREATE TABLE IF NOT EXISTS budgets (
            id TEXT PRIMARY KEY,
            category_id TEXT NOT NULL,
            month_key TEXT NOT NULL,
            limit_cents INTEGER NOT NULL,
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(category_id) REFERENCES categories(id)
        );

        CREATE TABLE IF NOT EXISTS transfers (
            id TEXT PRIMARY KEY,
            from_account_id TEXT NOT NULL,
            to_account_id TEXT NOT NULL,
            amount_cents INTEGER NOT NULL,
            date TEXT NOT NULL,
            notes TEXT NOT NULL DEFAULT '',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(from_account_id) REFERENCES accounts(id),
            FOREIGN KEY(to_account_id) REFERENCES accounts(id)
        );

        CREATE TABLE IF NOT EXISTS goals (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            target_cents INTEGER NOT NULL,
            current_cents INTEGER NOT NULL DEFAULT 0,
            due_date TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);
        CREATE INDEX IF NOT EXISTS idx_card_purchases_card ON card_purchases(card_id);
        CREATE INDEX IF NOT EXISTS idx_debt_payments_debt ON debt_payments(debt_id);
        CREATE INDEX IF NOT EXISTS idx_commitment_payments_month ON commitment_payments(month_key);
        CREATE INDEX IF NOT EXISTS idx_budgets_month ON budgets(month_key);
        CREATE UNIQUE INDEX IF NOT EXISTS idx_budgets_unique_active ON budgets(category_id, month_key) WHERE archived=0;
        "#,
    )
    .map_err(|e| e.to_string())?;

    let has_installment_month = {
        let mut stmt = conn.prepare("PRAGMA table_info(debt_payments)").map_err(|e| e.to_string())?;
        let names = stmt.query_map([], |row| row.get::<_, String>(1)).map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?;
        names.iter().any(|name| name == "installment_month")
    };
    if !has_installment_month {
        conn.execute("ALTER TABLE debt_payments ADD COLUMN installment_month TEXT", []).map_err(|e| e.to_string())?;
    }

    let has_expected_amount = {
        let mut stmt = conn.prepare("PRAGMA table_info(commitment_payments)").map_err(|e| e.to_string())?;
        let names = stmt.query_map([], |row| row.get::<_, String>(1)).map_err(|e| e.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>().map_err(|e| e.to_string())?;
        names.iter().any(|name| name == "expected_amount_cents")
    };
    if !has_expected_amount {
        conn.execute("ALTER TABLE commitment_payments ADD COLUMN expected_amount_cents INTEGER NOT NULL DEFAULT 0", []).map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE commitment_payments SET expected_amount_cents = COALESCE((SELECT amount_cents FROM commitments WHERE commitments.id = commitment_payments.commitment_id), amount_cents) WHERE expected_amount_cents = 0",
            [],
        ).map_err(|e| e.to_string())?;
    }

    seed_categories(&conn)?;
    Ok(())
}

fn seed_categories(conn: &Connection) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Ok(());
    }

    let created = now_iso();
    let defaults = [
        ("Salário", "income", "wallet"),
        ("Renda extra", "income", "plus"),
        ("Outras receitas", "income", "arrow-down"),
        ("Moradia", "expense", "home"),
        ("Alimentação", "expense", "basket"),
        ("Feira", "expense", "cart"),
        ("Transporte", "expense", "car"),
        ("Combustível", "expense", "fuel"),
        ("Saúde", "expense", "heart"),
        ("Educação", "expense", "book"),
        ("Lazer", "expense", "sparkles"),
        ("Assinaturas", "expense", "repeat"),
        ("Dívidas", "expense", "receipt"),
        ("Pequenos gastos", "expense", "coins"),
        ("Outros", "expense", "tag"),
    ];

    for (name, kind, icon) in defaults {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO categories (id, name, kind, parent_id, icon, archived, created_at, updated_at) VALUES (?1, ?2, ?3, NULL, ?4, 0, ?5, ?5)",
            params![id, name, kind, icon, created],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn s(value: &Value, key: &str) -> String {
    value.get(key).and_then(Value::as_str).unwrap_or_default().trim().to_string()
}

fn os(value: &Value, key: &str) -> Option<String> {
    let text = s(value, key);
    if text.is_empty() { None } else { Some(text) }
}

fn i(value: &Value, key: &str) -> i64 {
    if let Some(v) = value.get(key).and_then(Value::as_i64) {
        return v;
    }
    value
        .get(key)
        .and_then(Value::as_f64)
        .map(|v| v.round() as i64)
        .unwrap_or(0)
}

fn f(value: &Value, key: &str) -> f64 {
    value.get(key).and_then(Value::as_f64).unwrap_or(0.0)
}

fn b(value: &Value, key: &str, default: bool) -> bool {
    value.get(key).and_then(Value::as_bool).unwrap_or(default)
}

fn id_for(payload: &Value) -> String {
    let id = s(payload, "id");
    if id.is_empty() { Uuid::new_v4().to_string() } else { id }
}

fn created_for(conn: &Connection, table: &str, id: &str) -> String {
    let allowed = [
        "accounts", "categories", "transactions", "commitments", "commitment_payments",
        "cards", "card_purchases", "card_payments", "debts", "debt_payments", "budgets",
        "transfers", "goals",
    ];
    if !allowed.contains(&table) {
        return now_iso();
    }
    let query = format!("SELECT created_at FROM {table} WHERE id = ?1");
    conn.query_row(&query, params![id], |row| row.get::<_, String>(0))
        .optional()
        .ok()
        .flatten()
        .unwrap_or_else(now_iso)
}

pub fn save(app: &AppHandle, entity_type: &str, payload: &Value) -> Result<String, String> {
    let conn = open(app)?;
    let id = id_for(payload);
    let updated = now_iso();

    match entity_type {
        "profile" => {
            let name = s(payload, "name");
            let usage_mode_raw = s(payload, "usageMode");
            let theme_raw = s(payload, "theme");
            let usage_mode = if usage_mode_raw.is_empty() { "personal".to_string() } else { usage_mode_raw };
            let theme = if theme_raw.is_empty() { "system".to_string() } else { theme_raw };
            let existing: Option<String> = conn
                .query_row("SELECT created_at FROM profile WHERE id = 1", [], |row| row.get(0))
                .optional()
                .map_err(|e| e.to_string())?;
            let created = existing.unwrap_or_else(|| updated.clone());
            conn.execute(
                "INSERT INTO profile (id, name, usage_mode, theme, created_at, updated_at) VALUES (1, ?1, ?2, ?3, ?4, ?5) ON CONFLICT(id) DO UPDATE SET name=excluded.name, usage_mode=excluded.usage_mode, theme=excluded.theme, updated_at=excluded.updated_at",
                params![name, usage_mode, theme, created, updated],
            ).map_err(|e| e.to_string())?;
            return Ok("1".to_string());
        }
        "account" => {
            let created = created_for(&conn, "accounts", &id);
            conn.execute(
                "INSERT INTO accounts (id,name,institution,account_type,opening_balance_cents,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,0,?6,?7) ON CONFLICT(id) DO UPDATE SET name=excluded.name,institution=excluded.institution,account_type=excluded.account_type,opening_balance_cents=excluded.opening_balance_cents,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"name"), s(payload,"institution"), s(payload,"accountType"), i(payload,"openingBalanceCents"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "category" => {
            let created = created_for(&conn, "categories", &id);
            conn.execute(
                "INSERT INTO categories (id,name,kind,parent_id,icon,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,0,?6,?7) ON CONFLICT(id) DO UPDATE SET name=excluded.name,kind=excluded.kind,parent_id=excluded.parent_id,icon=excluded.icon,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"name"), s(payload,"kind"), os(payload,"parentId"), s(payload,"icon"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "transaction" => {
            let created = created_for(&conn, "transactions", &id);
            conn.execute(
                "INSERT INTO transactions (id,kind,amount_cents,description,date,category_id,account_id,payment_method,notes,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,0,?10,?11) ON CONFLICT(id) DO UPDATE SET kind=excluded.kind,amount_cents=excluded.amount_cents,description=excluded.description,date=excluded.date,category_id=excluded.category_id,account_id=excluded.account_id,payment_method=excluded.payment_method,notes=excluded.notes,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"kind"), i(payload,"amountCents"), s(payload,"description"), s(payload,"date"), os(payload,"categoryId"), os(payload,"accountId"), s(payload,"paymentMethod"), s(payload,"notes"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "commitment" => {
            let created = created_for(&conn, "commitments", &id);
            conn.execute(
                "INSERT INTO commitments (id,name,kind,amount_cents,due_day,category_id,account_id,start_month,end_month,active,notes,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,0,?12,?13) ON CONFLICT(id) DO UPDATE SET name=excluded.name,kind=excluded.kind,amount_cents=excluded.amount_cents,due_day=excluded.due_day,category_id=excluded.category_id,account_id=excluded.account_id,start_month=excluded.start_month,end_month=excluded.end_month,active=excluded.active,notes=excluded.notes,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"name"), s(payload,"kind"), i(payload,"amountCents"), i(payload,"dueDay"), os(payload,"categoryId"), os(payload,"accountId"), s(payload,"startMonth"), os(payload,"endMonth"), if b(payload,"active",true){1}else{0}, s(payload,"notes"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "commitment_payment" => {
            let created = created_for(&conn, "commitment_payments", &id);
            conn.execute(
                "INSERT INTO commitment_payments (id,commitment_id,month_key,amount_cents,expected_amount_cents,date,account_id,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,0,?8,?9) ON CONFLICT(id) DO UPDATE SET commitment_id=excluded.commitment_id,month_key=excluded.month_key,amount_cents=excluded.amount_cents,expected_amount_cents=excluded.expected_amount_cents,date=excluded.date,account_id=excluded.account_id,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"commitmentId"), s(payload,"monthKey"), i(payload,"amountCents"), i(payload,"expectedAmountCents"), s(payload,"date"), os(payload,"accountId"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "card" => {
            let created = created_for(&conn, "cards", &id);
            conn.execute(
                "INSERT INTO cards (id,name,bank,brand,last4,limit_cents,close_day,due_day,account_id,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,0,?10,?11) ON CONFLICT(id) DO UPDATE SET name=excluded.name,bank=excluded.bank,brand=excluded.brand,last4=excluded.last4,limit_cents=excluded.limit_cents,close_day=excluded.close_day,due_day=excluded.due_day,account_id=excluded.account_id,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"name"), s(payload,"bank"), s(payload,"brand"), s(payload,"last4"), i(payload,"limitCents"), i(payload,"closeDay"), i(payload,"dueDay"), os(payload,"accountId"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "card_purchase" => {
            let created = created_for(&conn, "card_purchases", &id);
            conn.execute(
                "INSERT INTO card_purchases (id,card_id,description,total_cents,purchase_date,installments,category_id,notes,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,0,?9,?10) ON CONFLICT(id) DO UPDATE SET card_id=excluded.card_id,description=excluded.description,total_cents=excluded.total_cents,purchase_date=excluded.purchase_date,installments=excluded.installments,category_id=excluded.category_id,notes=excluded.notes,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"cardId"), s(payload,"description"), i(payload,"totalCents"), s(payload,"purchaseDate"), i(payload,"installments").max(1), os(payload,"categoryId"), s(payload,"notes"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "card_payment" => {
            let created = created_for(&conn, "card_payments", &id);
            conn.execute(
                "INSERT INTO card_payments (id,card_id,invoice_month,amount_cents,date,account_id,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,0,?7,?8) ON CONFLICT(id) DO UPDATE SET card_id=excluded.card_id,invoice_month=excluded.invoice_month,amount_cents=excluded.amount_cents,date=excluded.date,account_id=excluded.account_id,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"cardId"), s(payload,"invoiceMonth"), i(payload,"amountCents"), s(payload,"date"), os(payload,"accountId"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "debt" => {
            let created = created_for(&conn, "debts", &id);
            conn.execute(
                "INSERT INTO debts (id,debt_type,name,creditor,original_cents,total_contract_cents,balance_cents,installment_cents,installments_total,installments_paid,due_day,interest_rate,start_date,status,priority,notes,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,0,?17,?18) ON CONFLICT(id) DO UPDATE SET debt_type=excluded.debt_type,name=excluded.name,creditor=excluded.creditor,original_cents=excluded.original_cents,total_contract_cents=excluded.total_contract_cents,balance_cents=excluded.balance_cents,installment_cents=excluded.installment_cents,installments_total=excluded.installments_total,installments_paid=excluded.installments_paid,due_day=excluded.due_day,interest_rate=excluded.interest_rate,start_date=excluded.start_date,status=excluded.status,priority=excluded.priority,notes=excluded.notes,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"debtType"), s(payload,"name"), s(payload,"creditor"), i(payload,"originalCents"), i(payload,"totalContractCents"), i(payload,"balanceCents"), i(payload,"installmentCents"), i(payload,"installmentsTotal"), i(payload,"installmentsPaid"), i(payload,"dueDay"), f(payload,"interestRate"), s(payload,"startDate"), s(payload,"status"), i(payload,"priority"), s(payload,"notes"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "debt_payment" => {
            let created = created_for(&conn, "debt_payments", &id);
            conn.execute(
                "INSERT INTO debt_payments (id,debt_id,amount_cents,principal_cents,interest_cents,date,installment_month,account_id,notes,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,0,?10,?11) ON CONFLICT(id) DO UPDATE SET debt_id=excluded.debt_id,amount_cents=excluded.amount_cents,principal_cents=excluded.principal_cents,interest_cents=excluded.interest_cents,date=excluded.date,installment_month=excluded.installment_month,account_id=excluded.account_id,notes=excluded.notes,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"debtId"), i(payload,"amountCents"), i(payload,"principalCents"), i(payload,"interestCents"), s(payload,"date"), os(payload,"installmentMonth"), os(payload,"accountId"), s(payload,"notes"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "budget" => {
            let created = created_for(&conn, "budgets", &id);
            conn.execute(
                "INSERT INTO budgets (id,category_id,month_key,limit_cents,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,0,?5,?6) ON CONFLICT(id) DO UPDATE SET category_id=excluded.category_id,month_key=excluded.month_key,limit_cents=excluded.limit_cents,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"categoryId"), s(payload,"monthKey"), i(payload,"limitCents"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "transfer" => {
            let created = created_for(&conn, "transfers", &id);
            conn.execute(
                "INSERT INTO transfers (id,from_account_id,to_account_id,amount_cents,date,notes,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,0,?7,?8) ON CONFLICT(id) DO UPDATE SET from_account_id=excluded.from_account_id,to_account_id=excluded.to_account_id,amount_cents=excluded.amount_cents,date=excluded.date,notes=excluded.notes,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"fromAccountId"), s(payload,"toAccountId"), i(payload,"amountCents"), s(payload,"date"), s(payload,"notes"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        "goal" => {
            let created = created_for(&conn, "goals", &id);
            conn.execute(
                "INSERT INTO goals (id,name,target_cents,current_cents,due_date,status,archived,created_at,updated_at) VALUES (?1,?2,?3,?4,?5,?6,0,?7,?8) ON CONFLICT(id) DO UPDATE SET name=excluded.name,target_cents=excluded.target_cents,current_cents=excluded.current_cents,due_date=excluded.due_date,status=excluded.status,archived=0,updated_at=excluded.updated_at",
                params![id, s(payload,"name"), i(payload,"targetCents"), i(payload,"currentCents"), os(payload,"dueDate"), s(payload,"status"), created, updated],
            ).map_err(|e| e.to_string())?;
        }
        _ => return Err("Tipo de registro não reconhecido".to_string()),
    }

    Ok(id)
}


fn dependency_exists(conn: &Connection, entity_type: &str, id: &str) -> Result<Option<String>, String> {
    let checks: &[(&str, &str)] = match entity_type {
        "account" => &[
            ("SELECT EXISTS(SELECT 1 FROM transactions WHERE archived=0 AND account_id=?1)", "Esta conta possui lançamentos vinculados."),
            ("SELECT EXISTS(SELECT 1 FROM commitments WHERE archived=0 AND account_id=?1)", "Esta conta está vinculada a um valor recorrente."),
            ("SELECT EXISTS(SELECT 1 FROM cards WHERE archived=0 AND account_id=?1)", "Esta conta está vinculada a um cartão."),
            ("SELECT EXISTS(SELECT 1 FROM commitment_payments WHERE archived=0 AND account_id=?1)", "Esta conta possui pagamentos recorrentes no histórico."),
            ("SELECT EXISTS(SELECT 1 FROM card_payments WHERE archived=0 AND account_id=?1)", "Esta conta possui pagamentos de fatura no histórico."),
            ("SELECT EXISTS(SELECT 1 FROM debt_payments WHERE archived=0 AND account_id=?1)", "Esta conta possui pagamentos de dívida no histórico."),
            ("SELECT EXISTS(SELECT 1 FROM transfers WHERE archived=0 AND (from_account_id=?1 OR to_account_id=?1))", "Esta conta possui transferências no histórico."),
        ],
        "category" => &[
            ("SELECT EXISTS(SELECT 1 FROM categories WHERE archived=0 AND parent_id=?1)", "Esta categoria possui subcategorias."),
            ("SELECT EXISTS(SELECT 1 FROM transactions WHERE archived=0 AND category_id=?1)", "Esta categoria possui lançamentos vinculados."),
            ("SELECT EXISTS(SELECT 1 FROM commitments WHERE archived=0 AND category_id=?1)", "Esta categoria possui valores recorrentes vinculados."),
            ("SELECT EXISTS(SELECT 1 FROM card_purchases WHERE archived=0 AND category_id=?1)", "Esta categoria possui compras de cartão vinculadas."),
            ("SELECT EXISTS(SELECT 1 FROM budgets WHERE archived=0 AND category_id=?1)", "Esta categoria possui orçamento vinculado."),
        ],
        "commitment" => &[
            ("SELECT EXISTS(SELECT 1 FROM commitment_payments WHERE archived=0 AND commitment_id=?1)", "Este valor recorrente já possui pagamentos/recebimentos no histórico."),
        ],
        "card" => &[
            ("SELECT EXISTS(SELECT 1 FROM card_purchases WHERE archived=0 AND card_id=?1)", "Este cartão possui compras vinculadas."),
            ("SELECT EXISTS(SELECT 1 FROM card_payments WHERE archived=0 AND card_id=?1)", "Este cartão possui pagamentos de fatura no histórico."),
        ],
        _ => &[],
    };

    for (query, message) in checks {
        let found: i64 = conn.query_row(query, params![id], |row| row.get(0)).map_err(|e| e.to_string())?;
        if found == 1 {
            return Ok(Some((*message).to_string()));
        }
    }

    if entity_type == "debt" {
        let balance: Option<i64> = conn.query_row(
            "SELECT MAX(0, balance_cents - COALESCE((SELECT SUM(CASE WHEN principal_cents > 0 THEN principal_cents ELSE amount_cents END) FROM debt_payments WHERE archived=0 AND debt_id=debts.id), 0)) FROM debts WHERE id=?1",
            params![id],
            |row| row.get(0),
        ).optional().map_err(|e| e.to_string())?;
        if balance.unwrap_or(0) > 0 {
            return Ok(Some("Quite ou ajuste o saldo desta dívida antes de arquivá-la.".to_string()));
        }
    }

    Ok(None)
}

pub fn archive(app: &AppHandle, entity_type: &str, id: &str) -> Result<(), String> {
    let table = match entity_type {
        "account" => "accounts",
        "category" => "categories",
        "transaction" => "transactions",
        "commitment" => "commitments",
        "commitment_payment" => "commitment_payments",
        "card" => "cards",
        "card_purchase" => "card_purchases",
        "card_payment" => "card_payments",
        "debt" => "debts",
        "debt_payment" => "debt_payments",
        "budget" => "budgets",
        "transfer" => "transfers",
        "goal" => "goals",
        _ => return Err("Tipo de registro não reconhecido".to_string()),
    };
    let conn = open(app)?;
    if let Some(message) = dependency_exists(&conn, entity_type, id)? {
        return Err(message);
    }
    let sql = format!("UPDATE {table} SET archived = 1, updated_at = ?1 WHERE id = ?2");
    conn.execute(&sql, params![now_iso(), id]).map_err(|e| e.to_string())?;
    Ok(())
}

fn rows(conn: &Connection, query: &str, mapper: fn(&rusqlite::Row<'_>) -> rusqlite::Result<Value>) -> Result<Vec<Value>, String> {
    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
    let result = stmt
        .query_map([], mapper)
        .map_err(|e| e.to_string())?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())?;
    Ok(result)
}

pub fn state(app: &AppHandle) -> Result<Value, String> {
    let conn = open(app)?;
    let profile = conn
        .query_row(
            "SELECT name, usage_mode, theme, created_at, updated_at FROM profile WHERE id = 1",
            [],
            |row| {
                Ok(json!({
                    "name": row.get::<_, String>(0)?,
                    "usageMode": row.get::<_, String>(1)?,
                    "theme": row.get::<_, String>(2)?,
                    "createdAt": row.get::<_, String>(3)?,
                    "updatedAt": row.get::<_, String>(4)?
                }))
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let accounts = rows(&conn, "SELECT id,name,institution,account_type,opening_balance_cents,created_at,updated_at FROM accounts WHERE archived=0 ORDER BY created_at", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "name":row.get::<_,String>(1)?, "institution":row.get::<_,String>(2)?, "accountType":row.get::<_,String>(3)?, "openingBalanceCents":row.get::<_,i64>(4)?, "createdAt":row.get::<_,String>(5)?, "updatedAt":row.get::<_,String>(6)?
    })))?;

    let categories = rows(&conn, "SELECT id,name,kind,parent_id,icon,created_at,updated_at FROM categories WHERE archived=0 ORDER BY kind,name", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "name":row.get::<_,String>(1)?, "kind":row.get::<_,String>(2)?, "parentId":row.get::<_,Option<String>>(3)?, "icon":row.get::<_,String>(4)?, "createdAt":row.get::<_,String>(5)?, "updatedAt":row.get::<_,String>(6)?
    })))?;

    let transactions = rows(&conn, "SELECT id,kind,amount_cents,description,date,category_id,account_id,payment_method,notes,created_at,updated_at FROM transactions WHERE archived=0 ORDER BY date DESC,created_at DESC", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "kind":row.get::<_,String>(1)?, "amountCents":row.get::<_,i64>(2)?, "description":row.get::<_,String>(3)?, "date":row.get::<_,String>(4)?, "categoryId":row.get::<_,Option<String>>(5)?, "accountId":row.get::<_,Option<String>>(6)?, "paymentMethod":row.get::<_,String>(7)?, "notes":row.get::<_,String>(8)?, "createdAt":row.get::<_,String>(9)?, "updatedAt":row.get::<_,String>(10)?
    })))?;

    let commitments = rows(&conn, "SELECT id,name,kind,amount_cents,due_day,category_id,account_id,start_month,end_month,active,notes,created_at,updated_at FROM commitments WHERE archived=0 ORDER BY due_day,name", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "name":row.get::<_,String>(1)?, "kind":row.get::<_,String>(2)?, "amountCents":row.get::<_,i64>(3)?, "dueDay":row.get::<_,i64>(4)?, "categoryId":row.get::<_,Option<String>>(5)?, "accountId":row.get::<_,Option<String>>(6)?, "startMonth":row.get::<_,String>(7)?, "endMonth":row.get::<_,Option<String>>(8)?, "active":row.get::<_,i64>(9)?==1, "notes":row.get::<_,String>(10)?, "createdAt":row.get::<_,String>(11)?, "updatedAt":row.get::<_,String>(12)?
    })))?;

    let commitment_payments = rows(&conn, "SELECT id,commitment_id,month_key,amount_cents,expected_amount_cents,date,account_id,created_at,updated_at FROM commitment_payments WHERE archived=0 ORDER BY date DESC", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "commitmentId":row.get::<_,String>(1)?, "monthKey":row.get::<_,String>(2)?, "amountCents":row.get::<_,i64>(3)?, "expectedAmountCents":row.get::<_,i64>(4)?, "date":row.get::<_,String>(5)?, "accountId":row.get::<_,Option<String>>(6)?, "createdAt":row.get::<_,String>(7)?, "updatedAt":row.get::<_,String>(8)?
    })))?;

    let cards = rows(&conn, "SELECT id,name,bank,brand,last4,limit_cents,close_day,due_day,account_id,created_at,updated_at FROM cards WHERE archived=0 ORDER BY created_at", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "name":row.get::<_,String>(1)?, "bank":row.get::<_,String>(2)?, "brand":row.get::<_,String>(3)?, "last4":row.get::<_,String>(4)?, "limitCents":row.get::<_,i64>(5)?, "closeDay":row.get::<_,i64>(6)?, "dueDay":row.get::<_,i64>(7)?, "accountId":row.get::<_,Option<String>>(8)?, "createdAt":row.get::<_,String>(9)?, "updatedAt":row.get::<_,String>(10)?
    })))?;

    let card_purchases = rows(&conn, "SELECT id,card_id,description,total_cents,purchase_date,installments,category_id,notes,created_at,updated_at FROM card_purchases WHERE archived=0 ORDER BY purchase_date DESC,created_at DESC", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "cardId":row.get::<_,String>(1)?, "description":row.get::<_,String>(2)?, "totalCents":row.get::<_,i64>(3)?, "purchaseDate":row.get::<_,String>(4)?, "installments":row.get::<_,i64>(5)?, "categoryId":row.get::<_,Option<String>>(6)?, "notes":row.get::<_,String>(7)?, "createdAt":row.get::<_,String>(8)?, "updatedAt":row.get::<_,String>(9)?
    })))?;

    let card_payments = rows(&conn, "SELECT id,card_id,invoice_month,amount_cents,date,account_id,created_at,updated_at FROM card_payments WHERE archived=0 ORDER BY date DESC", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "cardId":row.get::<_,String>(1)?, "invoiceMonth":row.get::<_,String>(2)?, "amountCents":row.get::<_,i64>(3)?, "date":row.get::<_,String>(4)?, "accountId":row.get::<_,Option<String>>(5)?, "createdAt":row.get::<_,String>(6)?, "updatedAt":row.get::<_,String>(7)?
    })))?;

    let debts = rows(&conn, "SELECT id,debt_type,name,creditor,original_cents,total_contract_cents,balance_cents,installment_cents,installments_total,installments_paid,due_day,interest_rate,start_date,status,priority,notes,created_at,updated_at FROM debts WHERE archived=0 ORDER BY priority,name", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "debtType":row.get::<_,String>(1)?, "name":row.get::<_,String>(2)?, "creditor":row.get::<_,String>(3)?, "originalCents":row.get::<_,i64>(4)?, "totalContractCents":row.get::<_,i64>(5)?, "balanceCents":row.get::<_,i64>(6)?, "installmentCents":row.get::<_,i64>(7)?, "installmentsTotal":row.get::<_,i64>(8)?, "installmentsPaid":row.get::<_,i64>(9)?, "dueDay":row.get::<_,i64>(10)?, "interestRate":row.get::<_,f64>(11)?, "startDate":row.get::<_,String>(12)?, "status":row.get::<_,String>(13)?, "priority":row.get::<_,i64>(14)?, "notes":row.get::<_,String>(15)?, "createdAt":row.get::<_,String>(16)?, "updatedAt":row.get::<_,String>(17)?
    })))?;

    let debt_payments = rows(&conn, "SELECT id,debt_id,amount_cents,principal_cents,interest_cents,date,installment_month,account_id,notes,created_at,updated_at FROM debt_payments WHERE archived=0 ORDER BY date DESC", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "debtId":row.get::<_,String>(1)?, "amountCents":row.get::<_,i64>(2)?, "principalCents":row.get::<_,i64>(3)?, "interestCents":row.get::<_,i64>(4)?, "date":row.get::<_,String>(5)?, "installmentMonth":row.get::<_,Option<String>>(6)?, "accountId":row.get::<_,Option<String>>(7)?, "notes":row.get::<_,String>(8)?, "createdAt":row.get::<_,String>(9)?, "updatedAt":row.get::<_,String>(10)?
    })))?;

    let budgets = rows(&conn, "SELECT id,category_id,month_key,limit_cents,created_at,updated_at FROM budgets WHERE archived=0 ORDER BY month_key DESC", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "categoryId":row.get::<_,String>(1)?, "monthKey":row.get::<_,String>(2)?, "limitCents":row.get::<_,i64>(3)?, "createdAt":row.get::<_,String>(4)?, "updatedAt":row.get::<_,String>(5)?
    })))?;

    let transfers = rows(&conn, "SELECT id,from_account_id,to_account_id,amount_cents,date,notes,created_at,updated_at FROM transfers WHERE archived=0 ORDER BY date DESC", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "fromAccountId":row.get::<_,String>(1)?, "toAccountId":row.get::<_,String>(2)?, "amountCents":row.get::<_,i64>(3)?, "date":row.get::<_,String>(4)?, "notes":row.get::<_,String>(5)?, "createdAt":row.get::<_,String>(6)?, "updatedAt":row.get::<_,String>(7)?
    })))?;

    let goals = rows(&conn, "SELECT id,name,target_cents,current_cents,due_date,status,created_at,updated_at FROM goals WHERE archived=0 ORDER BY created_at", |row| Ok(json!({
        "id":row.get::<_,String>(0)?, "name":row.get::<_,String>(1)?, "targetCents":row.get::<_,i64>(2)?, "currentCents":row.get::<_,i64>(3)?, "dueDate":row.get::<_,Option<String>>(4)?, "status":row.get::<_,String>(5)?, "createdAt":row.get::<_,String>(6)?, "updatedAt":row.get::<_,String>(7)?
    })))?;

    Ok(json!({
        "profile": profile,
        "accounts": accounts,
        "categories": categories,
        "transactions": transactions,
        "commitments": commitments,
        "commitmentPayments": commitment_payments,
        "cards": cards,
        "cardPurchases": card_purchases,
        "cardPayments": card_payments,
        "debts": debts,
        "debtPayments": debt_payments,
        "budgets": budgets,
        "transfers": transfers,
        "goals": goals
    }))
}

pub fn create_backup(app: &AppHandle) -> Result<String, String> {
    let source = database_path(app)?;
    {
        let conn = open(app)?;
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").map_err(|e| e.to_string())?;
    }
    let backup_dir = app_dir(app)?.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let name = format!("sos_financa_{}.db", Local::now().format("%Y%m%d_%H%M%S"));
    let target = backup_dir.join(&name);
    fs::copy(&source, &target).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().to_string())
}

pub fn list_backups(app: &AppHandle) -> Result<Vec<Value>, String> {
    let backup_dir = app_dir(app)?.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for entry in fs::read_dir(&backup_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("db") {
            continue;
        }
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        items.push(json!({
            "name": entry.file_name().to_string_lossy().to_string(),
            "path": path.to_string_lossy().to_string(),
            "size": metadata.len()
        }));
    }
    items.sort_by(|a, b| b.get("name").and_then(Value::as_str).cmp(&a.get("name").and_then(Value::as_str)));
    Ok(items)
}

pub fn restore_backup(app: &AppHandle, name: &str) -> Result<(), String> {
    if name.contains('/') || name.contains('\\') || !name.ends_with(".db") {
        return Err("Nome de backup inválido".to_string());
    }
    let backup_dir = app_dir(app)?.join("backups");
    let source = backup_dir.join(name);
    if !source.exists() {
        return Err("Backup não encontrado".to_string());
    }
    let current = database_path(app)?;
    let safety = backup_dir.join(format!("antes_restauracao_{}.db", Local::now().format("%Y%m%d_%H%M%S")));
    if current.exists() {
        {
            let conn = open(app)?;
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").map_err(|e| e.to_string())?;
        }
        fs::copy(&current, safety).map_err(|e| e.to_string())?;
    }
    let wal = PathBuf::from(format!("{}-wal", current.to_string_lossy()));
    let shm = PathBuf::from(format!("{}-shm", current.to_string_lossy()));
    let _ = fs::remove_file(wal);
    let _ = fs::remove_file(shm);
    fs::copy(source, current).map_err(|e| e.to_string())?;
    init(app)?;
    Ok(())
}

pub fn database_info(app: &AppHandle) -> Result<Value, String> {
    let path = database_path(app)?;
    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let conn = open(app)?;
    let tables = [
        "accounts", "categories", "transactions", "commitments", "commitment_payments",
        "cards", "card_purchases", "card_payments", "debts", "debt_payments", "budgets",
        "transfers", "goals",
    ];
    let mut counts = Map::new();
    for table in tables {
        let query = format!("SELECT COUNT(*) FROM {table} WHERE archived=0");
        let count: i64 = conn.query_row(&query, [], |row| row.get(0)).map_err(|e| e.to_string())?;
        counts.insert(table.to_string(), json!(count));
    }
    Ok(json!({
        "path": path.to_string_lossy().to_string(),
        "size": size,
        "counts": counts
    }))
}

pub fn create_sync_snapshot(app: &AppHandle) -> Result<PathBuf, String> {
    let source = database_path(app)?;
    let dir = app.path().app_cache_dir().map_err(|e| e.to_string())?.join("sync");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let target = dir.join(format!("sos_financa_sync_{}.db", Uuid::new_v4().simple()));
    let conn = open(app)?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); BEGIN IMMEDIATE;").map_err(|e| e.to_string())?;
    let copied = fs::copy(&source, &target).map_err(|e| e.to_string());
    let _ = conn.execute_batch("COMMIT;");
    copied?;
    if let Err(err) = validate_database_file(&target) {
        let _ = fs::remove_file(&target);
        return Err(err);
    }
    Ok(target)
}

fn validate_database_file(path: &PathBuf) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() < 16 || &bytes[..16] != b"SQLite format 3\0" {
        return Err("O arquivo recebido não é um banco SQLite válido.".into());
    }
    let conn = Connection::open(path).map_err(|e| format!("Não foi possível abrir o banco recebido: {e}"))?;
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if integrity.to_lowercase() != "ok" {
        return Err(format!("O banco recebido falhou na verificação de integridade: {integrity}"));
    }
    let required = ["profile", "accounts", "transactions", "commitments", "cards", "card_purchases", "debts", "budgets"];
    for table in required {
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            [table],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        if exists != 1 {
            return Err(format!("O banco recebido não parece pertencer ao SOS Finança (tabela {table} ausente)."));
        }
    }
    Ok(())
}

pub fn import_sync_database(app: &AppHandle, bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("Nenhum dado foi recebido.".into());
    }
    let dir = app.path().app_cache_dir().map_err(|e| e.to_string())?.join("sync");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let incoming = dir.join(format!("incoming_{}.db", Uuid::new_v4().simple()));
    fs::write(&incoming, bytes).map_err(|e| e.to_string())?;
    if let Err(err) = validate_database_file(&incoming) {
        let _ = fs::remove_file(&incoming);
        return Err(err);
    }

    let backup = create_backup(app)?;
    let current = database_path(app)?;
    let wal = PathBuf::from(format!("{}-wal", current.to_string_lossy()));
    let shm = PathBuf::from(format!("{}-shm", current.to_string_lossy()));
    let _ = fs::remove_file(&wal);
    let _ = fs::remove_file(&shm);

    if let Err(err) = fs::copy(&incoming, &current) {
        let _ = fs::remove_file(&incoming);
        return Err(format!("Não foi possível aplicar os dados recebidos: {err}"));
    }
    let _ = fs::remove_file(&incoming);

    if let Err(err) = init(app) {
        let _ = fs::copy(&backup, &current);
        let _ = init(app);
        return Err(format!("A atualização do banco recebido falhou e o backup anterior foi restaurado: {err}"));
    }
    Ok(backup)
}
