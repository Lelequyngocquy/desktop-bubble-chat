pub mod ffi;
pub mod state;
pub mod types;
pub mod wndproc;

pub use ffi::*;
pub use state::*;
pub use types::*;
pub use wndproc::*;

use std::sync::Once;
use tauri::{AppHandle, Manager};

static REGISTER_CLASS_ONCE: Once = Once::new();

pub fn ensure_window_class_registered() {
    REGISTER_CLASS_ONCE.call_once(|| unsafe {
        let class_name_u16: Vec<u16> = CLASS_NAME.encode_utf16().collect();
        let hinstance = GetModuleHandleW(std::ptr::null());

        let wndclass = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfn_wnd_proc: Some(native_bubble_wndproc),
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

        RegisterClassExW(&wndclass);
    });
}

/// Đại diện cho 1 cửa sổ Bong bóng nổi Native Win32 Layered Window
pub struct NativeBubbleWindow {
    pub hwnd: *mut std::ffi::c_void,
    pub account_index: u32,
    pub bubble_size: u32,
    pub width: u32,
    pub height: u32,
}

unsafe impl Send for NativeBubbleWindow {}
unsafe impl Sync for NativeBubbleWindow {}

impl NativeBubbleWindow {
    /// Khởi tạo cửa sổ Bong bóng Native chuẩn 32-bit ARGB kèm tương tác chuột
    pub fn new(
        app: Option<AppHandle>,
        account_index: u32,
        x: i32,
        y: i32,
        size: u32,
    ) -> Result<Self, String> {
        ensure_window_class_registered();

        let class_name_u16: Vec<u16> = CLASS_NAME.encode_utf16().collect();
        let title_u16: Vec<u16> = format!("NativeBubble-{:02}\0", account_index)
            .encode_utf16()
            .collect();

        let is_on_top = app
            .as_ref()
            .and_then(|a| a.try_state::<std::sync::Arc<crate::state::ChatState>>())
            .map(|s| s.is_always_on_top.load(std::sync::atomic::Ordering::SeqCst))
            .unwrap_or(true);

        let canvas_w = size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        let win_x = x - BUBBLE_CANVAS_PAD_X;
        let win_y = y - BUBBLE_CANVAS_PAD_Y;

        unsafe {
            let hinstance = GetModuleHandleW(std::ptr::null());
            let topmost_flag = if is_on_top { WS_EX_TOPMOST } else { 0 };

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | topmost_flag | WS_EX_TOOLWINDOW,
                class_name_u16.as_ptr(),
                title_u16.as_ptr(),
                WS_POPUP,
                win_x,
                win_y,
                canvas_w as i32,
                canvas_h as i32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null_mut(),
            );

            if hwnd.is_null() {
                return Err("Failed to CreateWindowExW for NativeBubble".into());
            }

            let hdc_screen = GetDC(std::ptr::null_mut());
            let hdc_mem = CreateCompatibleDC(hdc_screen);
            ReleaseDC(std::ptr::null_mut(), hdc_screen);

            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmi_header.bi_width = canvas_w as i32;
            bmi.bmi_header.bi_height = -(canvas_h as i32); // Top-down DIB buffer
            bmi.bmi_header.bi_planes = 1;
            bmi.bmi_header.bi_bit_count = 32;
            bmi.bmi_header.bi_compression = BI_RGB;

            let mut p_bits: *mut u8 = std::ptr::null_mut();
            let hbitmap = CreateDIBSection(
                hdc_mem,
                &bmi,
                DIB_RGB_COLORS,
                &mut p_bits,
                std::ptr::null_mut(),
                0,
            );

            if hbitmap.is_null() || p_bits.is_null() {
                DeleteDC(hdc_mem);
                DestroyWindow(hwnd);
                return Err("Failed to CreateDIBSection for NativeBubble".into());
            }

            let old_bitmap = SelectObject(hdc_mem, hbitmap);

            let state = Box::new(BubbleWindowState {
                hwnd,
                account_index,
                bubble_size: size,
                width: canvas_w,
                height: canvas_h,
                x,
                y,
                is_dragging: false,
                has_dragged: false,
                drag_start_cursor: POINT { x: 0, y: 0 },
                drag_start_win: POINT { x: 0, y: 0 },
                is_hovered: false,
                unread_count: 0,
                drag_alpha: 255,
                anim_state: crate::windows::native_bubble::animator::SpringAnimationState::default(),
                app_handle: app,
                hdc_mem,
                hbitmap,
                old_bitmap,
                p_bits,
            });

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

            let win = Self {
                hwnd,
                account_index,
                bubble_size: size,
                width: canvas_w,
                height: canvas_h,
            };

            win.update();
            Ok(win)
        }
    }

    /// Cập nhật và vẽ lại bộ đệm đồ họa Skia
    pub fn update(&self) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut BubbleWindowState;
            if let Some(state) = ptr.as_mut() {
                state.update_layered_buffer();
            }
        }
    }

    /// Đặt tọa độ mới cho cửa sổ Native Bubble
    pub fn set_position(&self, x: i32, y: i32) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut BubbleWindowState;
            if let Some(state) = ptr.as_mut() {
                state.x = x;
                state.y = y;
                state.update_layered_buffer();
            }
            SetWindowPos(
                self.hwnd,
                std::ptr::null_mut(),
                x - BUBBLE_CANVAS_PAD_X,
                y - BUBBLE_CANVAS_PAD_Y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
    }

    /// Thiết lập chế độ luôn hiển thị trên cùng (Always-on-top / Topmost)
    pub fn set_always_on_top(&self, enabled: bool) {
        unsafe {
            let insert_after = if enabled {
                -1 as isize as *mut std::ffi::c_void // HWND_TOPMOST
            } else {
                -2 as isize as *mut std::ffi::c_void // HWND_NOTOPMOST
            };
            SetWindowPos(
                self.hwnd,
                insert_after,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    /// Thay đổi kích thước cửa sổ Native Bubble và cập nhật bộ đệm Skia
    pub fn resize(&mut self, new_size: u32) {
        self.bubble_size = new_size;
        self.width = new_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        self.height = new_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut BubbleWindowState;
            if let Some(state) = ptr.as_mut() {
                state.resize(new_size);
            }
        }
    }

    /// Hiển thị cửa sổ
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
        }
    }

    /// Ẩn cửa sổ
    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    /// Kiểm tra xem cửa sổ có đang hiển thị hay không
    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.hwnd) != 0 }
    }

    /// Cập nhật số tin nhắn chưa đọc và vẽ lại badge
    pub fn set_unread_count(&self, count: u32) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut BubbleWindowState;
            if let Some(state) = ptr.as_mut() {
                if state.unread_count != count {
                    state.unread_count = count;
                    state.update_layered_buffer();
                }
            }
        }
    }

    /// Kích hoạt hoạt ảnh sóng lan tỏa (Pulse Wave) - Đã được gỡ bỏ hoàn toàn để tối ưu hiệu năng
    pub fn trigger_pulse(&self) {
        // No-op: Đã loại bỏ hoàn toàn pulse animation để giảm tải tối đa GPU/CPU/Timer
    }

    /// Kích hoạt hoạt ảnh lò xo (Spring Animation 60 FPS) trượt mượt mà tới tọa độ đích
    pub fn animate_to(&self, target_x: i32, target_y: i32, duration_ms: u64) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut BubbleWindowState;
            if let Some(state) = ptr.as_mut() {
                if state.x == target_x && state.y == target_y {
                    return;
                }
                state.anim_state.start(state.x, state.y, target_x, target_y, duration_ms);
                SetTimer(self.hwnd, TIMER_ID_SPRING, 16, None);
            }
        }
    }

    /// Lấy tọa độ và kích thước hiện tại của cửa sổ Native Bubble
    pub fn get_bounds(&self) -> (i32, i32, u32, u32) {
        unsafe {
            let ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut BubbleWindowState;
            if let Some(state) = ptr.as_ref() {
                return (state.x, state.y, state.bubble_size, state.bubble_size);
            }
        }
        (0, 0, self.bubble_size, self.bubble_size)
    }
}

impl Drop for NativeBubbleWindow {
    fn drop(&mut self) {
        unsafe {
            if !self.hwnd.is_null() {
                DestroyWindow(self.hwnd);
                self.hwnd = std::ptr::null_mut();
            }
        }
    }
}
