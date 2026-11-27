use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::platforms::messenger::verifier::is_window_logged_in;
use crate::platforms::messenger::windows::get_messenger_webview;
use crate::state::ChatState;
use crate::presentation::bubble::ensure_bubble_positioned_and_visible;

/// Daemon thread monitoring real-time Messenger login state and unread message counts
pub fn start_auth_watcher(app_handle: AppHandle, chat_state: Arc<ChatState>) {
    std::thread::spawn(move || {
        let mut logged_in_map: std::collections::HashMap<u32, bool> = std::collections::HashMap::new();
        loop {
            for _ in 0..8 {
                if chat_state.is_explicit_exit.load(Ordering::SeqCst) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            if chat_state.is_explicit_exit.load(Ordering::SeqCst) {
                break;
            }
            for idx in 1..=5 {
                if chat_state.is_account_suspended(idx) {
                    logged_in_map.insert(idx, false);
                    continue;
                }
                if let Some(chat_window) = get_messenger_webview(&app_handle, idx) {
                    let win_label = chat_window.label().to_string();
                    let auth_info = chat_state.get_account_auth(idx);
                    let state_logged_in = auth_info.as_ref().map(|a| a.connected).unwrap_or(false);
                    let (win_logged_in, win_user_name) = is_window_logged_in(&chat_window);
                    let is_logged_in = if state_logged_in {
                        if let Ok(curr_url) = chat_window.url() {
                            let u = curr_url.to_string();
                            !(u.contains("messenger.com/login") || (u.contains("facebook.com/login") && !win_logged_in))
                        } else {
                            true
                        }
                    } else {
                        win_logged_in
                    };
                    let user_name = if !win_user_name.is_empty() {
                        win_user_name
                    } else if state_logged_in {
                        auth_info.as_ref().map(|a| a.name.clone()).unwrap_or_default()
                    } else {
                        String::new()
                    };
                    let was_logged = *logged_in_map.get(&idx).unwrap_or(&false);
                    if is_logged_in != was_logged {
                        logged_in_map.insert(idx, is_logged_in);
                        if is_logged_in {
                            chat_state.set_account_auth(
                                idx,
                                true,
                                if user_name.is_empty() { None } else { Some(user_name.clone()) },
                            );
                            chat_state.update_user_config(&app_handle, |c| {
                                if !c.authenticated_accounts.contains(&idx) {
                                    c.authenticated_accounts.push(idx);
                                    c.authenticated_accounts.sort();
                                }
                            });
                            if let Ok(curr_url) = chat_window.url() {
                                let u = curr_url.to_string();
                                if (u.contains("facebook.com") || u.contains("meta.com"))
                                    && let Ok(target) = tauri::Url::parse("https://www.messenger.com/") {
                                        let _ = chat_window.navigate(target);
                                    }
                            }
                            let _ = chat_window.eval("window.__IS_AUTH_MODE__ = false;");
                            if chat_state.is_window_in_auth_mode(&win_label) {
                                chat_state.set_window_auth_mode(&win_label, false);
                                let _ = chat_window.hide();
                                let _ = chat_window.set_decorations(false);
                                let _ = chat_window.set_resizable(chat_state.get_chat_window_custom_size(idx).is_some());
                                let _ = chat_window.set_skip_taskbar(true);
                                let _ = app_handle.emit("chat-visibility-changed", serde_json::json!({
                                    "accountIndex": idx,
                                    "visible": false,
                                    "windowLabel": &win_label
                                }));
                                let _ = ensure_bubble_positioned_and_visible(&app_handle, idx);
                            }
                        } else if !chat_state.is_window_in_auth_mode(&win_label) {
                            chat_state.clear_account_auth(idx);
                            chat_state.set_account_suspended(idx, true);
                            let _ = chat_window.eval("window.__IS_AUTH_MODE__ = true;");
                            crate::commands::bubble::suspend_account_bubble_and_chat(&app_handle, idx);
                            chat_state.update_user_config(&app_handle, |c| {
                                c.authenticated_accounts.retain(|&x| x != idx);
                                c.active_bubble_accounts.retain(|&x| x != idx);
                            });
                        }
                        let _ = app_handle.emit("messenger-auth-status", serde_json::json!({
                            "accountIndex": idx,
                            "connected": is_logged_in,
                            "name": if is_logged_in { &user_name } else { "" }
                        }));
                        let _ = crate::tray::update_tray_menu(&app_handle);
                    }
                    if is_logged_in
                        && let Ok(title) = chat_window.title() {
                            crate::commands::notification::sync_unread_from_window_title(
                                &app_handle,
                                &chat_state,
                                idx,
                                &title,
                            );
                        }
                }
            }
        }
    });
}
