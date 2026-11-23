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
    crate::presentation::main_window::destroy_or_hibernate_main_window(&app);
}

/// Shows or recreates main dashboard window
#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    let _ = crate::presentation::main_window::get_or_create_main_window(&app);
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

/// Updates display language in SSOT user cache, refreshes tray, and broadcasts settings snapshot
pub fn apply_app_language(
    app: &AppHandle,
    state: &crate::state::ChatState,
    language: &str,
) {
    let normalized = if language == "vi" { "vi" } else { "en" };
    state.update_user_config(app, |c| {
        c.locale = Some(normalized.to_string());
    });
    let _ = crate::tray::update_tray_menu(app);
}

/// Sets application display language, updates persistent cache, and updates system tray
#[tauri::command]
pub fn set_app_language(
    app: AppHandle,
    state: tauri::State<'_, Arc<crate::state::ChatState>>,
    language: String,
) -> Result<(), String> {
    apply_app_language(&app, &state, &language);
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
    if !matches!(theme_mode.as_str(), "light" | "dark" | "system") {
        return Err(format!("Invalid theme_mode '{}': expected light, dark, or system", theme_mode));
    }
    state.update_user_config(&app, |c| {
        c.theme_mode = Some(theme_mode);
    });
    Ok(())
}

/// Sets application update mode (auto or manual) in user configuration cache
#[tauri::command]
pub fn set_update_mode(
    app: AppHandle,
    state: tauri::State<'_, Arc<crate::state::ChatState>>,
    mode: String,
) -> Result<(), String> {
    if !matches!(mode.as_str(), "auto" | "manual") {
        return Err(format!("Invalid update_mode '{}': expected auto or manual", mode));
    }
    state.update_user_config(&app, |c| {
        c.update_mode = Some(mode);
    });
    Ok(())
}

/// Returns OS-appropriate global shortcut accelerators for toggling all bubbles
pub fn global_shortcuts_for_os(os: &str) -> &'static [&'static str] {
    match os {
        "windows" => &["CommandOrControl+Shift+M", "Super+Shift+M"],
        "macos" => &["CommandOrControl+Shift+M", "Ctrl+Shift+M"],
        "linux" => &["CommandOrControl+Shift+M"],
        _ => &["CommandOrControl+Shift+M"],
    }
}

/// Applies global shortcut registration state for bubble toggle hotkeys
pub fn apply_global_shortcut_registration(app: &AppHandle, enabled: bool) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
    for &sc in global_shortcuts_for_os(std::env::consts::OS) {
        if let Ok(shortcut) = sc.parse::<Shortcut>() {
            if enabled {
                let _ = app.global_shortcut().register(shortcut);
            } else {
                let _ = app.global_shortcut().unregister(shortcut);
            }
        }
    }
}

/// Enables or disables global bubble toggle shortcuts and persists the setting
#[tauri::command]
pub fn set_global_shortcut_enabled(
    app: AppHandle,
    state: tauri::State<'_, Arc<crate::state::ChatState>>,
    enabled: bool,
) -> Result<(), String> {
    state.update_user_config(&app, |c| {
        c.is_global_shortcut_enabled = enabled;
    });
    apply_global_shortcut_registration(&app, enabled);
    Ok(())
}

/// Exits application completely
#[tauri::command]
pub fn exit_app(app: AppHandle) {
    println!("\x1b[33m[AppLifecycle] Explicit application exit initiated...\x1b[0m");
    if let Some(state) = app.try_state::<Arc<crate::state::ChatState>>() {
        state.is_explicit_exit.store(true, std::sync::atomic::Ordering::SeqCst);
        state.update_user_config(&app, |c| {
            c.is_dashboard_open = true;
        });
    }
    for (_, window) in app.webview_windows() {
        let _ = window.destroy();
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
        let _ = window.destroy();
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
    crate::operating_systems::current::bubble::set_native_bubble_always_on_top(enabled);
    for idx in 1..=crate::state::MAX_ACCOUNTS {
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

static LAST_OPENED_URL: std::sync::Mutex<Option<(String, std::time::Instant)>> = std::sync::Mutex::new(None);

/// Validates external HTTP(S) URL strictly and avoids shell command injection
fn validate_external_http_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("Invalid external URL".into());
    }
    let parsed = tauri::Url::parse(trimmed).map_err(|_| "Invalid external URL".to_string())?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err("Invalid URL scheme".into());
    }
    Ok(parsed.to_string())
}

