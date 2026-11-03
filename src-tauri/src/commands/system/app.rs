use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Command thu nhỏ cửa sổ Dashboard chính xuống Taskbar
#[tauri::command]
pub fn minimize_main_window(app: AppHandle) {
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.minimize();
    }
}

/// Command phóng to / khôi phục kích thước cửa sổ Dashboard chính
#[tauri::command]
pub fn toggle_maximize_main_window(app: AppHandle) -> Result<bool, String> {
    if let Some(main_win) = app.get_webview_window("main") {
        let is_max = main_win.is_maximized().map_err(|e| e.to_string())?;
        if is_max {
            main_win.unmaximize().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            main_win.maximize().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        Err("Main window not found".to_string())
    }
}

/// Command ẩn cửa sổ Dashboard chính về khay hệ thống (System Tray) - Kích hoạt Dynamic Destruction (0 MB UI RAM)
#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    crate::windows::main_window::destroy_or_hibernate_main_window(&app);
}

/// Command mở lại cửa sổ Dashboard chính (Tái sinh hoặc hiển thị lại)
#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    let _ = crate::windows::main_window::get_or_create_main_window(&app);
}

/// Command đồng bộ trạng thái hiển thị menu System Tray từ Frontend / Store
#[tauri::command]
pub fn update_tray_menu_status(
    app: AppHandle,
    _has_connected_accounts: Option<bool>,
) -> Result<(), String> {
    crate::tray::update_tray_menu(&app).map_err(|e| e.to_string())
}

/// Command lấy snapshot cấu hình người dùng từ cache
#[tauri::command]
pub fn get_user_settings(state: tauri::State<'_, Arc<crate::state::ChatState>>) -> Result<crate::utils::user_cache::AppUserConfig, String> {
    Ok(state.get_user_config())
}

/// Command lưu cấu hình người dùng vào cache
#[tauri::command]
pub fn save_user_settings(
    app: AppHandle,
    state: tauri::State<'_, Arc<crate::state::ChatState>>,
    settings: crate::utils::user_cache::AppUserConfig,
) -> Result<(), String> {
    state.set_user_config(&app, settings);
    Ok(())
}

/// Command thoát ứng dụng hoàn toàn
#[tauri::command]
pub fn exit_app(app: AppHandle) {
    if let Some(state) = app.try_state::<Arc<crate::state::ChatState>>() {
        state.is_explicit_exit.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    app.exit(0);
}

/// Command điều khiển chế độ Luôn nổi trên cùng (Always on Top) dành riêng cho các Bong bóng nổi
#[tauri::command]
pub fn set_always_on_top(app: AppHandle, enabled: bool) -> Result<(), String> {
    use std::sync::atomic::Ordering;

    // 1. Lưu trạng thái vào ChatState để các Bubble khởi tạo / hiển thị sau này tuân thủ
    if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
        chat_state.is_always_on_top.store(enabled, Ordering::SeqCst);
        chat_state.update_user_config(&app, |c| {
            c.is_always_on_top = enabled;
        });
    }

    // 2. Dashboard chính tuyệt đối KHÔNG ghim Always-on-top (luôn là cửa sổ tiêu chuẩn của OS)
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.set_always_on_top(false);
    }

    // 3. Áp dụng Always-on-top cho tất cả các Bong bóng nổi Native Win32 (SSOT)
    #[cfg(target_os = "windows")]
    crate::windows::native_bubble::set_native_bubble_always_on_top(enabled);

    for idx in 1..=5 {
        let bubble_label = format!("bubble-{:02}", idx);
        if let Some(w) = app.get_webview_window(&bubble_label) {
            let _ = w.set_always_on_top(enabled);
        }
    }

    // 4. Khung chat Messenger đồng bộ trạng thái nổi cùng bong bóng
    for idx in 1..=5 {
        let chat_label = format!("chat-messenger-{:02}", idx);
        if let Some(w) = app.get_webview_window(&chat_label) {
            let _ = w.set_always_on_top(enabled);
        }
    }
    if let Some(w) = app.get_webview_window("chat-messenger") {
        let _ = w.set_always_on_top(enabled);
    }
    Ok(())
}

/// Command mở liên kết ngoài an toàn bằng trình duyệt mặc định của hệ điều hành
#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;

            let wide: Vec<u16> = OsStr::new(&url)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            #[link(name = "shell32")]
            unsafe extern "system" {
                fn ShellExecuteW(
                    hwnd: *mut std::ffi::c_void,
                    lpOperation: *const u16,
                    lpFile: *const u16,
                    lpParameters: *const u16,
                    lpDirectory: *const u16,
                    nShowCmd: i32,
                ) -> isize;
            }

            unsafe {
                ShellExecuteW(
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    wide.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                    1, // SW_SHOWNORMAL
                );
            }
            return Ok(());
        }
        #[cfg(not(target_os = "windows"))]
        {
            return Ok(());
        }
    }
    Err("Invalid URL scheme".into())
}

/// Command cầu nối chuyển log từ Frontend React / Injected Script thẳng vào Terminal Backend và file logs/messenger_inject.log
#[tauri::command]
pub fn log_frontend(app: AppHandle, level: Option<String>, message: String) {
    let lvl = level.as_deref().unwrap_or("info");
    let tag = match lvl.to_lowercase().as_str() {
        "error" => "\x1b[31m[FE - ERROR]\x1b[0m",
        "warn" => "\x1b[33m[FE - WARN]\x1b[0m",
        "debug" => "\x1b[35m[FE - DEBUG]\x1b[0m",
        _ => "\x1b[36m[FE - INFO]\x1b[0m",
    };
    println!("{} {}", tag, message);

    // Ghi an toàn vào AppData để tránh lỗi quyền khi cài đặt trong C:\Program Files
    if let Ok(data_dir) = app.path().app_data_dir() {
        let log_dir = data_dir.join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let log_file = log_dir.join("messenger_inject.log");
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&log_file) {
            let now = crate::utils::time::now_millis();
            let _ = writeln!(file, "[{}] [{}] {}", now, lvl.to_uppercase(), message);
        }
    }
}
