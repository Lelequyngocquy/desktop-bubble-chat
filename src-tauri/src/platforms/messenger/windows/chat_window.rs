mod events;
mod navigation;

pub use events::setup_window_events;

use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use super::theme::{apply_window_theme_color, get_account_theme};
use crate::platforms::messenger::constants::{MESSENGER_BASE_URL, MESSENGER_INJECT_SCRIPT};
use crate::platforms::messenger::services::ChatService;
use crate::presentation::bubble::get_bubble_window_label;
use crate::state::{ChatState, MAX_ACCOUNTS};

/// Returns window label for a Messenger account index (e.g. chat-messenger-01)
pub fn get_messenger_window_label(account_index: u32) -> String {
    format!("chat-messenger-{:02}", account_index)
}

/// Generates a deterministic 16-byte WKWebsiteDataStore identifier per account slot (macOS 14+)
pub fn data_store_identifier_for_account(account_index: u32) -> [u8; 16] {
    let mut id: [u8; 16] = [0x64, 0x62, 0x63, 0x2d, 0x6d, 0x73, 0x67, 0x2d, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let bytes = account_index.to_be_bytes();
    id[12] = bytes[0];
    id[13] = bytes[1];
    id[14] = bytes[2];
    id[15] = bytes[3];
    id[6] = (id[6] & 0x0F) | 0x40;
    id[8] = (id[8] & 0x3F) | 0x80;
    id
}

/// Returns a platform-native browser User-Agent string matching the underlying WebView engine
pub fn platform_user_agent(os: &str) -> &'static str {
    match os {
        "windows" => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
        "macos" => "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
        "linux" => "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
        _ => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36 Edg/131.0.0.0",
    }
}

