use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Clone, Debug, Default)]
pub struct AccountAuthInfo {
    pub connected: bool,
    pub user_id: String,
    pub name: String,
}

/// Recent incoming message payload
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

/// Maximum number of supported concurrent accounts
pub const MAX_ACCOUNTS: u32 = 5;

/// Multi-account SSOT state for chat and authentication
pub struct ChatState {
    pub auth_windows: Mutex<HashSet<String>>,
    pub account_auth: Mutex<HashMap<u32, AccountAuthInfo>>,
    pub is_always_on_top: AtomicBool,
    pub bypass_close_guard: Mutex<HashSet<String>>,
    pub transition_locks: Mutex<HashSet<u32>>,
    pub suspended_accounts: Mutex<HashSet<u32>>,
    pub ready_accounts: Mutex<HashSet<u32>>,
    pub is_explicit_exit: AtomicBool,
    pub active_bubble_rects: Mutex<HashMap<u32, crate::movement::BubbleRect>>,
    pub unread_counts: Mutex<HashMap<u32, u32>>,
    pub recent_messages: Mutex<Vec<IncomingMessage>>,
    pub muted_accounts: Mutex<HashSet<u32>>,
    pub pinned_accounts: Mutex<HashSet<u32>>,
    pub chat_window_sizes: Mutex<HashMap<u32, (u32, u32)>>,
    pub user_config: Mutex<crate::utils::user_cache::AppUserConfig>,
    pub user_cache_write_lock: Mutex<()>,
    pub has_shown_tray_notification: AtomicBool,
    pub active_dialog_windows: Mutex<HashSet<String>>,
    pub chat_anim_tokens: Mutex<HashMap<u32, u64>>,
    pub session_clean_tokens: Mutex<HashMap<u32, u64>>,
}

impl Default for ChatState {
    fn default() -> Self {
        let mut suspended = HashSet::new();
        for idx in 1..=MAX_ACCOUNTS {
            suspended.insert(idx);
        }
        Self {
            auth_windows: Mutex::new(HashSet::new()),
            account_auth: Mutex::new(HashMap::new()),
            is_always_on_top: AtomicBool::new(true),
            bypass_close_guard: Mutex::new(HashSet::new()),
            transition_locks: Mutex::new(HashSet::new()),
            suspended_accounts: Mutex::new(suspended),
            ready_accounts: Mutex::new(HashSet::new()),
            is_explicit_exit: AtomicBool::new(false),
            active_bubble_rects: Mutex::new(HashMap::new()),
            unread_counts: Mutex::new(HashMap::new()),
            recent_messages: Mutex::new(Vec::new()),
            muted_accounts: Mutex::new(HashSet::new()),
            pinned_accounts: Mutex::new(HashSet::new()),
            chat_window_sizes: Mutex::new(HashMap::new()),
            user_config: Mutex::new(crate::utils::user_cache::AppUserConfig::default()),
            user_cache_write_lock: Mutex::new(()),
            has_shown_tray_notification: AtomicBool::new(false),
            active_dialog_windows: Mutex::new(HashSet::new()),
            chat_anim_tokens: Mutex::new(HashMap::new()),
            session_clean_tokens: Mutex::new(HashMap::new()),
        }
    }
}

impl ChatState {
    pub fn set_dialog_active(&self, label: &str, active: bool) {
        if let Ok(mut set) = self.active_dialog_windows.lock() {
            if active {
                set.insert(label.to_string());
            } else {
                set.remove(label);
            }
        }
    }

    pub fn is_dialog_active(&self, label: &str) -> bool {
        if let Ok(set) = self.active_dialog_windows.lock() {
            return set.contains(label);
        }
        false
    }

    pub fn is_any_dialog_active(&self) -> bool {
        if let Ok(set) = self.active_dialog_windows.lock() {
            return !set.is_empty();
        }
        false
    }

    pub fn set_window_auth_mode(&self, label: &str, is_auth: bool) {
        if let Ok(mut set) = self.auth_windows.lock() {
            if is_auth {
                set.insert(label.to_string());
            } else {
                set.remove(label);
            }
        }
    }

