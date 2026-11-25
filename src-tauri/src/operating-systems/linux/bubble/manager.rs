use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use tauri::{AppHandle, Manager};
use super::window::LinuxBubbleWindowState;

/// Control actions sent to the Linux Native Bubble manager
pub enum NativeBubbleAction {
    Show { account_index: u32, x: i32, y: i32, size: u32, app: AppHandle },
    Hide { account_index: u32 },
    SetPosition { account_index: u32, x: i32, y: i32 },
    AnimateTo { account_index: u32, target_x: i32, target_y: i32, duration_ms: u64 },
    SetUnread { account_index: u32, count: u32 },
    TriggerPulse { account_index: u32 },
    Tuck { account_index: u32, app: AppHandle },
    Untuck { account_index: u32, app: AppHandle, mode: crate::movement::UntuckMode },
    SetAlwaysOnTop { enabled: bool },
    AssertTopmost,
    SetSize { account_index: u32, size: u32 },
    Shutdown,
}

pub type LinuxBubbleBoundsCache = Arc<RwLock<HashMap<u32, (i32, i32, u32, u32)>>>;

/// Linux Native Bubble Manager singleton
pub struct NativeBubbleManager {
    bubbles: Arc<Mutex<HashMap<u32, LinuxBubbleWindowState>>>,
    bounds_cache: LinuxBubbleBoundsCache,
    #[allow(dead_code)]
    pending_actions: Mutex<VecDeque<NativeBubbleAction>>,
    is_always_on_top: AtomicBool,
}

static GLOBAL_MANAGER: OnceLock<NativeBubbleManager> = OnceLock::new();

#[cfg(target_os = "linux")]
unsafe extern "C" fn drain_pending_gtk_actions(_user_data: *mut std::ffi::c_void) -> i32 {
    NativeBubbleManager::global().process_pending_on_main_thread();
    0
}

