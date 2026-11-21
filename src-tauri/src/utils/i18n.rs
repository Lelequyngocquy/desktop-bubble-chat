use serde::Deserialize;
use std::sync::OnceLock;

// Tray localization dictionary loaded from shared locales/
#[derive(Debug, Clone, Deserialize)]
pub struct TrayLocale {
    pub open_dashboard: String,
    pub close_dashboard: String,
    pub toggle_all_bubbles: String,
    pub hide_all_bubbles: String,
    pub show_all_bubbles: String,
    pub account_label: String,
    pub quit_app: String,
    #[serde(default)]
    pub confirm_exit_title: String,
    #[serde(default)]
    pub confirm_exit_text: String,
}

const EN_TRAY_JSON: &str = include_str!("../../../locales/en/tray.json");
const VI_TRAY_JSON: &str = include_str!("../../../locales/vi/tray.json");
static EN_TRAY_CACHE: OnceLock<TrayLocale> = OnceLock::new();
static VI_TRAY_CACHE: OnceLock<TrayLocale> = OnceLock::new();

fn default_en_tray_locale() -> TrayLocale {
    TrayLocale {
        open_dashboard: "Open Dashboard".into(),
        close_dashboard: "Close Dashboard".into(),
        toggle_all_bubbles: "Toggle All Bubbles".into(),
        hide_all_bubbles: "Hide All Bubbles".into(),
        show_all_bubbles: "Show All Bubbles".into(),
        account_label: "Messenger {{index}}".into(),
        quit_app: "Quit Application".into(),
        confirm_exit_title: "Confirm Exit DesktopBubbleChat".into(),
        confirm_exit_text: "Are you sure you want to completely exit DesktopBubbleChat?\n\n• All floating bubbles will close.\n• Messenger chat windows will stop running.".into(),
    }
}

/// Returns cached localized tray strings from embedded JSON (defaults unknown locales to English)
pub fn get_tray_locale(locale: &str) -> TrayLocale {
    if locale.starts_with("vi") {
        VI_TRAY_CACHE
            .get_or_init(|| serde_json::from_str(VI_TRAY_JSON).unwrap_or_else(|_| default_en_tray_locale()))
            .clone()
    } else {
        EN_TRAY_CACHE
            .get_or_init(|| serde_json::from_str(EN_TRAY_JSON).unwrap_or_else(|_| default_en_tray_locale()))
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tray_locale_caching_and_fallback() {
        let en = get_tray_locale("en");
        let vi = get_tray_locale("vi");
        let unknown = get_tray_locale("fr");
        assert_eq!(en.open_dashboard, unknown.open_dashboard);
        assert_ne!(en.open_dashboard, vi.open_dashboard);
    }
}
