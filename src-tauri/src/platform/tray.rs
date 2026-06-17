use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, Manager};

use crate::core::vault::VaultSession;

const MENU_SHOW: &str = "show";
const MENU_LOCK: &str = "lock";
const MENU_QUIT: &str = "quit";

pub fn configure_tray(app: &mut App) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    let show = MenuItem::with_id(app, MENU_SHOW, "Show", true, None::<&str>)?;
    let lock = MenuItem::with_id(app, MENU_LOCK, "Lock Vault", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &lock, &quit])?;

    let mut builder = TrayIconBuilder::new()
        .tooltip("Prompt Password")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_SHOW => crate::platform::window::show_main_window(app),
            MENU_LOCK => {
                if let Some(vault) = app.try_state::<std::sync::Mutex<VaultSession>>() {
                    if let Ok(mut vault) = vault.lock() {
                        vault.lock();
                    }
                }
                if let Some(window) =
                    app.get_webview_window(crate::platform::window::MAIN_WINDOW_LABEL)
                {
                    let _ = window.hide();
                }
            }
            MENU_QUIT => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;
    Ok(())
}
