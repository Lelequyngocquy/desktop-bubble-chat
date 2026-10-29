use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::ChatState;
use crate::utils::parser::parse_unread_from_title;
use crate::windows::chat::{is_window_logged_in, MESSENGER_INJECT_SCRIPT};

/// Luồng giám sát trạng thái đăng nhập và URL của Messenger theo thời gian thực
pub fn start_auth_watcher(app_handle: AppHandle, chat_state: Arc<ChatState>) {
    std::thread::spawn(move || {
        let mut was_logged_in = false;
        loop {
            std::thread::sleep(Duration::from_millis(1500));

            if let Some(chat_window) = app_handle.get_webview_window("chat-messenger") {
                // Định kỳ nạp script kiểm tra DOM và responsive CSS
                let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);

                let (is_logged_in, user_name) = is_window_logged_in(&chat_window);

                if is_logged_in != was_logged_in {
                    was_logged_in = is_logged_in;

                    let _ = app_handle.emit("messenger-auth-status", serde_json::json!({
                        "connected": is_logged_in,
                        "name": if is_logged_in { &user_name } else { "" }
                    }));

                    if is_logged_in {
                        // Đã đăng nhập xong: Chuyển về chế độ chat bình thường (tắt auth mode để cho phép auto-hide on blur)
                        chat_state.is_auth_mode.store(false, Ordering::SeqCst);
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
    });
}
