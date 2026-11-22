use tauri::WebviewWindow;

pub struct MacOSThemeManager;

impl MacOSThemeManager {
    /// Parses a 6-digit hex color into normalized RGB components (0.0..=1.0)
    pub fn parse_hex_rgb(hex: &str) -> Option<(f64, f64, f64)> {
        let clean = hex.trim().trim_start_matches('#');
        if !clean.is_ascii() || clean.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&clean[0..2], 16).ok()? as f64 / 255.0;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()? as f64 / 255.0;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()? as f64 / 255.0;
        Some((r, g, b))
    }

    /// Evaluates whether parsed RGB color requires dark titlebar appearance
    pub fn is_dark_color(r: f64, g: f64, b: f64) -> bool {
        (0.299 * r + 0.587 * g + 0.114 * b) < 0.55
    }

    /// Configures macOS window appearance, titlebar transparency, and accent background color on main thread
    pub fn apply_window_theme_color(window: &WebviewWindow, hex: &str, _border_hex: &str) {
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::{id, nil, YES};
            use cocoa::foundation::NSString;
            use objc::{class, msg_send, sel, sel_impl};
            let rgb = Self::parse_hex_rgb(hex);
            let win_clone = window.clone();
            let _ = window.run_on_main_thread(move || {
                if let Ok(ns_win) = win_clone.ns_window() {
                    super::bubble::window::ffi::with_autorelease_pool(|| unsafe {
                        let win = ns_win as id;
                        let _: () = msg_send![win, setTitlebarAppearsTransparent: YES];
                        if let Some((r, g, b)) = rgb {
                            let bg: id = msg_send![class!(NSColor), colorWithSRGBRed: r green: g blue: b alpha: 1.0f64];
                            if bg != nil {
                                let _: () = msg_send![win, setBackgroundColor: bg];
                            }
                            let app_name = if Self::is_dark_color(r, g, b) { "NSAppearanceNameDarkAqua" } else { "NSAppearanceNameAqua" };
                            let name_ns = NSString::alloc(nil).init_str(app_name);
                            let appearance: id = msg_send![class!(NSAppearance), appearanceNamed: name_ns];
                            if appearance != nil {
                                let _: () = msg_send![win, setAppearance: appearance];
                            }
                            let _: () = msg_send![name_ns, release];
                        }
                    });
                }
            });
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (window, hex);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_rgb_and_dark_detection() {
        let (r, g, b) = MacOSThemeManager::parse_hex_rgb("#06B6D4").unwrap();
        assert!((r - 6.0 / 255.0).abs() < 1e-6);
        assert!((g - 182.0 / 255.0).abs() < 1e-6);
        assert!((b - 212.0 / 255.0).abs() < 1e-6);
        assert!(MacOSThemeManager::is_dark_color(0.05, 0.05, 0.1));
        assert!(!MacOSThemeManager::is_dark_color(0.95, 0.95, 0.98));
        assert!(MacOSThemeManager::parse_hex_rgb("invalid").is_none());
    }
}
