pub mod animator;
pub mod renderer;

#[cfg(target_os = "windows")]
pub mod manager;
#[cfg(target_os = "windows")]
pub mod window;

pub use animator::*;
pub use renderer::*;
#[cfg(target_os = "windows")]
pub use manager::*;
#[cfg(target_os = "windows")]
pub use window::*;

/// Helper to toggle a native Win32 bubble (delegates to dedicated UI thread).
#[cfg(target_os = "windows")]
pub fn toggle_native_bubble_prototype(
    app: Option<tauri::AppHandle>,
    account_index: u32,
    visible: bool,
) -> Result<bool, String> {
    if visible {
        let y_pos = 180 + ((account_index.max(1) - 1) as i32) * 72;
        if let Some(app_handle) = app {
            let config = crate::movement::get_bubble_physics_config(&app_handle);
            let screen = crate::movement::get_target_dashboard_screen(Some(&app_handle));
            let size = (config.size * screen.scale_factor).round() as u32;
            NativeBubbleManager::global().dispatch(NativeBubbleAction::Show {
                account_index,
                x: screen.x + 30,
                y: screen.y + y_pos,
                size,
                app: app_handle,
            });
        }
        Ok(true)
    } else {
        NativeBubbleManager::global().dispatch(NativeBubbleAction::Hide { account_index });
        Ok(false)
    }
}

/// Checks if a native bubble is currently displayed.
#[cfg(target_os = "windows")]
pub fn has_native_bubble(account_index: u32) -> bool {
    NativeBubbleManager::global().has_bubble(account_index)
}

/// Updates unread message count for the corresponding native bubble.
#[cfg(target_os = "windows")]
pub fn set_native_bubble_unread_count(account_index: u32, count: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetUnread { account_index, count });
}

/// Updates unread badge count and heartbeat alert eligibility for native bubble.
#[cfg(target_os = "windows")]
pub fn set_native_bubble_alert_state(account_index: u32, unread_count: u32, alert_active: bool) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetAlertState {
        account_index,
        unread_count,
        alert_active,
    });
}

/// Updates whether the account chat window is open to pause or resume heartbeat animation.
#[cfg(target_os = "windows")]
pub fn set_native_bubble_chat_open(account_index: u32, is_open: bool) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetChatOpen {
        account_index,
        is_open,
    });
}

/// Triggers pulse wave animation for native bubble on new message.
#[cfg(target_os = "windows")]
pub fn trigger_native_bubble_pulse(account_index: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::TriggerPulse { account_index });
}

/// Checks if cursor is currently over the native bubble.
#[cfg(target_os = "windows")]
pub fn is_cursor_over_native_bubble(account_index: u32) -> bool {
    NativeBubbleManager::global().is_cursor_over(account_index)
}

/// Triggers spring animation to tuck native bubble to the edge.
#[cfg(target_os = "windows")]
pub fn tuck_native_bubble(app: &tauri::AppHandle, account_index: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::Tuck {
        account_index,
        app: app.clone(),
    });
}

/// Triggers spring animation to untuck native bubble from the edge with explicit reason
#[cfg(target_os = "windows")]
pub fn untuck_native_bubble_with_mode(app: &tauri::AppHandle, account_index: u32, mode: crate::movement::UntuckMode) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::Untuck {
        account_index,
        app: app.clone(),
        mode,
    });
}

/// Triggers spring animation to untuck native bubble from the edge on user interaction
#[cfg(target_os = "windows")]
pub fn untuck_native_bubble(app: &tauri::AppHandle, account_index: u32) {
    untuck_native_bubble_with_mode(app, account_index, crate::movement::UntuckMode::UserInteraction);
}

/// Sets always-on-top mode for all native bubbles.
#[cfg(target_os = "windows")]
pub fn set_native_bubble_always_on_top(enabled: bool) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetAlwaysOnTop { enabled });
}

/// Re-asserts topmost Z-order for visible native bubbles if enabled and not fullscreen.
#[cfg(target_os = "windows")]
pub fn reassert_native_bubble_topmost() {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::AssertTopmost);
}

/// Sets display size for the native bubble.
#[cfg(target_os = "windows")]
pub fn set_native_bubble_size(account_index: u32, size: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetSize { account_index, size });
}

/// Shuts down native bubble worker thread cleanly on application exit.
#[cfg(target_os = "windows")]
pub fn shutdown_native_bubbles() {
    NativeBubbleManager::shutdown_if_initialized();
}
