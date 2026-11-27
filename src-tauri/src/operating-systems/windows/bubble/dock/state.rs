use std::sync::{Arc, RwLock};
use std::time::Instant;
use tauri::AppHandle;

use crate::operating_systems::windows::bubble::window::{
    CreateDIBSection, DeleteObject, KillTimer, SelectObject, SetTimer, SetWindowPos, ShowWindow,
    UpdateLayeredWindow, AC_SRC_ALPHA, AC_SRC_OVER, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    BLENDFUNCTION, DIB_RGB_COLORS, POINT, SIZE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE, ULW_ALPHA,
};
use crate::presentation::bubble::dock::{
    render_action_dock_with_tooltip, ActionDockLayout, DockActionStateSnapshot, DockAnimationState,
    DockRenderOptions, DockVisibilityPhase, ScreenRect,
};
use super::tooltip_text::{composite_tooltip_text_bgra, measure_tooltip_text_width_px};

pub const ACTION_DOCK_CLASS_NAME: &str = "DesktopBubbleChat_ActionDock\0";
pub const TIMER_ID_DOCK_ANIM: usize = 2001;
pub const DOCK_ANIM_INTERVAL_MS: u32 = 16;

/// Shared lock-protected Hover Bridge screen rect for O(1) cursor queries
pub type DockHoverCorridorCache = Arc<RwLock<Option<ScreenRect>>>;

/// Internal Win32 state attached to singleton ActionDock HWND via GWLP_USERDATA
pub struct ActionDockWindowState {
    pub hwnd: *mut std::ffi::c_void,
    pub account_index: u32,
    pub layout: ActionDockLayout,
    pub snapshot: DockActionStateSnapshot,
    pub animator: DockAnimationState,
    pub last_tick: Instant,
    pub anim_timer_active: bool,
    pub is_tracking_mouse: bool,
    pub is_visible: bool,
    pub is_always_on_top: bool,
    pub app_handle: Option<AppHandle>,
    pub hover_corridor_cache: DockHoverCorridorCache,
    pub hdc_mem: *mut std::ffi::c_void,
    pub hbitmap: *mut std::ffi::c_void,
    pub old_bitmap: *mut std::ffi::c_void,
    pub p_bits: *mut u8,
    pub buf_width: u32,
    pub buf_height: u32,
}

impl ActionDockWindowState {
    /// Updates shared hover corridor cache according to current visibility and layout
    pub fn sync_hover_corridor_cache(&self) {
        if let Ok(mut guard) = self.hover_corridor_cache.write() {
            *guard = if self.is_visible && self.animator.phase != DockVisibilityPhase::Hiding {
                Some(self.layout.hover_corridor)
            } else {
                None
            };
        }
    }

    /// Reallocates GDI DIBSection only when canvas dimensions change
    pub fn ensure_dib_size(&mut self, width: u32, height: u32) -> bool {
        if width == 0 || height == 0 {
            return false;
        }
        if self.buf_width == width && self.buf_height == height && !self.p_bits.is_null() {
            return true;
        }
        unsafe {
            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmi_header.bi_width = width as i32;
            bmi.bmi_header.bi_height = -(height as i32);
            bmi.bmi_header.bi_planes = 1;
            bmi.bmi_header.bi_bit_count = 32;
            bmi.bmi_header.bi_compression = BI_RGB;
            let mut p_bits: *mut u8 = std::ptr::null_mut();
            let new_hbitmap = CreateDIBSection(
                self.hdc_mem,
                &bmi,
                DIB_RGB_COLORS,
                &mut p_bits,
                std::ptr::null_mut(),
                0,
            );
            if new_hbitmap.is_null() || p_bits.is_null() {
                if !new_hbitmap.is_null() {
                    DeleteObject(new_hbitmap);
                }
                return false;
            }
            if !self.hbitmap.is_null() {
                SelectObject(self.hdc_mem, self.old_bitmap);
                DeleteObject(self.hbitmap);
            }
            self.old_bitmap = SelectObject(self.hdc_mem, new_hbitmap);
            self.hbitmap = new_hbitmap;
            self.p_bits = p_bits;
            self.buf_width = width;
            self.buf_height = height;
            true
        }
    }

    /// Starts or kills 16ms animation timer to guarantee 0% CPU usage when idle
    pub fn sync_anim_timer(&mut self) {
        if self.animator.needs_frame() {
            if !self.anim_timer_active {
                self.last_tick = Instant::now();
                let res = unsafe { SetTimer(self.hwnd, TIMER_ID_DOCK_ANIM, DOCK_ANIM_INTERVAL_MS, None) };
                self.anim_timer_active = res != 0;
            }
        } else if self.anim_timer_active {
            unsafe { KillTimer(self.hwnd, TIMER_ID_DOCK_ANIM); }
            self.anim_timer_active = false;
        }
    }

