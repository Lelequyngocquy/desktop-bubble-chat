use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Minimizes main dashboard window to taskbar
#[tauri::command]
pub fn minimize_main_window(app: AppHandle) {
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.minimize();
    }
}

/// Toggles maximize/unmaximize on main dashboard window
#[tauri::command]
pub fn toggle_maximize_main_window(app: AppHandle) -> Result<bool, String> {
    if let Some(main_win) = app.get_webview_window("main") {
        let is_max = main_win.is_maximized().map_err(|e| e.to_string())?;
        if is_max {
            main_win.unmaximize().map_err(|e| e.to_string())?;
            Ok(false)
        } else {
            main_win.maximize().map_err(|e| e.to_string())?;
            Ok(true)
        }
    } else {
        Err("Main window not found".to_string())
    }
}

/// Hides main dashboard window to system tray (triggers hibernation)
#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    crate::windows::main_window::destroy_or_hibernate_main_window(&app);
}

/// Shows or recreates main dashboard window
#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    let _ = crate::windows::main_window::get_or_create_main_window(&app);
}

/// Updates system tray menu status from frontend/store
#[tauri::command]
pub fn update_tray_menu_status(
    app: AppHandle,
    _has_connected_accounts: Option<bool>,
) -> Result<(), String> {
    crate::tray::update_tray_menu(&app).map_err(|e| e.to_string())
}

/// Returns user settings snapshot from cache
#[tauri::command]
pub fn get_user_settings(state: tauri::State<'_, Arc<crate::state::ChatState>>) -> Result<crate::utils::user_cache::AppUserConfig, String> {
    Ok(state.get_user_config())
}

/// Saves user settings to cache
#[tauri::command]
pub fn save_user_settings(
    app: AppHandle,
    state: tauri::State<'_, Arc<crate::state::ChatState>>,
    settings: crate::utils::user_cache::AppUserConfig,
) -> Result<(), String> {
    state.set_user_config(&app, settings);
    Ok(())
}

/// Sets application display language, updates persistent cache, and updates system tray
#[tauri::command]
pub fn set_app_language(
    app: AppHandle,
    state: tauri::State<'_, Arc<crate::state::ChatState>>,
    language: String,
) -> Result<(), String> {
    state.update_user_config(&app, |c| {
        c.locale = Some(language.clone());
    });
    let _ = crate::tray::update_tray_menu(&app);
    Ok(())
}

/// Sets auto-trim enabled flag in user configuration cache
#[tauri::command]
pub fn set_auto_trim_enabled(
    app: AppHandle,
    state: tauri::State<'_, Arc<crate::state::ChatState>>,
    enabled: bool,
) -> Result<(), String> {
    state.update_user_config(&app, |c| {
        c.is_auto_trim_enabled = enabled;
    });
    Ok(())
}

/// Sets application display theme mode in user configuration cache
#[tauri::command]
pub fn set_theme_mode(
    app: AppHandle,
    state: tauri::State<'_, Arc<crate::state::ChatState>>,
    theme_mode: String,
) -> Result<(), String> {
    state.update_user_config(&app, |c| {
        c.theme_mode = Some(theme_mode);
    });
    Ok(())
}

/// Exits application completely
#[tauri::command]
pub fn exit_app(app: AppHandle) {
    if let Some(state) = app.try_state::<Arc<crate::state::ChatState>>() {
        state.is_explicit_exit.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    app.exit(0);
}

/// Restarts application (used for updates or resets)
#[tauri::command]
pub fn restart_app(app: AppHandle) {
    if let Some(state) = app.try_state::<Arc<crate::state::ChatState>>() {
        state.is_explicit_exit.store(true, std::sync::atomic::Ordering::SeqCst);
    }
    for (_, window) in app.webview_windows() {
        let _ = window.close();
    }
    app.request_restart();
}

/// Sets always on top mode for floating bubbles
#[tauri::command]
pub fn set_always_on_top(app: AppHandle, enabled: bool) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
        chat_state.is_always_on_top.store(enabled, Ordering::SeqCst);
        chat_state.update_user_config(&app, |c| {
            c.is_always_on_top = enabled;
        });
    }
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.set_always_on_top(false);
    }
    #[cfg(target_os = "windows")]
    crate::windows::native_bubble::set_native_bubble_always_on_top(enabled);
    for idx in 1..=5 {
        let bubble_label = format!("bubble-{:02}", idx);
        if let Some(w) = app.get_webview_window(&bubble_label) {
            let _ = w.set_always_on_top(enabled);
        }
    }
    for idx in 1..=5 {
        let chat_label = format!("chat-messenger-{:02}", idx);
        if let Some(w) = app.get_webview_window(&chat_label) {
            let _ = w.set_always_on_top(enabled);
        }
    }
    if let Some(w) = app.get_webview_window("chat-messenger") {
        let _ = w.set_always_on_top(enabled);
    }
    Ok(())
}

/// Opens external URL safely using default OS browser
#[tauri::command]
pub fn open_external_url(url: String) -> Result<(), String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            let wide: Vec<u16> = OsStr::new(&url)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            #[link(name = "shell32")]
            unsafe extern "system" {
                fn ShellExecuteW(
                    hwnd: *mut std::ffi::c_void,
                    lpOperation: *const u16,
                    lpFile: *const u16,
                    lpParameters: *const u16,
                    lpDirectory: *const u16,
                    nShowCmd: i32,
                ) -> isize;
            }
            unsafe {
                ShellExecuteW(
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    wide.as_ptr(),
                    std::ptr::null(),
                    std::ptr::null(),
                    1,
                );
            }
            return Ok(());
        }
        #[cfg(not(target_os = "windows"))]
        {
            return Ok(());
        }
    }
    Err("Invalid URL scheme".into())
}

/// Opens profiles directory in File Explorer
#[tauri::command]
pub fn open_profiles_folder(app: AppHandle) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let profiles_dir = app_data_dir.join("profiles");
    if !profiles_dir.exists() {
        let _ = std::fs::create_dir_all(&profiles_dir);
    }
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let path_str = profiles_dir.to_string_lossy().to_string();
        let wide: Vec<u16> = OsStr::new(&path_str)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let op_open: Vec<u16> = OsStr::new("open")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        #[link(name = "shell32")]
        unsafe extern "system" {
            fn ShellExecuteW(
                hwnd: *mut std::ffi::c_void,
                lpOperation: *const u16,
                lpFile: *const u16,
                lpParameters: *const u16,
                lpDirectory: *const u16,
                nShowCmd: i32,
            ) -> isize;
        }
        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                op_open.as_ptr(),
                wide.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            );
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

/// Forwards log messages from Frontend / Injected scripts to terminal and log file
#[tauri::command]
pub fn log_frontend(app: AppHandle, level: Option<String>, message: String) {
    let lvl = level.as_deref().unwrap_or("info");
    let tag = match lvl.to_lowercase().as_str() {
        "error" => "\x1b[31m[FE - ERROR]\x1b[0m",
        "warn" => "\x1b[33m[FE - WARN]\x1b[0m",
        "debug" => "\x1b[35m[FE - DEBUG]\x1b[0m",
        _ => "\x1b[36m[FE - INFO]\x1b[0m",
    };
    println!("{} {}", tag, message);
    if let Ok(data_dir) = app.path().app_data_dir() {
        let log_dir = data_dir.join("logs");
        let _ = std::fs::create_dir_all(&log_dir);
        let log_file = log_dir.join("messenger_inject.log");
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&log_file) {
            let now = crate::utils::time::now_millis();
            let _ = writeln!(file, "[{}] [{}] {}", now, lvl.to_uppercase(), message);
        }
    }
}
