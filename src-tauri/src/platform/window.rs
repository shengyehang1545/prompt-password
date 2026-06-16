use tauri::{Manager, WebviewWindow};

pub const MAIN_WINDOW_LABEL: &str = "main";

pub fn configure_main_window(window: &WebviewWindow) {
    #[cfg(debug_assertions)]
    {
        window.open_devtools();
    }

    hide_on_blur(window);
}

pub fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn hide_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window
            .hide()
            .map_err(|e| format!("Failed to hide main window: {}", e))?;
    }

    Ok(())
}

fn hide_on_blur(window: &WebviewWindow) {
    let w = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Focused(false) = event {
            let _ = w.hide();
        }
    });
}
