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

/// Command receiving new message notification via unified pipeline
#[tauri::command]
pub fn report_new_message(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    sender_name: String,
    body: Option<String>,
    avatar_url: Option<String>,
    thread_id: Option<String>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = ChatState::validate_account_index(account_index)?;
    let ts = crate::utils::time::now_millis();
    crate::commands::notification::dispatch_incoming_message(
        &app,
        &state,
        idx,
        &sender_name,
        body.as_deref().unwrap_or(""),
        avatar_url.as_deref().unwrap_or(""),
        thread_id.as_deref().unwrap_or(""),
        ts,
    );
    Ok(())
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
    let idx = ChatState::validate_account_index(account_index)?;
    ChatService::open_chat(&app, &state, idx)
}

/// Validates a Messenger thread ID against safe path characters and rejects synthetic prefixes
fn is_valid_thread_id(thread_id: &str) -> bool {
    let clean = thread_id.trim();
    !clean.is_empty()
        && !clean.starts_with("sender-")
        && !clean.starts_with("account-msg-")
        && clean.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// Command opening a specific conversation thread and docking next to bubble
#[tauri::command]
pub async fn open_thread(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    thread_id: String,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = ChatState::validate_account_index(account_index)?;
    let is_connected = state.get_account_auth(idx).map(|a| a.connected).unwrap_or(false);
    if !is_connected || state.is_account_suspended(idx) {
        return Err(format!("Account {} is not active or connected", idx));
    }
    ChatService::open_chat(&app, &state, idx)?;
    if let Some(chat_window) = crate::platforms::messenger::windows::get_messenger_webview(&app, idx) {
        let clean_id = thread_id.trim();
        if is_valid_thread_id(clean_id) {
            let url = format!("https://www.messenger.com/t/{}", clean_id);
            if let Ok(quoted_url) = serde_json::to_string(&url) {
                let script = format!("window.location.href = {};", quoted_url);
                let _ = chat_window.eval(&script);
            }
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
    let idx = ChatState::validate_account_index(account_index)?;
    ChatService::toggle_chat(&app, &state, idx)
}

/// Command reloading Messenger page for specified account
#[tauri::command]
pub fn refresh_messenger(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = ChatState::validate_account_index(account_index)?;
    ChatService::refresh(&app, idx)
}

/// Command closing and destroying webview window for deleted account
#[tauri::command]
pub fn close_messenger_window(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = ChatState::validate_account_index(account_index)?;
    StorageService::clean_account_session(&app, &state, idx)
}

/// Command clearing storage and cookies for account
#[tauri::command]
pub fn clear_account_storage(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = ChatState::validate_account_index(account_index)?;
    StorageService::clean_account_session(&app, &state, idx)
}

/// Command preloading Messenger webview in background
#[tauri::command]
pub async fn preload_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = ChatState::validate_account_index(account_index)?;
    let _ = crate::platforms::messenger::windows::get_or_create_messenger_window(&app, idx, Arc::clone(&state))?;
    Ok(())
}

/// SSOT core helper applying account audio mute state across cache, bubble alert, and webview
pub fn apply_account_muted(
    app: &AppHandle,
    state: &ChatState,
    account_index: u32,
    muted: bool,
) -> Result<(), String> {
    let idx = ChatState::validate_account_index(Some(account_index))?;
    let was_alert = state.is_account_alert_active(idx);
    state.set_account_muted(idx, muted);
    state.update_user_config(app, |c| {
        if muted {
            if !c.muted_accounts.contains(&idx) {
                c.muted_accounts.push(idx);
                c.muted_accounts.sort();
            }
        } else {
            c.muted_accounts.retain(|&x| x != idx);
        }
    });
    crate::commands::notification::sync_bubble_alert_state(app, state, idx, Some(was_alert));
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
                    if let Ok(core_webview) = controller.CoreWebView2()
                        && let Ok(webview_8) = core_webview.cast::<ICoreWebView2_8>() {
                            let _ = webview_8.SetIsMuted(muted);
                        }
                }
            });
            let js = format!(
                "(function(){{window.__isMuted={muted};document.querySelectorAll('audio, video').forEach(function(el){{el.muted={muted};}});}})();"
            );
            let _ = win.eval(&js);
        }
        #[cfg(target_os = "linux")]
        {
            crate::operating_systems::linux::memory::LinuxMemoryOptimizer::set_webview_muted(&win, muted);
            let js = format!(
                "(function(){{window.__isMuted={muted};document.querySelectorAll('audio, video').forEach(function(el){{el.muted={muted};}});}})();"
            );
            let _ = win.eval(&js);
        }
        #[cfg(target_os = "macos")]
        {
            let js = format!(
                "(function(){{window.__isMuted={muted};if(!window.__dbcAudioGuardInstalled){{window.__dbcAudioGuardInstalled=true;var origPlay=HTMLMediaElement.prototype.play;HTMLMediaElement.prototype.play=function(){{if(window.__isMuted){{this.muted=true;}}return origPlay.apply(this,arguments);}};}}document.querySelectorAll('audio, video').forEach(function(el){{el.muted={muted};}});}})();"
            );
            let _ = win.eval(&js);
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let js = format!("document.querySelectorAll('audio, video').forEach(el => el.muted = {});", muted);
            let _ = win.eval(&js);
        }
    }
    crate::presentation::bubble::dock::sync_action_dock_for_account(app, idx);
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
    let idx = ChatState::validate_account_index(account_index)?;
    apply_account_muted(&app, &state, idx, muted)
}

