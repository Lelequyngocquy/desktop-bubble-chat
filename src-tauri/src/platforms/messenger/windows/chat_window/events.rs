use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use super::get_bubble_window_label;
use crate::operating_systems::current::CursorTracker;
use crate::platforms::messenger::constants::MESSENGER_BASE_URL;
use crate::platforms::messenger::services::ChatService;
use crate::state::ChatState;

/// Attaches focus/blur and close protection listeners to chat window
pub fn setup_window_events(app: &AppHandle, chat_window: &tauri::WebviewWindow, chat_state: Arc<ChatState>) {
    let chat_clone = chat_window.clone();
    let app_handle = app.clone();
    let state_clone = Arc::clone(&chat_state);
    let win_label = chat_window.label().to_string();
    chat_window.on_window_event(move |event| {
        let idx = win_label
            .strip_prefix("chat-messenger-")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);
        match event {
            tauri::WindowEvent::Focused(false) => {
                if !state_clone.is_window_in_auth_mode(&win_label) && !state_clone.is_chat_pinned(idx) && !state_clone.is_dialog_active(&win_label) {
                    let bubble_label = get_bubble_window_label(idx);
                    let over_bubble = app_handle
                        .get_webview_window(&bubble_label)
                        .map(|b| CursorTracker::is_cursor_over_window(&b))
                        .unwrap_or_else(|| crate::operating_systems::current::bubble::is_cursor_over_native_bubble(idx));
                    let over_chat = CursorTracker::is_cursor_over_window(&chat_clone);
                    if !over_bubble && !over_chat {
                        let _ = ChatService::hide_chat(&app_handle, idx);
                    }
                }
            }
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if state_clone.is_bypass_close_guard(&win_label) {
                    state_clone.set_bypass_close_guard(&win_label, false);
                    return;
                }
                api.prevent_close();
                state_clone.set_window_auth_mode(&win_label, false);
                let _ = chat_clone.eval("window.__IS_AUTH_MODE__ = false;");
                let _ = chat_clone.set_decorations(false);
                let _ = chat_clone.set_skip_taskbar(true);
                if !state_clone.get_account_auth(idx).as_ref().map(|a| a.connected).unwrap_or(false) {
                    state_clone.set_account_suspended(idx, true);
                    crate::commands::bubble::suspend_account_bubble_and_chat(&app_handle, idx);
                    state_clone.update_user_config(&app_handle, |c| {
                        c.active_bubble_accounts.retain(|&x| x != idx);
                        c.authenticated_accounts.retain(|&x| x != idx);
                    });
                    if let Ok(clean_url) = tauri::Url::parse(MESSENGER_BASE_URL) {
                        let _ = chat_clone.navigate(clean_url);
                    }
                }
                let _ = app_handle.emit("messenger-auth-closed", serde_json::json!({ "label": &win_label }));
                let _ = ChatService::hide_chat(&app_handle, idx);
            }
            tauri::WindowEvent::Resized(physical_size)
                if physical_size.width > 0
                    && physical_size.height > 0
                    && !state_clone.is_window_in_auth_mode(&win_label) => {
                    let scale = chat_clone.scale_factor().unwrap_or(1.0);
                    let logical_w = (physical_size.width as f64 / scale).round() as u32;
                    let logical_h = (physical_size.height as f64 / scale).round() as u32;
                    if logical_w >= 200 && logical_h >= 200 {
                        if let Some((cur_w, cur_h)) = state_clone.get_chat_window_custom_size(idx) {
                            let clamped_w = logical_w.max(crate::utils::user_cache::MIN_CHAT_WINDOW_WIDTH);
                            let clamped_h = logical_h.max(crate::utils::user_cache::MIN_CHAT_WINDOW_HEIGHT);
                            if cur_w.abs_diff(clamped_w) >= 2 || cur_h.abs_diff(clamped_h) >= 2 {
                                state_clone.set_chat_window_custom_size(idx, Some((clamped_w, clamped_h)));
                                let token = state_clone.next_chat_resize_token(idx);
                                let app_bg = app_handle.clone();
                                let state_bg = Arc::clone(&state_clone);
                                tauri::async_runtime::spawn(async move {
                                    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                                    if state_bg.is_explicit_exit.load(std::sync::atomic::Ordering::SeqCst) {
                                        return;
                                    }
                                    if state_bg.is_chat_resize_token_valid(idx, token) {
                                        crate::commands::messenger::chat::record_resized_chat_window_size(
                                            &app_bg,
                                            &state_bg,
                                            idx,
                                            clamped_w,
                                            clamped_h,
                                        );
                                    }
                                });
                            }
                        }
                        let _ = app_handle.emit("chat-window-resized", serde_json::json!({
                            "accountIndex": idx,
                            "width": logical_w,
                            "height": logical_h,
                        }));
                    }
                }
            _ => {}
        }
    });
}

/// Applies hardening settings and initial mute state to Windows WebView2 instance
#[cfg(target_os = "windows")]
pub fn configure_webview2_settings(window: &tauri::WebviewWindow, is_muted: bool) {
    let _ = window.with_webview(move |webview| {
        unsafe {
            use webview2_com::Microsoft::Web::WebView2::Win32::{
                ICoreWebView2_8, ICoreWebView2Settings4, ICoreWebView2Settings6,
            };
            use windows_core::Interface;
            let controller = webview.controller();
            if let Ok(core_webview) = controller.CoreWebView2() {
                if is_muted
                    && let Ok(webview_8) = core_webview.cast::<ICoreWebView2_8>() {
                        let _ = webview_8.SetIsMuted(true);
                    }
                if let Ok(settings) = core_webview.Settings() {
                    let _ = settings.SetIsStatusBarEnabled(false);
                    let _ = settings.SetIsZoomControlEnabled(false);
                    let _ = settings.SetIsBuiltInErrorPageEnabled(false);
                    let _ = settings.SetAreDefaultContextMenusEnabled(true);
                    #[cfg(debug_assertions)]
                    let _ = settings.SetAreDevToolsEnabled(true);
                    #[cfg(not(debug_assertions))]
                    let _ = settings.SetAreDevToolsEnabled(false);
                    if let Ok(settings4) = settings.cast::<ICoreWebView2Settings4>() {
                        let _ = settings4.SetIsPasswordAutosaveEnabled(false);
                        let _ = settings4.SetIsGeneralAutofillEnabled(false);
                    }
                    if let Ok(settings6) = settings.cast::<ICoreWebView2Settings6>() {
                        let _ = settings6.SetIsSwipeNavigationEnabled(false);
                    }
                }
            }
        }
    });
}
