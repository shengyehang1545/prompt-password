use tauri::AppHandle;

#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    crate::platform::window::hide_main_window(&app);
}