/// Persists a user-dragged custom chat window size without re-docking the live window
pub fn record_resized_chat_window_size(
    app: &AppHandle,
    state: &ChatState,
    account_index: u32,
    width: u32,
    height: u32,
) {
    let clamped_w = width.max(crate::utils::user_cache::MIN_CHAT_WINDOW_WIDTH);
    let clamped_h = height.max(crate::utils::user_cache::MIN_CHAT_WINDOW_HEIGHT);
    if state.get_chat_window_custom_size(account_index).is_none() {
        return;
    }
    let current_cfg = state.get_user_config().resolved_custom_chat_size(account_index);
    state.set_chat_window_custom_size(account_index, Some((clamped_w, clamped_h)));
    let changed = current_cfg.is_none_or(|(cw, ch)| cw.abs_diff(clamped_w) >= 2 || ch.abs_diff(clamped_h) >= 2);
    if changed {
        state.update_user_config(app, |c| {
            c.upsert_chat_window_size_entry(account_index, true, Some((clamped_w, clamped_h)));
        });
    }
}

/// Resolves target logical custom size from explicit request, persisted entry, live window, or default
fn resolve_chat_window_target_size(
    is_custom: bool,
    width: Option<u32>,
    height: Option<u32>,
    stored: Option<(u32, u32)>,
    live: Option<(u32, u32)>,
) -> Option<(u32, u32)> {
    if !is_custom {
        return None;
    }
    let min_w = crate::utils::user_cache::MIN_CHAT_WINDOW_WIDTH;
    let min_h = crate::utils::user_cache::MIN_CHAT_WINDOW_HEIGHT;
    let def_w = crate::utils::user_cache::DEFAULT_CHAT_WINDOW_WIDTH;
    let def_h = crate::utils::user_cache::DEFAULT_CHAT_WINDOW_HEIGHT;
    let valid = |pair: Option<(u32, u32)>| pair.filter(|&(w, h)| w >= min_w && h >= min_h);
    valid(width.zip(height))
        .or_else(|| valid(stored))
        .or_else(|| valid(live))
        .or(Some((def_w, def_h)))
}

