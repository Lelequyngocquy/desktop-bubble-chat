use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::platforms::messenger::services::{ChatService, StorageService};
use crate::state::ChatState;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IncomingMessageDto {
    pub account_index: u32,
    pub sender_name: String,
    pub body: Option<String>,
    pub avatar_url: Option<String>,
    pub thread_id: Option<String>,
    pub timestamp: u64,
}

/// Command nhận báo cáo tin nhắn mới từ webview inject script và phát event cho Frontend SSOT
#[tauri::command]
pub fn report_new_message(
    app: AppHandle,
    sender_name: String,
    body: Option<String>,
    avatar_url: Option<String>,
    thread_id: Option<String>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let payload = IncomingMessageDto {
        account_index: idx,
        sender_name,
        body,
        avatar_url,
        thread_id,
        timestamp: now,
    };

    // 1. Phát âm thanh Pop thuần Native trong Rust Core
    crate::utils::native_sound::play_bubble_pop();

    // 2. Gửi Windows Toast Notification thuần Native nếu khung chat của account này đang không mở
    use tauri_plugin_notification::NotificationExt;
    let chat_label = crate::platforms::messenger::windows::get_messenger_window_label(idx);
    let is_chat_open = app
        .get_webview_window(&chat_label)
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false);

    if !is_chat_open {
        let notif_body = payload.body.as_deref().unwrap_or("Bạn có tin nhắn mới");
        let _ = app
            .notification()
            .builder()
            .title(&payload.sender_name)
            .body(notif_body)
            .show();
    }

    app.emit("new-message-received", &payload).map_err(|e| e.to_string())
}

/// Command nhận danh sách hội thoại gần nhất trích xuất an toàn từ webview
#[tauri::command]
pub fn sync_recent_conversations(
    app: AppHandle,
    conversations: Vec<IncomingMessageDto>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    app.emit(
        "recent-conversations-synced",
        serde_json::json!({
            "accountIndex": idx,
            "conversations": conversations,
        }),
    )
    .map_err(|e| e.to_string())
}

/// Command mở trực tiếp khung chat cạnh bong bóng nổi cho tài khoản chỉ định
#[tauri::command]
pub async fn open_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    ChatService::open_chat(&app, &state, idx)
}

/// Command mở trực tiếp một thread hội thoại cụ thể và dock khung chat cạnh Bubble
#[tauri::command]
pub async fn open_thread(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    thread_id: String,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    ChatService::open_chat(&app, &state, idx)?;

    let chat_label = format!("chat-{:02}", idx);
    if let Some(chat_window) = app.get_webview_window(&chat_label) {
        let clean_id = thread_id.trim();
        if !clean_id.is_empty() && !clean_id.starts_with("sender-") {
            let url = format!("https://www.messenger.com/t/{}", clean_id);
            let script = format!("window.location.href = '{}';", url);
            let _ = chat_window.eval(&script);
        }
    }
    Ok(())
}

/// Command bật/tắt khung chat Messenger khi người dùng click vào bong bóng nổi
#[tauri::command]
pub async fn toggle_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<bool, String> {
    let idx = account_index.unwrap_or(1);
    ChatService::toggle_chat(&app, &state, idx)
}

/// Command tải lại trang Messenger cho tài khoản chỉ định
#[tauri::command]
pub fn refresh_messenger(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    ChatService::refresh(&app, idx)
}

/// Command đóng và hủy hẳn cửa sổ webview của tài khoản bị xóa, xóa sạch cache/storage
#[tauri::command]
pub fn close_messenger_window(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    StorageService::clean_account_session(&app, &state, idx)
}

/// Command xóa toàn bộ bộ nhớ đệm, cookies và storage của một tài khoản cụ thể trên ổ cứng
#[tauri::command]
pub fn clear_account_storage(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    StorageService::clean_account_session(&app, &state, idx)
}

/// Command chủ động tải trước Webview Messenger trong nền cho tài khoản chỉ định
#[tauri::command]
pub async fn preload_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let _ = crate::platforms::messenger::windows::get_or_create_messenger_window(&app, idx, Arc::clone(&state))?;
    Ok(())
}
