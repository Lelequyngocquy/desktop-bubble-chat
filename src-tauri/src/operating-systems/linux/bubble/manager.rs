use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use tauri::{AppHandle, Manager};
use super::window::LinuxBubbleWindowState;

/// Control actions sent to the Linux Native Bubble manager
pub enum NativeBubbleAction {
    Show {
        account_index: u32,
        x: i32,
        y: i32,
        size: u32,
        app: AppHandle,
    },
    Hide {
        account_index: u32,
    },
    SetPosition {
        account_index: u32,
        x: i32,
        y: i32,
    },
    AnimateTo {
        account_index: u32,
        target_x: i32,
        target_y: i32,
        duration_ms: u64,
    },
    SetUnread {
        account_index: u32,
        count: u32,
    },
    TriggerPulse {
        account_index: u32,
    },
    Tuck {
        account_index: u32,
        app: AppHandle,
    },
    Untuck {
        account_index: u32,
        app: AppHandle,
    },
    SetAlwaysOnTop {
        enabled: bool,
    },
    SetSize {
        account_index: u32,
        size: u32,
    },
}

/// Linux Native Bubble Manager singleton
pub struct NativeBubbleManager {
    bubbles: Arc<Mutex<HashMap<u32, LinuxBubbleWindowState>>>,
    bounds_cache: Arc<RwLock<HashMap<u32, (i32, i32, u32, u32)>>>,
}

static GLOBAL_MANAGER: OnceLock<NativeBubbleManager> = OnceLock::new();

