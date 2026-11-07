#[derive(Clone, Copy, Debug)]
pub struct AccountTheme {
    pub hex: &'static str,
    pub border_hex: &'static str,
}

pub fn get_account_theme(idx: u32) -> AccountTheme {
    match idx {
        1 => AccountTheme {
            hex: "#06B6D4",
            border_hex: "#0891B2",
        },
        2 => AccountTheme {
            hex: "#EC4899",
            border_hex: "#DB2777",
        },
        3 => AccountTheme {
            hex: "#F97316",
            border_hex: "#EA580C",
        },
        4 => AccountTheme {
            hex: "#8B5CF6",
            border_hex: "#7C3AED",
        },
        5 => AccountTheme {
            hex: "#10B981",
            border_hex: "#059669",
        },
        _ => AccountTheme {
            hex: "#06B6D4",
            border_hex: "#0891B2",
        },
    }
}

pub fn hex_to_colorref(hex: &str) -> u32 {
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

/// Applies window theme color via platform theme manager
pub fn apply_window_theme_color(window: &tauri::WebviewWindow, hex: &str, border_hex: &str) {
    crate::operating_systems::current::ThemeManager::apply_window_theme_color(window, hex, border_hex);
}
