use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use crate::extensions::{self, ProfileExtension};
use crate::platforms::messenger::windows::chat_window::get_messenger_window_label;

/// Retrieves list of installed extensions for a Messenger profile
#[tauri::command]
pub fn get_profile_extensions(app: AppHandle, account_index: u32) -> Result<Vec<ProfileExtension>, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    extensions::list_profile_extensions(&app_data, account_index)
}

/// Loads and installs an unpacked extension folder into a Messenger profile
#[tauri::command]
pub fn load_profile_extension(app: AppHandle, account_index: u32, source_path: Option<String>) -> Result<ProfileExtension, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = match source_path {
        Some(ref p) if !p.trim().is_empty() => PathBuf::from(p.trim()),
        _ => extensions::pick_folder_dialog().ok_or_else(|| "Folder selection cancelled".to_string())?,
    };
    extensions::install_unpacked_extension(&app_data, account_index, &path)
}

/// Toggles an extension between enabled and disabled states
#[tauri::command]
pub fn toggle_profile_extension(app: AppHandle, account_index: u32, extension_id: String, enabled: bool) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    extensions::toggle_profile_extension(&app_data, account_index, &extension_id, enabled)
}

/// Deletes an extension from a Messenger profile
#[tauri::command]
pub fn remove_profile_extension(app: AppHandle, account_index: u32, extension_id: String) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    extensions::remove_profile_extension(&app_data, account_index, &extension_id)
}

/// Opens the profile extensions folder in OS file explorer
#[tauri::command]
pub fn open_profile_extensions_folder(app: AppHandle, account_index: u32) -> Result<(), String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let (enabled_dir, _) = extensions::get_profile_extension_dirs(&app_data, account_index);
    use tauri_plugin_opener::OpenerExt;
    if app.opener().open_path(enabled_dir.to_string_lossy().to_string(), None::<&str>).is_ok() {
        return Ok(());
    }
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let path_str = enabled_dir.to_string_lossy().to_string();
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
        let _ = std::process::Command::new("open").arg(&enabled_dir).spawn();
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&enabled_dir).spawn();
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Ok(())
    }
}

/// Reloads the webview of a specific Messenger account
#[tauri::command]
pub fn reload_messenger_window(app: AppHandle, account_index: u32) -> Result<(), String> {
    let label = get_messenger_window_label(account_index);
    if let Some(win) = app.get_webview_window(&label) {
        win.eval("window.location.reload();").map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err(format!("Messenger window '{}' not found", label))
}
