use rusqlite::{Connection, Result};

pub fn initialize(db_path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    initialize_connection(conn)
}

pub fn initialize_connection(conn: Connection) -> Result<Connection> {
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id             TEXT PRIMARY KEY,
            name           TEXT NOT NULL,
            url            TEXT NOT NULL DEFAULT '',
            description    TEXT NOT NULL DEFAULT '',
            alias          TEXT NOT NULL DEFAULT '',
            account        TEXT NOT NULL DEFAULT '',
            password_enc   TEXT NOT NULL,
            password_nonce TEXT NOT NULL DEFAULT '',
            tags           TEXT NOT NULL DEFAULT '',
            created_at     TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS vault_meta (
            id             INTEGER PRIMARY KEY CHECK (id = 1),
            salt_b64       TEXT NOT NULL,
            verifier_enc   TEXT NOT NULL,
            verifier_nonce TEXT NOT NULL,
            created_at     TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS password_templates (
            id             TEXT PRIMARY KEY,
            name           TEXT NOT NULL,
            description    TEXT NOT NULL DEFAULT '',
            password_enc   TEXT NOT NULL,
            password_nonce TEXT NOT NULL DEFAULT '',
            created_at     TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key            TEXT PRIMARY KEY,
            value          TEXT NOT NULL,
            updated_at     TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    add_column_if_missing(&conn, "entries", "account", "TEXT NOT NULL DEFAULT ''")?;

    rebuild_entries_fts_if_needed(&conn)?;
    rebuild_templates_fts_if_needed(&conn)?;
    create_entries_triggers(&conn)?;
    create_template_triggers(&conn)?;

    Ok(conn)
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<()> {
    if !column_exists(conn, table, column)? {
        conn.execute_batch(&format!(
            "ALTER TABLE {} ADD COLUMN {} {};",
            table, column, definition
        ))?;
    }
    Ok(())
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [table],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn rebuild_entries_fts_if_needed(conn: &Connection) -> Result<()> {
    let needs_rebuild =
        !table_exists(conn, "entries_fts")? || !column_exists(conn, "entries_fts", "account")?;
    if !needs_rebuild {
        return Ok(());
    }

    conn.execute_batch(
        "DROP TRIGGER IF EXISTS entries_ai;
        DROP TRIGGER IF EXISTS entries_ad;
        DROP TRIGGER IF EXISTS entries_au;
        DROP TABLE IF EXISTS entries_fts;

        CREATE VIRTUAL TABLE entries_fts USING fts5(
            name, url, description, alias, account, tags,
            content=entries,
            content_rowid=rowid
        );

        INSERT INTO entries_fts(rowid, name, url, description, alias, account, tags)
        SELECT rowid, name, url, description, alias, account, tags FROM entries;",
    )
}

fn rebuild_templates_fts_if_needed(conn: &Connection) -> Result<()> {
    if table_exists(conn, "password_templates_fts")? {
        return Ok(());
    }

    conn.execute_batch(
        "DROP TRIGGER IF EXISTS password_templates_ai;
        DROP TRIGGER IF EXISTS password_templates_ad;
        DROP TRIGGER IF EXISTS password_templates_au;

        CREATE VIRTUAL TABLE password_templates_fts USING fts5(
            name, description,
            content=password_templates,
            content_rowid=rowid
        );

        INSERT INTO password_templates_fts(rowid, name, description)
        SELECT rowid, name, description FROM password_templates;",
    )
}

fn create_entries_triggers(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(rowid, name, url, description, alias, account, tags)
            VALUES (new.rowid, new.name, new.url, new.description, new.alias, new.account, new.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, name, url, description, alias, account, tags)
            VALUES ('delete', old.rowid, old.name, old.url, old.description, old.alias, old.account, old.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, name, url, description, alias, account, tags)
            VALUES ('delete', old.rowid, old.name, old.url, old.description, old.alias, old.account, old.tags);
            INSERT INTO entries_fts(rowid, name, url, description, alias, account, tags)
            VALUES (new.rowid, new.name, new.url, new.description, new.alias, new.account, new.tags);
        END;",
    )
}

fn create_template_triggers(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS password_templates_ai AFTER INSERT ON password_templates BEGIN
            INSERT INTO password_templates_fts(rowid, name, description)
            VALUES (new.rowid, new.name, new.description);
        END;

        CREATE TRIGGER IF NOT EXISTS password_templates_ad AFTER DELETE ON password_templates BEGIN
            INSERT INTO password_templates_fts(password_templates_fts, rowid, name, description)
            VALUES ('delete', old.rowid, old.name, old.description);
        END;

        CREATE TRIGGER IF NOT EXISTS password_templates_au AFTER UPDATE ON password_templates BEGIN
            INSERT INTO password_templates_fts(password_templates_fts, rowid, name, description)
            VALUES ('delete', old.rowid, old.name, old.description);
            INSERT INTO password_templates_fts(rowid, name, description)
            VALUES (new.rowid, new.name, new.description);
        END;",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_connection_creates_entries_schema() {
        let conn = initialize_connection(Connection::open_in_memory().unwrap()).unwrap();

        let table_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name IN ('entries', 'entries_fts', 'password_templates', 'password_templates_fts', 'vault_meta', 'app_settings')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let trigger_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'trigger' AND name IN ('entries_ai', 'entries_ad', 'entries_au', 'password_templates_ai', 'password_templates_ad', 'password_templates_au')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let account_columns: i64 = conn
            .query_row(
                "SELECT count(*) FROM pragma_table_info('entries') WHERE name = 'account'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_count, 6);
        assert_eq!(trigger_count, 6);
        assert_eq!(account_columns, 1);
    }
}
