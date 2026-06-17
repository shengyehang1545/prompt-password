use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::core::vault::{self, VaultSession};

#[tauri::command]
pub fn unlock_vault(
    master_password: String,
    conn: State<'_, Mutex<Connection>>,
    vault_state: State<'_, Mutex<VaultSession>>,
) -> Result<bool, String> {
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    let mut vault = vault_state
        .lock()
        .map_err(|e| format!("Failed to access vault: {}", e))?;

    vault::unlock_or_initialize(&db, &mut vault, &master_password)?;
    Ok(true)
}

#[tauri::command]
pub fn lock_vault(vault_state: State<'_, Mutex<VaultSession>>) -> Result<(), String> {
    let mut vault = vault_state
        .lock()
        .map_err(|e| format!("Failed to access vault: {}", e))?;
    vault.lock();
    Ok(())
}
