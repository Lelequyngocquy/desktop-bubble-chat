use tauri::{AppHandle, Emitter, Manager};
use crate::windows::chat::{is_window_logged_in, MESSENGER_INJECT_SCRIPT};

// Command kiểm tra trạng thái đăng nhập thực tế của Messenger
#[tauri::command]
pub fn check_messenger_auth(app: AppHandle) -> Result<serde_json::Value, String> {
    if let Some(chat_window) = app.get_webview_window("chat-messenger") {
        let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);
        let (is_logged_in, user_name) = is_window_logged_in(&chat_window);
        let url_str = chat_window.url().map(|u| u.to_string()).unwrap_or_default();
        let title = chat_window.title().unwrap_or_default();

        return Ok(serde_json::json!({
            "connected": is_logged_in,
            "name": if is_logged_in { user_name } else { "".to_string() },
            "url": url_str,
            "title": title
        }));
    }
    Ok(serde_json::json!({ "connected": false, "name": "" }))
}

// Command nhận báo cáo trạng thái xác thực từ webview và đồng bộ sang UI
#[tauri::command]
pub fn report_auth_status(app: AppHandle, connected: bool, name: Option<String>) -> Result<(), String> {
    let _ = app.emit("messenger-auth-status", serde_json::json!({
        "connected": connected,
        "name": name
    }));
    Ok(())
}
