use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Manager};

use crate::platforms::messenger::windows::get_messenger_window_label;
use crate::state::ChatState;
use crate::windows::bubble::get_bubble_window_label;

pub struct StorageService;

impl StorageService {
    /// Đóng và hủy cửa sổ webview, ẩn bubble và dọn dẹp profile data trên ổ cứng
    pub fn clean_account_session(
        app: &AppHandle,
        state: &Arc<ChatState>,
        account_index: u32,
    ) -> Result<(), String> {
        let idx = account_index;
        let label = get_messenger_window_label(idx);
        println!("[StorageService] Đóng cửa sổ và dọn dẹp profile cho Messenger {:02}", idx);
        state.clear_account_auth(idx);

        // 1. Huỷ cửa sổ chat webview
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

        // 2. Ẩn cửa sổ bubble tương ứng (giữ nguyên handle để có thể show lại tức thì khi cần)
        let bubble_label = get_bubble_window_label(idx);
        if let Some(bw) = app.get_webview_window(&bubble_label) {
            let _ = bw.hide();
        }

        // 3. Xoá triệt để thư mục profile/storage trên ổ cứng (User Data Directory)
        // Chạy ngắt tiến trình msedgewebview2 phụ thuộc trước để nhả file lock
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
                        eprintln!("[StorageService ERROR] Thực thi lệnh PowerShell thất bại: {}", e);
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
                                        "[StorageService] Đã dọn dẹp profile directory '{:?}' ở lần thử {}",
                                        profile_dir, attempt
                                    );
                                    deleted = true;
                                    break;
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[StorageService WARNING] Lần thử {} xóa thư mục thất bại: {}. Đang thử lại...",
                                        attempt, e
                                    );
                                    std::thread::sleep(Duration::from_millis(200));
                                }
                            }
                        }
                        if !deleted {
                            eprintln!(
                                "[StorageService ERROR] Không thể xóa toàn bộ thư mục '{:?}' sau 5 lần thử.",
                                profile_dir
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[StorageService ERROR] Không lấy được app_data_dir: {}", e);
                }
            }
        });

        Ok(())
    }
}
