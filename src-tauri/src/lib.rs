use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

mod commands;
mod core;
mod crypto;
mod db;
mod platform;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        platform::window::toggle_main_window(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::search::search_entries,
            commands::entry::get_entry,
            commands::entry::create_entry,
            commands::clipboard::copy_to_clipboard,
            commands::clipboard::copy_entry_password,
            commands::vault::unlock_vault,
            commands::vault::lock_vault,
            commands::template::search_password_templates,
            commands::template::create_password_template,
            commands::template::use_password_template,
        ])
        .setup(|app| {
            let window = app
                .get_webview_window(platform::window::MAIN_WINDOW_LABEL)
                .expect("main window must exist");
            platform::window::configure_main_window(&window);

            let hotkey = platform::hotkey::default_hotkey_spec();
            app.global_shortcut()
                .register(hotkey.to_tauri_shortcut())
                .unwrap_or_else(|e| {
                    panic!(
                        "failed to register global shortcut {}: {}",
                        hotkey.label(),
                        e
                    )
                });

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            let db_path = app_data_dir.join("prompt-password.db");
            let conn = db::init::initialize(&db_path).expect("failed to initialize database");
            app.manage(Mutex::new(conn));
            app.manage(Mutex::new(core::vault::VaultSession::default()));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
