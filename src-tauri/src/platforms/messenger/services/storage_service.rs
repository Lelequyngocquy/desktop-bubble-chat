use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::platforms::messenger::windows::get_messenger_window_label;
use crate::state::ChatState;
use crate::windows::bubble::get_bubble_window_label;

pub struct StorageService;

impl StorageService {
    /// Closes and destroys webview window, hides bubble, and cleans profile data on disk
    pub fn clean_account_session(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        println!("[StorageService] Closing window and cleaning profile for Messenger {:02}", idx);
        state.clear_account_auth(idx);
        let _ = crate::tray::update_tray_menu(app);
        let win = app
            .get_webview_window(&label)
            .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });
        if let Some(chat_window) = win {
            let _ = chat_window.hide();
            let _ = chat_window.destroy();
        }
        if idx == 1 {
            if let Some(old) = app.get_webview_window("chat-messenger") {
                let _ = old.hide();
                let _ = old.destroy();
            }
        }
        let bubble_label = get_bubble_window_label(idx);
        if let Some(bw) = app.get_webview_window(&bubble_label) {
            let _ = bw.hide();
        }
        let target_needle = format!("messenger_{:02}", idx);
        let app_handle = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let kill_cmd = format!(
                    "Get-CimInstance Win32_Process | Where-Object {{ $_.Name -like '*msedgewebview2*' -and $_.CommandLine -like '*{}*' }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }}",
                    target_needle
                );
                match std::process::Command::new("powershell")
                    .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &kill_cmd])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output()
                {
                    Ok(output) => {
                        if !output.status.success() {
                            eprintln!(
                                "[StorageService WARNING] Stop-Process WebView2 exit with status: {:?}",
                                output.status
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[StorageService ERROR] PowerShell command failed: {}", e);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(150));
            match app_handle.path().app_data_dir() {
                Ok(mut profile_dir) => {
                    profile_dir.push("profiles");
                    profile_dir.push(&target_needle);
                    if profile_dir.exists() {
                        let mut deleted = false;
                        for attempt in 1..=5 {
                            match std::fs::remove_dir_all(&profile_dir) {
                                Ok(_) => {
                                    println!(
                                        "[StorageService] Cleaned profile directory '{:?}' on attempt {}",
                                        profile_dir, attempt
                                    );
                                    deleted = true;
                                    break;
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[StorageService WARNING] Attempt {} failed to delete directory: {}. Retrying...",
                                        attempt, e
                                    );
                                    std::thread::sleep(Duration::from_millis(200));
                                }
                            }
                        }
                        if !deleted {
                            eprintln!(
                                "[StorageService ERROR] Failed to delete directory '{:?}' after 5 attempts.",
                                profile_dir
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[StorageService ERROR] Failed to get app_data_dir: {}", e);
                }
            }
        });
        Ok(())
    }

    /// Terminates WebView2 process of specified account to reclaim RAM, preserving profile/cache on disk
    pub fn terminate_profile_process_only(account_index: u32) {
        let target_needle = format!("messenger_{:02}", account_index);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let kill_cmd = format!(
                "Get-CimInstance Win32_Process | Where-Object {{ $_.Name -like '*msedgewebview2*' -and $_.CommandLine -like '*{}*' }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }}",
                target_needle
            );
            let output = std::process::Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &kill_cmd])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
            match output {
                Ok(out) => {
                    println!(
                        "[StorageService] Terminated WebView2 process for profile '{}' (status: {:?})",
                        target_needle, out.status
                    );
                }
                Err(e) => {
                    eprintln!(
                        "[StorageService ERROR] Failed to invoke PowerShell to terminate process {}: {}",
                        target_needle, e
                    );
                }
            }
        }
    }
}
