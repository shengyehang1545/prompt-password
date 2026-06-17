use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::core::entry_input::{normalize_create_entry, normalize_update_entry};
use crate::core::vault::{entry_aad, VaultSession};
use crate::crypto;
use crate::db::queries::{self, EntrySearchResult};

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_entry(
    name: String,
    url: Option<String>,
    description: Option<String>,
    alias: Option<String>,
    account: Option<String>,
    password: String,
    tags: Option<String>,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<EntrySearchResult, String> {
    let input = normalize_create_entry(name, url, description, alias, account, password, tags)?;

    let id = crypto::generate_id();
    let sealed = {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        crypto::encrypt_secret(vault.key()?, &entry_aad(&id), &input.password)?
    };
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;

    let entry = queries::insert_entry_with_id(
        &db,
        &id,
        &input.name,
        &input.url,
        &input.description,
        &input.alias,
        &input.account,
        &sealed.ciphertext_b64,
        &sealed.nonce_b64,
        &input.tags,
    )
    .map_err(|e| format!("Failed to create entry: {}", e))?;

    Ok(EntrySearchResult {
        id: entry.id,
        name: entry.name,
        url: entry.url,
        description: entry.description,
        alias: entry.alias,
        account: entry.account,
        tags: entry.tags,
        password_preview: "********".to_string(),
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn update_entry(
    id: String,
    name: String,
    url: Option<String>,
    description: Option<String>,
    alias: Option<String>,
    account: Option<String>,
    password: Option<String>,
    tags: Option<String>,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<EntrySearchResult, String> {
    if id.trim().is_empty() {
        return Err("entry id is required".to_string());
    }

    let input = normalize_update_entry(name, url, description, alias, account, password, tags)?;
    let sealed = if input.password.is_empty() {
        None
    } else {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        Some(crypto::encrypt_secret(
            vault.key()?,
            &entry_aad(&id),
            &input.password,
        )?)
    };

    if sealed.is_none() {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        vault.key()?;
    }

    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;

    match sealed {
        Some(sealed) => queries::update_entry_metadata_and_secret(
            &db,
            &id,
            &input.name,
            &input.url,
            &input.description,
            &input.alias,
            &input.account,
            &sealed.ciphertext_b64,
            &sealed.nonce_b64,
            &input.tags,
        ),
        None => queries::update_entry_metadata(
            &db,
            &id,
            &input.name,
            &input.url,
            &input.description,
            &input.alias,
            &input.account,
            &input.tags,
        ),
    }
    .map_err(|e| format!("Failed to update entry {}: {}", id, e))
}

#[tauri::command]
pub fn delete_entry(
    id: String,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<(), String> {
    if id.trim().is_empty() {
        return Err("entry id is required".to_string());
    }
    {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        vault.key()?;
    }
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    queries::delete_entry(&db, &id).map_err(|e| format!("Failed to delete entry {}: {}", id, e))
}
