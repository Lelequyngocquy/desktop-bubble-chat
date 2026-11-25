#[cfg(target_os = "windows")]
mod actions;

#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(target_os = "windows")]
use std::sync::mpsc::{channel, Sender};
#[cfg(target_os = "windows")]
use std::sync::{Arc, OnceLock, RwLock};
#[cfg(target_os = "windows")]
use tauri::AppHandle;

#[cfg(target_os = "windows")]
use super::window::{
    ensure_window_class_registered, is_window_fullscreen, CreateWindowExW, DestroyWindow,
    GetForegroundWindow, GetModuleHandleW, KillTimer, NativeBubbleWindow, SetTimer,
    SetWinEventHook, UnhookWinEvent, EVENT_SYSTEM_FOREGROUND, POINT, WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS, WM_TIMER,
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
pub(super) const TIMER_ID_SETTLE_TOPMOST: usize = 1;
#[cfg(target_os = "windows")]
const HWND_MESSAGE: *mut std::ffi::c_void = (-3_isize) as *mut std::ffi::c_void;

#[cfg(target_os = "windows")]
static FOREGROUND_MSG_HWND: AtomicUsize = AtomicUsize::new(0);

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetMessageW(
        lpMsg: *mut MSG,
        hWnd: *mut std::ffi::c_void,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
    ) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostMessageW(
        hWnd: *mut std::ffi::c_void,
        Msg: u32,
        wParam: usize,
        lParam: isize,
    ) -> i32;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
}

/// WinEvent hook callback forwarding external foreground switches to worker message window
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
    let msg_hwnd_val = FOREGROUND_MSG_HWND.load(Ordering::Acquire);
    if event == EVENT_SYSTEM_FOREGROUND && !hwnd.is_null() && msg_hwnd_val != 0 {
        unsafe { PostMessageW(msg_hwnd_val as *mut std::ffi::c_void, WM_BUBBLE_ASSERT_TOPMOST, hwnd as usize, 0); }
    }
}

/// Re-asserts HWND_TOPMOST on visible idle bubbles unless foreground window is fullscreen
#[cfg(target_os = "windows")]
pub(super) fn reassert_topmost_for_bubbles(
    bubbles: &HashMap<u32, NativeBubbleWindow>,
    fg_hwnd: *mut std::ffi::c_void,
) {
    if bubbles.is_empty()
        || crate::operating_systems::windows::fullscreen_overlay::has_active_fullscreen_overlay_session()
        || unsafe { is_window_fullscreen(fg_hwnd) }
    {
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
        mode: crate::movement::UntuckMode,
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

pub type BubbleBoundsCache = Arc<RwLock<HashMap<u32, (i32, i32, u32, u32)>>>;

/// Dedicated Native Bubble UI Thread Manager
#[cfg(target_os = "windows")]
pub struct NativeBubbleManager {
    msg_hwnd: usize,
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
            let (init_tx, init_rx) = channel::<usize>();
            let (cmd_tx, cmd_rx) = channel::<NativeBubbleAction>();
            let bounds_cache = Arc::new(RwLock::new(HashMap::new()));
            let bounds_clone = Arc::clone(&bounds_cache);
            std::thread::Builder::new()
                .name("native-bubble-worker".into())
                .spawn(move || {
                    ensure_window_class_registered();
                    let static_class: Vec<u16> = "STATIC\0".encode_utf16().collect();
                    let msg_hwnd = unsafe {
                        CreateWindowExW(
                            0,
                            static_class.as_ptr(),
                            std::ptr::null(),
                            0,
                            0,
                            0,
                            0,
                            0,
                            HWND_MESSAGE,
                            std::ptr::null_mut(),
                            GetModuleHandleW(std::ptr::null()),
                            std::ptr::null_mut(),
                        )
                    };
                    if msg_hwnd.is_null() {
                        eprintln!("[NativeBubble ERROR] Failed to create message-only dispatcher window");
                        let _ = init_tx.send(0);
                        return;
                    }
                    FOREGROUND_MSG_HWND.store(msg_hwnd as usize, Ordering::Release);
                    let mut msg: MSG = unsafe { std::mem::zeroed() };
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
                    let _ = init_tx.send(msg_hwnd as usize);
                    let mut bubbles: HashMap<u32, NativeBubbleWindow> = HashMap::new();
                    let mut is_always_on_top = true;
                    let mut settle_timer_active = false;
                    let mut should_shutdown = false;
                    while unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) } > 0 {
                        if msg.message == WM_BUBBLE_DISPATCH {
                            while let Ok(action) = cmd_rx.try_recv() {
                                actions::handle_bubble_action(
                                    action,
                                    &mut bubbles,
                                    &bounds_clone,
                                    &mut is_always_on_top,
                                    &mut settle_timer_active,
                                    &mut should_shutdown,
                                    msg_hwnd,
                                );
                            }
                            if should_shutdown {
                                break;
                            }
                        } else if msg.message == WM_BUBBLE_ASSERT_TOPMOST {
                            if is_always_on_top && !bubbles.is_empty() {
                                let fg_hwnd = msg.wparam as *mut std::ffi::c_void;
                                reassert_topmost_for_bubbles(&bubbles, fg_hwnd);
                                if settle_timer_active {
                                    unsafe { KillTimer(msg_hwnd, TIMER_ID_SETTLE_TOPMOST); }
                                }
                                settle_timer_active = unsafe { SetTimer(msg_hwnd, TIMER_ID_SETTLE_TOPMOST, TOPMOST_SETTLE_DELAY_MS, None) } != 0;
                            }
                        } else if msg.message == WM_TIMER && msg.hwnd == msg_hwnd && settle_timer_active && msg.wparam == TIMER_ID_SETTLE_TOPMOST {
                            unsafe { KillTimer(msg_hwnd, TIMER_ID_SETTLE_TOPMOST); }
                            settle_timer_active = false;
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
                    if settle_timer_active {
                        unsafe { KillTimer(msg_hwnd, TIMER_ID_SETTLE_TOPMOST); }
                    }
                    FOREGROUND_MSG_HWND.store(0, Ordering::Release);
                    if !event_hook.is_null() {
                        unsafe { UnhookWinEvent(event_hook); }
                    }
                    unsafe { DestroyWindow(msg_hwnd); }
                })
                .expect("Failed to create Native Bubble Worker thread");
            let msg_hwnd = init_rx.recv().expect("Failed to receive msg_hwnd from Native Bubble Worker");
            Self {
                msg_hwnd,
                sender: cmd_tx,
                bounds_cache,
            }
        })
    }

    /// Shuts down the worker thread and unhooks global WinEvent listener if initialized
    pub fn shutdown_if_initialized() {
        if let Some(mgr) = GLOBAL_MANAGER.get() {
            mgr.dispatch(NativeBubbleAction::Shutdown);
        }
    }

    /// Dispatches action to dedicated thread via message-only HWND and wakes up GetMessageW
    pub fn dispatch(&self, action: NativeBubbleAction) {
        if self.msg_hwnd != 0 && self.sender.send(action).is_ok() {
            unsafe {
                PostMessageW(self.msg_hwnd as *mut std::ffi::c_void, WM_BUBBLE_DISPATCH, 0, 0);
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