/// Opens validated HTTP(S) URL using system default browser with dedup protection
pub fn open_external_url_internal(app: &AppHandle, url: &str) -> Result<(), String> {
    let safe_url = validate_external_http_url(url)?;
    if let Ok(mut guard) = LAST_OPENED_URL.lock() {
        if let Some((ref last_url, ref last_time)) = *guard
            && last_url == &safe_url && last_time.elapsed() < std::time::Duration::from_millis(1000) {
                return Ok(());
            }
        *guard = Some((safe_url.clone(), std::time::Instant::now()));
    }
    use tauri_plugin_opener::OpenerExt;
    if app.opener().open_url(&safe_url, None::<&str>).is_ok() {
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let wide_op: Vec<u16> = OsStr::new("open").encode_wide().chain(std::iter::once(0)).collect();
        let wide_url: Vec<u16> = OsStr::new(&safe_url).encode_wide().chain(std::iter::once(0)).collect();
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
        let res = unsafe { ShellExecuteW(std::ptr::null_mut(), wide_op.as_ptr(), wide_url.as_ptr(), std::ptr::null(), std::ptr::null(), 1) };
        if res > 32 {
            return Ok(());
        }
        Err(format!("ShellExecuteW failed with code {}", res))
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(&safe_url).spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(&safe_url).spawn().map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Failed to open external URL".to_string())
    }
}

/// Opens external URL safely using default OS browser
#[tauri::command]
pub fn open_external_url(app: AppHandle, url: String) -> Result<(), String> {
    open_external_url_internal(&app, &url)
}

/// Opens profiles directory in File Explorer
#[tauri::command]
pub fn open_profiles_folder(app: AppHandle) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let profiles_dir = app_data_dir.join("profiles");
    if !profiles_dir.exists() {
        let _ = std::fs::create_dir_all(&profiles_dir);
    }
    use tauri_plugin_opener::OpenerExt;
    if app.opener().open_path(profiles_dir.to_string_lossy().to_string(), None::<&str>).is_ok() {
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let path_str = profiles_dir.to_string_lossy().to_string();
        let wide: Vec<u16> = OsStr::new(&path_str).encode_wide().chain(std::iter::once(0)).collect();
        let op_open: Vec<u16> = OsStr::new("open").encode_wide().chain(std::iter::once(0)).collect();
        #[link(name = "shell32")]
        unsafe extern "system" {
            fn ShellExecuteW(hwnd: *mut std::ffi::c_void, lpOperation: *const u16, lpFile: *const u16, lpParameters: *const u16, lpDirectory: *const u16, nShowCmd: i32) -> isize;
        }
        unsafe {
            ShellExecuteW(std::ptr::null_mut(), op_open.as_ptr(), wide.as_ptr(), std::ptr::null(), std::ptr::null(), 1);
        }
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&profiles_dir).spawn();
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&profiles_dir).spawn();
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Ok(())
    }
}

/// Forwards log messages from Frontend / Injected scripts to terminal and rotated log file
#[tauri::command]
pub fn log_frontend(app: AppHandle, level: Option<String>, message: String) {
    const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;
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
        if let Ok(meta) = std::fs::metadata(&log_file)
            && meta.len() >= MAX_LOG_BYTES {
                let rotated = log_dir.join("messenger_inject.log.old");
                let _ = std::fs::remove_file(&rotated);
                let _ = std::fs::rename(&log_file, &rotated);
            }
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&log_file) {
            let now = crate::utils::time::now_millis();
            let _ = writeln!(file, "[{}] [{}] {}", now, lvl.to_uppercase(), message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_external_http_url() {
        assert!(validate_external_http_url("https://github.com/test?a=1").is_ok());
        assert!(validate_external_http_url("http://localhost:1420").is_ok());
        assert!(validate_external_http_url("javascript:alert(1)").is_err());
        assert!(validate_external_http_url("file:///C:/Windows/win.ini").is_err());
        assert!(validate_external_http_url("https://example.com & calc.exe").is_err());
        assert!(validate_external_http_url("https://example.com\ncalc").is_err());
        assert!(validate_external_http_url("").is_err());
    }
}
