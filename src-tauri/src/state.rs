use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Mutex;

#[derive(Clone, Debug, Default)]
pub struct AccountAuthInfo {
    pub connected: bool,
    pub user_id: String,
    pub name: String,
}

/// Trạng thái theo dõi thời điểm ẩn cửa sổ và chế độ xác thực độc lập cho từng tài khoản (SSOT)
pub struct ChatState {
    pub last_blur_time: AtomicI64,
    pub is_auth_mode: AtomicBool,
    pub auth_windows: Mutex<HashSet<String>>,
    pub account_auth: Mutex<HashMap<u32, AccountAuthInfo>>,
    pub is_always_on_top: AtomicBool,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            last_blur_time: AtomicI64::new(0),
            is_auth_mode: AtomicBool::new(false),
            auth_windows: Mutex::new(HashSet::new()),
            account_auth: Mutex::new(HashMap::new()),
            is_always_on_top: AtomicBool::new(true),
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

    pub fn set_account_auth(&self, account_index: u32, connected: bool, user_id: Option<String>) {
        if let Ok(mut map) = self.account_auth.lock() {
            let uid = user_id.clone().unwrap_or_default();
            map.insert(
                account_index,
                AccountAuthInfo {
                    connected,
                    user_id: uid.clone(),
                    name: uid,
                },
            );
        }
    }

    pub fn get_account_auth(&self, account_index: u32) -> Option<AccountAuthInfo> {
        if let Ok(map) = self.account_auth.lock() {
            return map.get(&account_index).cloned();
        }
        None
    }

    pub fn clear_account_auth(&self, account_index: u32) {
        if let Ok(mut map) = self.account_auth.lock() {
            map.remove(&account_index);
        }
    }
}
