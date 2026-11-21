pub use crate::presentation::bubble::renderer::theme::{get_account_theme, AccountTheme};

pub fn hex_to_colorref(hex: &str) -> u32 {
    let clean = hex.trim_start_matches('#');
    if clean.is_ascii() && clean.len() == 6 {
        let r = u32::from_str_radix(&clean[0..2], 16).unwrap_or(0);
        let g = u32::from_str_radix(&clean[2..4], 16).unwrap_or(0);
        let b = u32::from_str_radix(&clean[4..6], 16).unwrap_or(0);
        r | (g << 8) | (b << 16)
    } else {
        0
    }
}

/// Applies window theme color via platform theme manager
pub fn apply_window_theme_color(window: &tauri::WebviewWindow, hex: &str, border_hex: &str) {
    crate::operating_systems::current::ThemeManager::apply_window_theme_color(window, hex, border_hex);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_colorref_rejects_non_ascii_without_panic() {
        assert_eq!(hex_to_colorref("#06B6D4"), 0x00D4B606);
        assert_eq!(hex_to_colorref("ééé"), 0);
        assert_eq!(hex_to_colorref("🔥a"), 0);
    }
}
