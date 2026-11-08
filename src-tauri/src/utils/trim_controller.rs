use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

/// Debounce duration after bubble/window is tucked before trimming memory (5s)
pub const TUCK_DEBOUNCE_DURATION: Duration = Duration::from_secs(5);

/// Debounce duration after incoming message notification before trimming memory (5s)
pub const MESSAGE_DEBOUNCE_DURATION: Duration = Duration::from_secs(5);

/// Settle duration after webview preload finishes before trimming memory
pub const PRELOAD_SETTLE_DURATION: Duration = Duration::from_millis(5000);

/// Interval duration between periodic idle sweeps (5 minutes)
pub const IDLE_SWEEP_INTERVAL: Duration = Duration::from_secs(300);

/// Minimum cooldown duration between consecutive trims of the same account process tree (30s)
pub const TRIM_COOLDOWN_DURATION: Duration = Duration::from_secs(30);

/// Coordinates automated, non-intrusive working set memory trimming for Messenger accounts
pub struct AutoTrimController {
    last_trim_time: Mutex<HashMap<u32, Instant>>,
    last_interaction_time: Mutex<HashMap<u32, Instant>>,
    pending_tokens: Mutex<HashMap<u32, u64>>,
    is_tucked: Mutex<HashMap<u32, bool>>,
    token_counter: AtomicU64,
}

static AUTO_TRIM_CONTROLLER: OnceLock<AutoTrimController> = OnceLock::new();

fn is_chat_window_visible(app: &AppHandle, account_index: u32) -> bool {
    let chat_label = crate::platforms::messenger::windows::get_messenger_window_label(account_index);
    let win = app.get_webview_window(&chat_label).or_else(|| if account_index == 1 { app.get_webview_window("chat-messenger") } else { None });
    win.and_then(|w| w.is_visible().ok()).unwrap_or(false)
}

