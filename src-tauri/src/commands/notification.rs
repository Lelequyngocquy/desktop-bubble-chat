use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::state::{ChatState, IncomingMessage};

/// Command nhận thông báo mới từ HTML5 Notification Hook (Giai đoạn 1 & 2 & 4)
/// Nhiệm vụ:
/// 1. Bóc tách payload { senderName, body, avatarUrl, threadId, timestamp }
/// 2. Lưu trữ vào ChatState.recent_messages (tối đa 20 tin, khử trùng lặp 3000ms)
/// 3. Cập nhật huy hiệu số đỏ Native Bubble Win32
/// 4. Phát event đồng bộ sang Dashboard Frontend (badge-updated & new-message-received)
/// 5. In log chuẩn ra terminal và ghi file log: [M02:MSG] Từ: 'Nguyễn Văn A' | Lúc: 18:45:10 | Nội dung: 'Chào bạn nhé!'
#[tauri::command]
pub fn report_incoming_notification(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
    sender_name: Option<String>,
    body: Option<String>,
    avatar_url: Option<String>,
    thread_id: Option<String>,
    timestamp: Option<i64>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let s_name = sender_name.unwrap_or_default().trim().to_string();
    let b_text = body.unwrap_or_default().trim().to_string();
    let a_url = avatar_url.unwrap_or_default().trim().to_string();
    let t_id = thread_id.unwrap_or_default().trim().to_string();
    let ts = timestamp.unwrap_or_else(crate::utils::time::now_millis);

    let is_generic_sender = s_name.is_empty()
        || s_name.starts_with("M0")
        || s_name.starts_with("M1")
        || s_name == "M";

    let display_sender = if is_generic_sender {
        format!("Có tin nhắn mới đến Messenger {:02}", idx)
    } else {
        s_name.clone()
    };

    let msg = IncomingMessage {
        account_index: idx,
        sender_name: display_sender.clone(),
        body: b_text.clone(),
        avatar_url: a_url.clone(),
        thread_id: t_id.clone(),
        timestamp: ts,
    };

    // 1. Lưu tin nhắn vào bộ nhớ ChatState (lọc trùng lặp sliding window 3000ms)
    let is_new = state.add_recent_message(msg);

    // 2. Cập nhật unread count & Native Bubble
    let new_count = state.increment_unread_count(idx);
    crate::windows::native_bubble::set_native_bubble_unread_count(idx, new_count);

    // 3. Đồng bộ event sang Frontend Dashboard (Giai đoạn 4)
    let _ = app.emit("badge-updated", serde_json::json!({
        "accountIndex": idx,
        "count": new_count
    }));

    if is_new {
        let _ = app.emit("new-message-received", serde_json::json!({
            "accountIndex": idx,
            "senderName": display_sender,
            "body": b_text,
            "avatarUrl": a_url,
            "threadId": t_id,
            "timestamp": ts,
        }));

        let time_str = crate::utils::time::format_current_time();
        let log_msg = if is_generic_sender {
            format!(
                "[M{:02}:MSG] Có tin nhắn mới đến Messenger {:02} | Lúc: {}",
                idx, idx, time_str
            )
        } else {
            format!(
                "[M{:02}:MSG] Từ: '{}' | Lúc: {} | Nội dung: '{}'",
                idx, display_sender, time_str, b_text
            )
        };
        println!("\x1b[32m{}\x1b[0m", log_msg);
        crate::commands::system::app::log_frontend(Some("info".into()), log_msg);
    }

    Ok(())
}
