#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::mpsc::{channel, Sender};
#[cfg(target_os = "windows")]
use std::sync::{Arc, OnceLock, RwLock};
#[cfg(target_os = "windows")]
use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
use super::window::{
    ensure_window_class_registered, is_window_fullscreen, GetForegroundWindow, KillTimer,
    NativeBubbleWindow, SetTimer, SetWinEventHook, UnhookWinEvent, EVENT_SYSTEM_FOREGROUND,
    PM_NOREMOVE, POINT, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_TIMER,
};

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSG {
    pub hwnd: *mut std::ffi::c_void,
    pub message: u32,
    pub wparam: usize,
    pub lparam: isize,
    pub time: u32,
    pub pt: POINT,
    pub l_private: u32,
}

#[cfg(target_os = "windows")]
const WM_USER: u32 = 0x0400;
#[cfg(target_os = "windows")]
const WM_BUBBLE_DISPATCH: u32 = WM_USER + 200;
#[cfg(target_os = "windows")]
const WM_BUBBLE_ASSERT_TOPMOST: u32 = WM_USER + 201;
#[cfg(target_os = "windows")]
const TOPMOST_SETTLE_DELAY_MS: u32 = 150;

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetMessageW(
        lpMsg: *mut MSG,
        hWnd: *mut std::ffi::c_void,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
    ) -> i32;
    fn PeekMessageW(
        lpMsg: *mut MSG,
        hWnd: *mut std::ffi::c_void,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
        wRemoveMsg: u32,
    ) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostThreadMessageW(
        idThread: u32,
        Msg: u32,
        wParam: usize,
        lParam: isize,
    ) -> i32;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
}

/// WinEvent hook callback forwarding external foreground switches to worker loop
#[cfg(target_os = "windows")]
unsafe extern "system" fn foreground_win_event_proc(
    _h_win_event_hook: *mut std::ffi::c_void,
    event: u32,
    hwnd: *mut std::ffi::c_void,
    _id_object: i32,
    _id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    if event == EVENT_SYSTEM_FOREGROUND && !hwnd.is_null() {
        unsafe { PostThreadMessageW(GetCurrentThreadId(), WM_BUBBLE_ASSERT_TOPMOST, hwnd as usize, 0); }
    }
}

/// Re-asserts HWND_TOPMOST on visible idle bubbles unless foreground window is fullscreen
#[cfg(target_os = "windows")]
fn reassert_topmost_for_bubbles(
    bubbles: &HashMap<u32, NativeBubbleWindow>,
    fg_hwnd: *mut std::ffi::c_void,
) {
    if bubbles.is_empty() || unsafe { is_window_fullscreen(fg_hwnd) } {
        return;
    }
    for win in bubbles.values() {
        if win.is_visible() && !win.is_dragging() {
            win.set_always_on_top(true);
        }
    }
}

/// Control actions sent to the dedicated Win32 Native Bubble UI thread
#[cfg(target_os = "windows")]
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
}

pub type BubbleBoundsCache = Arc<RwLock<HashMap<u32, (i32, i32, u32, u32)>>>;

/// Dedicated Native Bubble UI Thread Manager
#[cfg(target_os = "windows")]
pub struct NativeBubbleManager {
    thread_id: u32,
    sender: Sender<NativeBubbleAction>,
    bounds_cache: BubbleBoundsCache,
}

#[cfg(target_os = "windows")]
static GLOBAL_MANAGER: OnceLock<NativeBubbleManager> = OnceLock::new();

