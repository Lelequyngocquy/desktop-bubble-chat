use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::ChatState;
use crate::utils::parser::parse_unread_from_title;
use crate::utils::positioning::{
    dock_chat_window, ensure_bubble_positioned_and_visible, is_cursor_over_bubble, tuck_bubble,
};
use crate::utils::time::now_millis;
use crate::windows::chat::MESSENGER_INJECT_SCRIPT;

// Command mở bong bóng và mở khung chat đăng nhập đồng nhất
#[tauri::command]
pub fn open_messenger_auth(app: AppHandle, state: State<'_, Arc<ChatState>>) -> Result<(), String> {
    // 1. Hiển thị bong bóng nổi nếu đang ẩn và đặt vị trí hợp lý
    let _ = ensure_bubble_positioned_and_visible(&app);

    // 2. Lấy cửa sổ chat Messenger
    let chat_window = match app.get_webview_window("chat-messenger") {
        Some(w) => w,
        None => return Err("Chat window 'chat-messenger' not found".into()),
    };

    // 3. Bật Auth Mode: Cửa sổ tuyệt đối KHÔNG tự động ẩn khi mất focus (blur) trong lúc nhập liệu / giải Captcha
    state.is_auth_mode.store(true, Ordering::SeqCst);

    // 4. Định vị chuẩn 420x620 borderless docked ngay cạnh bong bóng
    dock_chat_window(&app, &chat_window);

    // 5. Nạp injected script
    let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);

    // 6. Hiển thị và đưa lên tiêu điểm
    chat_window.show().map_err(|e| e.to_string())?;
    chat_window.set_focus().map_err(|e| e.to_string())?;

    // 7. Đồng bộ trạng thái bong bóng và chat bật sang frontend
    let _ = app.emit("bubble-visibility-changed", true);
    let _ = app.emit("chat-visibility-changed", true);

    Ok(())
}

// Command mở trực tiếp khung chat cạnh bong bóng nổi (không toggle tắt)
#[tauri::command]
pub fn open_chat(app: AppHandle, state: State<'_, Arc<ChatState>>) -> Result<(), String> {
    let _ = ensure_bubble_positioned_and_visible(&app);

    let chat_window = match app.get_webview_window("chat-messenger") {
        Some(w) => w,
        None => return Err("Chat window 'chat-messenger' not found".into()),
    };

    state.is_auth_mode.store(false, Ordering::SeqCst);
    dock_chat_window(&app, &chat_window);
    let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);

    if let Ok(title) = chat_window.title() {
        let count = parse_unread_from_title(&title);
        let _ = app.emit("badge-updated", count);
    }

    chat_window.show().map_err(|e| e.to_string())?;
    chat_window.set_focus().map_err(|e| e.to_string())?;
    let _ = app.emit("bubble-visibility-changed", true);
    let _ = app.emit("chat-visibility-changed", true);

    Ok(())
}

// Command bật/tắt khung chat Messenger khi người dùng click vào bong bóng nổi
#[tauri::command]
pub fn toggle_chat(app: AppHandle, state: State<'_, Arc<ChatState>>) -> Result<bool, String> {
    let chat_window = match app.get_webview_window("chat-messenger") {
        Some(w) => w,
        None => return Err("Chat window 'chat-messenger' not found".into()),
    };

    let is_visible = chat_window.is_visible().unwrap_or(false);

    if is_visible {
        state.last_blur_time.store(now_millis(), Ordering::SeqCst);
        chat_window.hide().map_err(|e| e.to_string())?;
        let _ = app.emit("chat-visibility-changed", false);
        if let Some(bubble_win) = app.get_webview_window("bubble") {
            if !is_cursor_over_bubble(&bubble_win) {
                tuck_bubble(&app);
            }
        }
        Ok(false)
    } else {
        let last_blur = state.last_blur_time.load(Ordering::SeqCst);
        if now_millis() - last_blur < 250 {
            return Ok(false);
        }

        dock_chat_window(&app, &chat_window);
        let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);

        if let Ok(title) = chat_window.title() {
            let count = parse_unread_from_title(&title);
            let _ = app.emit("badge-updated", count);
        }

        chat_window.show().map_err(|e| e.to_string())?;
        chat_window.set_focus().map_err(|e| e.to_string())?;
        let _ = app.emit("chat-visibility-changed", true);
        Ok(true)
    }
}

// Command ngắt kết nối Messenger, điều hướng về login và ẩn bong bóng nổi
#[tauri::command]
pub fn disconnect_messenger(app: AppHandle, state: State<'_, Arc<ChatState>>) -> Result<(), String> {
    state.is_auth_mode.store(false, Ordering::SeqCst);
    if let Some(chat_window) = app.get_webview_window("chat-messenger") {
        let _ = chat_window.eval("window.location.href = 'https://www.messenger.com/login/';");
        let _ = chat_window.hide();
        let _ = chat_window.set_decorations(false);
    }
    if let Some(bubble_window) = app.get_webview_window("bubble") {
        let _ = bubble_window.hide();
    }
    let _ = app.emit("messenger-auth-status", serde_json::json!({ "connected": false }));
    let _ = app.emit("badge-updated", 0);
    let _ = app.emit("bubble-visibility-changed", false);
    let _ = app.emit("chat-visibility-changed", false);
    tuck_bubble(&app);
    Ok(())
}

// Command tải lại trang Messenger
#[tauri::command]
pub fn refresh_messenger(app: AppHandle) -> Result<(), String> {
    if let Some(chat_window) = app.get_webview_window("chat-messenger") {
        let _ = chat_window.eval("window.location.reload();");
    }
    Ok(())
}