/// SSOT core helper applying chat window size mode, resizable lock, persistence, and live re-dock
pub fn apply_chat_window_size_mode(
    app: &AppHandle,
    state: &ChatState,
    account_index: u32,
    is_custom: bool,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<Option<(u32, u32)>, String> {
    let idx = ChatState::validate_account_index(Some(account_index))?;
    let label = crate::platforms::messenger::windows::get_messenger_window_label(idx);
    let win_opt = app.get_webview_window(&label);
    let stored = state
        .get_user_config()
        .get_chat_window_size_entry(idx)
        .and_then(|e| e.custom_width.zip(e.custom_height));
    let live = win_opt.as_ref().and_then(|win| {
        let phys_sz = win.inner_size().ok()?;
        let scale = win.scale_factor().unwrap_or(1.0);
        let cur_w = (phys_sz.width as f64 / scale).round() as u32;
        let cur_h = (phys_sz.height as f64 / scale).round() as u32;
        Some((cur_w, cur_h))
    });
    let size = resolve_chat_window_target_size(is_custom, width, height, stored, live);
    state.set_chat_window_custom_size(idx, size);
    state.update_user_config(app, |c| {
        c.upsert_chat_window_size_entry(idx, is_custom, size);
    });
    if let Some(ref win) = win_opt
        && !state.is_window_in_auth_mode(&label) {
            let _ = win.set_resizable(is_custom);
            if win.is_visible().unwrap_or(false) {
                let _ = ChatService::dock_chat_window(app, win, idx);
            }
        }
    crate::presentation::bubble::dock::sync_action_dock_for_account(app, idx);
    Ok(size)
}

/// Command setting chat window size configuration
#[tauri::command]
pub fn set_chat_window_size_config(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
    is_custom: bool,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<Option<(u32, u32)>, String> {
    let idx = ChatState::validate_account_index(account_index)?;
    apply_chat_window_size_mode(&app, &state, idx, is_custom, width, height)
}

/// SSOT core helper toggling or explicitly setting pinned state of a chat window
pub fn apply_chat_pinned(
    app: &AppHandle,
    state: &ChatState,
    account_index: u32,
    pinned: Option<bool>,
) -> Result<bool, String> {
    let idx = ChatState::validate_account_index(Some(account_index))?;
    let next_state = pinned.unwrap_or_else(|| !state.is_chat_pinned(idx));
    state.set_chat_pinned(idx, next_state);
    println!("\x1b[34m[BE - Pin] apply_chat_pinned: account_index={}, pinned={}\x1b[0m", idx, next_state);
    let label = crate::platforms::messenger::windows::get_messenger_window_label(idx);
    if let Some(win) = app.get_webview_window(&label) {
        let _ = win.eval(format!(
            "window.__IS_PINNED__ = {next_state}; if ({next_state} && typeof window.__cancelAutoUnselectCountdown === 'function') {{ window.__cancelAutoUnselectCountdown(); }}"
        ));
    }
    let _ = app.emit("chat-pin-changed", serde_json::json!({
        "accountIndex": idx,
        "pinned": next_state
    }));
    crate::presentation::bubble::dock::sync_action_dock_for_account(app, idx);
    Ok(next_state)
}

/// Command toggling or explicitly setting pinned state of chat window
#[tauri::command]
pub fn toggle_pin_chat(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
    pinned: Option<bool>,
) -> Result<bool, String> {
    let idx = ChatState::validate_account_index(account_index)?;
    apply_chat_pinned(&app, &state, idx, pinned)
}

/// Command querying whether chat window is currently pinned
#[tauri::command]
pub fn is_chat_pinned(
    state: State<'_, Arc<ChatState>>,
    account_index: Option<u32>,
) -> Result<bool, String> {
    let idx = ChatState::validate_account_index(account_index)?;
    Ok(state.is_chat_pinned(idx))
}

#[cfg(test)]
mod tests {
    use super::{is_valid_thread_id, resolve_chat_window_target_size};

    #[test]
    fn test_is_valid_thread_id() {
        assert!(is_valid_thread_id("1234567890"));
        assert!(is_valid_thread_id("t_abc.123-xyz"));
        assert!(!is_valid_thread_id(""));
        assert!(!is_valid_thread_id("   "));
        assert!(!is_valid_thread_id("sender-alice"));
        assert!(!is_valid_thread_id("account-msg-1"));
        assert!(!is_valid_thread_id("123';alert(1)//"));
        assert!(!is_valid_thread_id("e2ee/t/123"));
    }

    #[test]
    fn test_resolve_chat_window_target_size_precedence() {
        assert_eq!(resolve_chat_window_target_size(false, Some(500), Some(600), Some((480, 580)), Some((510, 610))), None);
        assert_eq!(resolve_chat_window_target_size(true, Some(520), Some(640), Some((480, 580)), Some((510, 610))), Some((520, 640)));
        assert_eq!(resolve_chat_window_target_size(true, Some(200), Some(300), Some((480, 580)), Some((510, 610))), Some((480, 580)));
        assert_eq!(resolve_chat_window_target_size(true, None, None, None, Some((510, 610))), Some((510, 610)));
        assert_eq!(resolve_chat_window_target_size(true, None, None, None, Some((200, 300))), Some((404, 550)));
    }
}
