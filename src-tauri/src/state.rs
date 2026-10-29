use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Mutex;

/// Trạng thái theo dõi thời điểm ẩn cửa sổ và chế độ xác thực độc lập cho từng tài khoản (SSOT)
pub struct ChatState {
    pub last_blur_time: AtomicI64,
    pub is_auth_mode: AtomicBool,
    pub auth_windows: Mutex<HashSet<String>>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            last_blur_time: AtomicI64::new(0),
            is_auth_mode: AtomicBool::new(false),
            auth_windows: Mutex::new(HashSet::new()),
        }
    }
}

impl ChatState {
    pub fn set_window_auth_mode(&self, label: &str, is_auth: bool) {
        if let Ok(mut set) = self.auth_windows.lock() {
            if is_auth {
                set.insert(label.to_string());
            } else {
                set.remove(label);
            }
        }
        self.is_auth_mode.store(is_auth, Ordering::SeqCst);
    }

    pub fn is_window_in_auth_mode(&self, label: &str) -> bool {
        if let Ok(set) = self.auth_windows.lock() {
            if set.contains(label) {
                return true;
            }
        }
        self.is_auth_mode.load(Ordering::SeqCst)
    }
}
