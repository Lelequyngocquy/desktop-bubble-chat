use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::platforms::messenger::services::ChatService;
use crate::platforms::messenger::windows::get_messenger_window_label;
use crate::state::ChatState;
use crate::utils::win32::{is_cursor_over_bubble, is_cursor_over_window, is_lbutton_down};
use crate::windows::bubble::get_bubble_window_label;

/// Luồng giám sát toàn cục: Hover 0.1s trên Bubble để tự động mở Chat, rời khỏi cả Chat lẫn Bubble thì tự động đóng
pub fn start_hover_and_autoclose_monitor(app: AppHandle, chat_state: Arc<ChatState>) {
    std::thread::spawn(move || {
        let mut current_hover_bubble: Option<u32> = None;
        let mut hover_start: Option<Instant> = None;
        let mut already_opened_for_hover = false;
        let mut outside_open_chat_start: Option<Instant> = None;

        loop {
            std::thread::sleep(Duration::from_millis(30));

            let lbutton_down = is_lbutton_down();

            // 1. Tìm xem con trỏ chuột hiện tại đang nằm trên Bubble nào
            let mut detected_hover_bubble: Option<u32> = None;
            if !lbutton_down {
                for idx in 1..=5 {
                    let label = get_bubble_window_label(idx);
                    if let Some(bubble_win) = app.get_webview_window(&label) {
                        if bubble_win.is_visible().unwrap_or(false) && is_cursor_over_bubble(&bubble_win) {
                            detected_hover_bubble = Some(idx);
                            break;
                        }
                    }
                }
            }

            // 2. Logic Hover 0.1s (100ms) để tự động mở Chat tương ứng
            if let Some(hovered_idx) = detected_hover_bubble {
                if current_hover_bubble == Some(hovered_idx) {
                    if !already_opened_for_hover {
                        if let Some(start) = hover_start {
                            if start.elapsed() >= Duration::from_millis(100) {
                                let _ = ChatService::open_chat(&app, &chat_state, hovered_idx);
                                already_opened_for_hover = true;
                            }
                        }
                    }
                } else {
                    current_hover_bubble = Some(hovered_idx);
                    hover_start = Some(Instant::now());
                    already_opened_for_hover = false;
                }
            } else {
                current_hover_bubble = None;
                hover_start = None;
                already_opened_for_hover = false;
            }

            // 3. Logic Auto-Close khi chuột rời khỏi cả Chat lẫn Bubble tương ứng
            let mut active_chat_info: Option<(u32, tauri::WebviewWindow)> = None;
            for idx in 1..=5 {
                let label = get_messenger_window_label(idx);
                if let Some(chat_win) = app.get_webview_window(&label) {
                    if chat_win.is_visible().unwrap_or(false) {
                        if !chat_state.is_window_in_auth_mode(&label) {
                            active_chat_info = Some((idx, chat_win));
                        }
                        break;
                    }
                }
            }

            if let Some((active_idx, chat_win)) = active_chat_info {
                let bubble_label = get_bubble_window_label(active_idx);
                let over_chat = is_cursor_over_window(&chat_win);
                let over_bubble = app
                    .get_webview_window(&bubble_label)
                    .map(|b| is_cursor_over_bubble(&b))
                    .unwrap_or(false);

                if over_chat || over_bubble {
                    outside_open_chat_start = None;
                } else {
                    match outside_open_chat_start {
                        None => {
                            outside_open_chat_start = Some(Instant::now());
                        }
                        Some(start_time) => {
                            if start_time.elapsed() >= Duration::from_millis(500) {
                                let _ = ChatService::hide_chat(&app, active_idx);
                                outside_open_chat_start = None;
                            }
                        }
                    }
                }
            } else {
                outside_open_chat_start = None;
            }
        }
    });
}
