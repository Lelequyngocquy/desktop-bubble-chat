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
    pub previously_visible_bubbles: Mutex<Vec<u32>>,
    pub bypass_close_guard: Mutex<HashSet<String>>,
    pub transition_locks: Mutex<HashSet<u32>>,
    pub suspended_accounts: Mutex<HashSet<u32>>,
    pub ready_accounts: Mutex<HashSet<u32>>,
    pub is_explicit_exit: AtomicBool,
    pub active_bubble_rects: Mutex<HashMap<u32, crate::movement::BubbleRect>>,
}

impl Default for ChatState {
    fn default() -> Self {
        let mut suspended = HashSet::new();
        for idx in 1..=5 {
            suspended.insert(idx);
        }
        Self {
            last_blur_time: AtomicI64::new(0),
            is_auth_mode: AtomicBool::new(false),
            auth_windows: Mutex::new(HashSet::new()),
            account_auth: Mutex::new(HashMap::new()),
            is_always_on_top: AtomicBool::new(true),
            previously_visible_bubbles: Mutex::new(Vec::new()),
            bypass_close_guard: Mutex::new(HashSet::new()),
            transition_locks: Mutex::new(HashSet::new()),
            suspended_accounts: Mutex::new(suspended),
            ready_accounts: Mutex::new(HashSet::new()),
            is_explicit_exit: AtomicBool::new(false),
            active_bubble_rects: Mutex::new(HashMap::new()),
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

    pub fn has_any_connected_account(&self) -> bool {
        if let Ok(map) = self.account_auth.lock() {
            return map.values().any(|info| info.connected);
        }
        false
    }

    pub fn save_previously_visible_bubbles(&self, indices: Vec<u32>) {
        if let Ok(mut prev) = self.previously_visible_bubbles.lock() {
            if !indices.is_empty() {
                *prev = indices;
            }
        }
    }

    pub fn get_previously_visible_bubbles(&self) -> Vec<u32> {
        if let Ok(prev) = self.previously_visible_bubbles.lock() {
            return prev.clone();
        }
        Vec::new()
    }

    pub fn set_bypass_close_guard(&self, label: &str, bypass: bool) {
        if let Ok(mut set) = self.bypass_close_guard.lock() {
            if bypass {
                set.insert(label.to_string());
            } else {
                set.remove(label);
            }
        }
    }

    pub fn is_bypass_close_guard(&self, label: &str) -> bool {
        if let Ok(set) = self.bypass_close_guard.lock() {
            return set.contains(label);
        }
        false
    }

    pub fn try_acquire_transition_lock(&self, account_index: u32) -> bool {
        if let Ok(mut set) = self.transition_locks.lock() {
            if set.contains(&account_index) {
                return false;
            }
            set.insert(account_index);
            return true;
        }
        false
    }

    pub fn release_transition_lock(&self, account_index: u32) {
        if let Ok(mut set) = self.transition_locks.lock() {
            set.remove(&account_index);
        }
    }

    pub fn set_account_suspended(&self, account_index: u32, suspended: bool) {
        if let Ok(mut set) = self.suspended_accounts.lock() {
            if suspended {
                set.insert(account_index);
            } else {
                set.remove(&account_index);
            }
        }
    }

    pub fn is_account_suspended(&self, account_index: u32) -> bool {
        if let Ok(set) = self.suspended_accounts.lock() {
            return set.contains(&account_index);
        }
        false
    }

    pub fn set_chat_ready(&self, account_index: u32, ready: bool) {
        if let Ok(mut set) = self.ready_accounts.lock() {
            if ready {
                set.insert(account_index);
            } else {
                set.remove(&account_index);
            }
        }
    }

    pub fn is_chat_ready(&self, account_index: u32) -> bool {
        if let Ok(set) = self.ready_accounts.lock() {
            return set.contains(&account_index);
        }
        false
    }

    pub fn set_bubble_rect(&self, account_index: u32, rect: crate::movement::BubbleRect) {
        if let Ok(mut map) = self.active_bubble_rects.lock() {
            map.insert(account_index, rect);
        }
    }

    pub fn remove_bubble_rect(&self, account_index: u32) {
        if let Ok(mut map) = self.active_bubble_rects.lock() {
            map.remove(&account_index);
        }
    }

    pub fn get_bubble_rect(&self, account_index: u32) -> Option<crate::movement::BubbleRect> {
        if let Ok(map) = self.active_bubble_rects.lock() {
            return map.get(&account_index).copied();
        }
        None
    }
}
