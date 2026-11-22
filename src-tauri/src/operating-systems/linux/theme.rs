use tauri::WebviewWindow;

pub struct LinuxThemeManager;

impl LinuxThemeManager {
    /// Parses a #RRGGBB color string and returns true if its luminance is dark
    pub fn parse_hex_is_dark(hex: &str) -> bool {
        let clean = hex.trim().trim_start_matches('#');
        if clean.is_ascii() && clean.len() == 6 {
            let r = u8::from_str_radix(&clean[0..2], 16).unwrap_or(255) as f64;
            let g = u8::from_str_radix(&clean[2..4], 16).unwrap_or(255) as f64;
            let b = u8::from_str_radix(&clean[4..6], 16).unwrap_or(255) as f64;
            let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            return luma < 140.0;
        }
        false
    }

    /// Configures Linux window light or dark appearance from theme accent color
    pub fn apply_window_theme_color(window: &WebviewWindow, hex: &str, _border_hex: &str) {
        #[cfg(target_os = "linux")]
        {
            let theme = if Self::parse_hex_is_dark(hex) { tauri::Theme::Dark } else { tauri::Theme::Light };
            let _ = window.set_theme(Some(theme));
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (window, hex);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_manager_exists() {
        let _ = LinuxThemeManager;
    }
}
