use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};

use crate::utils::positioning::ensure_bubble_positioned_and_visible;

/// Khởi tạo System Tray Icon & Menu điều khiển
pub fn setup_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_home = MenuItem::with_id(app, "show_home", "Mở Trang chủ", true, None::<&str>)?;
    let toggle_bubble_item = MenuItem::with_id(app, "toggle_bubble_item", "Bật/Tắt Bong bóng nổi", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Thoát hoàn toàn", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_home, &toggle_bubble_item, &separator, &quit])?;

    let icon = app.default_window_icon().cloned().ok_or("Failed to get app icon")?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app_handle, event| {
            match event.id.as_ref() {
                "show_home" => {
                    if let Some(main_win) = app_handle.get_webview_window("main") {
                        let _ = main_win.show();
                        let _ = main_win.set_focus();
                    }
                }
                "toggle_bubble_item" => {
                    if let Some(bubble_win) = app_handle.get_webview_window("bubble") {
                        let is_vis = bubble_win.is_visible().unwrap_or(false);
                        if is_vis {
                            let _ = bubble_win.hide();
                            let _ = app_handle.emit("bubble-visibility-changed", false);
                        } else {
                            let _ = ensure_bubble_positioned_and_visible(&app_handle, 1);
                        }
                    }
                }
                "quit" => {
                    app_handle.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app_handle = tray.app_handle();
                if let Some(main_win) = app_handle.get_webview_window("main") {
                    let _ = main_win.show();
                    let _ = main_win.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
