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

    // 2. Kích hoạt hiệu ứng sóng lan tỏa Pulse trên Bong bóng Native Win32 (Giai đoạn 3)
    crate::windows::native_bubble::trigger_native_bubble_pulse(idx);

    // 3. Gửi Windows Toast Notification thuần Native nếu khung chat của account này đang không mở
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

/// Command bật/tắt âm thanh tab webview của tài khoản Messenger (Mute/Unmute tab)
#[tauri::command]
pub fn set_account_muted(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
    muted: bool,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    state.set_account_muted(idx, muted);
    state.update_user_config(&app, |c| {
        if muted {
            if !c.muted_accounts.contains(&idx) {
                c.muted_accounts.push(idx);
                c.muted_accounts.sort();
            }
        } else {
            c.muted_accounts.retain(|&x| x != idx);
        }
        c.is_notifications_enabled = c.muted_accounts.len() < 5;
    });
    let label = crate::platforms::messenger::windows::get_messenger_window_label(idx);

    println!(
        "\x1b[34m[BE - Audio] {} âm thanh tab Messenger {:02}\x1b[0m",
        if muted { "Tắt (Muted)" } else { "Bật (Unmuted)" },
        idx
    );

    if let Some(win) = app.get_webview_window(&label) {
        #[cfg(target_os = "windows")]
        {
            let _ = win.with_webview(move |webview| {
                unsafe {
                    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_8;
                    use windows_core::Interface;
                    let controller = webview.controller();
                    if let Ok(core_webview) = controller.CoreWebView2() {
                        if let Ok(webview_8) = core_webview.cast::<ICoreWebView2_8>() {
                            let _ = webview_8.SetIsMuted(muted);
                        }
                    }
                }
            });

            // Đồng thời eval script tắt Audio elements trong DOM
            let js = format!(
                r#"
                (function() {{
                    window.__isMuted = {muted};
                    document.querySelectorAll('audio, video').forEach(function(el) {{
                        el.muted = {muted};
                    }});
                }})();
                "#
            );
            let _ = win.eval(&js);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let js = format!("document.querySelectorAll('audio, video').forEach(el => el.muted = {});", muted);
            let _ = win.eval(&js);
        }
    }

    Ok(())
}
