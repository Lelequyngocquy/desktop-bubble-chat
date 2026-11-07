use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use super::theme::{apply_window_theme_color, get_account_theme};
use crate::platforms::messenger::constants::{MESSENGER_BASE_URL, MESSENGER_INJECT_SCRIPT};
use crate::platforms::messenger::parser::parse_unread_from_title;
use crate::platforms::messenger::services::ChatService;
use crate::state::ChatState;
use crate::utils::time::now_millis;
use crate::utils::win32::{is_cursor_over_bubble, is_cursor_over_window};
use crate::windows::bubble::{get_bubble_window_label, tuck_bubble};

/// Returns window label for a Messenger account index (e.g. chat-messenger-01)
pub fn get_messenger_window_label(account_index: u32) -> String {
    format!("chat-messenger-{:02}", account_index)
}

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
                if !state_clone.is_window_in_auth_mode(&win_label) {
                    let bubble_label = get_bubble_window_label(idx);
                    let over_bubble = app_handle
                        .get_webview_window(&bubble_label)
                        .map(|b| is_cursor_over_bubble(&b))
                        .unwrap_or_else(|| {
                            #[cfg(target_os = "windows")]
                            {
                                crate::windows::native_bubble::is_cursor_over_native_bubble(idx)
                            }
                            #[cfg(not(target_os = "windows"))]
                            false
                        });
                    let over_chat = is_cursor_over_window(&chat_clone);
                    if !over_bubble && !over_chat {
                        state_clone.last_blur_time.store(now_millis(), Ordering::SeqCst);
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
            _ => {}
        }
    });
}

