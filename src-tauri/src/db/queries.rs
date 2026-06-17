use rusqlite::{params, Connection, OptionalExtension, Result};

#[cfg(test)]
use crate::crypto;

const PASSWORD_PREVIEW: &str = "********";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub alias: String,
    pub account: String,
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
    pub account: String,
    pub tags: String,
    pub password_preview: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PasswordTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub password_enc: String,
    pub password_nonce: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PasswordTemplateSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub password_preview: String,
}

#[derive(Debug, Clone)]
pub struct VaultMeta {
    pub salt_b64: String,
    pub verifier_enc: String,
    pub verifier_nonce: String,
}

#[derive(Debug, Clone)]
pub struct LegacyPlaintextEntry {
    pub id: String,
    pub password: String,
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub fn insert_entry(
    conn: &Connection,
    name: &str,
    url: &str,
    description: &str,
    alias: &str,
    account: &str,
    password_enc: &str,
    password_nonce: &str,
    tags: &str,
) -> Result<Entry> {
    let id = crypto::generate_id();
    insert_entry_with_id(
        conn,
        &id,
        name,
        url,
        description,
        alias,
        account,
        password_enc,
        password_nonce,
        tags,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn insert_entry_with_id(
    conn: &Connection,
    id: &str,
    name: &str,
    url: &str,
    description: &str,
    alias: &str,
    account: &str,
    password_enc: &str,
    password_nonce: &str,
    tags: &str,
) -> Result<Entry> {
    conn.execute(
        "INSERT INTO entries (id, name, url, description, alias, account, password_enc, password_nonce, tags)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            name,
            url,
            description,
            alias,
            account,
            password_enc,
            password_nonce,
            tags
        ],
    )?;
    get_entry(conn, id)
}

pub fn get_entry(conn: &Connection, id: &str) -> Result<Entry> {
    conn.query_row(
        "SELECT id, name, url, description, alias, account, password_enc, password_nonce, tags, created_at, updated_at
         FROM entries WHERE id = ?1",
        params![id],
        map_entry,
    )
}

pub fn all_entries(conn: &Connection) -> Result<Vec<Entry>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, description, alias, account, password_enc, password_nonce, tags, created_at, updated_at
         FROM entries
         ORDER BY created_at ASC, rowid ASC",
    )?;
    let entries = stmt.query_map([], map_entry)?.collect();
    entries
}

fn map_entry(row: &rusqlite::Row<'_>) -> Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        description: row.get(3)?,
        alias: row.get(4)?,
        account: row.get(5)?,
        password_enc: row.get(6)?,
        password_nonce: row.get(7)?,
        tags: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

pub fn entry_exists_by_id_or_name(conn: &Connection, id: &str, name: &str) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM entries WHERE id = ?1 OR name = ?2",
        params![id, name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn get_entry_search_result(conn: &Connection, id: &str) -> Result<EntrySearchResult> {
    conn.query_row(
        "SELECT id, name, url, description, alias, account, tags
         FROM entries WHERE id = ?1",
        params![id],
        map_entry_search_result,
    )
}

pub fn search_entries(conn: &Connection, query: &str) -> Result<Vec<EntrySearchResult>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return list_entries(conn);
    }

    let fts_query = build_fts_query(trimmed);

    let mut stmt = conn.prepare(
        "SELECT e.id, e.name, e.url, e.description, e.alias, e.account, e.tags
         FROM entries e
         JOIN entries_fts fts ON e.rowid = fts.rowid
         WHERE entries_fts MATCH ?1
         ORDER BY rank
         LIMIT 20",
    )?;

    let results = stmt
        .query_map(params![fts_query], map_entry_search_result)?
        .collect::<Result<Vec<_>>>()?;

    Ok(results)
}

pub fn list_entries(conn: &Connection) -> Result<Vec<EntrySearchResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, description, alias, account, tags
         FROM entries
         ORDER BY updated_at DESC, rowid DESC
         LIMIT 20",
    )?;

    let results = stmt
        .query_map([], map_entry_search_result)?
        .collect::<Result<Vec<_>>>()?;

    Ok(results)
}

