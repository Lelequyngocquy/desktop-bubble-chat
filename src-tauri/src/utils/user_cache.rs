use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const DEFAULT_CHAT_WINDOW_WIDTH: u32 = 404;
pub const DEFAULT_CHAT_WINDOW_HEIGHT: u32 = 550;
pub const MIN_CHAT_WINDOW_WIDTH: u32 = 320;
pub const MIN_CHAT_WINDOW_HEIGHT: u32 = 450;

/// Persistent per-account chat window sizing preference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatWindowSizeConfigEntry {
    pub account_index: u32,
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_height: Option<u32>,
}

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
    #[serde(default = "default_authenticated_accounts")]
    pub authenticated_accounts: Vec<u32>,
    #[serde(default = "default_update_mode")]
    pub update_mode: Option<String>,
    #[serde(default = "default_true", alias = "is_global_shortcut_enabled")]
    pub is_global_shortcut_enabled: bool,
    #[serde(default, alias = "chat_window_settings")]
    pub chat_window_settings: Vec<ChatWindowSizeConfigEntry>,
}

fn default_authenticated_accounts() -> Vec<u32> {
    vec![1]
}

fn default_update_mode() -> Option<String> {
    Some("auto".to_string())
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
            authenticated_accounts: vec![1],
            update_mode: Some("auto".to_string()),
            is_global_shortcut_enabled: true,
            chat_window_settings: Vec::new(),
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
    pub fn is_auto_update_enabled(&self) -> bool {
        self.update_mode.as_deref().unwrap_or("auto") == "auto"
    }
    /// Finds persisted chat window size entry for an account
    pub fn get_chat_window_size_entry(&self, account_index: u32) -> Option<&ChatWindowSizeConfigEntry> {
        self.chat_window_settings.iter().find(|e| e.account_index == account_index)
    }
    /// Returns active custom chat size when account uses remember_custom mode
    pub fn resolved_custom_chat_size(&self, account_index: u32) -> Option<(u32, u32)> {
        let entry = self.get_chat_window_size_entry(account_index)?;
        if entry.mode != "remember_custom" {
            return None;
        }
        let w = entry.custom_width.filter(|&v| v >= MIN_CHAT_WINDOW_WIDTH).unwrap_or(DEFAULT_CHAT_WINDOW_WIDTH);
        let h = entry.custom_height.filter(|&v| v >= MIN_CHAT_WINDOW_HEIGHT).unwrap_or(DEFAULT_CHAT_WINDOW_HEIGHT);
        Some((w, h))
    }
    /// Upserts per-account chat window size mode while preserving stored custom dimensions
    pub fn upsert_chat_window_size_entry(&mut self, account_index: u32, is_custom: bool, size: Option<(u32, u32)>) {
        let mode_str = if is_custom { "remember_custom" } else { "default" };
        let valid_size = size.filter(|&(w, h)| w >= MIN_CHAT_WINDOW_WIDTH && h >= MIN_CHAT_WINDOW_HEIGHT);
        if let Some(existing) = self.chat_window_settings.iter_mut().find(|e| e.account_index == account_index) {
            existing.mode = mode_str.to_string();
            if let Some((w, h)) = valid_size {
                existing.custom_width = Some(w);
                existing.custom_height = Some(h);
            } else if is_custom && (existing.custom_width.is_none() || existing.custom_height.is_none()) {
                existing.custom_width = Some(DEFAULT_CHAT_WINDOW_WIDTH);
                existing.custom_height = Some(DEFAULT_CHAT_WINDOW_HEIGHT);
            }
        } else {
            let (custom_width, custom_height) = match valid_size {
                Some((w, h)) => (Some(w), Some(h)),
                None if is_custom => (Some(DEFAULT_CHAT_WINDOW_WIDTH), Some(DEFAULT_CHAT_WINDOW_HEIGHT)),
                None => (None, None),
            };
            self.chat_window_settings.push(ChatWindowSizeConfigEntry {
                account_index,
                mode: mode_str.to_string(),
                custom_width,
                custom_height,
            });
            self.chat_window_settings.sort_by_key(|e| e.account_index);
        }
    }
}

// Returns user_cache.json path in OS app data directory
pub fn get_user_cache_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|p| p.join("user_cache.json"))
}

