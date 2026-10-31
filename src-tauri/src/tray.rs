use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub const TRAY_ID: &str = "main_tray";

/// Cập nhật menu của System Tray theo trạng thái có tài khoản kết nối hay không
pub fn update_tray_menu(app_handle: &AppHandle, has_connected_accounts: bool) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
        let show_home = MenuItem::with_id(app_handle, "show_home", "Mở Trang chủ", true, None::<&str>)?;
        let separator = PredefinedMenuItem::separator(app_handle)?;
        let quit = MenuItem::with_id(app_handle, "quit", "Thoát hoàn toàn", true, None::<&str>)?;

        let menu = if has_connected_accounts {
            let toggle_bubble_item = MenuItem::with_id(app_handle, "toggle_bubble_item", "Bật/Tắt Bong bóng nổi", true, None::<&str>)?;
            Menu::with_items(app_handle, &[&show_home, &toggle_bubble_item, &separator, &quit])?
        } else {
            Menu::with_items(app_handle, &[&show_home, &separator, &quit])?
        };

        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

/// Khởi tạo System Tray Icon & Menu điều khiển
pub fn setup_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_home = MenuItem::with_id(app, "show_home", "Mở Trang chủ", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Thoát hoàn toàn", true, None::<&str>)?;

    // Mặc định ban đầu: Ẩn nút "Bật/Tắt Bong bóng nổi" khi chưa có liên kết đăng nhập nào
    let menu = Menu::with_items(app, &[&show_home, &separator, &quit])?;

    let icon = app.default_window_icon().cloned().ok_or("Failed to get app icon")?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
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
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        crate::commands::bubble::toggle_all_bubbles_impl(&handle, None, None).await;
                    });
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