fn map_entry_search_result(row: &rusqlite::Row<'_>) -> Result<EntrySearchResult> {
    Ok(EntrySearchResult {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        description: row.get(3)?,
        alias: row.get(4)?,
        account: row.get(5)?,
        tags: row.get(6)?,
        password_preview: PASSWORD_PREVIEW.to_string(),
    })
}

pub fn legacy_plaintext_entries(conn: &Connection) -> Result<Vec<LegacyPlaintextEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, password_enc
         FROM entries
         WHERE password_nonce = ''",
    )?;

    let entries = stmt
        .query_map([], |row| {
            Ok(LegacyPlaintextEntry {
                id: row.get(0)?,
                password: row.get(1)?,
            })
        })?
        .collect();
    entries
}

pub fn update_entry_secret(
    conn: &Connection,
    id: &str,
    password_enc: &str,
    password_nonce: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE entries
         SET password_enc = ?1, password_nonce = ?2
         WHERE id = ?3",
        params![password_enc, password_nonce, id],
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn update_entry_metadata(
    conn: &Connection,
    id: &str,
    name: &str,
    url: &str,
    description: &str,
    alias: &str,
    account: &str,
    tags: &str,
) -> Result<EntrySearchResult> {
    let changed = conn.execute(
        "UPDATE entries
         SET name = ?1,
             url = ?2,
             description = ?3,
             alias = ?4,
             account = ?5,
             tags = ?6,
             updated_at = datetime('now')
         WHERE id = ?7",
        params![name, url, description, alias, account, tags, id],
    )?;

    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    get_entry_search_result(conn, id)
}

#[allow(clippy::too_many_arguments)]
pub fn update_entry_metadata_and_secret(
    conn: &Connection,
    id: &str,
    name: &str,
    url: &str,
    description: &str,
    alias: &str,
    account: &str,
    password_enc: &str,
    password_nonce: &str,
    tags: &str,
) -> Result<EntrySearchResult> {
    let changed = conn.execute(
        "UPDATE entries
         SET name = ?1,
             url = ?2,
             description = ?3,
             alias = ?4,
             account = ?5,
             password_enc = ?6,
             password_nonce = ?7,
             tags = ?8,
             updated_at = datetime('now')
         WHERE id = ?9",
        params![
            name,
            url,
            description,
            alias,
            account,
            password_enc,
            password_nonce,
            tags,
            id
        ],
    )?;

    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    get_entry_search_result(conn, id)
}

pub fn delete_entry(conn: &Connection, id: &str) -> Result<()> {
    let changed = conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}

#[cfg(test)]
pub fn insert_password_template(
    conn: &Connection,
    name: &str,
    description: &str,
    password_enc: &str,
    password_nonce: &str,
) -> Result<PasswordTemplate> {
    let id = crypto::generate_id();
    insert_password_template_with_id(conn, &id, name, description, password_enc, password_nonce)
}

pub fn insert_password_template_with_id(
    conn: &Connection,
    id: &str,
    name: &str,
    description: &str,
    password_enc: &str,
    password_nonce: &str,
) -> Result<PasswordTemplate> {
    conn.execute(
        "INSERT INTO password_templates (id, name, description, password_enc, password_nonce)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, description, password_enc, password_nonce],
    )?;
    get_password_template(conn, id)
}

pub fn get_password_template(conn: &Connection, id: &str) -> Result<PasswordTemplate> {
    conn.query_row(
        "SELECT id, name, description, password_enc, password_nonce, created_at, updated_at
         FROM password_templates WHERE id = ?1",
        params![id],
        map_password_template,
    )
}

pub fn all_password_templates(conn: &Connection) -> Result<Vec<PasswordTemplate>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, password_enc, password_nonce, created_at, updated_at
         FROM password_templates
         ORDER BY created_at ASC, rowid ASC",
    )?;
    let templates = stmt.query_map([], map_password_template)?.collect();
    templates
}

