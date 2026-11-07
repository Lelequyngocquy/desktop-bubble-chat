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

/// Command receiving new message notification from injected script
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
    crate::utils::native_sound::play_bubble_pop();
    crate::windows::native_bubble::trigger_native_bubble_pulse(idx);
    crate::utils::trim_controller::schedule_auto_trim_on_message(idx, app.clone());
    use tauri_plugin_notification::NotificationExt;
    let chat_label = crate::platforms::messenger::windows::get_messenger_window_label(idx);
    let is_chat_open = app
        .get_webview_window(&chat_label)
        .map(|w| w.is_visible().unwrap_or(false))
        .unwrap_or(false);
    if !is_chat_open {
        let notif_body = payload.body.as_deref().unwrap_or("You have a new message");
        let _ = app
            .notification()
            .builder()
            .title(&payload.sender_name)
            .body(notif_body)
            .show();
    }
    app.emit("new-message-received", &payload).map_err(|e| e.to_string())
}

/// Command receiving recent conversations extracted from webview
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

/// Command opening chat window docked next to bubble for specified account
#[tauri::command]
pub async fn open_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    ChatService::open_chat(&app, &state, idx)
}

/// Command opening a specific conversation thread and docking next to bubble
#[tauri::command]
pub async fn open_thread(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    thread_id: String,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    let is_connected = state.get_account_auth(idx).map(|a| a.connected).unwrap_or(false);
    if !is_connected || state.is_account_suspended(idx) {
        return Err(format!("Account {} is not active or connected", idx));
    }
    ChatService::open_chat(&app, &state, idx)?;
    let chat_label = crate::platforms::messenger::windows::get_messenger_window_label(idx);
    let chat_win = app.get_webview_window(&chat_label).or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });
    if let Some(chat_window) = chat_win {
        let clean_id = thread_id.trim();
        if !clean_id.is_empty() && !clean_id.starts_with("sender-") {
            let url = format!("https://www.messenger.com/t/{}", clean_id);
            let script = format!("window.location.href = '{}';", url);
            let _ = chat_window.eval(&script);
        }
    }
    Ok(())
}

/// Command toggling Messenger chat window
#[tauri::command]
pub async fn toggle_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<bool, String> {
    let idx = account_index.unwrap_or(1);
    ChatService::toggle_chat(&app, &state, idx)
}

/// Command reloading Messenger page for specified account
#[tauri::command]
pub fn refresh_messenger(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    ChatService::refresh(&app, idx)
}

/// Command closing and destroying webview window for deleted account
#[tauri::command]
pub fn close_messenger_window(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    StorageService::clean_account_session(&app, &state, idx)
}

/// Command clearing storage and cookies for account
#[tauri::command]
pub fn clear_account_storage(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);
    StorageService::clean_account_session(&app, &state, idx)
}

/// Command preloading Messenger webview in background
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

/// Command setting account tab audio muted state
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
        "\x1b[34m[BE - Audio] {} Messenger {:02} tab audio\x1b[0m",
        if muted { "Muting" } else { "Unmuting" },
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
