use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::{ChatState, IncomingMessage};

/// Checks if sender string is a generic placeholder rather than a real display name
fn is_generic_sender_name(name: &str) -> bool {
    name.is_empty()
        || name == "M"
        || (name.len() == 3 && name.starts_with('M') && name[1..].bytes().all(|b| b.is_ascii_digit()))
        || name.starts_with("Messenger ")
}

/// Applies badge count to state, clears recent messages when zero, and updates bubble/UI listeners
pub fn apply_badge_state(app: &AppHandle, state: &ChatState, idx: u32, count: u32) {
    state.set_unread_count(idx, count);
    if count == 0 {
        state.clear_recent_messages_for_account(idx);
    }
    crate::operating_systems::current::bubble::set_native_bubble_unread_count(idx, count);
    let _ = app.emit("badge-updated", serde_json::json!({
        "accountIndex": idx,
        "count": count
    }));
}

/// Triggers sound, pulse, OS toast, and auto-trim when account is unmuted and notifications enabled
fn trigger_alert_effects(app: &AppHandle, state: &ChatState, idx: u32, title: &str, body: &str) {
    crate::utils::trim_controller::schedule_auto_trim_on_message(idx, app.clone());
    let notif_enabled = state.get_user_config().is_notifications_enabled;
    if state.is_account_muted(idx) || !notif_enabled {
        return;
    }
    crate::utils::native_sound::play_bubble_pop();
    crate::operating_systems::current::bubble::trigger_native_bubble_pulse(idx);
    use tauri_plugin_notification::NotificationExt;
    let chat_label = crate::platforms::messenger::windows::get_messenger_window_label(idx);
    let is_chat_open = app
        .get_webview_window(&chat_label)
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false);
    if !is_chat_open {
        let notif_body = if body.is_empty() { "You have a new message" } else { body };
        let _ = app.notification().builder().title(title).body(notif_body).show();
    }
}

/// Unified pipeline handling an incoming message event with 3s dedupe and mute enforcement
#[allow(clippy::too_many_arguments)]
pub fn dispatch_incoming_message(
    app: &AppHandle,
    state: &ChatState,
    idx: u32,
    raw_sender: &str,
    raw_body: &str,
    raw_avatar: &str,
    raw_thread: &str,
    ts: i64,
) -> bool {
    let s_name = raw_sender.trim().to_string();
    let b_text = raw_body.trim().to_string();
    let a_url = raw_avatar.trim().to_string();
    let t_id = raw_thread.trim().to_string();
    let is_generic_sender = is_generic_sender_name(&s_name);
    let display_sender = if is_generic_sender {
        format!("New message for Messenger {:02}", idx)
    } else {
        s_name
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
    if !is_new {
        return false;
    }
    let new_count = state.increment_unread_count(idx);
    apply_badge_state(app, state, idx, new_count);
    trigger_alert_effects(app, state, idx, &display_sender, &b_text);
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
    true
}

/// Synchronizes unread badge count from Messenger window title without duplicating recent alerts
pub fn sync_unread_from_window_title(app: &AppHandle, state: &ChatState, idx: u32, title: &str) {
    if !title.contains("Messenger") {
        return;
    }
    let count = crate::platforms::messenger::parser::parse_unread_from_title(title);
    let prev = state.get_unread_count(idx);
    if count == prev {
        return;
    }
    apply_badge_state(app, state, idx, count);
    if count > prev && count > 0 {
        let ts = crate::utils::time::now_millis();
        if !state.has_recent_message_within(idx, ts, 3000) {
            let display_title = format!("New message for Messenger {:02}", idx);
            let msg = IncomingMessage {
                account_index: idx,
                sender_name: display_title.clone(),
                body: "New message".to_string(),
                avatar_url: String::new(),
                thread_id: String::new(),
                timestamp: ts,
            };
            if state.add_recent_message(msg) {
                trigger_alert_effects(app, state, idx, &display_title, "New message");
                let time_str = crate::utils::time::format_current_time();
                let log_msg = format!(
                    "[M{:02}:MSG] New message for Messenger {:02} | Time: {}",
                    idx, idx, time_str
                );
                println!("\x1b[32m{}\x1b[0m", log_msg);
                crate::commands::system::app::log_frontend(app.clone(), Some("info".into()), log_msg);
                let _ = app.emit("new-message-received", serde_json::json!({
                    "accountIndex": idx,
                    "senderName": display_title,
                    "body": "New message",
                    "avatarUrl": "",
                    "threadId": "",
                    "timestamp": ts,
                }));
            }
        }
    }
}

/// Command handling incoming notification from HTML5 Notification Hook via unified pipeline
#[tauri::command]
#[allow(clippy::too_many_arguments)]
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
    let idx = ChatState::validate_account_index(account_index)?;
    let s_name = sender_name.unwrap_or_default();
    let b_text = body.unwrap_or_default();
    let a_url = avatar_url.unwrap_or_default();
    let t_id = thread_id.unwrap_or_default();
    let ts = timestamp.unwrap_or_else(crate::utils::time::now_millis);
    dispatch_incoming_message(&app, &state, idx, &s_name, &b_text, &a_url, &t_id, ts);
    Ok(())
}
