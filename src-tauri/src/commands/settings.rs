use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::db::queries;

const HOTKEY_KEY: &str = "hotkey";
const AUTO_START_KEY: &str = "auto_start_enabled";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub auto_start_enabled: bool,
    pub hotkey: String,
}

#[tauri::command]
pub fn get_app_settings(conn: State<'_, Mutex<Connection>>) -> Result<AppSettings, String> {
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    read_app_settings(&db)
}

#[tauri::command]
pub fn set_auto_start_enabled(
    enabled: bool,
    conn: State<'_, Mutex<Connection>>,
) -> Result<AppSettings, String> {
    crate::platform::startup::set_auto_start_enabled(enabled)?;
    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    queries::set_app_setting(&db, AUTO_START_KEY, bool_to_setting(enabled))
        .map_err(|e| format!("Failed to save auto-start setting: {}", e))?;
    read_app_settings(&db)
}

#[tauri::command]
pub fn set_global_hotkey(
    hotkey: String,
    app: AppHandle,
    conn: State<'_, Mutex<Connection>>,
) -> Result<AppSettings, String> {
    let normalized = normalize_hotkey(&hotkey)?;
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| format!("Failed to unregister current hotkey: {}", e))?;
    if let Err(err) = app.global_shortcut().register(normalized.as_str()) {
        let fallback = default_hotkey();
        let _ = app.global_shortcut().register(fallback.as_str());
        return Err(format!("Failed to register hotkey {}: {}", normalized, err));
    }

    let db = conn
        .lock()
        .map_err(|e| format!("Failed to access database: {}", e))?;
    queries::set_app_setting(&db, HOTKEY_KEY, &normalized)
        .map_err(|e| format!("Failed to save hotkey setting: {}", e))?;
    read_app_settings(&db)
}

pub fn read_app_settings(conn: &Connection) -> Result<AppSettings, String> {
    let auto_start_enabled = queries::get_app_setting(conn, AUTO_START_KEY)
        .map_err(|e| format!("Failed to read auto-start setting: {}", e))?
        .as_deref()
        .map(setting_to_bool)
        .unwrap_or(false);
    let hotkey = queries::get_app_setting(conn, HOTKEY_KEY)
        .map_err(|e| format!("Failed to read hotkey setting: {}", e))?
        .unwrap_or_else(default_hotkey);

    Ok(AppSettings {
        auto_start_enabled,
        hotkey,
    })
}

pub fn stored_hotkey_or_default(conn: &Connection) -> String {
    queries::get_app_setting(conn, HOTKEY_KEY)
        .ok()
        .flatten()
        .unwrap_or_else(default_hotkey)
}

fn normalize_hotkey(hotkey: &str) -> Result<String, String> {
    let trimmed = hotkey.trim();
    if trimmed.is_empty() {
        return Err("Hotkey is required".to_string());
    }
    trimmed
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|e| format!("Invalid hotkey: {}", e))?;
    Ok(trimmed.to_string())
}

fn default_hotkey() -> String {
    crate::platform::hotkey::default_hotkey_spec()
        .label()
        .to_string()
}

fn bool_to_setting(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

fn setting_to_bool(value: &str) -> bool {
    matches!(value, "true" | "1" | "yes")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_conn() -> Connection {
        crate::db::init::initialize_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    #[test]
    fn read_app_settings_returns_defaults() {
        let conn = test_conn();

        let settings = read_app_settings(&conn).unwrap();

        assert!(!settings.auto_start_enabled);
        assert_eq!(settings.hotkey, default_hotkey());
    }

    #[test]
    fn read_app_settings_uses_stored_values() {
        let conn = test_conn();
        queries::set_app_setting(&conn, AUTO_START_KEY, "true").unwrap();
        queries::set_app_setting(&conn, HOTKEY_KEY, "Cmd+Option+P").unwrap();

        let settings = read_app_settings(&conn).unwrap();

        assert!(settings.auto_start_enabled);
        assert_eq!(settings.hotkey, "Cmd+Option+P");
    }

    #[test]
    fn normalize_hotkey_rejects_empty_values() {
        assert_eq!(normalize_hotkey("   ").unwrap_err(), "Hotkey is required");
    }
}
