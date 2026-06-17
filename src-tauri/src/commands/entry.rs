use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::core::entry_input::normalize_create_entry;
use crate::core::vault::{entry_aad, VaultSession};
use crate::crypto;
use crate::db::queries::{self, Entry};

#[tauri::command]
pub fn get_entry(
    id: String,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<Entry, String> {
    {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        vault.key()?;
    }
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    queries::get_entry(&db, &id).map_err(|e| format!("Failed to retrieve entry {}: {}", id, e))
}

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
) -> Result<Entry, String> {
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

    queries::insert_entry_with_id(
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
    .map_err(|e| format!("Failed to create entry: {}", e))
}