/// Finds or creates WebviewWindow with isolated data_directory profile
pub fn get_or_create_messenger_window(
    app: &AppHandle,
    account_index: u32,
    chat_state: Arc<ChatState>,
) -> Result<tauri::WebviewWindow, String> {
    let label = get_messenger_window_label(account_index);
    println!("[BE - Window] get_or_create_messenger_window for label='{}' (index={})", label, account_index);
    if let Some(win) = app.get_webview_window(&label) {
        #[cfg(target_os = "windows")]
        {
            #[link(name = "user32")]
            unsafe extern "system" {
                fn IsWindow(hWnd: *mut std::ffi::c_void) -> i32;
            }
            let is_alive = if let Ok(hwnd) = win.hwnd() {
                unsafe { IsWindow(hwnd.0 as *mut std::ffi::c_void) != 0 }
            } else {
                false
            };
            if is_alive {
                println!("[BE - Window] Window '{}' exists and is valid, reusing.", label);
                return Ok(win);
            } else {
                println!("[BE - Window] Window '{}' was destroyed previously, cleaning to rebuild.", label);
                let _ = win.destroy();
            }
        }
        #[cfg(not(target_os = "windows"))]
        return Ok(win);
    }
    if account_index == 1 {
        if let Some(win) = app.get_webview_window("chat-messenger") {
            println!("[BE - Window] Found legacy 'chat-messenger' window for account 1.");
            return Ok(win);
        }
    }
    let mut profile_dir = app.path().app_data_dir().map_err(|e| {
        let msg = format!("Failed to get app_data_dir: {}", e);
        eprintln!("[BE - Window ERROR] {}", msg);
        msg
    })?;
    profile_dir.push("profiles");
    profile_dir.push(format!("messenger_{:02}", account_index));
    let _ = std::fs::create_dir_all(&profile_dir);
    println!("[BE - Window] Profile directory: {:?}", profile_dir);
    let parsed_url = tauri::Url::parse(MESSENGER_BASE_URL).map_err(|e| e.to_string())?;
    let theme = get_account_theme(account_index);
    let init_script = format!(
        "window.__ACCOUNT_INDEX__ = {};\n{}",
        account_index,
        MESSENGER_INJECT_SCRIPT
    );
    println!("[BE - Window] Building new WebviewWindow for '{}'...", label);
    let state_page_load = Arc::clone(&chat_state);
    let state_title = Arc::clone(&chat_state);
    let builder = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::External(parsed_url),
    )
    .background_color(tauri::utils::config::Color(255, 255, 255, 255))
    .data_directory(profile_dir)
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0")
    .initialization_script(&init_script)
    .title(format!("Messenger {:02}", account_index))
    .inner_size(404.0, 550.0)
    .min_inner_size(404.0, 550.0)
    .resizable(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .shadow(true)
    .additional_browser_args(crate::platforms::messenger::constants::CHROMIUM_BROWSER_ARGS)
    .visible(false)
    .on_document_title_changed(move |win, title| {
        let count = parse_unread_from_title(&title);
        let prev = state_title.get_unread_count(account_index);
        let is_visible = win.is_visible().unwrap_or(false);
        if !is_visible && count < prev {
            return;
        }
        let _ = win.set_title(&title);
        if count != prev {
            state_title.set_unread_count(account_index, count);
            crate::windows::native_bubble::set_native_bubble_unread_count(account_index, count);
            if count > prev && count > 0 {
                let display_title = format!("New message for Messenger {:02}", account_index);
                let account_tag = format!("M{:02}", account_index);
                let ts = now_millis();
                let msg = crate::state::IncomingMessage {
                    account_index,
                    sender_name: display_title.clone(),
                    body: "New message".to_string(),
                    avatar_url: String::new(),
                    thread_id: String::new(),
                    timestamp: ts,
                };
                if state_title.add_recent_message(msg) {
                    let time_str = crate::utils::time::format_current_time();
                    let log_msg = format!(
                        "[{}:MSG] New message for Messenger {:02} | Time: {}",
                        account_tag, account_index, time_str
                    );
                    println!("\x1b[32m{}\x1b[0m", log_msg);
                    crate::commands::system::app::log_frontend(win.app_handle().clone(), Some("info".into()), log_msg);
                    let _ = win.app_handle().emit("new-message-received", serde_json::json!({
                        "accountIndex": account_index,
                        "senderName": display_title,
                        "body": "New message",
                        "avatarUrl": "",
                        "threadId": "",
                        "timestamp": ts,
                    }));
                }
            }
            let _ = win.app_handle().emit("badge-updated", serde_json::json!({
                "accountIndex": account_index,
                "count": count
            }));
        }
    })
    .on_page_load(move |win, payload| {
        let in_auth = state_page_load.is_window_in_auth_mode(win.label());
        let _ = win.eval(&format!("window.__IS_AUTH_MODE__ = {};", in_auth));
        if let tauri::webview::PageLoadEvent::Finished = payload.event() {
            state_page_load.set_chat_ready(account_index, true);
            println!("\x1b[32m[BE - Window] Webview Messenger {:02} preloading finished (Ready)!\x1b[0m", account_index);
            crate::utils::trim_controller::schedule_auto_trim_on_preload(account_index, win.app_handle().clone());
        }
    });
    match builder.build() {
        Ok(window) => {
            println!("\x1b[32m[BE - Window] Successfully built Webview window: {}\x1b[0m", label);
            setup_window_events(app, &window, chat_state.clone());
            #[cfg(target_os = "windows")]
            apply_window_theme_color(&window, theme.hex, theme.border_hex);
            #[cfg(target_os = "windows")]
            {
                let is_muted = chat_state.is_account_muted(account_index);
                let _ = window.with_webview(move |webview| {
                    unsafe {
                        use webview2_com::Microsoft::Web::WebView2::Win32::{
                            ICoreWebView2_8, ICoreWebView2Settings4, ICoreWebView2Settings6,
                        };
                        use windows_core::Interface;
                        let controller = webview.controller();
                        if let Ok(core_webview) = controller.CoreWebView2() {
                            if is_muted {
                                if let Ok(webview_8) = core_webview.cast::<ICoreWebView2_8>() {
                                    let _ = webview_8.SetIsMuted(true);
                                }
                            }
                            if let Ok(settings) = core_webview.Settings() {
                                let _ = settings.SetIsStatusBarEnabled(false);
                                let _ = settings.SetIsZoomControlEnabled(false);
                                let _ = settings.SetIsBuiltInErrorPageEnabled(false);
                                let _ = settings.SetAreDefaultContextMenusEnabled(false);
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
            #[cfg(target_os = "windows")]
            set_webview_memory_target_level(&window, true);
            Ok(window)
        }
        Err(err) => {
            let msg = format!("Builder.build() failed for '{}': {:?}", label, err);
            eprintln!("\x1b[31m[BE - Window ERROR] {}\x1b[0m", msg);
            Err(msg)
        }
    }
}

/// Adjusts WebView memory usage target level
pub fn set_webview_memory_target_level(window: &tauri::WebviewWindow, is_low: bool) {
    crate::operating_systems::current::MemoryOptimizer::set_webview_memory_target_level(window, is_low);
}

#[cfg(not(target_os = "windows"))]
pub fn set_webview_memory_target_level(_window: &tauri::WebviewWindow, _is_low: bool) {}

/// Sets memory target level for a specific Messenger account chat window
pub fn set_chat_window_memory_target(app: &tauri::AppHandle, account_index: u32, is_low: bool) {
    let label = get_messenger_window_label(account_index);
    let win = app
        .get_webview_window(&label)
        .or_else(|| if account_index == 1 { app.get_webview_window("chat-messenger") } else { None });
    if let Some(chat_window) = win {
        set_webview_memory_target_level(&chat_window, is_low);
    }
}

/// Ensures only one chat/auth window is displayed at a time and others are tucked
pub fn hide_all_other_messenger_windows(app: &AppHandle, active_label: &str) {
    for idx in 1..=5 {
        let label = get_messenger_window_label(idx);
        if label != active_label {
            if let Some(w) = app.get_webview_window(&label) {
                let _ = w.hide();
            }
            let _ = app.emit("chat-visibility-changed", serde_json::json!({
                "accountIndex": idx,
                "visible": false,
                "windowLabel": &label
            }));
            tuck_bubble(app, Some(idx));
        }
    }
    if active_label != "chat-messenger" {
        if let Some(w) = app.get_webview_window("chat-messenger") {
            let _ = w.hide();
            let _ = app.emit("chat-visibility-changed", serde_json::json!({
                "accountIndex": 1,
                "visible": false,
                "windowLabel": "chat-messenger"
            }));
            tuck_bubble(app, Some(1));
        }
    }
}

/// Preloads Messenger Webview in background for specified account
pub fn preload_messenger_window(
    app: &AppHandle,
    account_index: u32,
    chat_state: Arc<ChatState>,
) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = get_or_create_messenger_window(&app_handle, account_index, chat_state);
    });
}

/// Scans authenticated Messenger accounts and preloads webviews
pub fn preload_existing_messenger_windows(app: &AppHandle, chat_state: Arc<ChatState>) {
    let app_handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        for idx in 1..=5 {
            let is_connected = chat_state.get_account_auth(idx).map(|a| a.connected).unwrap_or(false);
            if is_connected {
                println!("[BE - Window] Preloading Messenger {:02}...", idx);
                let _ = get_or_create_messenger_window(&app_handle, idx, Arc::clone(&chat_state));
            }
        }
    });
}

/// Sets up event listeners for Messenger chat window
pub fn setup_chat_window(app: &AppHandle, chat_state: Arc<ChatState>) {
    if let Some(chat_window) = app.get_webview_window("chat-messenger") {
        setup_window_events(app, &chat_window, Arc::clone(&chat_state));
        let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);
    }
}
