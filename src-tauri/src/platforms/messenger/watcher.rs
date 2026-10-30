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
                    let auth_info = chat_state.get_account_auth(idx);
                    let state_logged_in = auth_info.as_ref().map(|a| a.connected).unwrap_or(false);
                    let (win_logged_in, win_user_name) = is_window_logged_in(&chat_window);
                    let is_logged_in = state_logged_in || win_logged_in;

                    let user_name = if state_logged_in {
                        auth_info.as_ref().map(|a| a.name.clone()).unwrap_or_default()
                    } else {
                        win_user_name
                    };

                    let was_logged = *logged_in_map.get(&idx).unwrap_or(&false);

                    if is_logged_in != was_logged {
                        logged_in_map.insert(idx, is_logged_in);

                        if is_logged_in {
                            chat_state.set_account_auth(
                                idx,
                                true,
                                if user_name.is_empty() { None } else { Some(user_name.clone()) },
                            );

                            // Đảm bảo cửa sổ không bị lưu lại URL Facebook
                            if let Ok(curr_url) = chat_window.url() {
                                let u = curr_url.to_string();
                                if u.contains("facebook.com") {
                                    if let Ok(target) = tauri::Url::parse("https://www.messenger.com/") {
                                        let _ = chat_window.navigate(target);
                                    }
                                }
                            }

                            // Đã đăng nhập xong: Tắt auth mode, đóng popup đăng nhập, chuyển sang chế độ chat docked
                            chat_state.set_window_auth_mode(&win_label, false);
                            let _ = chat_window.hide();
                            let _ = chat_window.set_decorations(false);
                            let _ = chat_window.set_skip_taskbar(true);
                            let _ = app_handle.emit("chat-visibility-changed", serde_json::json!({
                                "accountIndex": idx,
                                "visible": false,
                                "windowLabel": &win_label
                            }));
                            // Bật bong bóng nổi cho tài khoản này
                            let _ = ensure_bubble_positioned_and_visible(&app_handle, idx);
                        } else {
                            chat_state.clear_account_auth(idx);
                        }

                        let _ = app_handle.emit("messenger-auth-status", serde_json::json!({
                            "accountIndex": idx,
                            "connected": is_logged_in,
                            "name": if is_logged_in { &user_name } else { "" }
                        }));
                    }

                    // Đồng bộ badge tin chưa đọc
                    if is_logged_in {
                        if let Ok(title) = chat_window.title() {
                            let count = parse_unread_from_title(&title);
                            let _ = app_handle.emit("badge-updated", serde_json::json!({
                                "accountIndex": idx,
                                "count": count
                            }));
                        }
                    }
                }
            }
        }
    });
}
