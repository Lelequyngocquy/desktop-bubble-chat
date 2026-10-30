use tauri::{AppHandle, Manager};

/// Cấu hình sự kiện cho cửa sổ chính (Home Page): Chặn thoát app khi ấn nút X, ẩn về tray
pub fn setup_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        let main_clone = main_window.clone();
        main_window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = main_clone.hide();
            }
        });
    }
}
