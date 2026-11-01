use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::movement::{BubbleMovementEngine, ScreenMetrics};
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
        #[cfg(target_os = "windows")]
        let native_bounds = crate::windows::native_bubble::NativeBubbleManager::global().get_bounds(account_index);
        #[cfg(not(target_os = "windows"))]
        let native_bounds: Option<(i32, i32, u32, u32)> = None;

        let bubble_label = get_bubble_window_label(account_index);
        let webview_bubble = app.get_webview_window(&bubble_label);

        // 1. Xác định tọa độ (X, Y, Width) của Bubble
        let (bubble_x, bubble_y, bubble_w) = if let Some((bx, by, bw, _)) = native_bounds {
            (bx, by, bw as i32)
        } else if let Some(bw) = &webview_bubble {
            let pos = bw.outer_position().unwrap_or_default();
            let size = bw.outer_size().unwrap_or_default();
            (pos.x, pos.y, size.width as i32)
        } else if let Some(cs) = app.try_state::<Arc<ChatState>>() {
            if let Some(rect) = cs.get_bubble_rect(account_index) {
                (rect.x, rect.y, rect.width)
            } else {
                (0, 0, 56)
            }
        } else {
            (0, 0, 56)
        };

        // 2. Xác định chính xác Monitor chứa Bubble (hỗ trợ hoàn hảo đa màn hình)
        let monitor = app.available_monitors().ok().and_then(|monitors| {
            monitors.into_iter().find(|m| {
                let pos = m.position();
                let size = m.size();
                bubble_x >= pos.x
                    && bubble_x < pos.x + size.width as i32
                    && bubble_y >= pos.y
                    && bubble_y < pos.y + size.height as i32
            })
        })
        .or_else(|| chat_window.current_monitor().ok().flatten())
        .or_else(|| app.get_webview_window("main").and_then(|m| m.current_monitor().ok().flatten()))
        .or_else(|| app.primary_monitor().ok().flatten());

        if let Some(mon) = monitor {
            let screen = ScreenMetrics::from_monitor(&mon);
            let scale = screen.scale_factor;
            let config = crate::movement::get_bubble_physics_config(app);

            let chat_w = (config.chat_width * scale).round() as i32;
            let chat_h = (config.chat_height * scale).round() as i32;
            let gap = (config.chat_gap * scale).round() as i32;
            let margin = (config.margin * scale).round() as i32;
            let taskbar_margin = (config.taskbar_margin * scale).round() as i32;

            let _ = chat_window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: chat_w as u32,
                height: chat_h as u32,
            }));

            // Tính toán vị trí khi trượt mở ra (Untucked X)
            let untucked_pos = BubbleMovementEngine::calculate_hover_pos(
                &screen,
                &config,
                bubble_x,
                bubble_y,
                true,
            );

            // Nếu là Webview Bubble, cập nhật lại tọa độ untuck cho cửa sổ
            if let Some(bw) = &webview_bubble {
                let _ = bw.set_position(tauri::Position::Physical(untucked_pos));
            }

            // Tính toán X của khung chat: nằm bên trái nếu Bubble ở mép phải, hoặc bên phải nếu Bubble ở mép trái
            let mid_x = screen.x + screen.width / 2;
            let target_x = if untucked_pos.x + bubble_w / 2 >= mid_x {
                untucked_pos.x - chat_w - gap
            } else {
                untucked_pos.x + bubble_w + gap
            };

            let min_x = screen.x + margin;
            let max_x = screen.x + screen.width - chat_w - margin;
            let final_x = target_x.clamp(min_x, max_x);

            // CĂN ĐỈNH NGANG BẰNG (Top-Aligned): Đỉnh khung chat ngang bằng đỉnh Bubble
            let bubble_top = bubble_y;
            let min_y = screen.y + margin;
            let max_y = (screen.y + screen.height - chat_h - taskbar_margin).max(min_y);
            let final_y = bubble_top.clamp(min_y, max_y);

            let _ = chat_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: final_x,
                y: final_y,
            }));
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

        // Chỉ định vị/hiển thị lại Bubble nếu bong bóng này chưa active trên màn hình
        let is_bubble_already_active = {
            #[cfg(target_os = "windows")]
            {
                crate::windows::native_bubble::NativeBubbleManager::global().has_bubble(idx)
            }
            #[cfg(not(target_os = "windows"))]
            {
                let bubble_label = get_bubble_window_label(idx);
                app.get_webview_window(&bubble_label)
                    .map(|w| w.is_visible().unwrap_or(false))
                    .unwrap_or(false)
            }
        };

        if !is_bubble_already_active {
            let _ = ensure_bubble_positioned_and_visible(app, idx);
        }

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
            if u.contains("facebook.com") || (!u.is_empty() && u != "about:blank" && !u.contains("messenger.com")) {
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
