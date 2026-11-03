use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Mutex;

#[derive(Clone, Debug, Default)]
pub struct AccountAuthInfo {
    pub connected: bool,
    pub user_id: String,
    pub name: String,
}

/// Cấu trúc lưu trữ tin nhắn đến gần nhất (Giai đoạn 2)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomingMessage {
    pub account_index: u32,
    pub sender_name: String,
    pub body: String,
    pub avatar_url: String,
    pub thread_id: String,
    pub timestamp: i64,
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
    pub unread_counts: Mutex<HashMap<u32, u32>>,
    pub recent_messages: Mutex<Vec<IncomingMessage>>,
    pub muted_accounts: Mutex<HashSet<u32>>,
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
            unread_counts: Mutex::new(HashMap::new()),
            recent_messages: Mutex::new(Vec::new()),
            muted_accounts: Mutex::new(HashSet::new()),
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

    pub fn increment_unread_count(&self, account_index: u32) -> u32 {
        if let Ok(mut map) = self.unread_counts.lock() {
            let entry = map.entry(account_index).or_insert(0);
            *entry = entry.saturating_add(1);
            *entry
        } else {
            1
        }
    }

    pub fn set_unread_count(&self, account_index: u32, count: u32) {
        if let Ok(mut map) = self.unread_counts.lock() {
            map.insert(account_index, count);
        }
    }

    pub fn get_unread_count(&self, account_index: u32) -> u32 {
        if let Ok(map) = self.unread_counts.lock() {
            return *map.get(&account_index).unwrap_or(&0);
        }
        0
    }

    pub fn clear_unread_count(&self, account_index: u32) {
        if let Ok(mut map) = self.unread_counts.lock() {
            map.insert(account_index, 0);
        }
    }

    pub fn set_account_muted(&self, account_index: u32, muted: bool) {
        if let Ok(mut set) = self.muted_accounts.lock() {
            if muted {
                set.insert(account_index);
            } else {
                set.remove(&account_index);
            }
        }
    }

    pub fn is_account_muted(&self, account_index: u32) -> bool {
        if let Ok(set) = self.muted_accounts.lock() {
            set.contains(&account_index)
        } else {
            false
        }
    }

    /// Thêm tin nhắn mới vào danh sách gần nhất (giữ tối đa 20 tin, lọc trùng lặp sliding window 3000ms)
    pub fn add_recent_message(&self, msg: IncomingMessage) -> bool {
        if let Ok(mut list) = self.recent_messages.lock() {
            let is_duplicate = list.iter().take(5).any(|m| {
                m.account_index == msg.account_index
                    && m.sender_name == msg.sender_name
                    && m.body == msg.body
                    && (msg.timestamp - m.timestamp).abs() < 3000
            });
            if is_duplicate {
                return false;
            }
            list.insert(0, msg);
            if list.len() > 20 {
                list.truncate(20);
            }
            true
        } else {
            false
        }
    }

    /// Lấy danh sách tin nhắn gần nhất
    pub fn get_recent_messages(&self, account_index: Option<u32>) -> Vec<IncomingMessage> {
        if let Ok(list) = self.recent_messages.lock() {
            if let Some(idx) = account_index {
                list.iter().filter(|m| m.account_index == idx).cloned().collect()
            } else {
                list.clone()
            }
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incoming_message_deduplication_and_limit() {
        let state = ChatState::default();

        let msg1 = IncomingMessage {
            account_index: 2,
            sender_name: "Nguyễn Văn A".into(),
            body: "Chào bạn!".into(),
            avatar_url: "".into(),
            thread_id: "t_123".into(),
            timestamp: 10000,
        };

        // Tin nhắn đầu tiên: Được thêm thành công
        assert!(state.add_recent_message(msg1.clone()));
        assert_eq!(state.get_recent_messages(Some(2)).len(), 1);

        // Tin nhắn trùng lặp trong vòng 3000ms (cùng nội dung, timestamp 12000 => chênh 2000ms < 3000ms): Bị bỏ qua
        let mut msg_dup = msg1.clone();
        msg_dup.timestamp = 12000;
        assert!(!state.add_recent_message(msg_dup));
        assert_eq!(state.get_recent_messages(Some(2)).len(), 1);

        // Tin nhắn cùng nội dung nhưng sau 3000ms (timestamp 14000 => chênh 4000ms): Được thêm thành công
        let mut msg_later = msg1.clone();
        msg_later.timestamp = 14000;
        assert!(state.add_recent_message(msg_later));
        assert_eq!(state.get_recent_messages(Some(2)).len(), 2);

        // Kiểm tra giới hạn 20 tin nhắn
        for i in 0..25 {
            let msg = IncomingMessage {
                account_index: 2,
                sender_name: format!("Người gửi {}", i),
                body: format!("Nội dung {}", i),
                avatar_url: "".into(),
                thread_id: format!("t_{}", i),
                timestamp: 20000 + i as i64 * 100,
            };
            state.add_recent_message(msg);
        }

        let all_msgs = state.get_recent_messages(None);
        assert_eq!(all_msgs.len(), 20);
        // Tin mới nhất luôn ở đầu (index 0)
        assert_eq!(all_msgs[0].sender_name, "Người gửi 24");
    }
}
