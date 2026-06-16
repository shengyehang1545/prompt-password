use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub alias: String,
    pub password_enc: String,
    pub password_nonce: String,
    pub tags: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EntrySearchResult {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub alias: String,
    pub tags: String,
    pub password_enc: String,
}

pub fn insert_entry(
    conn: &Connection,
    name: &str,
    url: &str,
    description: &str,
    alias: &str,
    password: &str,
    tags: &str,
) -> Result<Entry> {
    conn.execute(
        "INSERT INTO entries (name, url, description, alias, password_enc, password_nonce, tags)
         VALUES (?1, ?2, ?3, ?4, ?5, '', ?6)",
        params![name, url, description, alias, password, tags],
    )?;
    let id: String = conn
        .query_row("SELECT last_insert_rowid()", [], |row| row.get(0))
        .map(|rowid: i64| {
            conn.query_row(
                "SELECT id FROM entries WHERE rowid = ?1",
                params![rowid],
                |r| r.get(0),
            )
        })??;
    get_entry(conn, &id)
}

pub fn get_entry(conn: &Connection, id: &str) -> Result<Entry> {
    conn.query_row(
        "SELECT id, name, url, description, alias, password_enc, password_nonce, tags, created_at, updated_at
         FROM entries WHERE id = ?1",
        params![id],
        |row| {
            Ok(Entry {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                alias: row.get(4)?,
                password_enc: row.get(5)?,
                password_nonce: row.get(6)?,
                tags: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        },
    )
}

pub fn search_entries(conn: &Connection, query: &str) -> Result<Vec<EntrySearchResult>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return list_entries(conn);
    }

    let fts_query = build_fts_query(trimmed);

    let mut stmt = conn.prepare(
        "SELECT e.id, e.name, e.url, e.description, e.alias, e.tags, e.password_enc
         FROM entries e
         JOIN entries_fts fts ON e.rowid = fts.rowid
         WHERE entries_fts MATCH ?1
         ORDER BY rank
         LIMIT 20",
    )?;

    let results = stmt
        .query_map(params![fts_query], |row| {
            Ok(EntrySearchResult {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                alias: row.get(4)?,
                tags: row.get(5)?,
                password_enc: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(results)
}

pub fn list_entries(conn: &Connection) -> Result<Vec<EntrySearchResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, description, alias, tags, password_enc
         FROM entries
         ORDER BY updated_at DESC, rowid DESC
         LIMIT 20",
    )?;

    let results = stmt
        .query_map([], |row| {
            Ok(EntrySearchResult {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                description: row.get(3)?,
                alias: row.get(4)?,
                tags: row.get(5)?,
                password_enc: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(results)
}

fn build_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .filter_map(|term| {
            let cleaned = term.replace('"', "\"\"");
            if cleaned.is_empty() {
                return None;
            }
            Some(format!("\"{}\"*", cleaned))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        crate::db::init::initialize_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn insert_entry_persists_all_fields() {
        let conn = test_conn();

        let entry = insert_entry(
            &conn,
            "GitHub",
            "https://github.com",
            "Developer account",
            "work login",
            "secret-123",
            "dev,code",
        )
        .unwrap();

        assert_eq!(entry.name, "GitHub");
        assert_eq!(entry.url, "https://github.com");
        assert_eq!(entry.description, "Developer account");
        assert_eq!(entry.alias, "work login");
        assert_eq!(entry.password_enc, "secret-123");
        assert_eq!(entry.password_nonce, "");
        assert_eq!(entry.tags, "dev,code");
        assert!(!entry.id.is_empty());
        assert!(!entry.created_at.is_empty());
        assert!(!entry.updated_at.is_empty());
    }

    #[test]
    fn empty_search_lists_recent_entries() {
        let conn = test_conn();
        insert_entry(&conn, "First", "", "", "", "one", "").unwrap();
        insert_entry(&conn, "Second", "", "", "", "two", "").unwrap();

        let results = search_entries(&conn, "").unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "Second");
        assert_eq!(results[1].name, "First");
    }

    #[test]
    fn search_entries_matches_metadata_fields() {
        let conn = test_conn();
        insert_entry(
            &conn,
            "GitHub",
            "https://github.com",
            "Developer account",
            "work login",
            "secret-123",
            "dev,code",
        )
        .unwrap();

        let alias_results = search_entries(&conn, "work").unwrap();
        let tag_results = search_entries(&conn, "code").unwrap();

        assert_eq!(alias_results.len(), 1);
        assert_eq!(alias_results[0].name, "GitHub");
        assert_eq!(tag_results.len(), 1);
        assert_eq!(tag_results[0].name, "GitHub");
    }
}