    pub fn is_window_in_auth_mode(&self, label: &str) -> bool {
        if let Ok(set) = self.auth_windows.lock() {
            return set.contains(label);
        }
        false
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

    pub fn set_chat_pinned(&self, account_index: u32, pinned: bool) {
        if let Ok(mut set) = self.pinned_accounts.lock() {
            if pinned {
                set.insert(account_index);
            } else {
                set.remove(&account_index);
            }
        }
    }

    pub fn is_chat_pinned(&self, account_index: u32) -> bool {
        if let Ok(set) = self.pinned_accounts.lock() {
            set.contains(&account_index)
        } else {
            false
        }
    }

    pub fn set_chat_window_custom_size(&self, account_index: u32, size: Option<(u32, u32)>) {
        if let Ok(mut map) = self.chat_window_sizes.lock() {
            if let Some((w, h)) = size {
                map.insert(account_index, (w, h));
            } else {
                map.remove(&account_index);
            }
        }
    }

    pub fn get_chat_window_custom_size(&self, account_index: u32) -> Option<(u32, u32)> {
        if let Ok(map) = self.chat_window_sizes.lock() {
            return map.get(&account_index).copied();
        }
        None
    }

    /// Initializes user config from disk on startup and seeds authenticated accounts
    pub fn init_user_config(&self, mut config: crate::utils::user_cache::AppUserConfig) {
        config.active_bubble_accounts.retain(|idx| config.authenticated_accounts.contains(idx));
        for &idx in &config.authenticated_accounts {
            self.set_account_auth(idx, true, None);
        }
        if let Ok(mut lock) = self.user_config.lock() {
            *lock = config;
        }
    }

    /// Returns a clone of current user config
    pub fn get_user_config(&self) -> crate::utils::user_cache::AppUserConfig {
        self.user_config
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    /// Replaces entire user config and saves to disk
    pub fn set_user_config(&self, app: &tauri::AppHandle, config: crate::utils::user_cache::AppUserConfig) {
        if let Ok(mut lock) = self.user_config.lock() {
            *lock = config.clone();
        }
        self.persist_and_emit_user_config(app);
    }

    /// Updates user config with closure and saves to disk
    pub fn update_user_config<F>(&self, app: &tauri::AppHandle, updater: F)
    where
        F: FnOnce(&mut crate::utils::user_cache::AppUserConfig),
    {
        if let Ok(mut lock) = self.user_config.lock() {
            updater(&mut lock);
        }
        self.persist_and_emit_user_config(app);
    }

    /// Serializes disk persistence and settings broadcast without holding user_config lock across I/O
    fn persist_and_emit_user_config(&self, app: &tauri::AppHandle) {
        let _write_guard = self.user_cache_write_lock.lock();
        let snapshot = if let Ok(lock) = self.user_config.lock() {
            Some(lock.clone())
        } else {
            None
        };
        if let Some(config) = snapshot {
            crate::utils::user_cache::save_user_cache(app, &config);
            use tauri::Emitter;
            let _ = app.emit("user-settings-changed", config);
        }
    }

    /// Validates that an optional account index falls within the supported range
    pub fn validate_account_index(account_index: Option<u32>) -> Result<u32, String> {
        let idx = account_index.unwrap_or(1);
        if (1..=MAX_ACCOUNTS).contains(&idx) {
            Ok(idx)
        } else {
            Err(format!("Invalid account_index {}: must be within 1..={}", idx, MAX_ACCOUNTS))
        }
    }

    /// Returns true if a recent message was already recorded for this account inside the window
    pub fn has_recent_message_within(&self, account_index: u32, now_ms: i64, window_ms: u64) -> bool {
        if let Ok(list) = self.recent_messages.lock() {
            return list
                .iter()
                .any(|m| m.account_index == account_index && now_ms.abs_diff(m.timestamp) < window_ms);
        }
        false
    }

    /// Adds incoming message to recent list (max 20 items, 3000ms sliding window deduplication)
    pub fn add_recent_message(&self, msg: IncomingMessage) -> bool {
        if let Ok(mut list) = self.recent_messages.lock() {
            let is_duplicate = list.iter().take(5).any(|m| {
                m.account_index == msg.account_index
                    && m.sender_name == msg.sender_name
                    && m.body == msg.body
                    && msg.timestamp.abs_diff(m.timestamp) < 3000
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

    /// Returns recent messages
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

    /// Clears recent messages for a specific account
    pub fn clear_recent_messages_for_account(&self, account_index: u32) {
        if let Ok(mut list) = self.recent_messages.lock() {
            list.retain(|m| m.account_index != account_index);
        }
    }

    /// Marks tray notification as shown once per session
    pub fn mark_tray_notification_shown(&self) -> bool {
        self.has_shown_tray_notification.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok()
    }

    /// Increments and returns the next per-account token for monotonic cancellation checks
    fn advance_token(map_lock: &Mutex<HashMap<u32, u64>>, account_index: u32) -> u64 {
        if let Ok(mut map) = map_lock.lock() {
            let entry = map.entry(account_index).or_insert(0);
            *entry = entry.wrapping_add(1);
            *entry
        } else {
            0
        }
    }

    /// Checks whether a per-account monotonic token is still current
    fn is_token_valid(map_lock: &Mutex<HashMap<u32, u64>>, account_index: u32, token: u64) -> bool {
        if let Ok(map) = map_lock.lock() {
            map.get(&account_index).copied().unwrap_or(0) == token
        } else {
            false
        }
    }

    /// Increments and returns the next chat animation token for debounce cancellation
    pub fn next_chat_anim_token(&self, account_index: u32) -> u64 {
        Self::advance_token(&self.chat_anim_tokens, account_index)
    }

    /// Verifies if a specific chat animation token is still valid
    pub fn is_chat_anim_token_valid(&self, account_index: u32, token: u64) -> bool {
        Self::is_token_valid(&self.chat_anim_tokens, account_index, token)
    }

    /// Increments and returns the next session clean token to cancel stale wipe tasks
    pub fn next_session_clean_token(&self, account_index: u32) -> u64 {
        Self::advance_token(&self.session_clean_tokens, account_index)
    }

    /// Verifies if a session clean token is still current
    pub fn is_session_clean_token_valid(&self, account_index: u32, token: u64) -> bool {
        Self::is_token_valid(&self.session_clean_tokens, account_index, token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_notification_flag_one_shot() {
        let state = ChatState::default();
        assert!(state.mark_tray_notification_shown());
        assert!(!state.mark_tray_notification_shown());
        assert!(!state.mark_tray_notification_shown());
    }

    #[test]
    fn test_incoming_message_deduplication_and_limit() {
        let state = ChatState::default();
        let msg1 = IncomingMessage {
            account_index: 2,
            sender_name: "Alice Smith".into(),
            body: "Hello there!".into(),
            avatar_url: "".into(),
            thread_id: "t_123".into(),
            timestamp: 10000,
        };
        assert!(state.add_recent_message(msg1.clone()));
        assert_eq!(state.get_recent_messages(Some(2)).len(), 1);
        let mut msg_dup = msg1.clone();
        msg_dup.timestamp = 12000;
        assert!(!state.add_recent_message(msg_dup));
        assert_eq!(state.get_recent_messages(Some(2)).len(), 1);
        let mut msg_later = msg1.clone();
        msg_later.timestamp = 14000;
        assert!(state.add_recent_message(msg_later));
        assert_eq!(state.get_recent_messages(Some(2)).len(), 2);
        for i in 0..25 {
            let msg = IncomingMessage {
                account_index: 2,
                sender_name: format!("Sender {}", i),
                body: format!("Message body {}", i),
                avatar_url: "".into(),
                thread_id: format!("t_{}", i),
                timestamp: 20000 + i as i64 * 100,
            };
            state.add_recent_message(msg);
        }
        let all_msgs = state.get_recent_messages(None);
        assert_eq!(all_msgs.len(), 20);
        assert_eq!(all_msgs[0].sender_name, "Sender 24");
    }

    #[test]
    fn test_clear_recent_messages_for_account() {
        let state = ChatState::default();
        let msg1 = IncomingMessage {
            account_index: 1,
            sender_name: "Alice".into(),
            body: "Msg 1".into(),
            avatar_url: "".into(),
            thread_id: "t_1".into(),
            timestamp: 10000,
        };
        let msg2 = IncomingMessage {
            account_index: 2,
            sender_name: "Bob".into(),
            body: "Msg 2".into(),
            avatar_url: "".into(),
            thread_id: "t_2".into(),
            timestamp: 10100,
        };
        state.add_recent_message(msg1);
        state.add_recent_message(msg2);
        assert_eq!(state.get_recent_messages(None).len(), 2);
        state.clear_recent_messages_for_account(1);
        let remaining = state.get_recent_messages(None);
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].account_index, 2);
    }

    #[test]
    fn test_chat_animation_token_lifecycle() {
        let state = ChatState::default();
        let tok1 = state.next_chat_anim_token(1);
        assert_eq!(tok1, 1);
        assert!(state.is_chat_anim_token_valid(1, tok1));
        assert!(!state.is_chat_anim_token_valid(1, 999));
        let tok2 = state.next_chat_anim_token(1);
        assert_eq!(tok2, 2);
        assert!(!state.is_chat_anim_token_valid(1, tok1));
        assert!(state.is_chat_anim_token_valid(1, tok2));
        let tok_other = state.next_chat_anim_token(2);
        assert_eq!(tok_other, 1);
        assert!(state.is_chat_anim_token_valid(2, tok_other));
        assert!(state.is_chat_anim_token_valid(1, tok2));
    }

    #[test]
    fn test_per_window_auth_mode_isolation() {
        let state = ChatState::default();
        state.set_window_auth_mode("chat-messenger-02", true);
        assert!(state.is_window_in_auth_mode("chat-messenger-02"));
        assert!(!state.is_window_in_auth_mode("chat-messenger-01"));
        state.set_window_auth_mode("chat-messenger-02", false);
        assert!(!state.is_window_in_auth_mode("chat-messenger-02"));
    }

    #[test]
    fn test_session_clean_token_lifecycle() {
        let state = ChatState::default();
        let tok1 = state.next_session_clean_token(1);
        assert_eq!(tok1, 1);
        assert!(state.is_session_clean_token_valid(1, tok1));
        let tok2 = state.next_session_clean_token(1);
        assert_eq!(tok2, 2);
        assert!(!state.is_session_clean_token_valid(1, tok1));
        assert!(state.is_session_clean_token_valid(1, tok2));
    }

    #[test]
    fn test_atomic_save_user_cache_to_path() {
        let temp_dir = std::env::temp_dir().join(format!("dbc_cache_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        let target_path = temp_dir.join("user_cache.json");
        let mut cfg = crate::utils::user_cache::AppUserConfig::default();
        cfg.locale = Some("en".into());
        crate::utils::user_cache::save_user_cache_to_path(&target_path, &cfg).expect("initial save succeeds");
        cfg.locale = Some("vi".into());
        crate::utils::user_cache::save_user_cache_to_path(&target_path, &cfg).expect("atomic overwrite succeeds");
        let raw = std::fs::read_to_string(&target_path).expect("target readable");
        let parsed: crate::utils::user_cache::AppUserConfig = serde_json::from_str(&raw).expect("valid json");
        assert_eq!(parsed.locale.as_deref(), Some("vi"));
        let leftover_tmp = std::fs::read_dir(&temp_dir).expect("dir readable").filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().ends_with(".tmp"));
        assert!(!leftover_tmp, "temp file should be renamed cleanly");
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_account_index_bounds() {
        assert_eq!(ChatState::validate_account_index(None), Ok(1));
        assert_eq!(ChatState::validate_account_index(Some(1)), Ok(1));
        assert_eq!(ChatState::validate_account_index(Some(MAX_ACCOUNTS)), Ok(MAX_ACCOUNTS));
        assert!(ChatState::validate_account_index(Some(0)).is_err());
        assert!(ChatState::validate_account_index(Some(MAX_ACCOUNTS + 1)).is_err());
    }

    #[test]
    fn test_has_recent_message_within_window() {
        let state = ChatState::default();
        assert!(!state.has_recent_message_within(1, 5000, 3000));
        state.add_recent_message(IncomingMessage {
            account_index: 1,
            sender_name: "Alice".into(),
            body: "Hi".into(),
            avatar_url: "".into(),
            thread_id: "t1".into(),
            timestamp: 4000,
        });
        assert!(state.has_recent_message_within(1, 5500, 3000));
        assert!(!state.has_recent_message_within(1, 7500, 3000));
        assert!(!state.has_recent_message_within(2, 5500, 3000));
    }
}
