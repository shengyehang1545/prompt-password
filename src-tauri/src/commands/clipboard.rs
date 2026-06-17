use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::core::vault::{entry_aad, VaultSession};
use crate::crypto;
use crate::db::queries;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostCopyWindowAction {
    KeepMainPanelVisible,
}

pub fn post_copy_window_action() -> PostCopyWindowAction {
    PostCopyWindowAction::KeepMainPanelVisible
}

fn apply_post_copy_window_action(action: PostCopyWindowAction) -> Result<(), String> {
    match action {
        PostCopyWindowAction::KeepMainPanelVisible => Ok(()),
    }
}

#[tauri::command]
pub fn copy_to_clipboard(text: String, app: tauri::AppHandle) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    apply_post_copy_window_action(post_copy_window_action())
}

#[tauri::command]
pub fn copy_entry_password(
    id: String,
    app: tauri::AppHandle,
    conn: State<'_, Mutex<Connection>>,
    vault: State<'_, Mutex<VaultSession>>,
) -> Result<(), String> {
    let entry = {
        let db = conn
            .lock()
            .map_err(|e| format!("Failed to access database: {}", e))?;
        queries::get_entry(&db, &id)
            .map_err(|e| format!("Failed to retrieve entry {}: {}", id, e))?
    };
    let password = {
        let vault = vault
            .lock()
            .map_err(|e| format!("Failed to access vault: {}", e))?;
        crypto::decrypt_secret(
            vault.key()?,
            &entry_aad(&entry.id),
            &entry.password_enc,
            &entry.password_nonce,
        )?
    };

    app.clipboard()
        .write_text(password)
        .map_err(|e| format!("Failed to copy password to clipboard: {}", e))?;

    apply_post_copy_window_action(post_copy_window_action())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_copy_keeps_main_panel_visible_for_feedback() {
        assert_eq!(
            post_copy_window_action(),
            PostCopyWindowAction::KeepMainPanelVisible
        );
    }
}