// Loads user cache from disk or returns default configuration
pub fn load_user_cache(app: &AppHandle) -> AppUserConfig {
    if let Some(path) = get_user_cache_path(app)
        && path.exists()
            && let Ok(content) = fs::read_to_string(&path)
                && let Ok(config) = serde_json::from_str::<AppUserConfig>(&content) {
                    println!("\x1b[32m[UserCache] Successfully loaded user cache: {:?}\x1b[0m", config);
                    return config;
                }
    println!("[UserCache] No cache found on disk, initializing defaults");
    AppUserConfig::default()
}

// Saves user configuration to disk atomically
static USER_CACHE_TMP_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Atomically persists user config to a specific file path via temp write and rename
pub fn save_user_cache_to_path(path: &std::path::Path, config: &AppUserConfig) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let seq = USER_CACHE_TMP_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp_name = format!("user_cache.json.{}.{}.tmp", std::process::id(), seq);
    let tmp_path = path.with_file_name(tmp_name);
    fs::write(&tmp_path, json)?;
    if let Err(err) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(err);
    }
    Ok(())
}

/// Persists user configuration safely to cache file using atomic write-and-rename
pub fn save_user_cache(app: &AppHandle, config: &AppUserConfig) {
    if let Some(path) = get_user_cache_path(app) {
        match save_user_cache_to_path(&path, config) {
            Ok(()) => {
                println!("\x1b[36m[UserCache] Saved user cache to disk: {:?}\x1b[0m", config);
            }
            Err(err) => {
                eprintln!("\x1b[31m[UserCache] Failed to save user cache to {:?}: {}\x1b[0m", path, err);
            }
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
            authenticated_accounts: vec![1, 2],
            update_mode: Some("manual".to_string()),
            is_global_shortcut_enabled: false,
            chat_window_settings: vec![ChatWindowSizeConfigEntry {
                account_index: 1,
                mode: "remember_custom".to_string(),
                custom_width: Some(520),
                custom_height: Some(680),
            }],
        };
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: AppUserConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.is_global_shortcut_enabled, false);
        assert_eq!(deserialized.is_dashboard_open, false);
        assert_eq!(deserialized.active_bubble_accounts, vec![1, 2]);
        assert_eq!(deserialized.authenticated_accounts, vec![1, 2]);
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
        assert_eq!(deserialized.is_auto_update_enabled(), false);
        assert_eq!(deserialized.resolved_custom_chat_size(1), Some((520, 680)));
        assert_eq!(deserialized.resolved_custom_chat_size(2), None);
        let sm_config = AppUserConfig {
            bubble_size: "sm".to_string(),
            ..AppUserConfig::default()
        };
        assert_eq!(sm_config.bubble_size_px(), 48.0);
        assert_eq!(sm_config.current_locale(), "en");
        assert_eq!(sm_config.current_theme_mode(), "system");
        assert_eq!(sm_config.is_auto_trim_enabled, true);
        assert_eq!(sm_config.is_free_position_mode(), false);
        assert_eq!(sm_config.is_auto_update_enabled(), true);
        let legacy_json = r#"{"isDashboardOpen":true,"activeBubbleAccounts":[1],"isNotificationsEnabled":true,"mutedAccounts":[],"bubbleSize":"md","isAlwaysOnTop":true,"isAutoStart":false}"#;
        let legacy_config: AppUserConfig = serde_json::from_str(legacy_json).unwrap();
        assert_eq!(legacy_config.is_auto_trim_enabled, true);
        assert_eq!(legacy_config.current_theme_mode(), "system");
        assert_eq!(legacy_config.is_free_position_mode(), false);
        assert_eq!(legacy_config.bubble_position_mode, "auto_tuck");
        assert_eq!(legacy_config.authenticated_accounts, vec![1]);
        assert_eq!(legacy_config.is_auto_update_enabled(), true);
        assert!(legacy_config.chat_window_settings.is_empty());
    }

    #[test]
    fn test_chat_window_size_upsert_preserves_custom_size_across_toggle() {
        let mut config = AppUserConfig::default();
        config.upsert_chat_window_size_entry(2, true, Some((540, 700)));
        assert_eq!(config.resolved_custom_chat_size(2), Some((540, 700)));
        config.upsert_chat_window_size_entry(2, false, None);
        assert_eq!(config.resolved_custom_chat_size(2), None);
        let stored = config.get_chat_window_size_entry(2).unwrap();
        assert_eq!(stored.mode, "default");
        assert_eq!(stored.custom_width, Some(540));
        assert_eq!(stored.custom_height, Some(700));
        config.upsert_chat_window_size_entry(2, true, None);
        assert_eq!(config.resolved_custom_chat_size(2), Some((540, 700)));
    }
}
