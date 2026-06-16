use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub fn copy_to_clipboard(text: String, app: tauri::AppHandle) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))
}
