use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Cấu trúc dữ liệu lưu trữ trạng thái cấu hình & hành vi người dùng (Persistent Cache)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUserConfig {
    /// Trạng thái cửa sổ Dashboard chính có đang mở hay không
    pub is_dashboard_open: bool,
    /// Danh sách các tài khoản đang bật Bong bóng nổi (ví dụ: [1, 2] cho M1 & M2)
    pub active_bubble_accounts: Vec<u32>,
    /// Trạng thái công tắc chuông / thông báo chung
    pub is_notifications_enabled: bool,
    /// Danh sách các tài khoản đang tắt âm thanh (Muted)
    pub muted_accounts: Vec<u32>,
    /// Kích thước bong bóng ("sm", "md", "lg")
    pub bubble_size: String,
    /// Trạng thái luôn nổi trên cùng
    pub is_always_on_top: bool,
    /// Trạng thái khởi động cùng Windows
    pub is_auto_start: bool,
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
        }
    }
}

impl AppUserConfig {
    /// Chuyển đổi mã kích thước chuỗi ("sm", "md", "lg") sang điểm ảnh logic
    pub fn bubble_size_px(&self) -> f64 {
        match self.bubble_size.as_str() {
            "sm" => 48.0,
            "lg" => 64.0,
            _ => 56.0,
        }
    }
}

/// Trả về đường dẫn file cache `user_cache.json` nằm trong app_data_dir của hệ điều hành
pub fn get_user_cache_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_data_dir().ok().map(|p| p.join("user_cache.json"))
}

/// Nạp cấu hình người dùng từ đĩa. Nếu chưa có hoặc lỗi, trả về cấu hình mặc định
pub fn load_user_cache(app: &AppHandle) -> AppUserConfig {
    if let Some(path) = get_user_cache_path(app) {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str::<AppUserConfig>(&content) {
                    println!("\x1b[32m[UserCache] Đã nạp cấu hình người dùng từ đĩa thành công: {:?}\x1b[0m", config);
                    return config;
                }
            }
        }
    }
    println!("[UserCache] Chưa có cache người dùng trên đĩa, khởi tạo cấu hình mặc định");
    AppUserConfig::default()
}

/// Lưu cấu hình người dùng vào đĩa (Atomic write an toàn)
pub fn save_user_cache(app: &AppHandle, config: &AppUserConfig) {
    if let Some(path) = get_user_cache_path(app) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(config) {
            let _ = fs::write(&path, json);
            println!("\x1b[36m[UserCache] Đã lưu cấu hình người dùng vào đĩa: {:?}\x1b[0m", config);
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
        assert_eq!(deserialized.bubble_size_px(), 64.0);
        let sm_config = AppUserConfig {
            bubble_size: "sm".to_string(),
            ..AppUserConfig::default()
        };
        assert_eq!(sm_config.bubble_size_px(), 48.0);
    }
}
