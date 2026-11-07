use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::state::{ChatState, IncomingMessage};

/// Command handling incoming notification from HTML5 Notification Hook:
/// 1. Parse payload { senderName, body, avatarUrl, threadId, timestamp }
/// 2. Store in ChatState.recent_messages (max 20, 3000ms deduplication)
/// 3. Update native bubble unread badge
/// 4. Emit dashboard events (badge-updated & new-message-received)
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
        format!("New message for Messenger {:02}", idx)
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
    let is_new = state.add_recent_message(msg);
    let new_count = state.increment_unread_count(idx);
    crate::operating_systems::current::bubble::set_native_bubble_unread_count(idx, new_count);
    crate::utils::trim_controller::schedule_auto_trim_on_message(idx, app.clone());
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
                "[M{:02}:MSG] New message for Messenger {:02} | Time: {}",
                idx, idx, time_str
            )
        } else {
            format!(
                "[M{:02}:MSG] From: '{}' | Time: {} | Body: '{}'",
                idx, display_sender, time_str, b_text
            )
        };
        println!("\x1b[32m{}\x1b[0m", log_msg);
        crate::commands::system::app::log_frontend(app.clone(), Some("info".into()), log_msg);
    }
    Ok(())
}
