use rusqlite::{Connection, Result};

pub fn initialize(db_path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL;")?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS entries (
            id          TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
            name        TEXT NOT NULL,
            url         TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            alias       TEXT NOT NULL DEFAULT '',
            password_enc TEXT NOT NULL,
            password_nonce TEXT NOT NULL DEFAULT '',
            tags        TEXT NOT NULL DEFAULT '',
            created_at  TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
            name, url, description, alias, tags,
            content=entries,
            content_rowid=rowid
        );

        CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
            INSERT INTO entries_fts(rowid, name, url, description, alias, tags)
            VALUES (new.rowid, new.name, new.url, new.description, new.alias, new.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, name, url, description, alias, tags)
            VALUES ('delete', old.rowid, old.name, old.url, old.description, old.alias, old.tags);
        END;

        CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
            INSERT INTO entries_fts(entries_fts, rowid, name, url, description, alias, tags)
            VALUES ('delete', old.rowid, old.name, old.url, old.description, old.alias, old.tags);
            INSERT INTO entries_fts(rowid, name, url, description, alias, tags)
            VALUES (new.rowid, new.name, new.url, new.description, new.alias, new.tags);
        END;",
    )?;

    Ok(conn)
}
