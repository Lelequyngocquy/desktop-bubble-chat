use tauri::WebviewWindow;

#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmSetWindowAttribute(hwnd: *mut std::ffi::c_void, dwAttribute: u32, pvAttribute: *const std::ffi::c_void, cbAttribute: u32) -> i32;
}

fn hex_to_colorref(hex: &str) -> u32 {
    let clean = hex.trim_start_matches('#');
    if clean.len() == 6 {
        let r = u32::from_str_radix(&clean[0..2], 16).unwrap_or(0);
        let g = u32::from_str_radix(&clean[2..4], 16).unwrap_or(0);
        let b = u32::from_str_radix(&clean[4..6], 16).unwrap_or(0);
        r | (g << 8) | (b << 16)
    } else {
        0
    }
}

pub struct WindowsThemeManager;

impl WindowsThemeManager {
    /// Applies custom titlebar caption and border colors via Desktop Window Manager
    pub fn apply_window_theme_color(window: &WebviewWindow, hex: &str, border_hex: &str) {
        if let Ok(hwnd) = window.hwnd() {
            let raw_hwnd = hwnd.0 as *mut std::ffi::c_void;
            let caption_color = hex_to_colorref(hex);
            let border_color = hex_to_colorref(border_hex);
            let text_color: u32 = 0x00FFFFFF;
            const DWMWA_BORDER_COLOR: u32 = 34;
            const DWMWA_CAPTION_COLOR: u32 = 35;
            const DWMWA_TEXT_COLOR: u32 = 36;
            unsafe {
                let _ = DwmSetWindowAttribute(raw_hwnd, DWMWA_CAPTION_COLOR, &caption_color as *const _ as *const std::ffi::c_void, std::mem::size_of::<u32>() as u32);
                let _ = DwmSetWindowAttribute(raw_hwnd, DWMWA_TEXT_COLOR, &text_color as *const _ as *const std::ffi::c_void, std::mem::size_of::<u32>() as u32);
                let _ = DwmSetWindowAttribute(raw_hwnd, DWMWA_BORDER_COLOR, &border_color as *const _ as *const std::ffi::c_void, std::mem::size_of::<u32>() as u32);
            }
        }
    }
}
