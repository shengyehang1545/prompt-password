use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::core::vault::VaultSession;
use crate::db::queries::{self, EntrySearchResult};

#[tauri::command]
pub fn search_entries(
    query: String,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<Vec<EntrySearchResult>, String> {
    if query.len() > 256 {
        return Err("Search query too long (max 256 characters)".to_string());
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
    queries::search_entries(&db, &query).map_err(|e| format!("Failed to search entries: {}", e))
}