impl NativeBubbleManager {
    /// Returns global singleton instance
    pub fn global() -> &'static Self {
        GLOBAL_MANAGER.get_or_init(|| Self {
            bubbles: Arc::new(Mutex::new(HashMap::new())),
            bounds_cache: Arc::new(RwLock::new(HashMap::new())),
            pending_actions: Mutex::new(VecDeque::new()),
            is_always_on_top: AtomicBool::new(true),
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

    /// Dispatches control action to native bubbles, marshaling to GTK main thread on Linux
    pub fn dispatch(&self, action: NativeBubbleAction) {
        self.apply_bounds_cache_preview(&action);
        #[cfg(target_os = "linux")]
        {
            let on_main = unsafe {
                let ctx = super::window::ffi::g_main_context_default();
                !ctx.is_null() && super::window::ffi::g_main_context_is_owner(ctx) != 0
            };
            if !on_main && !cfg!(test) {
                if let Ok(mut q) = self.pending_actions.lock() {
                    let should_schedule = q.is_empty();
                    q.push_back(action);
                    if should_schedule {
                        unsafe { super::window::ffi::g_idle_add(Some(drain_pending_gtk_actions), std::ptr::null_mut()); }
                    }
                }
                return;
            }
        }
        self.execute_action_on_main_thread(action);
    }

    /// Drains queued GTK actions on the GLib main loop thread
    pub fn process_pending_on_main_thread(&self) {
        loop {
            let next = self.pending_actions.lock().ok().and_then(|mut q| q.pop_front());
            let Some(action) = next else { break; };
            self.execute_action_on_main_thread(action);
        }
    }

    fn animate_displaced_siblings(&self, siblings: Vec<(u32, i32, i32, u64)>) {
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
                    let entry = map.entry(account_index).or_insert_with(|| LinuxBubbleWindowState::new(account_index, x, y, size));
                    entry.app_handle = Some(app);
                    entry.set_always_on_top(top);
                    entry.set_position(x, y);
                    entry.resize(size);
                    entry.set_visible(true);
                }
            }
            NativeBubbleAction::Hide { account_index } => {
                if let Ok(mut map) = self.bubbles.lock()
                    && let Some(mut bubble) = map.remove(&account_index) {
                        bubble.stop_spring_timer();
                        bubble.stop_hover_leave_timer();
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
                let displaced = if let Ok(mut map) = self.bubbles.lock() {
                    map.get_mut(&account_index).map(|b| b.tuck(&app)).unwrap_or_default()
                } else {
                    Vec::new()
                };
                self.animate_displaced_siblings(displaced);
            }
            NativeBubbleAction::Untuck { account_index, app, mode } => {
                let displaced = if let Ok(mut map) = self.bubbles.lock() {
                    map.get_mut(&account_index).map(|b| b.untuck_with_mode(&app, mode)).unwrap_or_default()
                } else {
                    Vec::new()
                };
                self.animate_displaced_siblings(displaced);
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
                if self.is_always_on_top.load(Ordering::SeqCst)
                    && !crate::operating_systems::linux::cursor::LinuxCursorTracker::is_active_x11_window_fullscreen()
                    && let Ok(mut map) = self.bubbles.lock() {
                        for bubble in map.values_mut() {
                            if bubble.is_visible && !bubble.is_dragging {
                                bubble.assert_topmost();
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
        self.bounds_cache.read().map(|c| c.contains_key(&account_index)).unwrap_or(false)
    }

    /// Returns bounds (x, y, w, h) of native bubble
    pub fn get_bounds(&self, account_index: u32) -> Option<(i32, i32, u32, u32)> {
        self.bounds_cache.read().ok().and_then(|c| c.get(&account_index).copied())
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
            && let Some((cx, cy)) = crate::operating_systems::linux::cursor::LinuxCursorTracker::get_cursor_position() {
                return cx >= x && cx <= x + w as i32 && cy >= y && cy <= y + h as i32;
            }
        false
    }

    /// Dispatches mouse press event
    pub fn handle_mouse_down(&self, account_index: u32, mouse_x: i32, mouse_y: i32) {
        if let Ok(mut map) = self.bubbles.lock()
            && let Some(bubble) = map.get_mut(&account_index) {
                bubble.on_mouse_down(mouse_x, mouse_y);
            }
    }

    /// Dispatches right-click context action on Linux bubble
    pub fn handle_right_click(&self, account_index: u32, x: f64, y: f64) {
        let app_opt = self.bubbles.lock().ok().and_then(|map| map.get(&account_index).and_then(|b| b.app_handle.clone()));
        if let Some(app) = app_opt {
            crate::operating_systems::linux::tray::LinuxMenuManager::show_bubble_context_menu(&app, account_index, x, y);
        }
    }

    /// Dispatches mouse move event
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

    /// Dispatches mouse release event without holding bubbles lock across sibling animations
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
    }

    /// Dispatches mouse enter event without holding bubbles lock across sibling animations
    pub fn handle_mouse_enter(&self, account_index: u32) {
        let displaced = if let Ok(mut map) = self.bubbles.lock() {
            map.get_mut(&account_index).map(|b| b.on_mouse_enter()).unwrap_or_default()
        } else {
            Vec::new()
        };
        self.animate_displaced_siblings(displaced);
    }

    /// Dispatches mouse leave event
    pub fn handle_mouse_leave(&self, account_index: u32) {
        if let Ok(mut map) = self.bubbles.lock()
            && let Some(bubble) = map.get_mut(&account_index) {
                bubble.on_mouse_leave();
            }
    }

    /// Handles GLib spring physics animation tick (60 FPS)
    pub fn on_spring_timer_tick(&self, account_index: u32) -> bool {
        let mut finished = true;
        let mut cur_pos = None;
        let mut bubble_sz = 56u32;
        let mut app_handle_opt = None;
        if let Ok(mut map) = self.bubbles.lock()
            && let Some(bubble) = map.get_mut(&account_index) {
                bubble_sz = bubble.bubble_size;
                if let Some((cur_x, cur_y, is_finished)) = bubble.anim_state.step() {
                    bubble.set_position(cur_x, cur_y);
                    cur_pos = Some((cur_x, cur_y));
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
        if let Some((cx, cy)) = cur_pos {
            self.update_cached_bounds(account_index, cx, cy, bubble_sz, bubble_sz);
            if finished
                && let Some(app) = app_handle_opt
                && let Some(cs) = app.try_state::<Arc<crate::state::ChatState>>() {
                    cs.set_bubble_rect(account_index, crate::movement::BubbleRect { account_index, x: cx, y: cy, width: bubble_sz as i32, height: bubble_sz as i32 });
                }
        }
        !finished
    }

    /// Handles GLib hover leave debounce expiration with Windows parity guards
    pub fn on_hover_leave_timer_tick(&self, account_index: u32) {
        let mut app_handle_opt = None;
        if let Ok(mut map) = self.bubbles.lock()
            && let Some(bubble) = map.get_mut(&account_index) {
                bubble.hover_leave_timer_id = 0;
                if bubble.can_tuck_on_hover_leave() {
                    app_handle_opt = bubble.app_handle.clone();
                }
            }
        if let Some(app) = app_handle_opt {
            self.dispatch(NativeBubbleAction::Tuck { account_index, app });
        }
    }
}

/// GLib 16ms spring animation timer callback
///
/// # Safety
/// `user_data` encodes the numeric `account_index` value cast as a raw pointer.
#[cfg(target_os = "linux")]
pub unsafe extern "C" fn spring_timer_callback(user_data: *mut std::ffi::c_void) -> i32 {
    let account_index = user_data as usize as u32;
    if NativeBubbleManager::global().on_spring_timer_tick(account_index) { 1 } else { 0 }
}

/// Fallback for non-Linux targets
///
/// # Safety
/// Safe no-op stub for non-Linux targets.
#[cfg(not(target_os = "linux"))]
pub unsafe extern "C" fn spring_timer_callback(_user_data: *mut std::ffi::c_void) -> i32 { 0 }

/// GLib 80ms hover leave debounce timer callback
///
/// # Safety
/// `user_data` encodes the numeric `account_index` value cast as a raw pointer.
#[cfg(target_os = "linux")]
pub unsafe extern "C" fn hover_leave_timer_callback(user_data: *mut std::ffi::c_void) -> i32 {
    let account_index = user_data as usize as u32;
    NativeBubbleManager::global().on_hover_leave_timer_tick(account_index);
    0
}

/// Fallback for non-Linux targets
///
/// # Safety
/// Safe no-op stub for non-Linux targets.
#[cfg(not(target_os = "linux"))]
pub unsafe extern "C" fn hover_leave_timer_callback(_user_data: *mut std::ffi::c_void) -> i32 { 0 }

/// Utility function to update cached bounds
pub fn update_cached_bubble_bounds(account_index: u32, x: i32, y: i32, w: u32, h: u32) {
    NativeBubbleManager::global().update_cached_bounds(account_index, x, y, w, h);
}
