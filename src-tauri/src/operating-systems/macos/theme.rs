use tauri::WebviewWindow;

pub struct MacOSThemeManager;

impl MacOSThemeManager {
    /// Configures macOS window appearance and titlebar styling
    pub fn apply_window_theme_color(window: &WebviewWindow, _hex: &str, _border_hex: &str) {
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::id;
            use objc::{msg_send, sel, sel_impl};
            if let Ok(ns_win) = window.ns_window() {
                unsafe {
                    let win = ns_win as id;
                    let _: () = msg_send![win, setTitlebarAppearsTransparent: true];
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = window;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_exists() {
        let _ = MacOSThemeManager;
    }
}
