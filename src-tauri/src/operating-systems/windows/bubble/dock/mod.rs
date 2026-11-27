pub mod state;
pub mod wndproc;

pub use state::*;
pub use wndproc::*;

use std::sync::Once;
use std::time::Instant;
use tauri::AppHandle;

use crate::operating_systems::windows::bubble::window::{
    CreateCompatibleDC, CreateWindowExW, DestroyWindow, GetDC, GetModuleHandleW, GetWindowLongPtrW,
    LoadCursorW, RegisterClassExW, ReleaseDC, SetWindowLongPtrW, CS_HREDRAW, CS_VREDRAW,
    GWLP_USERDATA, IDC_ARROW, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};
use crate::presentation::bubble::dock::{
    compute_action_dock_layout, ActionDockLayout, DockActionStateSnapshot, DockAnimationState,
};

static REGISTER_DOCK_CLASS_ONCE: Once = Once::new();

/// Registers the Win32 window class for the singleton ActionDock overlay once
pub fn ensure_action_dock_class_registered() {
    REGISTER_DOCK_CLASS_ONCE.call_once(|| unsafe {
        let class_name_u16: Vec<u16> = ACTION_DOCK_CLASS_NAME.encode_utf16().collect();
        let hinstance = GetModuleHandleW(std::ptr::null());
        let wndclass = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfn_wnd_proc: Some(native_action_dock_wndproc),
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: hinstance,
            h_icon: std::ptr::null_mut(),
            h_cursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW as *const u16),
            hbr_background: std::ptr::null_mut(),
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: class_name_u16.as_ptr(),
            h_icon_sm: std::ptr::null_mut(),
        };
        let atom = RegisterClassExW(&wndclass);
        if atom == 0 {
            eprintln!("[ActionDock WARN] RegisterClassExW returned 0");
        }
    });
}

/// Singleton Win32 layered overlay window rendering the 6 Liquid Glass dock droplets
pub struct NativeActionDockWindow {
    pub hwnd: *mut std::ffi::c_void,
}

impl NativeActionDockWindow {
    /// Creates a hidden non-activating layered window confined to the NativeBubble worker thread
    pub fn new(
        is_always_on_top: bool,
        hover_corridor_cache: DockHoverCorridorCache,
    ) -> Result<Self, String> {
        ensure_action_dock_class_registered();
        let class_name_u16: Vec<u16> = ACTION_DOCK_CLASS_NAME.encode_utf16().collect();
        let title_u16: Vec<u16> = "NativeActionDock\0".encode_utf16().collect();
        let default_screen = crate::movement::ScreenMetrics { x: 0, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
        let init_layout = compute_action_dock_layout(12, 120, 56, &default_screen, 12.0, 60.0);
        unsafe {
            let hinstance = GetModuleHandleW(std::ptr::null());
            let topmost_flag = if is_always_on_top { WS_EX_TOPMOST } else { 0 };
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | topmost_flag | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class_name_u16.as_ptr(),
                title_u16.as_ptr(),
                WS_POPUP,
                init_layout.window_x,
                init_layout.window_y,
                init_layout.canvas_width as i32,
                init_layout.canvas_height as i32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null_mut(),
            );
            if hwnd.is_null() {
                return Err("Failed to CreateWindowExW for NativeActionDock".into());
            }
            let hdc_screen = GetDC(std::ptr::null_mut());
            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if !hdc_screen.is_null() {
                ReleaseDC(std::ptr::null_mut(), hdc_screen);
            }
            if hdc_mem.is_null() {
                DestroyWindow(hwnd);
                return Err("Failed to CreateCompatibleDC for NativeActionDock".into());
            }
            let state = Box::new(ActionDockWindowState {
                hwnd,
                account_index: 1,
                layout: init_layout,
                snapshot: DockActionStateSnapshot::default(),
                animator: DockAnimationState::default(),
                last_tick: Instant::now(),
                anim_timer_active: false,
                is_tracking_mouse: false,
                is_visible: false,
                is_always_on_top,
                app_handle: None,
                hover_corridor_cache,
                hdc_mem,
                hbitmap: std::ptr::null_mut(),
                old_bitmap: std::ptr::null_mut(),
                p_bits: std::ptr::null_mut(),
                buf_width: 0,
                buf_height: 0,
            });
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);
            Ok(Self { hwnd })
        }
    }

    /// Shows or repositions dock for an active bubble
    pub fn show_for_bubble(
        &self,
        account_index: u32,
        layout: ActionDockLayout,
        snapshot: DockActionStateSnapshot,
        app: AppHandle,
    ) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut ActionDockWindowState;
            if let Some(state) = ptr.as_mut() {
                state.show_for_bubble(account_index, layout, snapshot, app);
            }
        }
    }

    /// Syncs toggle states and optional updated layout while visible
    pub fn sync_state(
        &self,
        account_index: u32,
        snapshot: DockActionStateSnapshot,
        layout_opt: Option<ActionDockLayout>,
    ) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut ActionDockWindowState;
            if let Some(state) = ptr.as_mut() {
                state.sync_state(account_index, snapshot, layout_opt);
            }
        }
    }

    /// Hides the ActionDock immediately or with fast retract fade
    pub fn hide(&self, immediate: bool) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut ActionDockWindowState;
            if let Some(state) = ptr.as_mut() {
                state.hide(immediate);
            }
        }
    }

    /// Returns true if the ActionDock overlay is currently visible
    pub fn is_visible(&self) -> bool {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *const ActionDockWindowState;
            ptr.as_ref().is_some_and(|s| s.is_visible)
        }
    }

    /// Sets topmost Z-order without activating the window
    pub fn set_always_on_top(&self, enabled: bool) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut ActionDockWindowState;
            if let Some(state) = ptr.as_mut() {
                state.set_always_on_top(enabled);
            }
        }
    }
}

impl Drop for NativeActionDockWindow {
    fn drop(&mut self) {
        if !self.hwnd.is_null() {
            unsafe { DestroyWindow(self.hwnd); }
            self.hwnd = std::ptr::null_mut();
        }
    }
}