fn map_password_template(row: &rusqlite::Row<'_>) -> Result<PasswordTemplate> {
    Ok(PasswordTemplate {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        password_enc: row.get(3)?,
        password_nonce: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

pub fn password_template_exists_by_id_or_name(
    conn: &Connection,
    id: &str,
    name: &str,
) -> Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT count(*) FROM password_templates WHERE id = ?1 OR name = ?2",
        params![id, name],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

pub fn get_password_template_search_result(
    conn: &Connection,
    id: &str,
) -> Result<PasswordTemplateSearchResult> {
    conn.query_row(
        "SELECT id, name, description
         FROM password_templates WHERE id = ?1",
        params![id],
        map_template_search_result,
    )
}

pub fn search_password_templates(
    conn: &Connection,
    query: &str,
) -> Result<Vec<PasswordTemplateSearchResult>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return list_password_templates(conn);
    }

    let fts_query = build_fts_query(trimmed);
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.description
         FROM password_templates t
         JOIN password_templates_fts fts ON t.rowid = fts.rowid
         WHERE password_templates_fts MATCH ?1
         ORDER BY rank
         LIMIT 20",
    )?;

    let results = stmt
        .query_map(params![fts_query], map_template_search_result)?
        .collect();
    results
}

pub fn list_password_templates(conn: &Connection) -> Result<Vec<PasswordTemplateSearchResult>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description
         FROM password_templates
         ORDER BY updated_at DESC, rowid DESC
         LIMIT 20",
    )?;

    let results = stmt.query_map([], map_template_search_result)?.collect();
    results
}

fn map_template_search_result(row: &rusqlite::Row<'_>) -> Result<PasswordTemplateSearchResult> {
    Ok(PasswordTemplateSearchResult {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        password_preview: PASSWORD_PREVIEW.to_string(),
    })
}

pub fn update_password_template_metadata(
    conn: &Connection,
    id: &str,
    name: &str,
    description: &str,
) -> Result<PasswordTemplateSearchResult> {
    let changed = conn.execute(
        "UPDATE password_templates
         SET name = ?1,
             description = ?2,
             updated_at = datetime('now')
         WHERE id = ?3",
        params![name, description, id],
    )?;

    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    get_password_template_search_result(conn, id)
}

pub fn update_password_template_metadata_and_secret(
    conn: &Connection,
    id: &str,
    name: &str,
    description: &str,
    password_enc: &str,
    password_nonce: &str,
) -> Result<PasswordTemplateSearchResult> {
    let changed = conn.execute(
        "UPDATE password_templates
         SET name = ?1,
             description = ?2,
             password_enc = ?3,
             password_nonce = ?4,
             updated_at = datetime('now')
         WHERE id = ?5",
        params![name, description, password_enc, password_nonce, id],
    )?;

    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    get_password_template_search_result(conn, id)
}

