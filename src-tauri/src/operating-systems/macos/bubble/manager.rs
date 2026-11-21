use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use tauri::{AppHandle, Manager};
use super::window::MacOSBubbleWindowState;

/// Control actions sent to the macOS Native Bubble manager
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
    AssertTopmost,
    SetSize {
        account_index: u32,
        size: u32,
    },
    Shutdown,
}

pub type MacOSBubbleBoundsCache = Arc<RwLock<HashMap<u32, (i32, i32, u32, u32)>>>;

/// macOS Native Bubble Manager singleton
pub struct NativeBubbleManager {
    bubbles: Arc<Mutex<HashMap<u32, MacOSBubbleWindowState>>>,
    bounds_cache: MacOSBubbleBoundsCache,
    is_always_on_top: AtomicBool,
    anim_loop_running: AtomicBool,
}

static GLOBAL_MANAGER: OnceLock<NativeBubbleManager> = OnceLock::new();

impl NativeBubbleManager {
    /// Returns global singleton instance
    pub fn global() -> &'static Self {
        GLOBAL_MANAGER.get_or_init(|| {
            Self {
                bubbles: Arc::new(Mutex::new(HashMap::new())),
                bounds_cache: Arc::new(RwLock::new(HashMap::new())),
                is_always_on_top: AtomicBool::new(true),
                anim_loop_running: AtomicBool::new(false),
            }
        })
    }

    /// Shuts down native bubbles if manager was initialized
    pub fn shutdown_if_initialized() {
        if let Some(mgr) = GLOBAL_MANAGER.get() {
            mgr.dispatch(NativeBubbleAction::Shutdown);
        }
    }

    fn apply_bounds_cache_preview(&self, action: &NativeBubbleAction) {
        match action {
            NativeBubbleAction::Show { account_index, x, y, size, .. } => {
                if let Ok(mut cache) = self.bounds_cache.write() {
                    cache.insert(*account_index, (*x, *y, *size, *size));
                }
            }
            NativeBubbleAction::Hide { account_index } => {
                if let Ok(mut cache) = self.bounds_cache.write() {
                    cache.remove(account_index);
                }
            }
            NativeBubbleAction::SetPosition { account_index, x, y } => {
                if let Ok(mut cache) = self.bounds_cache.write()
                    && let Some(b) = cache.get_mut(account_index) {
                        b.0 = *x;
                        b.1 = *y;
                    }
            }
            NativeBubbleAction::AnimateTo { account_index, target_x, target_y, .. } => {
                if let Ok(mut cache) = self.bounds_cache.write()
                    && let Some(b) = cache.get_mut(account_index) {
                        b.0 = *target_x;
                        b.1 = *target_y;
                    }
            }
            NativeBubbleAction::SetSize { account_index, size } => {
                if let Ok(mut cache) = self.bounds_cache.write()
                    && let Some(b) = cache.get_mut(account_index) {
                        b.2 = *size;
                        b.3 = *size;
                    }
            }
            NativeBubbleAction::Shutdown => {
                if let Ok(mut cache) = self.bounds_cache.write() {
                    cache.clear();
                }
            }
            _ => {}
        }
    }

    /// Dispatches control action to native bubbles on AppKit main thread
    pub fn dispatch(&self, action: NativeBubbleAction) {
        self.apply_bounds_cache_preview(&action);
        #[cfg(target_os = "macos")]
        if !cfg!(test) {
            super::window::ffi::run_on_macos_main_thread(move || {
                Self::global().execute_action_on_main_thread(action);
            });
            return;
        }
        self.execute_action_on_main_thread(action);
    }

    fn ensure_animation_driver_running(&self) {
        #[cfg(target_os = "macos")]
        if !cfg!(test)
            && self.anim_loop_running.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                std::thread::spawn(|| {
                    loop {
                        std::thread::sleep(std::time::Duration::from_millis(16));
                        let (tx, rx) = std::sync::mpsc::channel::<bool>();
                        super::window::ffi::run_on_macos_main_thread(move || {
                            let still_animating = Self::global().step_all_animations_on_main_thread();
                            let _ = tx.send(still_animating);
                        });
                        if !rx.recv().unwrap_or(false) {
                            Self::global().anim_loop_running.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                });
            }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = self.anim_loop_running.load(Ordering::Relaxed);
            self.step_all_animations_on_main_thread();
        }
    }

    /// Advances all active spring animations by one frame on the main thread
    pub fn step_all_animations_on_main_thread(&self) -> bool {
        let mut any_active = false;
        let mut updates = Vec::new();
        if let Ok(mut map) = self.bubbles.lock() {
            for (&idx, bubble) in map.iter_mut() {
                if bubble.anim_state.is_animating
                    && let Some((nx, ny, finished)) = bubble.step_animation() {
                        updates.push((idx, nx, ny, bubble.bubble_size, finished, bubble.app_handle.clone()));
                        if !finished {
                            any_active = true;
                        } else {
                            bubble.render_and_update();
                        }
                    }
            }
        }
        for (idx, nx, ny, bsz, finished, app_opt) in updates {
            self.update_cached_bounds(idx, nx, ny, bsz, bsz);
            if finished
                && let Some(app) = app_opt
                && let Some(cs) = app.try_state::<Arc<crate::state::ChatState>>() {
                    cs.set_bubble_rect(idx, crate::movement::BubbleRect {
                        account_index: idx,
                        x: nx,
                        y: ny,
                        width: bsz as i32,
                        height: bsz as i32,
                    });
                }
        }
        any_active
    }

    fn animate_displaced_siblings(&self, siblings: Vec<(u32, i32, i32, u64)>) {
        let has_siblings = !siblings.is_empty();
        for (sib_idx, tx, ty, dur) in siblings {
            if let Ok(mut map) = self.bubbles.lock()
                && let Some(sib) = map.get_mut(&sib_idx) {
                    sib.animate_to(tx, ty, dur);
                }
            if let Ok(mut cache) = self.bounds_cache.write()
                && let Some(b) = cache.get_mut(&sib_idx) {
                    b.0 = tx;
                    b.1 = ty;
                }
        }
        if has_siblings {
            self.ensure_animation_driver_running();
        }
    }

    fn execute_action_on_main_thread(&self, action: NativeBubbleAction) {
        match action {
            NativeBubbleAction::Show { account_index, x, y, size, app } => {
                if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
                    let top = chat_state.is_always_on_top.load(Ordering::SeqCst);
                    self.is_always_on_top.store(top, Ordering::SeqCst);
                }
                let top = self.is_always_on_top.load(Ordering::SeqCst);
                if let Ok(mut map) = self.bubbles.lock() {
                    let entry = map.entry(account_index).or_insert_with(|| MacOSBubbleWindowState::new(account_index, x, y, size));
                    entry.app_handle = Some(app);
                    entry.set_always_on_top(top);
                    entry.set_position(x, y);
                    entry.resize(size);
                    entry.set_visible(true);
                }
            }
            NativeBubbleAction::Hide { account_index } => {
                if let Ok(mut map) = self.bubbles.lock()
                    && let Some(bubble) = map.get_mut(&account_index) {
                        bubble.anim_state.stop();
                        bubble.set_visible(false);
                    }
            }
            NativeBubbleAction::SetPosition { account_index, x, y } => {
                if let Ok(mut map) = self.bubbles.lock()
                    && let Some(bubble) = map.get_mut(&account_index) {
                        bubble.set_position(x, y);
                    }
            }
            NativeBubbleAction::AnimateTo { account_index, target_x, target_y, duration_ms } => {
                if let Ok(mut map) = self.bubbles.lock()
                    && let Some(bubble) = map.get_mut(&account_index) {
                        bubble.animate_to(target_x, target_y, duration_ms);
                    }
                self.ensure_animation_driver_running();
            }
            NativeBubbleAction::SetUnread { account_index, count } => {
                if let Ok(mut map) = self.bubbles.lock()
                    && let Some(bubble) = map.get_mut(&account_index) {
                        bubble.set_unread_count(count);
                    }
            }
            NativeBubbleAction::TriggerPulse { account_index } => {
                if let Ok(mut map) = self.bubbles.lock()
                    && let Some(bubble) = map.get_mut(&account_index) {
                        bubble.trigger_pulse();
                    }
            }
            NativeBubbleAction::Tuck { account_index, app } => {
                if let Ok(mut map) = self.bubbles.lock()
                    && let Some(bubble) = map.get_mut(&account_index) {
                        bubble.tuck(&app);
                    }
                self.ensure_animation_driver_running();
            }
            NativeBubbleAction::Untuck { account_index, app } => {
                let displaced = if let Ok(mut map) = self.bubbles.lock() {
                    map.get_mut(&account_index).map(|b| b.untuck(&app)).unwrap_or_default()
                } else {
                    Vec::new()
                };
                self.animate_displaced_siblings(displaced);
                self.ensure_animation_driver_running();
            }
            NativeBubbleAction::SetAlwaysOnTop { enabled } => {
                self.is_always_on_top.store(enabled, Ordering::SeqCst);
                if let Ok(mut map) = self.bubbles.lock() {
                    for bubble in map.values_mut() {
                        bubble.set_always_on_top(enabled);
                    }
                }
            }
            NativeBubbleAction::AssertTopmost => {
                let enabled = self.is_always_on_top.load(Ordering::SeqCst);
                if enabled
                    && let Ok(mut map) = self.bubbles.lock() {
                        for bubble in map.values_mut() {
                            if bubble.is_visible && !bubble.is_dragging {
                                bubble.set_always_on_top(true);
                            }
                        }
                    }
            }
            NativeBubbleAction::SetSize { account_index, size } => {
                if let Ok(mut map) = self.bubbles.lock()
                    && let Some(bubble) = map.get_mut(&account_index) {
                        bubble.resize(size);
                    }
            }
            NativeBubbleAction::Shutdown => {
                if let Ok(mut map) = self.bubbles.lock() {
                    map.clear();
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
        if let Some((x, y, w, h)) = self.get_bounds(account_index)
            && let Some((cx, cy)) = crate::operating_systems::macos::cursor::MacOSCursorTracker::get_cursor_position() {
                return cx >= x && cx <= x + w as i32 && cy >= y && cy <= y + h as i32;
            }
        false
    }

    /// Handles mouse press on NSPanel view
    pub fn handle_mouse_down(&self, account_index: u32, mouse_x: i32, mouse_y: i32) {
        if let Ok(mut map) = self.bubbles.lock()
            && let Some(bubble) = map.get_mut(&account_index) {
                bubble.on_mouse_down(mouse_x, mouse_y);
            }
    }

    /// Handles mouse drag motion on NSPanel view
    pub fn handle_mouse_move(&self, account_index: u32, mouse_x: i32, mouse_y: i32) {
        let mut updated_bounds = None;
        let mut displaced = Vec::new();
        if let Ok(mut map) = self.bubbles.lock()
            && let Some(bubble) = map.get_mut(&account_index) {
                let was_hovered = bubble.is_hovered;
                bubble.on_mouse_move(mouse_x, mouse_y);
                updated_bounds = Some((bubble.x, bubble.y, bubble.bubble_size));
                if !was_hovered && bubble.is_hovered
                    && let Some(app) = bubble.app_handle.clone() {
                        displaced = bubble.untuck(&app);
                    }
            }
        if let Some((bx, by, bsz)) = updated_bounds {
            self.update_cached_bounds(account_index, bx, by, bsz, bsz);
        }
        self.animate_displaced_siblings(displaced);
    }

    /// Handles mouse release on NSPanel view without holding bubbles lock across sibling animations
    pub fn handle_mouse_up(&self, account_index: u32, mouse_x: i32, mouse_y: i32) {
        let mut updated_bounds = None;
        let mut displaced = Vec::new();
        if let Ok(mut map) = self.bubbles.lock()
            && let Some(bubble) = map.get_mut(&account_index) {
                displaced = bubble.on_mouse_up(mouse_x, mouse_y);
                updated_bounds = Some((bubble.x, bubble.y, bubble.bubble_size));
            }
        if let Some((bx, by, bsz)) = updated_bounds {
            self.update_cached_bounds(account_index, bx, by, bsz, bsz);
        }
        self.animate_displaced_siblings(displaced);
        self.ensure_animation_driver_running();
    }

    /// Handles mouse enter on NSPanel view without holding bubbles lock across sibling animations
    pub fn handle_mouse_enter(&self, account_index: u32) {
        let displaced = if let Ok(mut map) = self.bubbles.lock() {
            map.get_mut(&account_index).map(|b| b.on_mouse_enter()).unwrap_or_default()
        } else {
            Vec::new()
        };
        self.animate_displaced_siblings(displaced);
        self.ensure_animation_driver_running();
    }

    /// Handles mouse exit on NSPanel view and schedules tuck
    pub fn handle_mouse_leave(&self, account_index: u32) {
        let mut app_opt = None;
        if let Ok(mut map) = self.bubbles.lock()
            && let Some(bubble) = map.get_mut(&account_index) {
                bubble.on_mouse_leave();
                app_opt = bubble.app_handle.clone();
            }
        if let Some(app) = app_opt {
            self.dispatch(NativeBubbleAction::Tuck { account_index, app });
        }
    }
}

/// Utility function to update cached bounds
pub fn update_cached_bubble_bounds(account_index: u32, x: i32, y: i32, w: u32, h: u32) {
    NativeBubbleManager::global().update_cached_bounds(account_index, x, y, w, h);
}
