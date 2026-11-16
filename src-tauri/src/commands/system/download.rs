use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;

use crate::state::ChatState;

/// Internal helper to present native save dialog on isolated STA thread
pub fn show_save_dialog_internal(
    app: &AppHandle,
    state: &ChatState,
    window_label: &str,
    default_name: Option<&str>,
    default_dir: Option<PathBuf>,
) -> Option<PathBuf> {
    let default_file = default_name.unwrap_or("downloaded_file").to_string();
    println!("\x1b[35m[BE - Download] show_save_dialog_internal: window='{}', default='{}'\x1b[0m", window_label, default_file);
    state.set_dialog_active(window_label, true);
    let win_opt = app.get_webview_window(window_label);
    let was_topmost = win_opt.as_ref().map(|w| w.is_always_on_top().unwrap_or(false)).unwrap_or(false);
    if let Some(w) = win_opt.as_ref() {
        let _ = w.set_always_on_top(false);
    }
    let handle = std::thread::spawn(move || {
        let mut dialog = rfd::FileDialog::new().set_file_name(&default_file);
        if let Some(ref dir) = default_dir {
            dialog = dialog.set_directory(dir);
        }
        dialog.save_file()
    });
    let result = handle.join().ok().flatten();
    println!("\x1b[35m[BE - Download] show_save_dialog_internal result: '{:?}'\x1b[0m", result);
    state.set_dialog_active(window_label, false);
    if let Some(w) = win_opt.as_ref() {
        let _ = w.set_always_on_top(was_topmost);
        let _ = w.set_focus();
    }
    result
}

/// Command prompting native file save dialog
#[tauri::command]
pub fn prompt_save_file(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
    window_label: String,
    default_name: Option<String>,
) -> Result<Option<String>, String> {
    println!("\x1b[35m[BE - Download] prompt_save_file command called for window='{}', default='{:?}'\x1b[0m", window_label, default_name);
    let path_opt = show_save_dialog_internal(&app, &state, &window_label, default_name.as_deref(), None);
    Ok(path_opt.map(|p| p.to_string_lossy().to_string()))
}

/// Command appending or creating binary file from Base64 chunk
#[tauri::command]
pub fn save_blob_chunk(
    file_path: String,
    chunk_base64: String,
    is_first_chunk: bool,
) -> Result<(), String> {
    use base64::Engine;
    use std::fs::OpenOptions;
    use std::io::Write;
    println!("\x1b[35m[BE - Download] save_blob_chunk: path='{}', chunk_size={}, is_first={}\x1b[0m", file_path, chunk_base64.len(), is_first_chunk);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&chunk_base64)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    let mut file = if is_first_chunk {
        std::fs::File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?
    } else {
        OpenOptions::new()
            .append(true)
            .open(&file_path)
            .map_err(|e| format!("Failed to open file for append: {}", e))?
    };
    file.write_all(&bytes).map_err(|e| format!("Failed to write chunk: {}", e))?;
    Ok(())
}

/// Command revealing downloaded file in system file manager
#[tauri::command]
pub fn show_file_in_folder(path: String) -> Result<(), String> {
    println!("\x1b[35m[BE - Download] show_file_in_folder: path='{}'\x1b[0m", path);
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("File path does not exist".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        let parent = p.parent().unwrap_or(p);
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Command notifying user of download completion
#[tauri::command]
pub fn notify_download_complete(
    app: AppHandle,
    file_name: String,
    _file_path: String,
) -> Result<(), String> {
    println!("\x1b[35m[BE - Download] notify_download_complete: file='{}', path='{}'\x1b[0m", file_name, _file_path);
    app.notification()
        .builder()
        .title("DesktopBubbleChat")
        .body(format!("Downloaded: {}", file_name))
        .show()
        .map_err(|e| e.to_string())?;
    Ok(())
}
