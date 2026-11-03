use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::state::ChatState;

// Command nhận số unread từ injected script và phát event cho toàn bộ app
#[tauri::command]
pub fn update_unread_badge(
    app: AppHandle,
    count: u32,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    crate::windows::native_bubble::set_native_bubble_unread_count(idx, count);
    let _ = app.emit("badge-updated", serde_json::json!({
        "accountIndex": idx,
        "count": count
    }));
    Ok(())
}

// Command giả lập / thiết lập trực tiếp số badge (tiện cho test và điều khiển)
#[tauri::command]
pub fn set_badge_count(
    app: AppHandle,
    count: u32,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    crate::windows::native_bubble::set_native_bubble_unread_count(idx, count);
    let _ = app.emit("badge-updated", serde_json::json!({
        "accountIndex": idx,
        "count": count
    }));
    Ok(())
}

/// Command truy vấn số đếm chưa đọc của một tài khoản cụ thể (SSOT)
#[tauri::command]
pub fn get_unread_count(
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<u32, String> {
    let idx = account_index.unwrap_or(1);
    Ok(state.get_unread_count(idx))
}

/// Command truy vấn toàn bộ số đếm chưa đọc của tất cả tài khoản từ SSOT Backend (Giai đoạn 4)
#[tauri::command]
pub fn get_all_unread_counts(
    state: State<'_, Arc<ChatState>>,
) -> Result<std::collections::HashMap<u32, u32>, String> {
    if let Ok(map) = state.unread_counts.lock() {
        Ok(map.clone())
    } else {
        Ok(std::collections::HashMap::new())
    }
}
