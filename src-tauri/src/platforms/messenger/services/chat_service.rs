use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::movement::{BubbleMovementEngine, BubblePhysicsConfig, ScreenMetrics};
use crate::platforms::messenger::parser::parse_unread_from_title;
use crate::platforms::messenger::constants::MESSENGER_INJECT_SCRIPT;
use crate::platforms::messenger::windows::{
    get_messenger_window_label, get_or_create_messenger_window, hide_all_other_messenger_windows,
};
use crate::state::ChatState;
use crate::windows::bubble::{ensure_bubble_positioned_and_visible, get_bubble_window_label, tuck_bubble};

pub struct ChatService;

impl ChatService {
    /// Dock cửa sổ chat Messenger vào đúng Bubble của tài khoản tương ứng
    pub fn dock_chat_window(app: &AppHandle, chat_window: &tauri::WebviewWindow, account_index: u32) {
        let bubble_label = get_bubble_window_label(account_index);
        let bubble_win = app.get_webview_window(&bubble_label);

        if let Some(bubble_window) = bubble_win {
            if let Ok(Some(monitor)) = bubble_window.current_monitor() {
                let screen = ScreenMetrics::from_monitor(&monitor);
                let scale = screen.scale_factor;
                let config = BubblePhysicsConfig::default();

                let chat_w = (config.chat_width * scale).round() as i32;
                let chat_h = (config.chat_height * scale).round() as i32;
                let bubble_w = (config.size * scale).round() as i32;
                let gap = (config.chat_gap * scale).round() as i32;
                let margin = (config.margin * scale).round() as i32;
                let taskbar_margin = (config.taskbar_margin * scale).round() as i32;

                let _ = chat_window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                    width: chat_w as u32,
                    height: chat_h as u32,
                }));

