use tauri_plugin_clipboard_manager::ClipboardExt;

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
