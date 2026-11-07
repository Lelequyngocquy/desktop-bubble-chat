use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::platforms::messenger::constants::{MESSENGER_BASE_URL, MESSENGER_INJECT_SCRIPT};
use crate::platforms::messenger::verifier::is_window_logged_in;
use crate::platforms::messenger::windows::{
    apply_window_theme_color, get_account_theme, get_messenger_window_label,
    get_or_create_messenger_window, hide_all_other_messenger_windows,
};
use crate::state::ChatState;
use crate::windows::bubble::{ensure_bubble_positioned_and_visible, get_bubble_window_label};

pub struct AuthService;

impl AuthService {
    /// Opens independent login popup for specified account
    pub async fn open_auth(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        println!("\x1b[32m[BE - Auth] ========================================================\x1b[0m");
        println!("\x1b[32m[BE - Auth] >>> Received command to open auth popup: account_index = {} \x1b[0m", idx);
        state.clear_account_auth(idx);
        state.set_account_suspended(idx, false);
        let chat_window = match get_or_create_messenger_window(app, idx, Arc::clone(state)) {
            Ok(win) => {
                println!("[BE - Auth] Created window: '{}'", win.label());
                win
            }
            Err(err) => {
                eprintln!("\x1b[31m[BE - Auth ERROR] get_or_create_messenger_window failed: {}\x1b[0m", err);
                return Err(err);
            }
        };
        hide_all_other_messenger_windows(app, &chat_window.label());
        state.set_window_auth_mode(&chat_window.label(), true);
        let theme = get_account_theme(idx);
        let _ = chat_window.set_title(&format!("Messenger Login {:02}", idx));
        let _ = chat_window.set_decorations(true);
        let _ = chat_window.set_skip_taskbar(false);
        let _ = chat_window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 680.0,
            height: 820.0,
        }));
        let _ = chat_window.center();
        let _ = chat_window.unminimize();
        let _ = chat_window.set_always_on_top(true);
        #[cfg(target_os = "windows")]
        apply_window_theme_color(&chat_window, theme.hex, theme.border_hex);
        if let Ok(url) = tauri::Url::parse(MESSENGER_BASE_URL) {
            let current_url = chat_window.url().map(|u| u.to_string()).unwrap_or_default();
            println!("[BE - Auth] Current window URL: '{}'", current_url);
            let in_2fa = current_url.contains("checkpoint") || current_url.contains("two_step_verification");
            let in_messenger = current_url.contains("messenger.com");
            if current_url.is_empty() || current_url == "about:blank" || (!in_2fa && !in_messenger) {
                println!("[BE - Auth] Navigating to: {}", MESSENGER_BASE_URL);
                let _ = chat_window.navigate(url);
            }
        }
        let init_code = format!(
            "window.__ACCOUNT_INDEX__ = {};\nwindow.__IS_AUTH_MODE__ = true;\n{}",
            idx,
            MESSENGER_INJECT_SCRIPT
        );
        let _ = chat_window.eval(&init_code);
        println!("[BE - Auth] Calling chat_window.show()...");
        if let Err(e) = chat_window.show() {
            eprintln!("\x1b[31m[BE - Auth ERROR] chat_window.show() error: {:?}\x1b[0m", e);
            return Err(e.to_string());
        }
        println!("[BE - Auth] Calling chat_window.set_focus()...");
        if let Err(e) = chat_window.set_focus() {
            eprintln!("\x1b[31m[BE - Auth ERROR] chat_window.set_focus() error: {:?}\x1b[0m", e);
        }
        let _ = app.emit("chat-visibility-changed", serde_json::json!({
            "accountIndex": idx,
            "visible": true,
            "windowLabel": chat_window.label()
        }));
        println!("\x1b[32m[BE - Auth] Completed: Messenger Login {:02} window shown and focused!\x1b[0m", idx);
        println!("\x1b[32m[BE - Auth] ========================================================\x1b[0m");
        Ok(())
    }

    /// Disconnects Messenger, navigates to login, and hides window
    pub fn disconnect(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        println!("[BE - Messenger] Disconnecting Messenger account {:02}", idx);
        state.set_window_auth_mode(&label, false);
        state.clear_account_auth(idx);
        state.set_account_suspended(idx, true);
        let win = app
            .get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });
        if let Some(chat_window) = win {
            let nav_cmd = format!("window.location.href = '{}';", MESSENGER_BASE_URL);
            let _ = chat_window.eval(&nav_cmd);
            let _ = chat_window.hide();
            let _ = chat_window.set_decorations(false);
        }
        let bubble_label = get_bubble_window_label(idx);
        if let Some(bubble_win) = app.get_webview_window(&bubble_label) {
            let _ = bubble_win.hide();
        }
        crate::commands::bubble::suspend_account_bubble_and_chat(app, idx);
        state.update_user_config(app, |c| {
            c.authenticated_accounts.retain(|&x| x != idx);
            c.active_bubble_accounts.retain(|&x| x != idx);
        });
        let _ = app.emit("messenger-auth-status", serde_json::json!({
            "accountIndex": idx,
            "connected": false
        }));
        let _ = crate::tray::update_tray_menu(app);
        Ok(())
    }

    /// Checks actual authentication status of Messenger account
    pub fn check_auth(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<serde_json::Value, String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        if let Some(auth_info) = state.get_account_auth(idx) {
            if auth_info.connected {
                return Ok(serde_json::json!({
                    "accountIndex": idx,
                    "connected": true,
                    "name": auth_info.name,
                    "userId": auth_info.user_id,
                }));
            }
        }
        let win = app
            .get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });
        if let Some(chat_window) = win {
            let trigger_script = format!(
                "window.__ACCOUNT_INDEX__ = {}; if (typeof window.checkAndReportAuth === 'function') {{ window.checkAndReportAuth(); }}",
                idx
            );
            let _ = chat_window.eval(&trigger_script);
            let (is_logged_in, user_name) = is_window_logged_in(&chat_window);
            let url_str = chat_window.url().map(|u| u.to_string()).unwrap_or_default();
            let title = chat_window.title().unwrap_or_default();
            if is_logged_in {
                println!("\x1b[32m[BE - Auth] check_messenger_auth: Auth SUCCESS for account {:02}! (User: '{}', URL: {})\x1b[0m", idx, user_name, url_str);
                state.set_account_auth(idx, true, if user_name.is_empty() { None } else { Some(user_name.clone()) });
                state.update_user_config(app, |c| {
                    if !c.authenticated_accounts.contains(&idx) {
                        c.authenticated_accounts.push(idx);
                        c.authenticated_accounts.sort();
                    }
                });
                state.set_window_auth_mode(&label, false);
                let _ = chat_window.set_title("Messenger");
                let _ = chat_window.hide();
                let _ = chat_window.set_decorations(false);
                let _ = chat_window.set_skip_taskbar(true);
                let _ = app.emit("chat-visibility-changed", serde_json::json!({
                    "accountIndex": idx,
                    "visible": false,
                    "windowLabel": &label
                }));
                let _ = app.emit("messenger-auth-status", serde_json::json!({
                    "accountIndex": idx,
                    "connected": true,
                    "name": &user_name
                }));
            }
            return Ok(serde_json::json!({
                "accountIndex": idx,
                "connected": is_logged_in,
                "name": if is_logged_in { user_name } else { "".to_string() },
                "url": url_str,
                "title": title
            }));
        }
        Ok(serde_json::json!({ "accountIndex": idx, "connected": false, "name": "" }))
    }

    /// Receives auth status report from webview and synchronizes to UI
    pub fn report_status(
        app: &AppHandle,
        state: &Arc<ChatState>,
        connected: bool,
        name: Option<String>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        println!(
            "\x1b[32m[BE - Auth] report_auth_status received from webview: account={:02}, connected={}, user={:?}\x1b[0m",
            idx, connected, name
        );
        state.set_account_auth(idx, connected, name.clone());
        state.set_chat_ready(idx, true);
        if connected {
            state.update_user_config(app, |c| {
                if !c.authenticated_accounts.contains(&idx) {
                    c.authenticated_accounts.push(idx);
                    c.authenticated_accounts.sort();
                }
            });
            let was_in_auth_mode = state.is_window_in_auth_mode(&label);
            state.set_window_auth_mode(&label, false);
            if let Some(win) = app.get_webview_window(&label) {
                if let Ok(curr_url) = win.url() {
                    let u = curr_url.to_string();
                    if u.contains("facebook.com") || u.contains("meta.com") {
                        println!("[BE - Auth] report_auth_status: On Meta URL ('{}'), redirecting to https://www.messenger.com/", u);
                        if let Ok(target) = tauri::Url::parse("https://www.messenger.com/") {
                            let _ = win.navigate(target);
                        }
                    }
                }
                let _ = win.eval("window.__IS_AUTH_MODE__ = false;");
                let _ = win.set_title("Messenger");
                let _ = win.hide();
                let _ = win.set_decorations(false);
                let _ = win.set_skip_taskbar(true);
            }
            if was_in_auth_mode && !state.is_account_suspended(idx) {
                let _ = ensure_bubble_positioned_and_visible(app, idx);
            }
            let _ = app.emit("chat-visibility-changed", serde_json::json!({
                "accountIndex": idx,
                "visible": false,
                "windowLabel": &label
            }));
        } else {
            state.update_user_config(app, |c| {
                c.authenticated_accounts.retain(|&x| x != idx);
                c.active_bubble_accounts.retain(|&x| x != idx);
            });
            if let Some(win) = app.get_webview_window(&label) {
                let _ = win.eval("window.__IS_AUTH_MODE__ = true;");
            }
        }
        let _ = app.emit("messenger-auth-status", serde_json::json!({
            "accountIndex": idx,
            "connected": connected,
            "name": name
        }));
        let _ = crate::tray::update_tray_menu(app);
        Ok(())
    }
}