                if let Ok(bubble_pos) = bubble_window.outer_position() {
                    let untucked_pos = BubbleMovementEngine::calculate_hover_pos(
                        &screen,
                        &config,
                        bubble_pos.x,
                        bubble_pos.y,
                        true,
                    );
                    let _ = bubble_window.set_position(tauri::Position::Physical(untucked_pos));

                    let mid_x = screen.x + screen.width / 2;

                    let target_x = if untucked_pos.x + bubble_w / 2 >= mid_x {
                        untucked_pos.x - chat_w - gap
                    } else {
                        untucked_pos.x + bubble_w + gap
                    };

                    let min_x = screen.x + margin;
                    let max_x = screen.x + screen.width - chat_w - margin;
                    let final_x = target_x.clamp(min_x, max_x);

                    let min_y = screen.y + margin;
                    let max_y = screen.y + screen.height - chat_h - taskbar_margin;
                    let final_y = untucked_pos.y.clamp(min_y, max_y);

                    let _ = chat_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x: final_x,
                        y: final_y,
                    }));
                }
            }
        }

        // Đảm bảo tất cả các bubble khác đều bị thụt vào mép (tuck), không được thò ra ngoài
        for other_idx in 1..=5 {
            if other_idx != account_index {
                tuck_bubble(app, Some(other_idx));
            }
        }
    }

    /// Mở khung chat Messenger cho tài khoản chỉ định
    pub fn open_chat(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let _ = ensure_bubble_positioned_and_visible(app, idx);

        let chat_window = get_or_create_messenger_window(app, idx, Arc::clone(state))?;

        let is_already_visible = chat_window.is_visible().unwrap_or(false);
        if is_already_visible {
            return Ok(());
        }

        // Ẩn tất cả các cửa sổ chat của tài khoản khác
        hide_all_other_messenger_windows(app, &chat_window.label());

        // Đảm bảo URL luôn ở messenger.com, ngăn chặn tuyệt đối việc hiển thị giao diện facebook.com
        if let Ok(curr_url) = chat_window.url() {
            let u = curr_url.to_string();
            if !u.contains("messenger.com") || u.contains("facebook.com") {
                println!("[BE - Chat] Cửa sổ đang ở URL '{}', chuyển hướng bắt buộc về https://www.messenger.com/", u);
                if let Ok(target) = tauri::Url::parse("https://www.messenger.com/") {
                    let _ = chat_window.navigate(target);
                }
            }
        }

        state.set_window_auth_mode(&chat_window.label(), false);
        let _ = chat_window.set_decorations(false);
        let _ = chat_window.set_skip_taskbar(true);
        Self::dock_chat_window(app, &chat_window, idx);
        let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);

        if let Ok(title) = chat_window.title() {
            let count = parse_unread_from_title(&title);
            let _ = app.emit("badge-updated", serde_json::json!({
                "accountIndex": idx,
                "count": count
            }));
        }

        println!("[BE - Chat] Mở khung chat Messenger {:02} docked cạnh Bubble (Hover 0.1s)", idx);
        chat_window.show().map_err(|e| e.to_string())?;
        chat_window.set_focus().map_err(|e| e.to_string())?;
        let _ = app.emit("bubble-visibility-changed", serde_json::json!({
            "accountIndex": idx,
            "visible": true
        }));
        let _ = app.emit("chat-visibility-changed", serde_json::json!({
            "accountIndex": idx,
            "visible": true,
            "windowLabel": chat_window.label()
        }));

        Ok(())
    }

    /// Đóng khung chat Messenger cho tài khoản chỉ định và thụt bubble vào mép
    pub fn hide_chat(
        app: &AppHandle,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        let chat_window = app
            .get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });

        if let Some(chat_window) = chat_window {
            if chat_window.is_visible().unwrap_or(false) {
                // Bước 1: Ẩn khung chat trước ngay lập tức
                let _ = chat_window.hide();

                // Bước 2: Kích hoạt hoạt ảnh trượt ẩn (tuck) của Bubble ngay sau đó
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    // Chờ một nhịp ngắn (60ms) để khung chat hoàn tất ẩn trên OS
                    std::thread::sleep(std::time::Duration::from_millis(60));

                    // Báo cho frontend Bubble chạy hoạt ảnh trượt ẩn (tuck)
                    let _ = app_handle.emit("chat-visibility-changed", serde_json::json!({
                        "accountIndex": idx,
                        "visible": false,
                        "windowLabel": label
                    }));

                    // Chờ hoạt ảnh trượt ở frontend hoàn thành (180ms ease-out cubic + 40ms buffer)
                    std::thread::sleep(std::time::Duration::from_millis(220));

                    // Khóa cứng tọa độ tuck nếu khung chat vẫn đang ẩn (chưa bị hover mở lại)
                    if let Some(cw) = app_handle
                        .get_webview_window(&label)
                        .or_else(|| if idx == 1 { app_handle.get_webview_window("chat-messenger") } else { None })
                    {
                        if !cw.is_visible().unwrap_or(false) {
                            tuck_bubble(&app_handle, Some(idx));
                        }
                    }
                });
            }
        }
        Ok(())
    }

    /// Bật/tắt khung chat Messenger
    pub fn toggle_chat(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<bool, String> {
        let idx = account_index;
        let chat_window = get_or_create_messenger_window(app, idx, Arc::clone(state))?;

        let is_visible = chat_window.is_visible().unwrap_or(false);

        if is_visible {
            let _ = Self::hide_chat(app, idx);
            Ok(false)
        } else {
            let _ = Self::open_chat(app, state, idx);
            Ok(true)
        }
    }

    /// Tải lại trang Messenger cho tài khoản chỉ định
    pub fn refresh(
        app: &AppHandle,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        println!("[BE - Messenger] Tải lại phiên Messenger {:02}", idx);
        let win = app
            .get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });

        if let Some(chat_window) = win {
            let _ = chat_window.eval("window.location.reload();");
        }
        Ok(())
    }
}
