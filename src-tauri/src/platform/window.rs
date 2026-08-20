use tauri::{Emitter, Manager, PhysicalPosition, WebviewWindow};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const PANEL_SHOWN_EVENT: &str = "panel-shown";
pub const PANEL_HIDDEN_EVENT: &str = "panel-hidden";

pub fn configure_main_window(window: &WebviewWindow) {
    #[cfg(debug_assertions)]
    {
        window.open_devtools();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = window.set_visible_on_all_workspaces(true);
    }
}

pub fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            hide_window(&window);
        } else {
            show_window(&window);
        }
    }
}

pub fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        show_window(&window);
    }
}

pub fn hide_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        hide_window(&window);
    }
}

fn show_window(window: &WebviewWindow) {
    let _ = move_to_cursor_monitor(window);
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.emit(PANEL_SHOWN_EVENT, ());
}

fn hide_window(window: &WebviewWindow) {
    let _ = window.emit(PANEL_HIDDEN_EVENT, ());
    let _ = window.hide();
}

fn move_to_cursor_monitor(window: &WebviewWindow) -> tauri::Result<()> {
    let cursor = window.cursor_position()?;
    let monitor = window
        .available_monitors()?
        .into_iter()
        .find(|monitor| point_is_inside_monitor(cursor.x, cursor.y, monitor));

    if let Some(monitor) = monitor {
        let work_area = monitor.work_area();
        let window_size = window.outer_size()?;
        let x = centered_axis(
            work_area.position.x,
            work_area.size.width,
            window_size.width,
        );
        let y = centered_axis(
            work_area.position.y,
            work_area.size.height,
            window_size.height,
        );
        window.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

fn point_is_inside_monitor(x: f64, y: f64, monitor: &tauri::Monitor) -> bool {
    let position = monitor.position();
    let size = monitor.size();
    point_is_inside_rect(
        x,
        y,
        position.x as f64,
        position.y as f64,
        size.width as f64,
        size.height as f64,
    )
}

fn point_is_inside_rect(
    x: f64,
    y: f64,
    origin_x: f64,
    origin_y: f64,
    width: f64,
    height: f64,
) -> bool {
    x >= origin_x && x < origin_x + width && y >= origin_y && y < origin_y + height
}

fn centered_axis(origin: i32, available: u32, window: u32) -> i32 {
    let offset = available.saturating_sub(window) / 2;
    origin.saturating_add(i32::try_from(offset).unwrap_or(i32::MAX))
}

#[cfg(test)]
mod tests {
    use super::{centered_axis, point_is_inside_rect};

    #[test]
    fn centers_window_in_monitor_work_area() {
        assert_eq!(centered_axis(1728, 1440, 960), 1968);
    }

    #[test]
    fn centers_window_on_monitor_with_negative_origin() {
        assert_eq!(centered_axis(-1920, 1920, 960), -1440);
    }

    #[test]
    fn clamps_oversized_window_to_work_area_origin() {
        assert_eq!(centered_axis(100, 800, 1000), 100);
    }

    #[test]
    fn matches_cursor_inside_negative_coordinate_monitor() {
        assert!(point_is_inside_rect(
            -960.0, 348.0, -1920.0, -192.0, 1920.0, 1080.0
        ));
        assert!(!point_is_inside_rect(
            0.0, 348.0, -1920.0, -192.0, 1920.0, 1080.0
        ));
    }
}
