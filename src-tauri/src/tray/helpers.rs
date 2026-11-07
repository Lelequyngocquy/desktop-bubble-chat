use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Returns account indices that are authenticated and active in ChatState
pub fn get_available_account_indices(app_handle: &AppHandle) -> Vec<u32> {
    let mut available = std::collections::BTreeSet::new();
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

/// Checks if a bubble (Native or Webview) is currently visible
pub fn is_bubble_visible(app_handle: &AppHandle, account_index: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        if crate::operating_systems::current::bubble::has_native_bubble(account_index) {
            return true;
        }
    }
    let label = format!("bubble-{:02}", account_index);
    app_handle
        .get_webview_window(&label)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}

/// Toggles visibility of a single specified bubble from tray
pub async fn toggle_single_bubble_from_tray(app: &AppHandle, account_index: u32) {
    let is_visible = is_bubble_visible(app, account_index);
    let next_visible = !is_visible;
    let _ = crate::commands::bubble::toggle_bubble(app.clone(), next_visible, Some(account_index)).await;
}

/// Checks if main dashboard window is currently open and visible
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

/// Toggles main dashboard window from tray
pub fn toggle_main_window(app_handle: &AppHandle) {
    if is_main_window_open(app_handle) {
        crate::presentation::main_window::destroy_or_hibernate_main_window(app_handle);
    } else {
        let _ = crate::presentation::main_window::get_or_create_main_window(app_handle);
    }
}