/// Returns existing Messenger WebviewWindow for account index, falling back to legacy label for account 1
pub fn get_messenger_webview(app: &AppHandle, account_index: u32) -> Option<tauri::WebviewWindow> {
    let label = get_messenger_window_label(account_index);
    app.get_webview_window(&label)
        .or_else(|| if account_index == 1 { app.get_webview_window("chat-messenger") } else { None })
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
                unsafe { IsWindow(hwnd.0) != 0 }
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
    if account_index == 1
        && let Some(win) = app.get_webview_window("chat-messenger") {
            println!("[BE - Window] Found legacy 'chat-messenger' window for account 1.");
            return Ok(win);
        }
    let mut profile_dir = app.path().app_data_dir().map_err(|e| {
        let msg = format!("Failed to get app_data_dir: {}", e);
        eprintln!("[BE - Window ERROR] {}", msg);
        msg
    })?;
    profile_dir.push("profiles");
    profile_dir.push(format!("messenger_{:02}", account_index));
    let _ = std::fs::create_dir_all(&profile_dir);
    let ext_dir = profile_dir.join("extensions");
    let ext_disabled_dir = profile_dir.join("extensions_disabled");
    let _ = std::fs::create_dir_all(&ext_dir);
    let _ = std::fs::create_dir_all(&ext_disabled_dir);
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
    let state_nav = Arc::clone(&chat_state);
    let state_new_win = Arc::clone(&chat_state);
    let app_nav = app.clone();
    let app_new_win = app.clone();
    let label_nav = label.clone();
    let label_new_win = label.clone();
    let app_dl = app.clone();
    let state_dl = Arc::clone(&chat_state);
    let label_dl = label.clone();
    let last_download_name: Arc<std::sync::Mutex<Option<String>>> = Arc::new(std::sync::Mutex::new(None));
    let builder = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::External(parsed_url),
    ).on_navigation(move |url| {
        let in_auth = state_nav.is_window_in_auth_mode(&label_nav);
        match navigation::decide_navigation(url, in_auth) {
            navigation::NavigationDecision::Allow => true,
            navigation::NavigationDecision::OpenExternal(dest) => {
                println!("[BE - Nav] Opening external navigation in default browser: '{}'", dest);
                let app_open = app_nav.clone();
                std::thread::spawn(move || {
                    if let Err(err) = crate::commands::system::app::open_external_url_internal(&app_open, &dest) {
                        eprintln!("[BE - Nav WARN] Failed to open external URL '{}': {}", dest, err);
                    }
                });
                false
            }
            navigation::NavigationDecision::Block => false,
        }
    }).on_new_window(move |url, _features| {
        let in_auth = state_new_win.is_window_in_auth_mode(&label_new_win);
        match navigation::decide_new_window(&url, in_auth) {
            navigation::NavigationDecision::Allow => tauri::webview::NewWindowResponse::Allow,
            navigation::NavigationDecision::OpenExternal(dest) => {
                println!("[BE - Nav] Opening new-window request in default browser: '{}'", dest);
                let app_open = app_new_win.clone();
                std::thread::spawn(move || {
                    if let Err(err) = crate::commands::system::app::open_external_url_internal(&app_open, &dest) {
                        eprintln!("[BE - Nav WARN] Failed to open external URL '{}': {}", dest, err);
                    }
                });
                tauri::webview::NewWindowResponse::Deny
            }
            navigation::NavigationDecision::Block => tauri::webview::NewWindowResponse::Deny,
        }
    }).on_download(move |_webview, mut event| {
        match event {
            tauri::webview::DownloadEvent::Requested { ref url, ref mut destination } => {
                println!("\x1b[35m[BE - Download] on_download Requested: url='{}', default_dest='{:?}'\x1b[0m", url, destination);
                let default_name = destination.file_name().and_then(|n| n.to_str()).unwrap_or("downloaded_file");
                let default_dir = destination.parent().map(|p| p.to_path_buf());
                if let Some(target) = crate::commands::system::download::show_save_dialog_internal(
                    &app_dl,
                    &state_dl,
                    &label_dl,
                    Some(default_name),
                    default_dir,
                ) {
                    println!("\x1b[32m[BE - Download] on_download User selected target: '{:?}'\x1b[0m", target);
                    if let Ok(mut guard) = last_download_name.lock() {
                        *guard = target.file_name().and_then(|n| n.to_str()).map(|s| s.to_string());
                    }
                    **destination = target;
                    true
                } else {
                    println!("\x1b[33m[BE - Download] on_download User cancelled save dialog\x1b[0m");
                    false
                }
            }
            tauri::webview::DownloadEvent::Finished { ref url, ref path, success } => {
                println!("\x1b[35m[BE - Download] on_download Finished: url='{}', path='{:?}', success={}\x1b[0m", url, path, success);
                if success {
                    let cached_name = last_download_name.lock().ok().and_then(|mut g| g.take());
                    let file_name = path.as_ref()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                        .or(cached_name)
                        .unwrap_or_else(|| "file".to_string());
                    let _ = app_dl.notification()
                        .builder()
                        .title("DesktopBubbleChat")
                        .body(format!("Downloaded: {}", file_name))
                        .show();
                }
                true
            }
            _ => {
                println!("\x1b[35m[BE - Download] on_download Other event\x1b[0m");
                true
            }
        }
    });
    #[cfg(target_os = "windows")]
    let builder = builder.browser_extensions_enabled(true).extensions_path(&ext_dir);
    #[cfg(target_os = "macos")]
    let builder = builder.data_store_identifier(data_store_identifier_for_account(account_index));
    let custom_size = chat_state.get_chat_window_custom_size(account_index);
    let is_resizable = custom_size.is_some();
    let (init_w, init_h) = custom_size
        .map(|(w, h)| (w as f64, h as f64))
        .unwrap_or((404.0, 550.0));
    let builder = builder
        .background_color(tauri::utils::config::Color(255, 255, 255, 255))
        .data_directory(profile_dir)
        .user_agent(platform_user_agent(std::env::consts::OS))
        .initialization_script(&init_script)
        .title(format!("Messenger {:02}", account_index))
        .inner_size(init_w, init_h)
        .min_inner_size(320.0, 450.0)
        .resizable(is_resizable)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(true)
        .additional_browser_args(crate::platforms::messenger::constants::CHROMIUM_BROWSER_ARGS)
        .visible(false)
        .on_document_title_changed(move |win, title| {
            let is_pinned = title.contains("[PINNED]");
            state_title.set_chat_pinned(account_index, is_pinned);
            let clean_title = title.replace("[PINNED]", "").trim().to_string();
            if win.title().ok().as_deref() != Some(clean_title.as_str()) {
                let _ = win.set_title(&clean_title);
            }
            crate::commands::notification::sync_unread_from_window_title(
                win.app_handle(),
                &state_title,
                account_index,
                &clean_title,
            );
        })
        .on_page_load(move |win, payload| {
            let in_auth = state_page_load.is_window_in_auth_mode(win.label());
            let is_pinned = state_page_load.is_chat_pinned(account_index);
            let is_muted = state_page_load.is_account_muted(account_index);
            let _ = win.eval(format!(
                "window.__IS_AUTH_MODE__ = {in_auth}; window.__IS_PINNED__ = {is_pinned}; window.__isMuted = {is_muted};"
            ));
            if let tauri::webview::PageLoadEvent::Finished = payload.event() {
                state_page_load.set_chat_ready(account_index, true);
                if is_muted {
                    let _ = win.eval("document.querySelectorAll('audio, video').forEach(function(el){el.muted=true;});");
                }
                println!("\x1b[32m[BE - Window] Webview Messenger {:02} preloading finished (Ready)!\x1b[0m", account_index);
                crate::utils::trim_controller::schedule_auto_trim_on_preload(account_index, win.app_handle().clone());
            }
        });
    match builder.build() {
        Ok(window) => {
            println!("\x1b[32m[BE - Window] Successfully built Webview window: {}\x1b[0m", label);
            setup_window_events(app, &window, chat_state.clone());
            apply_window_theme_color(&window, theme.hex, theme.border_hex);
            #[cfg(target_os = "windows")]
            events::configure_webview2_settings(&window, chat_state.is_account_muted(account_index));
            #[cfg(target_os = "linux")]
            crate::operating_systems::linux::memory::LinuxMemoryOptimizer::set_webview_muted(&window, chat_state.is_account_muted(account_index));
            #[cfg(target_os = "macos")]
            {
                if let Ok(ns_win) = window.ns_window() {
                    let ns_win_ptr = ns_win as usize;
                    if ns_win_ptr != 0 {
                        crate::operating_systems::macos::bubble::window::ffi::run_on_macos_main_thread(move || unsafe {
                            use cocoa::base::{id, nil};
                            use objc::{msg_send, sel, sel_impl};
                            let win_obj = ns_win_ptr as id;
                            if win_obj != nil {
                                let behavior: u64 = (1 << 1) | (1 << 8);
                                let _: () = msg_send![win_obj, setCollectionBehavior: behavior];
                            }
                        });
                    }
                }
                if chat_state.is_account_muted(account_index) {
                    let _ = window.eval("(function(){window.__isMuted=true;if(!window.__dbcAudioGuardInstalled){window.__dbcAudioGuardInstalled=true;var origPlay=HTMLMediaElement.prototype.play;HTMLMediaElement.prototype.play=function(){if(window.__isMuted){this.muted=true;}return origPlay.apply(this,arguments);};}})();");
                }
            }
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

/// Sets memory target level for a specific Messenger account chat window
pub fn set_chat_window_memory_target(app: &tauri::AppHandle, account_index: u32, is_low: bool) {
    if let Some(chat_window) = get_messenger_webview(app, account_index) {
        set_webview_memory_target_level(&chat_window, is_low);
    }
}

/// Ensures only one chat/auth window is displayed at a time and others are tucked and cleaned
pub fn hide_all_other_messenger_windows(app: &AppHandle, active_label: &str) {
    for idx in 1..=MAX_ACCOUNTS {
        let label = get_messenger_window_label(idx);
        if label != active_label {
            let _ = ChatService::hide_chat(app, idx);
        }
    }
    if active_label != "chat-messenger" {
        let _ = ChatService::hide_chat(app, 1);
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
        for idx in 1..=MAX_ACCOUNTS {
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
