use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::db::queries::{self, Entry};

#[tauri::command]
pub fn get_entry(id: String, conn: State<'_, Mutex<Connection>>) -> Result<Entry, String> {
    let db = conn.lock().map_err(|e| format!("Failed to access database: {}", e))?;
    queries::get_entry(&db, &id).map_err(|e| format!("Failed to retrieve entry {}: {}", id, e))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_entry(
    name: String,
    url: Option<String>,
    description: Option<String>,
    alias: Option<String>,
    password: String,
    tags: Option<String>,
    conn: State<'_, Mutex<Connection>>,
) -> Result<Entry, String> {
    if name.trim().is_empty() {
        return Err("name is required".to_string());
    }
    if password.is_empty() {
        return Err("password is required".to_string());
    }

    let db = conn.lock().map_err(|e| format!("Failed to access database: {}", e))?;
    queries::insert_entry(
        &db,
        &name,
        &url.unwrap_or_default(),
        &description.unwrap_or_default(),
        &alias.unwrap_or_default(),
        &password,
        &tags.unwrap_or_default(),
    )
    .map_err(|e| format!("Failed to create entry: {}", e))
}