pub fn delete_password_template(conn: &Connection, id: &str) -> Result<()> {
    let changed = conn.execute("DELETE FROM password_templates WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}

pub fn get_vault_meta(conn: &Connection) -> Result<Option<VaultMeta>> {
    conn.query_row(
        "SELECT salt_b64, verifier_enc, verifier_nonce FROM vault_meta WHERE id = 1",
        [],
        |row| {
            Ok(VaultMeta {
                salt_b64: row.get(0)?,
                verifier_enc: row.get(1)?,
                verifier_nonce: row.get(2)?,
            })
        },
    )
    .optional()
}

pub fn store_vault_meta(
    conn: &Connection,
    salt_b64: &str,
    verifier_enc: &str,
    verifier_nonce: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO vault_meta (id, salt_b64, verifier_enc, verifier_nonce)
         VALUES (1, ?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET
             salt_b64 = excluded.salt_b64,
             verifier_enc = excluded.verifier_enc,
             verifier_nonce = excluded.verifier_nonce,
             updated_at = datetime('now')",
        params![salt_b64, verifier_enc, verifier_nonce],
    )?;
    Ok(())
}

pub fn get_app_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

pub fn set_app_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value)
         VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = datetime('now')",
        params![key, value],
    )?;
    Ok(())
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
            "octocat",
            "ciphertext-b64",
            "nonce-b64",
            "dev,code",
        )
        .unwrap();

        assert_eq!(entry.name, "GitHub");
        assert_eq!(entry.url, "https://github.com");
        assert_eq!(entry.description, "Developer account");
        assert_eq!(entry.alias, "work login");
        assert_eq!(entry.account, "octocat");
        assert_eq!(entry.password_enc, "ciphertext-b64");
        assert_eq!(entry.password_nonce, "nonce-b64");
        assert_eq!(entry.tags, "dev,code");
        assert!(!entry.id.is_empty());
        assert!(!entry.created_at.is_empty());
        assert!(!entry.updated_at.is_empty());
    }

    #[test]
    fn empty_search_lists_recent_entries() {
        let conn = test_conn();
        insert_entry(
            &conn,
            "First",
            "",
            "",
            "",
            "",
            "cipher-one",
            "nonce-one",
            "",
        )
        .unwrap();
        insert_entry(
            &conn,
            "Second",
            "",
            "",
            "",
            "",
            "cipher-two",
            "nonce-two",
            "",
        )
        .unwrap();

        let results = search_entries(&conn, "").unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "Second");
        assert_eq!(results[1].name, "First");
        assert_eq!(results[0].password_preview, "********");
    }

    #[test]
    fn search_entries_matches_metadata_fields_without_returning_secret_material() {
        let conn = test_conn();
        insert_entry(
            &conn,
            "GitHub",
            "https://github.com",
            "Developer account",
            "work login",
            "octocat",
            "ciphertext-secret-123",
            "nonce-secret-123",
            "dev,code",
        )
        .unwrap();

        let alias_results = search_entries(&conn, "work").unwrap();
        let account_results = search_entries(&conn, "octocat").unwrap();
        let tag_results = search_entries(&conn, "code").unwrap();

        assert_eq!(alias_results.len(), 1);
        assert_eq!(alias_results[0].name, "GitHub");
        assert_eq!(alias_results[0].account, "octocat");
        assert_eq!(alias_results[0].password_preview, "********");
        assert_eq!(account_results.len(), 1);
        assert_eq!(tag_results.len(), 1);
        assert_eq!(tag_results[0].name, "GitHub");
    }

    #[test]
    fn password_template_search_returns_metadata_only() {
        let conn = test_conn();
        let template = insert_password_template(
            &conn,
            "Birthday",
            "yyyymmdd",
            "ciphertext-template",
            "nonce-template",
        )
        .unwrap();

        let results = search_password_templates(&conn, "birth").unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, template.id);
        assert_eq!(results[0].name, "Birthday");
        assert_eq!(results[0].description, "yyyymmdd");
        assert_eq!(results[0].password_preview, "********");
    }

    #[test]
    fn update_password_template_metadata_preserves_secret_material() {
        let conn = test_conn();
        let template = insert_password_template(
            &conn,
            "Birthday",
            "yyyymmdd",
            "template-ciphertext",
            "template-nonce",
        )
        .unwrap();

        let updated =
            update_password_template_metadata(&conn, &template.id, "PIN", "last four digits")
                .unwrap();

        assert_eq!(updated.id, template.id);
        assert_eq!(updated.name, "PIN");
        assert_eq!(updated.description, "last four digits");
        assert_eq!(updated.password_preview, "********");

        let persisted = get_password_template(&conn, &template.id).unwrap();
        assert_eq!(persisted.password_enc, "template-ciphertext");
        assert_eq!(persisted.password_nonce, "template-nonce");
    }

    #[test]
    fn update_password_template_metadata_and_secret_only_changes_template() {
        let conn = test_conn();
        let entry = insert_entry(
            &conn,
            "GitHub",
            "",
            "",
            "",
            "octocat",
            "entry-ciphertext",
            "entry-nonce",
            "dev",
        )
        .unwrap();
        let template = insert_password_template(
            &conn,
            "Birthday",
            "yyyymmdd",
            "old-template-ciphertext",
            "old-template-nonce",
        )
        .unwrap();

        let updated = update_password_template_metadata_and_secret(
            &conn,
            &template.id,
            "PIN",
            "last four digits",
            "new-template-ciphertext",
            "new-template-nonce",
        )
        .unwrap();

        assert_eq!(updated.name, "PIN");
        let persisted_template = get_password_template(&conn, &template.id).unwrap();
        assert_eq!(persisted_template.password_enc, "new-template-ciphertext");
        assert_eq!(persisted_template.password_nonce, "new-template-nonce");

        let persisted_entry = get_entry(&conn, &entry.id).unwrap();
        assert_eq!(persisted_entry.password_enc, "entry-ciphertext");
        assert_eq!(persisted_entry.password_nonce, "entry-nonce");
    }

    #[test]
    fn delete_password_template_does_not_delete_existing_entries() {
        let conn = test_conn();
        let entry = insert_entry(
            &conn,
            "GitHub",
            "",
            "",
            "",
            "octocat",
            "entry-ciphertext",
            "entry-nonce",
            "dev",
        )
        .unwrap();
        let template = insert_password_template(
            &conn,
            "Birthday",
            "yyyymmdd",
            "template-ciphertext",
            "template-nonce",
        )
        .unwrap();

        delete_password_template(&conn, &template.id).unwrap();

        assert!(get_password_template(&conn, &template.id).is_err());
        let results = search_entries(&conn, "GitHub").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, entry.id);
    }

    #[test]
    fn app_settings_round_trip_values() {
        let conn = test_conn();

        assert_eq!(get_app_setting(&conn, "hotkey").unwrap(), None);

        set_app_setting(&conn, "hotkey", "Cmd+Option+P").unwrap();
        set_app_setting(&conn, "auto_start_enabled", "true").unwrap();

        assert_eq!(
            get_app_setting(&conn, "hotkey").unwrap(),
            Some("Cmd+Option+P".to_string())
        );
        assert_eq!(
            get_app_setting(&conn, "auto_start_enabled").unwrap(),
            Some("true".to_string())
        );
    }

    #[test]
    fn update_entry_metadata_changes_search_fields_without_secret_material() {
        let conn = test_conn();
        let entry = insert_entry(
            &conn,
            "GitHub",
            "https://github.com",
            "Developer account",
            "work login",
            "octocat",
            "ciphertext-secret-123",
            "nonce-secret-123",
            "dev,code",
        )
        .unwrap();

        let updated = update_entry_metadata(
            &conn,
            &entry.id,
            "GitLab",
            "https://gitlab.com",
            "Source control",
            "team login",
            "engineer@example.com",
            "work,source",
        )
        .unwrap();

        assert_eq!(updated.id, entry.id);
        assert_eq!(updated.name, "GitLab");
        assert_eq!(updated.url, "https://gitlab.com");
        assert_eq!(updated.account, "engineer@example.com");
        assert_eq!(updated.password_preview, "********");

        let persisted = get_entry(&conn, &entry.id).unwrap();
        assert_eq!(persisted.password_enc, "ciphertext-secret-123");
        assert_eq!(persisted.password_nonce, "nonce-secret-123");
    }

    #[test]
    fn update_entry_metadata_and_secret_replaces_secret_material() {
        let conn = test_conn();
        let entry = insert_entry(
            &conn,
            "GitHub",
            "",
            "",
            "",
            "octocat",
            "old-ciphertext",
            "old-nonce",
            "",
        )
        .unwrap();

        let updated = update_entry_metadata_and_secret(
            &conn,
            &entry.id,
            "GitHub",
            "",
            "",
            "",
            "octocat",
            "new-ciphertext",
            "new-nonce",
            "dev",
        )
        .unwrap();

        assert_eq!(updated.password_preview, "********");
        let persisted = get_entry(&conn, &entry.id).unwrap();
        assert_eq!(persisted.password_enc, "new-ciphertext");
        assert_eq!(persisted.password_nonce, "new-nonce");
        assert_eq!(persisted.tags, "dev");
    }

    #[test]
    fn delete_entry_removes_entry_from_search_results() {
        let conn = test_conn();
        let entry = insert_entry(
            &conn,
            "Disposable",
            "",
            "",
            "",
            "",
            "ciphertext",
            "nonce",
            "",
        )
        .unwrap();

        delete_entry(&conn, &entry.id).unwrap();

        let results = search_entries(&conn, "Disposable").unwrap();
        assert!(results.is_empty());
        assert!(get_entry(&conn, &entry.id).is_err());
    }
}