    /// Renders Liquid Glass droplets into BGRA DIBSection and commits via UpdateLayeredWindow
    pub fn render_layered_buffer(&mut self) {
        let w = self.layout.canvas_width;
        let h = self.layout.canvas_height;
        if !self.ensure_dib_size(w, h) {
            return;
        }
        let frames = self.animator.sample_droplets(self.layout.direction);
        let tooltip_label = self.animator.hovered_action.map(|act| self.snapshot.tooltip_label(act));
        let tooltip_width = tooltip_label.map(|lbl| measure_tooltip_text_width_px(lbl, self.layout.scale_factor));
        let opts = DockRenderOptions {
            layout: self.layout.clone(),
            state: self.snapshot,
            frames,
            hovered_action: self.animator.hovered_action,
            tooltip_text_width_px: tooltip_width,
        };
        if let Some((mut bgra, pill_opt)) = render_action_dock_with_tooltip(&opts) {
            if let (Some(pill), Some(label)) = (pill_opt, tooltip_label) {
                composite_tooltip_text_bgra(&mut bgra, w, h, &pill, label, self.layout.scale_factor);
            }
            let bytes = (w * h * 4) as usize;
            if bgra.len() == bytes && !self.p_bits.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(bgra.as_ptr(), self.p_bits, bytes);
                    let pt_dst = POINT { x: self.layout.window_x, y: self.layout.window_y };
                    let size = SIZE { cx: w as i32, cy: h as i32 };
                    let pt_src = POINT { x: 0, y: 0 };
                    let blend = BLENDFUNCTION {
                        blend_op: AC_SRC_OVER,
                        blend_flags: 0,
                        source_constant_alpha: 255,
                        alpha_format: AC_SRC_ALPHA,
                    };
                    UpdateLayeredWindow(
                        self.hwnd,
                        std::ptr::null_mut(),
                        &pt_dst,
                        &size,
                        self.hdc_mem,
                        &pt_src,
                        0,
                        &blend,
                        ULW_ALPHA,
                    );
                }
            }
        }
    }

    /// Shows or repositions dock for an active bubble and triggers staggered appear if newly shown
    pub fn show_for_bubble(
        &mut self,
        account_index: u32,
        layout: ActionDockLayout,
        snapshot: DockActionStateSnapshot,
        app: AppHandle,
    ) {
        let should_appear = !self.is_visible
            || self.account_index != account_index
            || self.animator.phase == DockVisibilityPhase::Hiding
            || self.animator.phase == DockVisibilityPhase::Hidden;
        self.account_index = account_index;
        self.layout = layout;
        self.snapshot = snapshot;
        self.app_handle = Some(app);
        self.is_visible = true;
        if should_appear {
            self.animator.start_appear();
        }
        self.sync_hover_corridor_cache();
        self.render_layered_buffer();
        unsafe {
            let z_after = if self.is_always_on_top { -1_isize as *mut std::ffi::c_void } else { std::ptr::null_mut() };
            let z_flag = if self.is_always_on_top { 0 } else { SWP_NOZORDER };
            SetWindowPos(
                self.hwnd,
                z_after,
                self.layout.window_x,
                self.layout.window_y,
                self.layout.canvas_width as i32,
                self.layout.canvas_height as i32,
                z_flag | SWP_NOACTIVATE,
            );
            ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
        self.sync_anim_timer();
    }

    /// Updates SSOT toggle state and optional new layout while visible without restarting appear animation
    pub fn sync_state(
        &mut self,
        account_index: u32,
        snapshot: DockActionStateSnapshot,
        layout_opt: Option<ActionDockLayout>,
    ) {
        if !self.is_visible || self.account_index != account_index {
            return;
        }
        self.snapshot = snapshot;
        if let Some(new_layout) = layout_opt {
            self.layout = new_layout;
            self.sync_hover_corridor_cache();
            unsafe {
                SetWindowPos(
                    self.hwnd,
                    std::ptr::null_mut(),
                    self.layout.window_x,
                    self.layout.window_y,
                    self.layout.canvas_width as i32,
                    self.layout.canvas_height as i32,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
        self.render_layered_buffer();
        self.sync_anim_timer();
    }

    /// Hides the ActionDock immediately or with fast 90ms fade-out
    pub fn hide(&mut self, immediate: bool) {
        if immediate || !self.is_visible {
            self.animator.hide_immediate();
            self.is_visible = false;
            self.is_tracking_mouse = false;
            self.sync_hover_corridor_cache();
            self.sync_anim_timer();
            unsafe { ShowWindow(self.hwnd, SW_HIDE); }
        } else {
            self.animator.start_hide();
            self.sync_hover_corridor_cache();
            self.render_layered_buffer();
            self.sync_anim_timer();
        }
    }

    /// Applies topmost or normal Z-order without activating the window
    pub fn set_always_on_top(&mut self, enabled: bool) {
        self.is_always_on_top = enabled;
        let insert_after = if enabled { -1_isize as *mut std::ffi::c_void } else { -2_isize as *mut std::ffi::c_void };
        unsafe {
            SetWindowPos(self.hwnd, insert_after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
    }
}
