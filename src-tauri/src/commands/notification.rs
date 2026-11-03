use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::state::ChatState;

/// Command nhận cờ báo hiệu thông báo mới từ HTML5 Notification Hook (Giai đoạn 1)
/// Nhiệm vụ: Tăng unread count -> Cập nhật huy hiệu số đỏ Native Bubble Win32
#[tauri::command]
pub fn report_incoming_notification(
    _app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);

    // 1. Tăng unread_count trong ChatState
    let new_count = state.increment_unread_count(idx);

    // 2. Cập nhật số unread cho Native Bubble Win32 (tiny-skia vẽ lại badge số đỏ nhọn trỏ vào bubble)
    crate::windows::native_bubble::set_native_bubble_unread_count(idx, new_count);

    println!(
        "\x1b[35m[BE - Notification] Nhận thông báo mới từ M{:02}! Số badge hiện tại: {}\x1b[0m",
        idx, new_count
    );

    Ok(())
}