impl Default for AutoTrimController {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoTrimController {
    /// Creates a new instance of AutoTrimController
    pub fn new() -> Self {
        Self {
            last_trim_time: Mutex::new(HashMap::new()),
            last_interaction_time: Mutex::new(HashMap::new()),
            pending_tokens: Mutex::new(HashMap::new()),
            is_tucked: Mutex::new(HashMap::new()),
            token_counter: AtomicU64::new(1),
        }
    }
    /// Returns the global singleton instance
    pub fn global() -> &'static Self {
        AUTO_TRIM_CONTROLLER.get_or_init(Self::new)
    }
    /// Records user interaction with an account
    pub fn record_interaction(&self, account_index: u32) {
        if let Ok(mut lock) = self.last_interaction_time.lock() {
            lock.insert(account_index, Instant::now());
        }
    }
    /// Checks whether user recently interacted with an account within duration
    pub fn has_recent_interaction(&self, account_index: u32, duration: Duration) -> bool {
        if let Ok(lock) = self.last_interaction_time.lock() {
            if let Some(last) = lock.get(&account_index) {
                return last.elapsed() < duration;
            }
        }
        false
    }
    /// Checks if an account is eligible to trim based on cooldown
    pub fn can_trim(&self, account_index: u32) -> bool {
        if let Ok(lock) = self.last_trim_time.lock() {
            if let Some(last) = lock.get(&account_index) {
                return last.elapsed() >= TRIM_COOLDOWN_DURATION;
            }
        }
        true
    }
    /// Records the current instant as the last trim time for an account
    pub fn record_trim(&self, account_index: u32) {
        if let Ok(mut lock) = self.last_trim_time.lock() {
            lock.insert(account_index, Instant::now());
        }
    }
    /// Updates tucked state and returns whether it changed
    pub fn set_tucked_state(&self, account_index: u32, tucked: bool) {
        if let Ok(mut lock) = self.is_tucked.lock() {
            lock.insert(account_index, tucked);
        }
    }
    /// Returns whether the account is currently marked as tucked
    pub fn is_account_tucked(&self, account_index: u32) -> bool {
        self.is_tucked.lock().ok().and_then(|m| m.get(&account_index).copied()).unwrap_or(true)
    }
    /// Cancels any scheduled trim debounce timer for an account
    pub fn cancel_pending_trim(&self, account_index: u32) {
        if let Ok(mut lock) = self.pending_tokens.lock() {
            lock.remove(&account_index);
        }
    }
    /// Generates and assigns a new debounce token for an account
    pub fn next_token(&self, account_index: u32) -> u64 {
        let token = self.token_counter.fetch_add(1, Ordering::SeqCst);
        if let Ok(mut lock) = self.pending_tokens.lock() {
            lock.insert(account_index, token);
        }
        token
    }
    /// Verifies if a token is still the active pending token for an account
    pub fn is_token_valid(&self, account_index: u32, token: u64) -> bool {
        self.pending_tokens.lock().ok().and_then(|m| m.get(&account_index).copied()) == Some(token)
    }
    /// Checks if auto-trim is enabled in user settings
    pub fn is_auto_trim_enabled(&self, app: &AppHandle) -> bool {
        if let Some(state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
            state.get_user_config().is_auto_trim_enabled
        } else {
            true
        }
    }
    /// Executes targeted working set trim on WebView2 processes belonging to account_index
    pub fn trim_account_process_tree(&self, account_index: u32) -> usize {
        let pids = crate::utils::performance::get_account_process_pids(account_index);
        let count = pids.len();
        for pid in &pids {
            crate::utils::performance::trim_single_process_working_set(*pid);
        }
        if count > 0 {
            self.record_trim(account_index);
            println!(
                "\x1b[32m[AutoTrimEngine] Trimmed working set for account {:02} ({} processes flushed to standby)\x1b[0m",
                account_index, count
            );
        }
        count
    }
    /// Schedules an auto-trim task after bubble/window is tucked (5s debounce)
    pub fn on_account_tucked(&self, account_index: u32, app: AppHandle) {
        self.set_tucked_state(account_index, true);
        let token = self.next_token(account_index);
        let controller = Self::global();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(TUCK_DEBOUNCE_DURATION).await;
            if !controller.is_token_valid(account_index, token) {
                return;
            }
            if !controller.is_account_tucked(account_index) {
                return;
            }
            if controller.has_recent_interaction(account_index, TUCK_DEBOUNCE_DURATION) {
                return;
            }
            if is_chat_window_visible(&app, account_index) {
                return;
            }
            if !controller.is_auto_trim_enabled(&app) {
                return;
            }
            if !controller.can_trim(account_index) {
                return;
            }
            crate::platforms::messenger::windows::set_chat_window_memory_target(&app, account_index, true);
            controller.trim_account_process_tree(account_index);
            controller.cancel_pending_trim(account_index);
        });
    }
    /// Cancels auto-trim when user untucks/interacts with bubble or window
    pub fn on_account_untucked(&self, account_index: u32) {
        self.set_tucked_state(account_index, false);
        self.cancel_pending_trim(account_index);
        self.record_interaction(account_index);
    }
    /// Schedules an auto-trim task after receiving a message notification (5s debounce)
    pub fn on_new_message(&self, account_index: u32, app: AppHandle) {
        let token = self.next_token(account_index);
        let controller = Self::global();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(MESSAGE_DEBOUNCE_DURATION).await;
            if !controller.is_token_valid(account_index, token) {
                return;
            }
            if !controller.is_account_tucked(account_index) {
                return;
            }
            if controller.has_recent_interaction(account_index, MESSAGE_DEBOUNCE_DURATION) {
                return;
            }
            if is_chat_window_visible(&app, account_index) {
                return;
            }
            if !controller.is_auto_trim_enabled(&app) {
                return;
            }
            if !controller.can_trim(account_index) {
                return;
            }
            crate::platforms::messenger::windows::set_chat_window_memory_target(&app, account_index, true);
            controller.trim_account_process_tree(account_index);
            controller.cancel_pending_trim(account_index);
        });
    }
    /// Schedules an auto-trim task after webview preloading finishes (5.0s settle duration)
    pub fn on_preload_finished(&self, account_index: u32, app: AppHandle) {
        let token = self.next_token(account_index);
        let controller = Self::global();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(PRELOAD_SETTLE_DURATION).await;
            if !controller.is_token_valid(account_index, token) {
                return;
            }
            if !controller.is_account_tucked(account_index) {
                return;
            }
            if is_chat_window_visible(&app, account_index) {
                return;
            }
            if !controller.is_auto_trim_enabled(&app) {
                return;
            }
            if !controller.can_trim(account_index) {
                return;
            }
            crate::platforms::messenger::windows::set_chat_window_memory_target(&app, account_index, true);
            let trimmed = controller.trim_account_process_tree(account_index);
            if trimmed > 0 {
                println!(
                    "\x1b[32m[AutoTrimEngine] Post-preload settle trim executed for Messenger {:02} ({} processes trimmed)\x1b[0m",
                    account_index, trimmed
                );
            }
            controller.cancel_pending_trim(account_index);
        });
    }
}

