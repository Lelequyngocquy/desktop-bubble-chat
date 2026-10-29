use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::ChatState;
use crate::utils::positioning::ensure_bubble_positioned_and_visible;
use super::parser::parse_unread_from_title;
use super::window::{get_messenger_window_label, is_window_logged_in};

/// Luồng giám sát trạng thái đăng nhập và số tin chưa đọc của các tài khoản Messenger theo thời gian thực
pub fn start_auth_watcher(app_handle: AppHandle, chat_state: Arc<ChatState>) {
    std::thread::spawn(move || {
        let mut logged_in_map: std::collections::HashMap<u32, bool> = std::collections::HashMap::new();

        loop {
            std::thread::sleep(Duration::from_millis(700));

            for idx in 1..=5 {
                let label = get_messenger_window_label(idx);
                let win = app_handle.get_webview_window(&label)
                    .or_else(|| if idx == 1 { app_handle.get_webview_window("chat-messenger") } else { None });

                if let Some(chat_window) = win {
                    let win_label = chat_window.label().to_string();
                    let (is_logged_in, user_name) = is_window_logged_in(&chat_window);

                    let was_logged = *logged_in_map.get(&idx).unwrap_or(&false);

                    if is_logged_in != was_logged {
                        logged_in_map.insert(idx, is_logged_in);

                        let _ = app_handle.emit("messenger-auth-status", serde_json::json!({
                            "accountIndex": idx,
                            "connected": is_logged_in,
                            "name": if is_logged_in { &user_name } else { "" }
                        }));

                        if is_logged_in {
                            // Đã đăng nhập xong: Tắt auth mode, đóng popup đăng nhập, chuyển sang chế độ chat docked
                            chat_state.set_window_auth_mode(&win_label, false);
                            let _ = chat_window.hide();
                            let _ = chat_window.set_decorations(false);
                            let _ = chat_window.set_skip_taskbar(true);
                            let _ = app_handle.emit("chat-visibility-changed", false);
                            // Bật bong bóng nổi cho tài khoản này
                            let _ = ensure_bubble_positioned_and_visible(&app_handle, idx);
                        }
                    }

                    // Đồng bộ badge tin chưa đọc
                    if is_logged_in {
                        if let Ok(title) = chat_window.title() {
                            let count = parse_unread_from_title(&title);
                            let _ = app_handle.emit("badge-updated", count);
                        }
                    }
                }
            }
        }
    });
}
