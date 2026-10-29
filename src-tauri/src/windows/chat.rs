use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::state::ChatState;
use crate::utils::parser::parse_unread_from_title;
use crate::utils::positioning::{is_cursor_over_bubble, tuck_bubble};
use crate::utils::time::now_millis;

pub const MESSENGER_INJECT_SCRIPT: &str = include_str!("../scripts/messenger_inject.js");

/// Cấu hình sự kiện tự động ẩn khi blur và chặn close cho cửa sổ chat Messenger
pub fn setup_chat_window(app: &AppHandle, chat_state: Arc<ChatState>) {
    if let Some(chat_window) = app.get_webview_window("chat-messenger") {
        let chat_clone = chat_window.clone();
        let app_handle = app.clone();
        let state_clone = Arc::clone(&chat_state);

        chat_window.on_window_event(move |event| {
            match event {
                tauri::WindowEvent::Focused(false) => {
                    // Chỉ tự động ẩn khi KHÔNG ở chế độ đăng nhập (Auth Mode)
                    if !state_clone.is_auth_mode.load(Ordering::SeqCst) {
                        state_clone.last_blur_time.store(now_millis(), Ordering::SeqCst);
                        let _ = chat_clone.hide();
                        let _ = app_handle.emit("chat-visibility-changed", false);

                        // Khi click ra ngoài cả popup lẫn bubble -> thụt bubble vào mép luôn
                        if let Some(bubble_win) = app_handle.get_webview_window("bubble") {
                            if !is_cursor_over_bubble(&bubble_win) {
                                tuck_bubble(&app_handle);
                            }
                        }
                    }
                }
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // Chặn đóng tiến trình, chỉ ẩn cửa sổ và reset chế độ Auth
                    api.prevent_close();
                    state_clone.is_auth_mode.store(false, Ordering::SeqCst);
                    let _ = chat_clone.hide();
                    let _ = chat_clone.set_decorations(false);
                    let _ = app_handle.emit("chat-visibility-changed", false);

                    if let Some(bubble_win) = app_handle.get_webview_window("bubble") {
                        if !is_cursor_over_bubble(&bubble_win) {
                            tuck_bubble(&app_handle);
                        }
                    }
                }
                _ => {}
            }
        });
    }
}

/// Kiểm tra toàn diện trạng thái đăng nhập của WebView2 Messenger
pub fn is_window_logged_in(chat_window: &tauri::WebviewWindow) -> (bool, String) {
    let title = chat_window.title().unwrap_or_default();
    let url_str = chat_window.url().map(|u| u.to_string()).unwrap_or_default();

    // 1. Thẻ xác thực thành công từ injected script (DOM đã load giao diện chat)
    if title.contains("[AUTH_OK]") {
        return (true, String::new());
    }

    // 2. Nhận diện các trang đăng nhập / checkpoint
    let is_login_page = url_str.contains("/login")
        || url_str.contains("/checkpoint")
        || url_str.contains("facebook.com/login")
        || title.to_lowercase().contains("log in to facebook")
        || title.to_lowercase().contains("log into facebook");

    if is_login_page {
        return (false, String::new());
    }

    // 3. Đường dẫn trực tiếp vào cuộc trò chuyện
    let is_inbox_path = url_str.contains("/t/") || url_str.contains("/e2ee/");
    if is_inbox_path {
        return (true, String::new());
    }

    // 4. Nếu đang ở domain messenger.com, có số unread hoặc tiêu đề Messenger hợp lệ
    if url_str.contains("messenger.com") {
        if parse_unread_from_title(&title) > 0 {
            return (true, String::new());
        }
        if title.contains("Messenger") || title.contains("Chats") || title.contains("Đoạn chat") {
            return (true, String::new());
        }
    }

    (false, String::new())
}
