use tauri::AppHandle;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
#[link(name = "netapi32")]
unsafe extern "system" {
    fn NetUserGetInfo(
        servername: *const u16,
        username: *const u16,
        level: u32,
        bufptr: *mut *mut u8,
    ) -> u32;

    fn NetApiBufferFree(buffer: *mut u8) -> u32;
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct USER_INFO_10 {
    _usri10_name: *mut u16,
    _usri10_comment: *mut u16,
    _usri10_usr_comment: *mut u16,
    usri10_full_name: *mut u16,
}

#[cfg(target_os = "windows")]
fn get_windows_display_name(username: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let username_wide: Vec<u16> = OsStr::new(username)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut buf_ptr: *mut u8 = std::ptr::null_mut();

    unsafe {
        let status = NetUserGetInfo(
            std::ptr::null(),
            username_wide.as_ptr(),
            10,
            &mut buf_ptr,
        );

        if status == 0 && !buf_ptr.is_null() {
            let info = &*(buf_ptr as *const USER_INFO_10);
            let full_name = if !info.usri10_full_name.is_null() {
                let mut len = 0;
                while *info.usri10_full_name.add(len) != 0 {
                    len += 1;
                }
                let slice = std::slice::from_raw_parts(info.usri10_full_name, len);
                String::from_utf16_lossy(slice).trim().to_string()
            } else {
                String::new()
            };

            NetApiBufferFree(buf_ptr);

            if !full_name.is_empty() {
                return Some(full_name);
            }
        }
    }

    None
}

// Command lấy tên tài khoản Windows (ưu tiên Win32 Full Name, fallback về username hệ thống)
#[tauri::command]
pub fn get_system_username() -> String {
    let raw_username = std::env::var("USERNAME").or_else(|_| std::env::var("USER"));

    if let Ok(username) = raw_username {
        let trimmed = username.trim();
        if !trimmed.is_empty() {
            #[cfg(target_os = "windows")]
            {
                if let Some(display_name) = get_windows_display_name(trimmed) {
                    return display_name;
                }
            }

            // Fallback về username nếu không có Full Name (viết hoa chữ cái đầu nếu cần)
            let mut chars = trimmed.chars();
            if let Some(first) = chars.next() {
                return first.to_uppercase().collect::<String>() + chars.as_str();
            }
            return trimmed.to_string();
        }
    }

    "Người dùng".to_string()
}

// Command bật/tắt khởi động cùng Windows
#[tauri::command]
pub fn toggle_autostart(enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_path = current_exe.to_string_lossy().to_string();

        if enabled {
            let _ = std::process::Command::new("reg")
                .args(&[
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "DesktopBubbleChat",
                    "/t",
                    "REG_SZ",
                    "/d",
                    &format!("\"{}\"", exe_path),
                    "/f",
                ])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .output();
        } else {
            let _ = std::process::Command::new("reg")
                .args(&[
                    "delete",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "DesktopBubbleChat",
                    "/f",
                ])
                .creation_flags(0x08000000)
                .output();
        }
    }
    Ok(enabled)
}

// Command kiểm tra xem app đã bật khởi động cùng Windows chưa
#[tauri::command]
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "DesktopBubbleChat",
            ])
            .creation_flags(0x08000000)
            .output();

        if let Ok(out) = output {
            return out.status.success();
        }
    }
    false
}

// Command thoát ứng dụng hoàn toàn
#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

/// Command điều khiển chế độ Luôn nổi trên cùng (Always on Top) dành riêng cho các Bong bóng nổi
#[tauri::command]
pub fn set_always_on_top(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri::Manager;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    // 1. Lưu trạng thái vào ChatState để các Bubble khởi tạo / hiển thị sau này tuân thủ
    if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
        chat_state.is_always_on_top.store(enabled, Ordering::SeqCst);
    }

    // 2. Dashboard chính tuyệt đối KHÔNG ghim Always-on-top (luôn là cửa sổ tiêu chuẩn của OS)
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.set_always_on_top(false);
    }

    // 3. Áp dụng Always-on-top cho tất cả các cửa sổ Bong bóng nổi (bubble-01 -> bubble-05)
    for idx in 1..=5 {
        let bubble_label = format!("bubble-{:02}", idx);
        if let Some(w) = app.get_webview_window(&bubble_label) {
            let _ = w.set_always_on_top(enabled);
        }
    }

    // 4. Khung chat Messenger đồng bộ trạng thái nổi cùng bong bóng
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

// Command mở liên kết ngoài an toàn bằng trình duyệt mặc định của hệ điều hành
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
                    1, // SW_SHOWNORMAL
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

/// Command cầu nối chuyển log từ Frontend React thẳng vào Terminal Backend
#[tauri::command]
pub fn log_frontend(level: String, message: String) {
    let tag = match level.to_lowercase().as_str() {
        "error" => "\x1b[31m[FE - ERROR]\x1b[0m",
        "warn" => "\x1b[33m[FE - WARN]\x1b[0m",
        _ => "\x1b[36m[FE - INFO]\x1b[0m",
    };
    println!("{} {}", tag, message);
}
