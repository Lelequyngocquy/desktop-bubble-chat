use std::path::PathBuf;
use tauri::{AppHandle, Manager};

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
