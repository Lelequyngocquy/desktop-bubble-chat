use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// Persistent user configuration state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUserConfig {
    pub is_dashboard_open: bool,
    pub active_bubble_accounts: Vec<u32>,
    pub is_notifications_enabled: bool,
    pub muted_accounts: Vec<u32>,
    pub bubble_size: String,
    pub is_always_on_top: bool,
    pub is_auto_start: bool,
    #[serde(default = "default_locale")]
    pub locale: Option<String>,
    #[serde(default = "default_true", alias = "is_auto_trim_enabled")]
    pub is_auto_trim_enabled: bool,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: Option<String>,
    #[serde(default = "default_position_mode")]
    pub bubble_position_mode: String,
}

fn default_locale() -> Option<String> {
    Some("en".to_string())
}

fn default_true() -> bool {
    true
}

fn default_theme_mode() -> Option<String> {
    Some("system".to_string())
}

fn default_position_mode() -> String {
    "auto_tuck".to_string()
}

impl Default for AppUserConfig {
    fn default() -> Self {
        Self {
            is_dashboard_open: true,
            active_bubble_accounts: vec![1],
            is_notifications_enabled: true,
            muted_accounts: Vec::new(),
            bubble_size: "md".to_string(),
            is_always_on_top: true,
            is_auto_start: false,
            locale: Some("en".to_string()),
            is_auto_trim_enabled: true,
            theme_mode: Some("system".to_string()),
            bubble_position_mode: "auto_tuck".to_string(),
        }
    }
}

impl AppUserConfig {
    pub fn bubble_size_px(&self) -> f64 {
        match self.bubble_size.as_str() {
            "sm" => 48.0,
            "lg" => 64.0,
            _ => 56.0,
        }
    }
    pub fn current_locale(&self) -> &str {
        self.locale.as_deref().unwrap_or("en")
    }
    pub fn current_theme_mode(&self) -> &str {
        self.theme_mode.as_deref().unwrap_or("system")
    }
    pub fn is_free_position_mode(&self) -> bool {
        self.bubble_position_mode == "free"
    }
}

// Returns user_cache.json path in OS app data directory
pub fn get_user_cache_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|p| p.join("user_cache.json"))
}

// Loads user cache from disk or returns default configuration
pub fn load_user_cache(app: &AppHandle) -> AppUserConfig {
    if let Some(path) = get_user_cache_path(app) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppUserConfig>(&content) {
                    println!("\x1b[32m[UserCache] Successfully loaded user cache: {:?}\x1b[0m", config);
                    return config;
                }
            }
        }
    }
    println!("[UserCache] No cache found on disk, initializing defaults");
    AppUserConfig::default()
}

// Saves user configuration to disk atomically
pub fn save_user_cache(app: &AppHandle, config: &AppUserConfig) {
    if let Some(path) = get_user_cache_path(app) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(config) {
            let _ = fs::write(&path, json);
            println!("\x1b[36m[UserCache] Saved user cache to disk: {:?}\x1b[0m", config);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_user_config_serialization() {
        let config = AppUserConfig {
            is_dashboard_open: false,
            active_bubble_accounts: vec![1, 2],
            is_notifications_enabled: false,
            muted_accounts: vec![1, 2, 3, 4, 5],
            bubble_size: "lg".to_string(),
            is_always_on_top: false,
            is_auto_start: true,
            locale: Some("en".to_string()),
            is_auto_trim_enabled: false,
            theme_mode: Some("dark".to_string()),
            bubble_position_mode: "free".to_string(),
        };
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: AppUserConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.is_dashboard_open, false);
        assert_eq!(deserialized.active_bubble_accounts, vec![1, 2]);
        assert_eq!(deserialized.is_notifications_enabled, false);
        assert_eq!(deserialized.muted_accounts, vec![1, 2, 3, 4, 5]);
        assert_eq!(deserialized.bubble_size, "lg");
        assert_eq!(deserialized.is_always_on_top, false);
        assert_eq!(deserialized.is_auto_start, true);
        assert_eq!(deserialized.current_locale(), "en");
        assert_eq!(deserialized.current_theme_mode(), "dark");
        assert_eq!(deserialized.bubble_size_px(), 64.0);
        assert_eq!(deserialized.is_auto_trim_enabled, false);
        assert_eq!(deserialized.is_free_position_mode(), true);
        assert_eq!(deserialized.bubble_position_mode, "free");
        let sm_config = AppUserConfig {
            bubble_size: "sm".to_string(),
            ..AppUserConfig::default()
        };
        assert_eq!(sm_config.bubble_size_px(), 48.0);
        assert_eq!(sm_config.current_locale(), "en");
        assert_eq!(sm_config.current_theme_mode(), "system");
        assert_eq!(sm_config.is_auto_trim_enabled, true);
        assert_eq!(sm_config.is_free_position_mode(), false);
        let legacy_json = r#"{"isDashboardOpen":true,"activeBubbleAccounts":[1],"isNotificationsEnabled":true,"mutedAccounts":[],"bubbleSize":"md","isAlwaysOnTop":true,"isAutoStart":false}"#;
        let legacy_config: AppUserConfig = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(legacy_config.is_auto_trim_enabled, true);
        assert_eq!(legacy_config.current_theme_mode(), "system");
        assert_eq!(legacy_config.is_free_position_mode(), false);
        assert_eq!(legacy_config.bubble_position_mode, "auto_tuck");
    }
}
