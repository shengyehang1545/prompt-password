use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::platform::window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostCopyWindowAction {
    HideMainPanel,
}

pub fn post_copy_window_action() -> PostCopyWindowAction {
    PostCopyWindowAction::HideMainPanel
}

fn apply_post_copy_window_action(
    app: &tauri::AppHandle,
    action: PostCopyWindowAction,
) -> Result<(), String> {
    match action {
        PostCopyWindowAction::HideMainPanel => window::hide_main_window(app),
    }
}

#[tauri::command]
pub fn copy_to_clipboard(text: String, app: tauri::AppHandle) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    apply_post_copy_window_action(&app, post_copy_window_action())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_copy_hides_main_panel() {
        assert_eq!(post_copy_window_action(), PostCopyWindowAction::HideMainPanel);
    }
}