impl NativeBubbleManager {
    /// Returns global singleton instance
    pub fn global() -> &'static Self {
        GLOBAL_MANAGER.get_or_init(|| {
            Self {
                bubbles: Arc::new(Mutex::new(HashMap::new())),
                bounds_cache: Arc::new(RwLock::new(HashMap::new())),
            }
        })
    }

    /// Dispatches control action to native bubbles
    pub fn dispatch(&self, action: NativeBubbleAction) {
        match action {
            NativeBubbleAction::Show { account_index, x, y, size, app } => {
                if let Ok(mut map) = self.bubbles.lock() {
                    let entry = map.entry(account_index).or_insert_with(|| LinuxBubbleWindowState::new(account_index, x, y, size));
                    entry.app_handle = Some(app);
                    entry.set_position(x, y);
                    entry.resize(size);
                    entry.set_visible(true);
                }
                if let Ok(mut cache) = self.bounds_cache.write() {
                    cache.insert(account_index, (x, y, size, size));
                }
            }
            NativeBubbleAction::Hide { account_index } => {
                if let Ok(mut map) = self.bubbles.lock() {
                    if let Some(bubble) = map.get_mut(&account_index) {
                        bubble.set_visible(false);
                    }
                }
                if let Ok(mut cache) = self.bounds_cache.write() {
                    cache.remove(&account_index);
                }
            }
            NativeBubbleAction::SetPosition { account_index, x, y } => {
                if let Ok(mut map) = self.bubbles.lock() {
                    if let Some(bubble) = map.get_mut(&account_index) {
                        bubble.set_position(x, y);
                    }
                }
                if let Ok(mut cache) = self.bounds_cache.write() {
                    if let Some(b) = cache.get_mut(&account_index) {
                        b.0 = x;
                        b.1 = y;
                    }
                }
            }
            NativeBubbleAction::AnimateTo { account_index, target_x, target_y, duration_ms } => {
                if let Ok(mut map) = self.bubbles.lock() {
                    if let Some(bubble) = map.get_mut(&account_index) {
                        bubble.animate_to(target_x, target_y, duration_ms);
                    }
                }
                if let Ok(mut cache) = self.bounds_cache.write() {
                    if let Some(b) = cache.get_mut(&account_index) {
                        b.0 = target_x;
                        b.1 = target_y;
                    }
                }
            }
            NativeBubbleAction::SetUnread { account_index, count } => {
                if let Ok(mut map) = self.bubbles.lock() {
                    if let Some(bubble) = map.get_mut(&account_index) {
                        bubble.set_unread_count(count);
                    }
                }
            }
            NativeBubbleAction::TriggerPulse { account_index } => {
                if let Ok(mut map) = self.bubbles.lock() {
                    if let Some(bubble) = map.get_mut(&account_index) {
                        bubble.trigger_pulse();
                    }
                }
            }
            NativeBubbleAction::Tuck { account_index, app } => {
                if let Ok(mut map) = self.bubbles.lock() {
                    if let Some(bubble) = map.get_mut(&account_index) {
                        bubble.tuck(&app);
                    }
                }
            }
            NativeBubbleAction::Untuck { account_index, app } => {
                if let Ok(mut map) = self.bubbles.lock() {
                    if let Some(bubble) = map.get_mut(&account_index) {
                        bubble.untuck(&app);
                    }
                }
            }
            NativeBubbleAction::SetAlwaysOnTop { .. } => {}
            NativeBubbleAction::SetSize { account_index, size } => {
                if let Ok(mut map) = self.bubbles.lock() {
                    if let Some(bubble) = map.get_mut(&account_index) {
                        bubble.resize(size);
                    }
                }
                if let Ok(mut cache) = self.bounds_cache.write() {
                    if let Some(b) = cache.get_mut(&account_index) {
                        b.2 = size;
                        b.3 = size;
                    }
                }
            }
        }
    }

    /// Checks if specified account has an active native bubble
    pub fn has_bubble(&self, account_index: u32) -> bool {
        if let Ok(cache) = self.bounds_cache.read() {
            return cache.contains_key(&account_index);
        }
        false
    }

    /// Returns bounds (x, y, w, h) of native bubble
    pub fn get_bounds(&self, account_index: u32) -> Option<(i32, i32, u32, u32)> {
        if let Ok(cache) = self.bounds_cache.read() {
            return cache.get(&account_index).copied();
        }
        None
    }

    /// Updates cached bounds
    pub fn update_cached_bounds(&self, account_index: u32, x: i32, y: i32, w: u32, h: u32) {
        if let Ok(mut cache) = self.bounds_cache.write() {
            cache.insert(account_index, (x, y, w, h));
        }
    }

    /// Checks if cursor is currently over native bubble
    pub fn is_cursor_over(&self, account_index: u32) -> bool {
        if let Some((x, y, w, h)) = self.get_bounds(account_index) {
            if let Some((cx, cy)) = crate::operating_systems::linux::cursor::LinuxCursorTracker::get_cursor_position() {
                return cx >= x && cx <= x + w as i32 && cy >= y && cy <= y + h as i32;
            }
        }
        false
    }

    /// Dispatches mouse press event
    pub fn handle_mouse_down(&self, account_index: u32, mouse_x: i32, mouse_y: i32) {
        if let Ok(mut map) = self.bubbles.lock() {
            if let Some(bubble) = map.get_mut(&account_index) {
                bubble.on_mouse_down(mouse_x, mouse_y);
            }
        }
    }

    /// Dispatches mouse move event
    pub fn handle_mouse_move(&self, account_index: u32, mouse_x: i32, mouse_y: i32) {
        if let Ok(mut map) = self.bubbles.lock() {
            if let Some(bubble) = map.get_mut(&account_index) {
                bubble.on_mouse_move(mouse_x, mouse_y);
                self.update_cached_bounds(account_index, bubble.x, bubble.y, bubble.bubble_size, bubble.bubble_size);
            }
        }
    }

    /// Dispatches mouse release event
    pub fn handle_mouse_up(&self, account_index: u32, mouse_x: i32, mouse_y: i32) {
        if let Ok(mut map) = self.bubbles.lock() {
            if let Some(bubble) = map.get_mut(&account_index) {
                bubble.on_mouse_up(mouse_x, mouse_y);
                self.update_cached_bounds(account_index, bubble.x, bubble.y, bubble.bubble_size, bubble.bubble_size);
            }
        }
    }

    /// Dispatches mouse enter event
    pub fn handle_mouse_enter(&self, account_index: u32) {
        if let Ok(mut map) = self.bubbles.lock() {
            if let Some(bubble) = map.get_mut(&account_index) {
                bubble.on_mouse_enter();
            }
        }
    }

    /// Dispatches mouse leave event
    pub fn handle_mouse_leave(&self, account_index: u32) {
        if let Ok(mut map) = self.bubbles.lock() {
            if let Some(bubble) = map.get_mut(&account_index) {
                bubble.on_mouse_leave();
            }
        }
    }

    /// Handles GLib spring physics animation tick (60 FPS)
    pub fn on_spring_timer_tick(&self, account_index: u32) -> bool {
        let mut finished = true;
        let mut cur_pos = (0, 0);
        let mut app_handle_opt = None;
        if let Ok(mut map) = self.bubbles.lock() {
            if let Some(bubble) = map.get_mut(&account_index) {
                if let Some((cur_x, cur_y, is_finished)) = bubble.anim_state.step() {
                    bubble.set_position(cur_x, cur_y);
                    cur_pos = (cur_x, cur_y);
                    finished = is_finished;
                    if is_finished {
                        bubble.spring_timer_id = 0;
                        bubble.render_and_update();
                        app_handle_opt = bubble.app_handle.clone();
                    }
                } else {
                    bubble.spring_timer_id = 0;
                }
            }
        }
        if cur_pos != (0, 0) {
            self.update_cached_bounds(account_index, cur_pos.0, cur_pos.1, 56, 56);
        }
        if finished {
            if let Some(app) = app_handle_opt {
                if let Some(cs) = app.try_state::<Arc<crate::state::ChatState>>() {
                    cs.set_bubble_rect(account_index, crate::movement::BubbleRect {
                        account_index,
                        x: cur_pos.0,
                        y: cur_pos.1,
                        width: 56,
                        height: 56,
                    });
                }
            }
            false
        } else {
            true
        }
    }

    /// Handles GLib hover leave debounce expiration (160ms)
    pub fn on_hover_leave_timer_tick(&self, account_index: u32) {
        let mut app_handle_opt = None;
        if let Ok(mut map) = self.bubbles.lock() {
            if let Some(bubble) = map.get_mut(&account_index) {
                bubble.hover_leave_timer_id = 0;
                bubble.is_hovered = false;
                bubble.render_and_update();
                app_handle_opt = bubble.app_handle.clone();
            }
        }
        if let Some(app) = app_handle_opt {
            self.dispatch(NativeBubbleAction::Tuck { account_index, app });
        }
    }
}

