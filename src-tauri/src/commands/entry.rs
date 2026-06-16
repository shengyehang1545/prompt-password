use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::core::entry_input::normalize_create_entry;
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
    let input = normalize_create_entry(name, url, description, alias, password, tags)?;

    let db = conn.lock().map_err(|e| format!("Failed to access database: {}", e))?;
    queries::insert_entry(
        &db,
        &input.name,
        &input.url,
        &input.description,
        &input.alias,
        &input.password,
        &input.tags,
    )
    .map_err(|e| format!("Failed to create entry: {}", e))
}
