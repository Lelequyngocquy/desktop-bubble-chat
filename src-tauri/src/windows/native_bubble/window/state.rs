use tauri::AppHandle;
use tauri::Manager;

use super::ffi::*;
use super::types::*;
use crate::movement::{BubbleMovementEngine, ScreenMetrics};
use crate::windows::native_bubble::renderer::{render_bubble_bgra, BubbleRenderOptions};

/// Cấu trúc lưu trữ trạng thái nội bộ của Native Bubble gắn liền với HWND qua GWLP_USERDATA
pub struct BubbleWindowState {
    pub hwnd: *mut std::ffi::c_void,
    pub account_index: u32,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,

    // Trạng thái kéo thả chuột (Mouse Interaction)
    pub is_dragging: bool,
    pub has_dragged: bool,
    pub drag_start_cursor: POINT,
    pub drag_start_win: POINT,

    // Trạng thái hiển thị
    pub is_hovered: bool,
    pub has_pulse: bool,
    pub pulse_progress: f32,
    pub pulse_frame: u32,
    pub unread_count: u32,
    pub drag_alpha: u8,

    // Hoạt ảnh lò xo 60 FPS (Native Spring Animation)
    pub anim_state: crate::windows::native_bubble::animator::SpringAnimationState,

    // Kết nối SSOT với Tauri AppHandle
    pub app_handle: Option<AppHandle>,

    // Bộ nhớ đệm đồ họa GDI DIBSection
    pub hdc_mem: *mut std::ffi::c_void,
    pub hbitmap: *mut std::ffi::c_void,
    pub old_bitmap: *mut std::ffi::c_void,
    pub p_bits: *mut u8,
}

impl BubbleWindowState {
    /// Thay đổi kích thước bộ đệm GDI DIBSection và cập nhật cửa sổ
    pub fn resize(&mut self, new_size: u32) {
        if self.width == new_size && self.height == new_size {
            return;
        }
        self.width = new_size;
        self.height = new_size;

        unsafe {
            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmi_header.bi_width = new_size as i32;
            bmi.bmi_header.bi_height = -(new_size as i32);
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

            if !new_hbitmap.is_null() && !p_bits.is_null() {
                SelectObject(self.hdc_mem, self.old_bitmap);
                DeleteObject(self.hbitmap);
                self.hbitmap = new_hbitmap;
                self.old_bitmap = SelectObject(self.hdc_mem, new_hbitmap);
                self.p_bits = p_bits;
            }

            SetWindowPos(
                self.hwnd,
                std::ptr::null_mut(),
                0,
                0,
                new_size as i32,
                new_size as i32,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }

        self.update_layered_buffer();
    }

    /// Cập nhật và vẽ lại nội dung Bong bóng qua UpdateLayeredWindow
    pub fn update_layered_buffer(&mut self) {
        let opts = BubbleRenderOptions {
            size: self.width,
            account_index: self.account_index,
            is_hovered: self.is_hovered,
            has_pulse: self.has_pulse,
            pulse_progress: self.pulse_progress,
            unread_count: self.unread_count,
        };

        if let Some(bgra) = render_bubble_bgra(&opts) {
            unsafe {
                let bytes_to_copy = (self.width * self.height * 4) as usize;
                if bgra.len() == bytes_to_copy && !self.p_bits.is_null() {
                    std::ptr::copy_nonoverlapping(bgra.as_ptr(), self.p_bits, bytes_to_copy);

                    let pt_dst = POINT { x: self.x, y: self.y };
                    let size = SIZE {
                        cx: self.width as i32,
                        cy: self.height as i32,
                    };
                    let pt_src = POINT { x: 0, y: 0 };
                    let blend = BLENDFUNCTION {
                        blend_op: AC_SRC_OVER,
                        blend_flags: 0,
                        source_constant_alpha: self.drag_alpha,
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

    /// Cập nhật nhanh độ mờ của cửa sổ qua UpdateLayeredWindow mà không cần render lại Skia
    pub fn set_layered_alpha(&mut self, alpha: u8) {
        if self.drag_alpha == alpha {
            return;
        }
        self.drag_alpha = alpha;
        unsafe {
            let blend = BLENDFUNCTION {
                blend_op: AC_SRC_OVER,
                blend_flags: 0,
                source_constant_alpha: alpha,
                alpha_format: AC_SRC_ALPHA,
            };
            UpdateLayeredWindow(
                self.hwnd,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                self.hdc_mem,
                std::ptr::null(),
                0,
                &blend,
                ULW_ALPHA,
            );
        }
    }

    /// Kích hoạt hoạt ảnh lò xo trượt ra mép (Untuck: appear_ratio = 0.0) 180ms ease-out cubic
    pub fn untuck(&mut self) {
        if let Some(app) = &self.app_handle {
            let monitor = app
                .get_webview_window("main")
                .and_then(|m| m.current_monitor().ok().flatten())
                .or_else(|| app.primary_monitor().ok().flatten());

            if let Some(mon) = monitor {
                let screen = ScreenMetrics::from_monitor(&mon);
                let config = crate::movement::get_bubble_physics_config(app);
                let target_pos = BubbleMovementEngine::calculate_hover_pos(
                    &screen,
                    &config,
                    self.x,
                    self.y,
                    true,
                );
                if self.x != target_pos.x || self.y != target_pos.y {
                    self.anim_state.start(self.x, self.y, target_pos.x, target_pos.y, 180);
                    unsafe {
                        SetTimer(self.hwnd, TIMER_ID_SPRING, 16, None);
                    }
                }
            }
        }
    }

    /// Kích hoạt hoạt ảnh lò xo thụt vào mép (Tuck: tuck_ratio = 0.75) 180ms ease-out cubic
    pub fn tuck(&mut self) {
        if let Some(app) = &self.app_handle {
            let monitor = app
                .get_webview_window("main")
                .and_then(|m| m.current_monitor().ok().flatten())
                .or_else(|| app.primary_monitor().ok().flatten());

            if let Some(mon) = monitor {
                let screen = ScreenMetrics::from_monitor(&mon);
                let config = crate::movement::get_bubble_physics_config(app);
                let target_pos = BubbleMovementEngine::calculate_hover_pos(
                    &screen,
                    &config,
                    self.x,
                    self.y,
                    false,
                );
                if self.x != target_pos.x || self.y != target_pos.y {
                    self.anim_state.start(self.x, self.y, target_pos.x, target_pos.y, 180);
                    unsafe {
                        SetTimer(self.hwnd, TIMER_ID_SPRING, 16, None);
                    }
                }
            }
        }
    }
}
