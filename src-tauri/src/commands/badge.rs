use tauri::{AppHandle, Emitter};

// Command nhận số unread từ injected script và phát event cho toàn bộ app
#[tauri::command]
pub fn update_unread_badge(app: AppHandle, count: u32) -> Result<(), String> {
    let _ = app.emit("badge-updated", count);
    Ok(())
}

// Command giả lập / thiết lập trực tiếp số badge (tiện cho test và điều khiển)
#[tauri::command]
pub fn set_badge_count(app: AppHandle, count: u32) -> Result<(), String> {
    let _ = app.emit("badge-updated", count);
    Ok(())
}
