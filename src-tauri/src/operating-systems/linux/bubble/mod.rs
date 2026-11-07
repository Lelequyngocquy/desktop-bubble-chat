pub mod animator;
pub mod renderer;
pub mod window;

pub use animator::*;
pub use renderer::*;
pub use window::*;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static BUBBLES: LazyLock<Mutex<HashMap<u32, window::LinuxBubbleWindowState>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// Shows or hides native bubble prototype on Linux
pub fn toggle_native_bubble_prototype(_app: Option<tauri::AppHandle>, account_index: u32, visible: bool) -> Result<bool, String> {
    if let Ok(mut map) = BUBBLES.lock() {
        let entry = map.entry(account_index).or_insert_with(|| {
            window::LinuxBubbleWindowState::new(account_index, 100, 100, 56)
        });
        entry.set_visible(visible);
        Ok(entry.is_visible)
    } else {
        Err("Failed to acquire bubble lock".into())
    }
}

/// Checks if native bubble is currently initialized for account
pub fn has_native_bubble(account_index: u32) -> bool {
    BUBBLES.lock().map(|m| m.contains_key(&account_index)).unwrap_or(false)
}

/// Updates unread badge count for native bubble
pub fn set_native_bubble_unread_count(account_index: u32, count: u32) {
    if let Ok(mut map) = BUBBLES.lock() {
        if let Some(bubble) = map.get_mut(&account_index) {
            bubble.set_unread_count(count);
        }
    }
}

/// Triggers pulse animation for native bubble
pub fn trigger_native_bubble_pulse(_account_index: u32) {}

/// Checks if cursor is over native bubble
pub fn is_cursor_over_native_bubble(account_index: u32) -> bool {
    if let Ok(map) = BUBBLES.lock() {
        if let Some(bubble) = map.get(&account_index) {
            if let Some((cx, cy)) = crate::operating_systems::linux::cursor::LinuxCursorTracker::get_cursor_position() {
                let size = bubble.bubble_size as i32;
                return cx >= bubble.x && cx <= bubble.x + size && cy >= bubble.y && cy <= bubble.y + size;
            }
        }
    }
    false
}

/// Tucks native bubble into screen edge
pub fn tuck_native_bubble(app: &tauri::AppHandle, account_index: u32) {
    if let Ok(mut map) = BUBBLES.lock() {
        if let Some(bubble) = map.get_mut(&account_index) {
            bubble.tuck(app);
        }
    }
}

/// Untucks native bubble from screen edge
pub fn untuck_native_bubble(app: &tauri::AppHandle, account_index: u32) {
    if let Ok(mut map) = BUBBLES.lock() {
        if let Some(bubble) = map.get_mut(&account_index) {
            bubble.untuck(app);
        }
    }
}

/// Sets always on top window level
pub fn set_native_bubble_always_on_top(_enabled: bool) {}

/// Resizes native bubble canvas and triggers redraw
pub fn set_native_bubble_size(account_index: u32, size: u32) {
    if let Ok(mut map) = BUBBLES.lock() {
        if let Some(bubble) = map.get_mut(&account_index) {
            bubble.bubble_size = size;
            bubble.render_and_update();
        }
    }
}