#[cfg(target_os = "windows")]
impl NativeBubbleManager {
    /// Returns global singleton NativeBubbleManager
    pub fn global() -> &'static Self {
        GLOBAL_MANAGER.get_or_init(|| {
            let (init_tx, init_rx) = channel::<u32>();
            let (cmd_tx, cmd_rx) = channel::<NativeBubbleAction>();
            let bounds_cache = Arc::new(RwLock::new(HashMap::new()));
            let bounds_clone = Arc::clone(&bounds_cache);

            std::thread::Builder::new()
                .name("native-bubble-worker".into())
                .spawn(move || {
                    let thread_id = unsafe { GetCurrentThreadId() };
                    ensure_window_class_registered();
                    let mut msg: MSG = unsafe { std::mem::zeroed() };
                    unsafe { PeekMessageW(&mut msg, std::ptr::null_mut(), WM_USER, WM_USER, PM_NOREMOVE); }
                    let event_hook = unsafe {
                        SetWinEventHook(
                            EVENT_SYSTEM_FOREGROUND,
                            EVENT_SYSTEM_FOREGROUND,
                            std::ptr::null_mut(),
                            Some(foreground_win_event_proc),
                            0,
                            0,
                            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                        )
                    };
                    if event_hook.is_null() {
                        eprintln!("[NativeBubble WARN] Failed to register foreground WinEvent hook");
                    }
                    let _ = init_tx.send(thread_id);
                    let mut bubbles: HashMap<u32, NativeBubbleWindow> = HashMap::new();
                    let mut is_always_on_top = true;
                    let mut settle_timer_id: usize = 0;
                    while unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) } > 0 {
                        if msg.message == WM_BUBBLE_DISPATCH {
                            while let Ok(action) = cmd_rx.try_recv() {
                                match action {
                                    NativeBubbleAction::Show { account_index, x, y, size, app } => {
                                        if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
                                            is_always_on_top = chat_state.is_always_on_top.load(std::sync::atomic::Ordering::SeqCst);
                                        }
                                        if let Some(win) = bubbles.get_mut(&account_index) {
                                            win.set_position(x, y);
                                            win.resize(size);
                                            win.show();
                                        } else {
                                            match NativeBubbleWindow::new(Some(app), account_index, x, y, size) {
                                                Ok(win) => {
                                                    win.show();
                                                    bubbles.insert(account_index, win);
                                                }
                                                Err(e) => {
                                                    eprintln!("[NativeBubble ERROR] Failed to create window for account {}: {}", account_index, e);
                                                }
                                            }
                                        }
                                        if let Ok(mut cache) = bounds_clone.write() {
                                            cache.insert(account_index, (x, y, size, size));
                                        }
                                    }
                                    NativeBubbleAction::Hide { account_index } => {
                                        if let Some(win) = bubbles.remove(&account_index) {
                                            win.hide();
                                        }
                                        if let Ok(mut cache) = bounds_clone.write() {
                                            cache.remove(&account_index);
                                        }
                                    }
                                    NativeBubbleAction::SetPosition { account_index, x, y } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            win.set_position(x, y);
                                            if let Ok(mut cache) = bounds_clone.write()
                                                && let Some(b) = cache.get_mut(&account_index) {
                                                    b.0 = x;
                                                    b.1 = y;
                                                }
                                        }
                                    }
                                    NativeBubbleAction::AnimateTo { account_index, target_x, target_y, duration_ms } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            win.animate_to(target_x, target_y, duration_ms);
                                            if let Ok(mut cache) = bounds_clone.write()
                                                && let Some(b) = cache.get_mut(&account_index) {
                                                    b.0 = target_x;
                                                    b.1 = target_y;
                                                }
                                        }
                                    }
                                    NativeBubbleAction::SetUnread { account_index, count } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            win.set_unread_count(count);
                                        }
                                    }
                                    NativeBubbleAction::TriggerPulse { account_index } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            win.trigger_pulse();
                                        }
                                    }
                                    NativeBubbleAction::Tuck { account_index, app } => {
                                        crate::utils::trim_controller::schedule_auto_trim_on_tuck(account_index, app.clone());
                                        let config = crate::movement::get_bubble_physics_config(&app);
                                        if config.is_free_position {
                                            continue;
                                        }
                                        if let Some(win) = bubbles.get(&account_index) {
                                            let screen = crate::movement::get_target_dashboard_screen(Some(&app));
                                            let (cur_x, cur_y, _, _) = win.get_bounds();
                                            let target_pos = crate::movement::BubbleMovementEngine::calculate_hover_pos(
                                                &screen,
                                                &config,
                                                cur_x,
                                                cur_y,
                                                false,
                                            );
                                            let mid_x = screen.x + screen.width / 2;
                                            let is_left_edge = cur_x + (config.size * screen.scale_factor) as i32 / 2 < mid_x;
                                            let all_bubbles: Vec<crate::movement::BubbleRect> = if let Ok(cache) = bounds_clone.read() {
                                                cache.iter().map(|(&idx, &(x, y, w, h))| crate::movement::BubbleRect {
                                                    account_index: idx,
                                                    x,
                                                    y,
                                                    width: w as i32,
                                                    height: h as i32,
                                                }).collect()
                                            } else {
                                                Vec::new()
                                            };
                                            let slots = crate::movement::CollisionResolver::resolve_mutual_column_layout(
                                                account_index,
                                                cur_y,
                                                is_left_edge,
                                                &screen,
                                                &config,
                                                &all_bubbles,
                                            );
                                            let resolved_y = slots.iter()
                                                .find(|s| s.account_index == account_index)
                                                .map(|s| s.target_y)
                                                .unwrap_or(cur_y);
                                            win.animate_to(target_pos.x, resolved_y, 180);
                                            if let Ok(mut cache) = bounds_clone.write()
                                                && let Some(b) = cache.get_mut(&account_index) {
                                                    b.0 = target_pos.x;
                                                    b.1 = resolved_y;
                                                }
                                            for slot in &slots {
                                                if slot.account_index == account_index {
                                                    continue;
                                                }
                                                let sibling_x = all_bubbles.iter()
                                                    .find(|b| b.account_index == slot.account_index)
                                                    .map(|b| b.x)
                                                    .unwrap_or(target_pos.x);
                                                if let Some(sibling_win) = bubbles.get(&slot.account_index) {
                                                    sibling_win.animate_to(sibling_x, slot.target_y, 180);
                                                }
                                                if let Ok(mut cache) = bounds_clone.write()
                                                    && let Some(b) = cache.get_mut(&slot.account_index) {
                                                        b.0 = sibling_x;
                                                        b.1 = slot.target_y;
                                                    }
                                            }
                                        }
                                    }
                                    NativeBubbleAction::Untuck { account_index, app } => {
                                        crate::utils::trim_controller::cancel_auto_trim(account_index);
                                        crate::utils::trim_controller::record_user_interaction(account_index);
                                        crate::platforms::messenger::windows::set_chat_window_memory_target(&app, account_index, false);
                                        let config = crate::movement::get_bubble_physics_config(&app);
                                        if config.is_free_position {
                                            continue;
                                        }
                                        if let Some(win) = bubbles.get(&account_index) {
                                            let screen = crate::movement::get_target_dashboard_screen(Some(&app));
                                            let config = crate::movement::get_bubble_physics_config(&app);
                                            let (cur_x, cur_y, _, _) = win.get_bounds();
                                            let target_pos = crate::movement::BubbleMovementEngine::calculate_hover_pos(
                                                &screen,
                                                &config,
                                                cur_x,
                                                cur_y,
                                                true,
                                            );
                                            let mid_x = screen.x + screen.width / 2;
                                            let is_left_edge = cur_x + (config.size * screen.scale_factor) as i32 / 2 < mid_x;
                                            let all_bubbles: Vec<crate::movement::BubbleRect> = if let Ok(cache) = bounds_clone.read() {
                                                cache.iter().map(|(&idx, &(x, y, w, h))| crate::movement::BubbleRect {
                                                    account_index: idx,
                                                    x,
                                                    y,
                                                    width: w as i32,
                                                    height: h as i32,
                                                }).collect()
                                            } else {
                                                Vec::new()
                                            };
                                            let slots = crate::movement::CollisionResolver::resolve_mutual_column_layout(
                                                account_index,
                                                cur_y,
                                                is_left_edge,
                                                &screen,
                                                &config,
                                                &all_bubbles,
                                            );
                                            let resolved_y = slots.iter()
                                                .find(|s| s.account_index == account_index)
                                                .map(|s| s.target_y)
                                                .unwrap_or(cur_y);
                                            win.animate_to(target_pos.x, resolved_y, 180);
                                            if let Ok(mut cache) = bounds_clone.write()
                                                && let Some(b) = cache.get_mut(&account_index) {
                                                    b.0 = target_pos.x;
                                                    b.1 = resolved_y;
                                                }
                                            for slot in &slots {
                                                if slot.account_index == account_index {
                                                    continue;
                                                }
                                                let sibling_x = all_bubbles.iter()
                                                    .find(|b| b.account_index == slot.account_index)
                                                    .map(|b| b.x)
                                                    .unwrap_or(target_pos.x);
                                                if let Some(sibling_win) = bubbles.get(&slot.account_index) {
                                                    sibling_win.animate_to(sibling_x, slot.target_y, 180);
                                                }
                                                if let Ok(mut cache) = bounds_clone.write()
                                                    && let Some(b) = cache.get_mut(&slot.account_index) {
                                                        b.0 = sibling_x;
                                                        b.1 = slot.target_y;
                                                    }
                                            }
                                        }
                                    }
                                    NativeBubbleAction::SetAlwaysOnTop { enabled } => {
                                        is_always_on_top = enabled;
                                        if !enabled && settle_timer_id != 0 {
                                            unsafe { KillTimer(std::ptr::null_mut(), settle_timer_id); }
                                            settle_timer_id = 0;
                                        }
                                        for win in bubbles.values() {
                                            win.set_always_on_top(enabled);
                                        }
                                    }
                                    NativeBubbleAction::AssertTopmost => {
                                        if is_always_on_top && !bubbles.is_empty() {
                                            let fg_hwnd = unsafe { GetForegroundWindow() };
                                            reassert_topmost_for_bubbles(&bubbles, fg_hwnd);
                                        }
                                    }
                                    NativeBubbleAction::SetSize { account_index, size } => {
                                        if let Some(win) = bubbles.get_mut(&account_index) {
                                            win.resize(size);
                                        }
                                        if let Ok(mut cache) = bounds_clone.write()
                                            && let Some(b) = cache.get_mut(&account_index) {
                                                b.2 = size;
                                                b.3 = size;
                                            }
                                    }
                                }
                            }
                        } else if msg.message == WM_BUBBLE_ASSERT_TOPMOST {
                            if is_always_on_top && !bubbles.is_empty() {
                                let fg_hwnd = msg.wparam as *mut std::ffi::c_void;
                                reassert_topmost_for_bubbles(&bubbles, fg_hwnd);
                                if settle_timer_id != 0 {
                                    unsafe { KillTimer(std::ptr::null_mut(), settle_timer_id); }
                                }
                                settle_timer_id = unsafe { SetTimer(std::ptr::null_mut(), 0, TOPMOST_SETTLE_DELAY_MS, None) };
                            }
                        } else if msg.message == WM_TIMER && msg.hwnd.is_null() && settle_timer_id != 0 && msg.wparam == settle_timer_id {
                            unsafe { KillTimer(std::ptr::null_mut(), settle_timer_id); }
                            settle_timer_id = 0;
                            if is_always_on_top && !bubbles.is_empty() {
                                let fg_hwnd = unsafe { GetForegroundWindow() };
                                reassert_topmost_for_bubbles(&bubbles, fg_hwnd);
                            }
                        } else {
                            unsafe {
                                TranslateMessage(&msg);
                                DispatchMessageW(&msg);
                            }
                        }
                    }
                    if settle_timer_id != 0 {
                        unsafe { KillTimer(std::ptr::null_mut(), settle_timer_id); }
                    }
                    if !event_hook.is_null() {
                        unsafe { UnhookWinEvent(event_hook); }
                    }
                })
                .expect("Failed to create Native Bubble Worker thread");
            let thread_id = init_rx.recv().expect("Failed to receive thread_id from Native Bubble Worker");
            Self {
                thread_id,
                sender: cmd_tx,
                bounds_cache,
            }
        })
    }

    /// Dispatches action to dedicated thread and wakes up GetMessageW
    pub fn dispatch(&self, action: NativeBubbleAction) {
        if self.sender.send(action).is_ok() {
            unsafe {
                PostThreadMessageW(self.thread_id, WM_BUBBLE_DISPATCH, 0, 0);
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
            let mut pt = POINT { x: 0, y: 0 };
            unsafe {
                if GetCursorPos(&mut pt) != 0 {
                    return pt.x >= x && pt.x <= x + w as i32 && pt.y >= y && pt.y <= y + h as i32;
                }
            }
        }
        false
    }
}

/// Utility to update cached bounds
#[cfg(target_os = "windows")]
pub fn update_cached_bubble_bounds(account_index: u32, x: i32, y: i32, w: u32, h: u32) {
    NativeBubbleManager::global().update_cached_bounds(account_index, x, y, w, h);
}
