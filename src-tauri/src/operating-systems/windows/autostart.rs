use std::os::windows::process::CommandExt;

#[link(name = "netapi32")]
unsafe extern "system" {
    fn NetUserGetInfo(servername: *const u16, username: *const u16, level: u32, bufptr: *mut *mut u8) -> u32;
    fn NetApiBufferFree(buffer: *mut u8) -> u32;
}

#[repr(C)]
struct USER_INFO_10 {
    _usri10_name: *mut u16,
    _usri10_comment: *mut u16,
    _usri10_usr_comment: *mut u16,
    usri10_full_name: *mut u16,
}

pub struct WindowsAutostartManager;

impl WindowsAutostartManager {
    /// Retrieves Windows account display name via NetUserGetInfo
    pub fn get_windows_display_name(username: &str) -> Option<String> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let username_wide: Vec<u16> = OsStr::new(username).encode_wide().chain(std::iter::once(0)).collect();
        let mut buf_ptr: *mut u8 = std::ptr::null_mut();
        unsafe {
            let status = NetUserGetInfo(std::ptr::null(), username_wide.as_ptr(), 10, &mut buf_ptr);
            if status == 0 && !buf_ptr.is_null() {
                let info = &*(buf_ptr as *const USER_INFO_10);
                let full_name = if !info.usri10_full_name.is_null() {
                    let mut len = 0;
                    while *info.usri10_full_name.add(len) != 0 { len += 1; }
                    let slice = std::slice::from_raw_parts(info.usri10_full_name, len);
                    String::from_utf16_lossy(slice).trim().to_string()
                } else {
                    String::new()
                };
                NetApiBufferFree(buf_ptr);
                if !full_name.is_empty() { return Some(full_name); }
            }
        }
        None
    }
    /// Toggles Windows registry Run key
    pub fn toggle_autostart_registry(enabled: bool) -> Result<bool, String> {
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_path = current_exe.to_string_lossy().to_string();
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        if enabled {
            let _ = std::process::Command::new("reg")
                .args(&["add", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "DesktopBubbleChat", "/t", "REG_SZ", "/d", &format!("\"{}\"", exe_path), "/f"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        } else {
            let _ = std::process::Command::new("reg")
                .args(&["delete", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "DesktopBubbleChat", "/f"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }
        Ok(enabled)
    }
    /// Queries Windows registry Run key status
    pub fn is_autostart_registry_enabled() -> bool {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let output = std::process::Command::new("reg")
            .args(&["query", "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run", "/v", "DesktopBubbleChat"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        if let Ok(out) = output { out.status.success() } else { false }
    }
}
