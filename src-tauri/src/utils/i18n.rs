use serde::Deserialize;

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

// Returns localized tray strings from embedded JSON
pub fn get_tray_locale(locale: &str) -> TrayLocale {
    let json_str = match locale {
        "en" => EN_TRAY_JSON,
        _ => VI_TRAY_JSON,
    };
    serde_json::from_str(json_str).unwrap_or_else(|_| TrayLocale {
        open_dashboard: "Open Dashboard".into(),
        close_dashboard: "Close Dashboard".into(),
        toggle_all_bubbles: "Toggle All Bubbles".into(),
        hide_all_bubbles: "Hide All Bubbles".into(),
        show_all_bubbles: "Show All Bubbles".into(),
        account_label: "Messenger {{index}}".into(),
        quit_app: "Quit Application".into(),
        confirm_exit_title: "Confirm Exit DesktopBubbleChat".into(),
        confirm_exit_text: "Are you sure you want to completely exit DesktopBubbleChat?\n\n• All floating bubbles will close.\n• Messenger chat windows will stop running.".into(),
    })
}
