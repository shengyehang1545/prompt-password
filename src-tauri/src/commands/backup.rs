use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use tauri::State;

use crate::core::vault::{self, entry_aad, template_aad};
use crate::crypto;
use crate::db::queries;

const BACKUP_APP: &str = "prompt-password";
const BACKUP_VERSION: u32 = 1;
const BACKUP_AAD: &str = "prompt-password:backup:v1";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BackupFile {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ImportSummary {
    pub imported_entries: usize,
    pub skipped_entries: usize,
    pub imported_templates: usize,
    pub skipped_templates: usize,
}

#[tauri::command]
pub fn export_vault_backup(
    master_password: String,
    format: String,
    conn: State<'_, Mutex<Connection>>,
) -> Result<BackupFile, String> {
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    export_backup_file(&db, &master_password, &format)
}

#[tauri::command]
pub fn import_vault_backup(
    master_password: String,
    backup_json: String,
    conn: State<'_, Mutex<Connection>>,
) -> Result<ImportSummary, String> {
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    import_backup_file(&db, &master_password, &backup_json)
}

pub fn export_backup_file(
    conn: &Connection,
    master_password: &str,
    format: &str,
) -> Result<BackupFile, String> {
    let vault_key = vault::verify_master_password(conn, master_password)?;
    let payload = build_backup_payload(conn, &vault_key[..])?;
    let timestamp = unix_timestamp();

    match normalized_format(format)?.as_str() {
        "json" => export_encrypted_json(master_password, payload, timestamp),
        "csv" => export_plain_csv(payload, timestamp),
        "txt" => export_plain_txt(payload, timestamp),
        _ => unreachable!("normalized_format returns only supported formats"),
    }
}

pub fn import_backup_file(
    conn: &Connection,
    master_password: &str,
    backup_json: &str,
) -> Result<ImportSummary, String> {
    let vault_key = vault::verify_master_password(conn, master_password)?;
    let envelope: BackupEnvelope = serde_json::from_str(backup_json)
        .map_err(|e| format!("Invalid backup JSON: {}", e))?;
    if envelope.app != BACKUP_APP || envelope.version != BACKUP_VERSION {
        return Err("Unsupported backup file".to_string());
    }

    let salt = crypto::decode_base64(&envelope.kdf.salt_b64)?;
    let backup_key = crypto::derive_key(master_password, &salt)?;
    let payload_json = crypto::decrypt_secret(
        &backup_key[..],
        BACKUP_AAD,
        &envelope.payload_enc,
        &envelope.payload_nonce,
    )
    .map_err(|_| "Failed to decrypt backup. Check the master password.".to_string())?;
    let payload: BackupPayload = serde_json::from_str(&payload_json)
        .map_err(|e| format!("Invalid backup payload: {}", e))?;

    let mut summary = ImportSummary {
        imported_entries: 0,
        skipped_entries: 0,
        imported_templates: 0,
        skipped_templates: 0,
    };

    for entry in payload.entries {
        if queries::entry_exists_by_id_or_name(conn, &entry.id, &entry.name)
            .map_err(|e| format!("Failed to check entry conflicts: {}", e))?
        {
            summary.skipped_entries += 1;
            continue;
        }

        let sealed = crypto::encrypt_secret(&vault_key[..], &entry_aad(&entry.id), &entry.password)?;
        queries::insert_entry_with_id(
            conn,
            &entry.id,
            &entry.name,
            &entry.url,
            &entry.description,
            &entry.alias,
            &entry.account,
            &sealed.ciphertext_b64,
            &sealed.nonce_b64,
            &entry.tags,
        )
        .map_err(|e| format!("Failed to import entry {}: {}", entry.name, e))?;
        summary.imported_entries += 1;
    }

    for template in payload.templates {
        if queries::password_template_exists_by_id_or_name(conn, &template.id, &template.name)
            .map_err(|e| format!("Failed to check template conflicts: {}", e))?
        {
            summary.skipped_templates += 1;
            continue;
        }

        let sealed =
            crypto::encrypt_secret(&vault_key[..], &template_aad(&template.id), &template.password)?;
        queries::insert_password_template_with_id(
            conn,
            &template.id,
            &template.name,
            &template.description,
            &sealed.ciphertext_b64,
            &sealed.nonce_b64,
        )
        .map_err(|e| format!("Failed to import template {}: {}", template.name, e))?;
        summary.imported_templates += 1;
    }

    Ok(summary)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BackupEnvelope {
    app: String,
    version: u32,
    exported_at_unix: u64,
    kdf: BackupKdf,
    payload_enc: String,
    payload_nonce: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BackupKdf {
    algorithm: String,
    salt_b64: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BackupPayload {
    entries: Vec<BackupEntry>,
    templates: Vec<BackupTemplate>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BackupEntry {
    id: String,
    name: String,
    url: String,
    description: String,
    alias: String,
    account: String,
    tags: String,
    password: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct BackupTemplate {
    id: String,
    name: String,
    description: String,
    password: String,
}

fn build_backup_payload(conn: &Connection, vault_key: &[u8]) -> Result<BackupPayload, String> {
    let entries = queries::all_entries(conn)
        .map_err(|e| format!("Failed to read entries for export: {}", e))?
        .into_iter()
        .map(|entry| {
            let password = crypto::decrypt_secret(
                vault_key,
                &entry_aad(&entry.id),
                &entry.password_enc,
                &entry.password_nonce,
            )?;
            Ok(BackupEntry {
                id: entry.id,
                name: entry.name,
                url: entry.url,
                description: entry.description,
                alias: entry.alias,
                account: entry.account,
                tags: entry.tags,
                password,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let templates = queries::all_password_templates(conn)
        .map_err(|e| format!("Failed to read templates for export: {}", e))?
        .into_iter()
        .map(|template| {
            let password = crypto::decrypt_secret(
                vault_key,
                &template_aad(&template.id),
                &template.password_enc,
                &template.password_nonce,
            )?;
            Ok(BackupTemplate {
                id: template.id,
                name: template.name,
                description: template.description,
                password,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(BackupPayload { entries, templates })
}

fn export_encrypted_json(
    master_password: &str,
    payload: BackupPayload,
    timestamp: u64,
) -> Result<BackupFile, String> {
    let salt = crypto::random_salt();
    let backup_key = crypto::derive_key(master_password, &salt)?;
    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| format!("Failed to serialize backup payload: {}", e))?;
    let sealed = crypto::encrypt_secret(&backup_key[..], BACKUP_AAD, &payload_json)?;
    let envelope = BackupEnvelope {
        app: BACKUP_APP.to_string(),
        version: BACKUP_VERSION,
        exported_at_unix: timestamp,
        kdf: BackupKdf {
            algorithm: "argon2id".to_string(),
            salt_b64: crypto::encode_base64(&salt),
        },
        payload_enc: sealed.ciphertext_b64,
        payload_nonce: sealed.nonce_b64,
    };
    let content = serde_json::to_string_pretty(&envelope)
        .map_err(|e| format!("Failed to serialize backup file: {}", e))?;

    Ok(BackupFile {
        filename: format!("prompt-password-backup-{}.json", timestamp),
        content,
    })
}

fn export_plain_csv(payload: BackupPayload, timestamp: u64) -> Result<BackupFile, String> {
    let mut lines = Vec::new();
    lines.push(csv_row(&[
        "type",
        "id",
        "name",
        "url",
        "description",
        "alias",
        "account",
        "tags",
        "password",
    ]));

    for entry in payload.entries {
        lines.push(csv_row(&[
            "entry",
            &entry.id,
            &entry.name,
            &entry.url,
            &entry.description,
            &entry.alias,
            &entry.account,
            &entry.tags,
            &entry.password,
        ]));
    }
    for template in payload.templates {
        lines.push(csv_row(&[
            "template",
            &template.id,
            &template.name,
            "",
            &template.description,
            "",
            "",
            "",
            &template.password,
        ]));
    }

    Ok(BackupFile {
        filename: format!("prompt-password-export-{}.csv", timestamp),
        content: lines.join("\n"),
    })
}

fn export_plain_txt(payload: BackupPayload, timestamp: u64) -> Result<BackupFile, String> {
    let mut content = String::new();
    for entry in payload.entries {
        content.push_str("[entry]\n");
        content.push_str(&format!("id: {}\n", entry.id));
        content.push_str(&format!("name: {}\n", entry.name));
        content.push_str(&format!("url: {}\n", entry.url));
        content.push_str(&format!("description: {}\n", entry.description));
        content.push_str(&format!("alias: {}\n", entry.alias));
        content.push_str(&format!("account: {}\n", entry.account));
        content.push_str(&format!("tags: {}\n", entry.tags));
        content.push_str(&format!("password: {}\n\n", entry.password));
    }
    for template in payload.templates {
        content.push_str("[template]\n");
        content.push_str(&format!("id: {}\n", template.id));
        content.push_str(&format!("name: {}\n", template.name));
        content.push_str(&format!("description: {}\n", template.description));
        content.push_str(&format!("password: {}\n\n", template.password));
    }

    Ok(BackupFile {
        filename: format!("prompt-password-export-{}.txt", timestamp),
        content,
    })
}

fn csv_row(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| csv_cell(value))
        .collect::<Vec<_>>()
        .join(",")
}

fn csv_cell(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn normalized_format(format: &str) -> Result<String, String> {
    let normalized = format.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "json" | "csv" | "txt" => Ok(normalized),
        _ => Err("Unsupported export format. Use json, csv, or txt.".to_string()),
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vault::{self, entry_aad, template_aad, VaultSession};
    use crate::crypto;
    use crate::db::{self, queries};

    fn test_conn() -> Connection {
        db::init::initialize_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn unlock(conn: &Connection, master_password: &str) -> VaultSession {
        let mut session = VaultSession::default();
        vault::unlock_or_initialize(conn, &mut session, master_password).unwrap();
        session
    }

    fn insert_dummy_entry(conn: &Connection, key: &[u8], name: &str, password: &str) -> String {
        let id = crypto::generate_id();
        let sealed = crypto::encrypt_secret(key, &entry_aad(&id), password).unwrap();
        queries::insert_entry_with_id(
            conn,
            &id,
            name,
            "https://example.com",
            "desc",
            "alias",
            "account",
            &sealed.ciphertext_b64,
            &sealed.nonce_b64,
            "tag",
        )
        .unwrap();
        id
    }

    fn insert_dummy_template(conn: &Connection, key: &[u8], name: &str, password: &str) -> String {
        let id = crypto::generate_id();
        let sealed = crypto::encrypt_secret(key, &template_aad(&id), password).unwrap();
        queries::insert_password_template_with_id(
            conn,
            &id,
            name,
            "template desc",
            &sealed.ciphertext_b64,
            &sealed.nonce_b64,
        )
        .unwrap();
        id
    }

    #[test]
    fn export_backup_requires_current_master_password() {
        let conn = test_conn();
        let session = unlock(&conn, "master-password");
        insert_dummy_entry(&conn, session.key().unwrap(), "GitHub", "entry-secret");

        let err = export_backup_file(&conn, "wrong-password", "json").unwrap_err();

        assert_eq!(err, "Master password is incorrect");
    }

    #[test]
    fn encrypted_backup_round_trips_entries_and_templates_without_overwrite() {
        let source = test_conn();
        let source_session = unlock(&source, "master-password");
        let source_entry_id =
            insert_dummy_entry(&source, source_session.key().unwrap(), "GitHub", "entry-secret");
        let source_template_id = insert_dummy_template(
            &source,
            source_session.key().unwrap(),
            "PIN",
            "template-secret",
        );
        let backup = export_backup_file(&source, "master-password", "json").unwrap();

        let target = test_conn();
        let target_session = unlock(&target, "master-password");
        let first_import =
            import_backup_file(&target, "master-password", &backup.content).unwrap();

        assert_eq!(
            first_import,
            ImportSummary {
                imported_entries: 1,
                skipped_entries: 0,
                imported_templates: 1,
                skipped_templates: 0,
            }
        );

        let imported_entry = queries::get_entry(&target, &source_entry_id).unwrap();
        assert_eq!(
            crypto::decrypt_secret(
                target_session.key().unwrap(),
                &entry_aad(&source_entry_id),
                &imported_entry.password_enc,
                &imported_entry.password_nonce,
            )
            .unwrap(),
            "entry-secret"
        );
        let imported_template =
            queries::get_password_template(&target, &source_template_id).unwrap();
        assert_eq!(
            crypto::decrypt_secret(
                target_session.key().unwrap(),
                &template_aad(&source_template_id),
                &imported_template.password_enc,
                &imported_template.password_nonce,
            )
            .unwrap(),
            "template-secret"
        );

        let second_import =
            import_backup_file(&target, "master-password", &backup.content).unwrap();

        assert_eq!(
            second_import,
            ImportSummary {
                imported_entries: 0,
                skipped_entries: 1,
                imported_templates: 0,
                skipped_templates: 1,
            }
        );
    }

    #[test]
    fn csv_export_quotes_passwords_with_commas_quotes_and_newlines() {
        let conn = test_conn();
        let session = unlock(&conn, "master-password");
        insert_dummy_entry(
            &conn,
            session.key().unwrap(),
            "Comma",
            "pa,ss\"word\nnext",
        );

        let export = export_backup_file(&conn, "master-password", "csv").unwrap();

        assert!(export.filename.ends_with(".csv"));
        assert!(export.content.contains("\"pa,ss\"\"word\nnext\""));
    }

    #[test]
    fn txt_export_contains_plaintext_after_master_password_verification() {
        let conn = test_conn();
        let session = unlock(&conn, "master-password");
        insert_dummy_entry(&conn, session.key().unwrap(), "GitHub", "entry-secret");

        let export = export_backup_file(&conn, "master-password", "txt").unwrap();

        assert!(export.filename.ends_with(".txt"));
        assert!(export.content.contains("[entry]"));
        assert!(export.content.contains("password: entry-secret"));
    }
}
