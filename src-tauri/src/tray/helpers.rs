use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Lấy danh sách các tài khoản đang có (có thư mục profile trên đĩa hoặc đã xác thực trong ChatState)
pub fn get_available_account_indices(app_handle: &AppHandle) -> Vec<u32> {
    let mut available = std::collections::BTreeSet::new();

    // 1. Quét các thư mục profile trên đĩa: profiles/messenger_01 .. profiles/messenger_05
    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        let profiles_dir = app_data_dir.join("profiles");
        for idx in 1..=5 {
            if profiles_dir.join(format!("messenger_{:02}", idx)).exists() {
                available.insert(idx);
            }
        }
    }

    // 2. Quét state xác thực trong ChatState
    if let Some(chat_state) = app_handle.try_state::<Arc<crate::state::ChatState>>() {
        if let Ok(map) = chat_state.account_auth.lock() {
            for (idx, info) in map.iter() {
                if info.connected {
                    available.insert(*idx);
                }
            }
        }
    }

    available.into_iter().collect()
}

/// Kiểm tra xem một Bong bóng (Native hoặc Webview) có đang hiển thị hay không
pub fn is_bubble_visible(app_handle: &AppHandle, account_index: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        if crate::windows::native_bubble::has_native_bubble(account_index) {
            return true;
        }
    }
    let label = format!("bubble-{:02}", account_index);
    app_handle
        .get_webview_window(&label)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}

/// Helper điều khiển bật/tắt riêng một bong bóng chỉ định từ Tray Icon
pub async fn toggle_single_bubble_from_tray(app: &AppHandle, account_index: u32) {
    let is_visible = is_bubble_visible(app, account_index);
    let next_visible = !is_visible;
    let _ = crate::commands::bubble::toggle_bubble(app.clone(), next_visible, Some(account_index)).await;
}

/// Kiểm tra xem cửa sổ Dashboard chính có đang mở và hiển thị hay không
pub fn is_main_window_open(app_handle: &AppHandle) -> bool {
    if let Some(main_win) = app_handle.get_webview_window("main") {
        #[cfg(target_os = "windows")]
        {
            #[link(name = "user32")]
            unsafe extern "system" {
                fn IsWindow(hWnd: *mut std::ffi::c_void) -> i32;
            }
            let is_alive = if let Ok(hwnd) = main_win.hwnd() {
                unsafe { IsWindow(hwnd.0 as *mut std::ffi::c_void) != 0 }
            } else {
                false
            };
            is_alive && main_win.is_visible().unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        main_win.is_visible().unwrap_or(false)
    } else {
        false
    }
}

/// Helper điều khiển Mở/Đóng Dashboard động từ System Tray
pub fn toggle_main_window(app_handle: &AppHandle) {
    if is_main_window_open(app_handle) {
        crate::windows::main_window::destroy_or_hibernate_main_window(app_handle);
    } else {
        let _ = crate::windows::main_window::get_or_create_main_window(app_handle);
    }
}