/// Convenience function to record user interaction with an account bubble or chat
pub fn record_user_interaction(account_index: u32) {
    AutoTrimController::global().record_interaction(account_index);
}

/// Convenience function to schedule debounced trim on tuck
pub fn schedule_auto_trim_on_tuck(account_index: u32, app: AppHandle) {
    AutoTrimController::global().on_account_tucked(account_index, app);
}

/// Convenience function to schedule debounced trim on preload finish
pub fn schedule_auto_trim_on_preload(account_index: u32, app: AppHandle) {
    AutoTrimController::global().on_preload_finished(account_index, app);
}

/// Starts the background idle trim sweeper daemon (runs every 5 minutes)
pub fn start_idle_trim_sweeper(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(IDLE_SWEEP_INTERVAL);
        interval.tick().await;
        let controller = AutoTrimController::global();
        loop {
            interval.tick().await;
            if !controller.is_auto_trim_enabled(&app) {
                continue;
            }
            for account_index in 1..=5 {
                if !controller.is_account_tucked(account_index) {
                    continue;
                }
                if controller.has_recent_interaction(account_index, Duration::from_secs(30)) {
                    continue;
                }
                if is_chat_window_visible(&app, account_index) {
                    continue;
                }
                if !controller.can_trim(account_index) {
                    continue;
                }
                let pids = crate::utils::performance::get_account_process_pids(account_index);
                if !pids.is_empty() {
                    crate::platforms::messenger::windows::set_chat_window_memory_target(&app, account_index, true);
                    controller.trim_account_process_tree(account_index);
                }
            }
        }
    });
}

/// Convenience function to cancel pending trim when untucked
pub fn cancel_auto_trim(account_index: u32) {
    AutoTrimController::global().on_account_untucked(account_index);
}

/// Convenience function to schedule debounced trim on new message
pub fn schedule_auto_trim_on_message(account_index: u32, app: AppHandle) {
    AutoTrimController::global().on_new_message(account_index, app);
}

/// Convenience function to immediately trim an account process tree
pub fn trim_account_memory(account_index: u32) -> usize {
    AutoTrimController::global().trim_account_process_tree(account_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_cooldown_logic() {
        let controller = AutoTrimController::new();
        assert!(controller.can_trim(1));
        assert!(controller.can_trim(2));
        controller.record_trim(1);
        assert!(!controller.can_trim(1));
        assert!(controller.can_trim(2));
    }

    #[test]
    fn test_token_invalidation_and_cancellation() {
        let controller = AutoTrimController::new();
        let token1 = controller.next_token(1);
        assert!(controller.is_token_valid(1, token1));
        let token2 = controller.next_token(1);
        assert!(!controller.is_token_valid(1, token1));
        assert!(controller.is_token_valid(1, token2));
        controller.cancel_pending_trim(1);
        assert!(!controller.is_token_valid(1, token2));
    }

    #[test]
    fn test_tuck_state_tracking() {
        let controller = AutoTrimController::new();
        assert!(controller.is_account_tucked(1));
        controller.set_tucked_state(1, false);
        assert!(!controller.is_account_tucked(1));
        controller.set_tucked_state(1, true);
        assert!(controller.is_account_tucked(1));
    }

    #[test]
    fn test_preload_and_idle_durations() {
        assert_eq!(PRELOAD_SETTLE_DURATION.as_secs(), 5);
        assert_eq!(IDLE_SWEEP_INTERVAL.as_secs(), 300);
    }

    #[test]
    fn test_recent_interaction_tracking() {
        let controller = AutoTrimController::new();
        assert!(!controller.has_recent_interaction(1, Duration::from_secs(1)));
        controller.record_interaction(1);
        assert!(controller.has_recent_interaction(1, Duration::from_secs(5)));
        assert!(!controller.has_recent_interaction(2, Duration::from_secs(5)));
    }
}