/// GLib 16ms spring animation timer callback
#[cfg(target_os = "linux")]
pub unsafe extern "C" fn spring_timer_callback(user_data: *mut std::ffi::c_void) -> i32 {
    let account_index = user_data as usize as u32;
    if NativeBubbleManager::global().on_spring_timer_tick(account_index) { 1 } else { 0 }
}

/// Fallback for non-Linux targets
#[cfg(not(target_os = "linux"))]
pub unsafe extern "C" fn spring_timer_callback(_user_data: *mut std::ffi::c_void) -> i32 { 0 }

/// GLib 160ms hover leave debounce timer callback
#[cfg(target_os = "linux")]
pub unsafe extern "C" fn hover_leave_timer_callback(user_data: *mut std::ffi::c_void) -> i32 {
    let account_index = user_data as usize as u32;
    NativeBubbleManager::global().on_hover_leave_timer_tick(account_index);
    0
}

/// Fallback for non-Linux targets
#[cfg(not(target_os = "linux"))]
pub unsafe extern "C" fn hover_leave_timer_callback(_user_data: *mut std::ffi::c_void) -> i32 { 0 }

/// Utility function to update cached bounds
pub fn update_cached_bubble_bounds(account_index: u32, x: i32, y: i32, w: u32, h: u32) {
    NativeBubbleManager::global().update_cached_bounds(account_index, x, y, w, h);
}
