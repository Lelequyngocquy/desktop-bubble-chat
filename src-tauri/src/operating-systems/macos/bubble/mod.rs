pub mod animator;
pub mod manager;
pub mod renderer;
pub mod window;

pub use animator::*;
pub use manager::*;
pub use renderer::*;
pub use window::*;

/// Shows or hides native bubble prototype on macOS
pub fn toggle_native_bubble_prototype(app: Option<tauri::AppHandle>, account_index: u32, visible: bool) -> Result<bool, String> {
    if visible {
        let app_handle = app.ok_or_else(|| "AppHandle is required to show native bubble".to_string())?;
        let config = crate::movement::get_bubble_physics_config(&app_handle);
        let screen = crate::movement::get_target_dashboard_screen(Some(&app_handle));
        let size = (config.size * screen.scale_factor).round() as u32;
        let y_pos = 180 + ((account_index.max(1) - 1) as i32) * 72;
        NativeBubbleManager::global().dispatch(NativeBubbleAction::Show {
            account_index,
            x: screen.x + 30,
            y: screen.y + y_pos,
            size,
            app: app_handle,
        });
        Ok(true)
    } else {
        NativeBubbleManager::global().dispatch(NativeBubbleAction::Hide { account_index });
        Ok(false)
    }
}

/// Checks if native bubble is currently initialized for account
pub fn has_native_bubble(account_index: u32) -> bool {
    NativeBubbleManager::global().has_bubble(account_index)
}

/// Updates unread badge count for native bubble
pub fn set_native_bubble_unread_count(account_index: u32, count: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetUnread { account_index, count });
}

/// Triggers pulse animation for native bubble
pub fn trigger_native_bubble_pulse(account_index: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::TriggerPulse { account_index });
}

/// Checks if cursor is over native bubble
pub fn is_cursor_over_native_bubble(account_index: u32) -> bool {
    NativeBubbleManager::global().is_cursor_over(account_index)
}

/// Tucks native bubble into screen edge
pub fn tuck_native_bubble(app: &tauri::AppHandle, account_index: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::Tuck { account_index, app: app.clone() });
}

/// Untucks native bubble from screen edge
pub fn untuck_native_bubble(app: &tauri::AppHandle, account_index: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::Untuck { account_index, app: app.clone() });
}

/// Sets always on top window level
pub fn set_native_bubble_always_on_top(enabled: bool) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetAlwaysOnTop { enabled });
}

/// Resizes native bubble canvas and triggers redraw
pub fn set_native_bubble_size(account_index: u32, size: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetSize { account_index, size });
}
