use tauri::AppHandle;

use super::ffi::*;
use super::types::*;
use crate::movement::BubbleMovementEngine;
use crate::operating_systems::windows::bubble::renderer::{render_bubble_bgra, BubbleRenderOptions};

/// Internal state of a native bubble attached to HWND via GWLP_USERDATA.
pub struct BubbleWindowState {
    pub hwnd: *mut std::ffi::c_void,
    pub account_index: u32,
    pub bubble_size: u32,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,

    // Mouse drag state
    pub is_dragging: bool,
    pub has_dragged: bool,
    pub drag_start_cursor: POINT,
    pub drag_start_win: POINT,

    // Display state
    pub is_hovered: bool,
    pub unread_count: u32,
    pub drag_alpha: u8,

    // 60 FPS spring animation state
    pub anim_state: crate::operating_systems::windows::bubble::animator::SpringAnimationState,

    // SSOT connection to Tauri AppHandle
    pub app_handle: Option<AppHandle>,

    // GDI DIBSection graphics buffer
    pub hdc_mem: *mut std::ffi::c_void,
    pub hbitmap: *mut std::ffi::c_void,
    pub old_bitmap: *mut std::ffi::c_void,
    pub p_bits: *mut u8,
}

impl BubbleWindowState {
    /// Resizes GDI DIBSection buffer and updates window.
    pub fn resize(&mut self, new_size: u32) {
        if self.bubble_size == new_size {
            return;
        }
        self.bubble_size = new_size;
        let canvas_w = new_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = new_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        self.width = canvas_w;
        self.height = canvas_h;
        unsafe {
            let mut bmi: BITMAPINFO = std::mem::zeroed();
            bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmi_header.bi_width = canvas_w as i32;
            bmi.bmi_header.bi_height = -(canvas_h as i32);
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
                self.x - BUBBLE_CANVAS_PAD_X,
                self.y - BUBBLE_CANVAS_PAD_Y,
                canvas_w as i32,
                canvas_h as i32,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        self.update_layered_buffer();
    }

    /// Determines if bubble is on the left or right half of the screen for badge orientation.
    pub fn is_at_left_edge(&self) -> bool {
        let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
        let screen_cx = screen.x + (screen.width / 2);
        let bubble_cx = self.x + (self.bubble_size as i32 / 2);
        bubble_cx < screen_cx
    }

    /// Updates and redraws bubble content via UpdateLayeredWindow.
    pub fn update_layered_buffer(&mut self) {
        let opts = BubbleRenderOptions {
            canvas_width: self.width,
            canvas_height: self.height,
            bubble_size: self.bubble_size,
            account_index: self.account_index,
            is_hovered: self.is_hovered,
            is_left_edge: self.is_at_left_edge(),
            unread_count: self.unread_count,
        };
        if let Some(bgra) = render_bubble_bgra(&opts) {
            unsafe {
                let bytes_to_copy = (self.width * self.height * 4) as usize;
                if bgra.len() == bytes_to_copy && !self.p_bits.is_null() {
                    std::ptr::copy_nonoverlapping(bgra.as_ptr(), self.p_bits, bytes_to_copy);
                    let pt_dst = POINT {
                        x: self.x - BUBBLE_CANVAS_PAD_X,
                        y: self.y - BUBBLE_CANVAS_PAD_Y,
                    };
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

    /// Updates window opacity via UpdateLayeredWindow without re-rendering.
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

    /// Triggers spring animation to untuck from edge (180ms ease-out cubic).
    pub fn untuck(&mut self) {
        crate::utils::trim_controller::cancel_auto_trim(self.account_index);
        crate::utils::trim_controller::record_user_interaction(self.account_index);
        if let Some(app) = &self.app_handle {
            crate::platforms::messenger::windows::set_chat_window_memory_target(app, self.account_index, false);
            let config = crate::movement::get_bubble_physics_config(app);
            if config.is_free_position {
                return;
            }
            let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
            let target_pos = BubbleMovementEngine::calculate_hover_pos(
                &screen,
                &config,
                self.x,
                self.y,
                true,
            );
            let mid_x = screen.x + screen.width / 2;
            let is_left_edge = self.x + (self.bubble_size as i32 / 2) < mid_x;
            let all_bubbles = crate::presentation::bubble::get_active_bubble_rects(app, None);
            let slots = crate::movement::collision::CollisionResolver::resolve_mutual_column_layout(
                self.account_index,
                self.y,
                is_left_edge,
                &screen,
                &config,
                &all_bubbles,
            );
            let resolved_y = slots
                .iter()
                .find(|s| s.account_index == self.account_index)
                .map(|s| s.target_y)
                .unwrap_or(self.y);
            if self.x != target_pos.x || self.y != resolved_y {
                self.anim_state.start(self.x, self.y, target_pos.x, resolved_y, 180);
                unsafe {
                    SetTimer(self.hwnd, TIMER_ID_SPRING, 16, None);
                }
            }
            for slot in &slots {
                if slot.account_index == self.account_index {
                    continue;
                }
                let sibling_x = all_bubbles
                    .iter()
                    .find(|b| b.account_index == slot.account_index)
                    .map(|b| b.x)
                    .unwrap_or(target_pos.x);
                crate::operating_systems::windows::bubble::NativeBubbleManager::global().dispatch(
                    crate::operating_systems::windows::bubble::NativeBubbleAction::AnimateTo {
                        account_index: slot.account_index,
                        target_x: sibling_x,
                        target_y: slot.target_y,
                        duration_ms: 180,
                    },
                );
            }
        }
    }

    /// Triggers spring animation to tuck into edge (180ms ease-out cubic).
    pub fn tuck(&mut self) {
        if let Some(app) = &self.app_handle {
            crate::utils::trim_controller::schedule_auto_trim_on_tuck(self.account_index, app.clone());
            let config = crate::movement::get_bubble_physics_config(app);
            if config.is_free_position {
                return;
            }
            let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
            let target_pos = BubbleMovementEngine::calculate_hover_pos(
                &screen,
                &config,
                self.x,
                self.y,
                false,
            );
            let mid_x = screen.x + screen.width / 2;
            let is_left_edge = self.x + (self.bubble_size as i32 / 2) < mid_x;
            let all_bubbles = crate::presentation::bubble::get_active_bubble_rects(app, None);
            let slots = crate::movement::collision::CollisionResolver::resolve_mutual_column_layout(
                self.account_index,
                self.y,
                is_left_edge,
                &screen,
                &config,
                &all_bubbles,
            );
            let resolved_y = slots
                .iter()
                .find(|s| s.account_index == self.account_index)
                .map(|s| s.target_y)
                .unwrap_or(self.y);
            if self.x != target_pos.x || self.y != resolved_y {
                self.anim_state.start(self.x, self.y, target_pos.x, resolved_y, 180);
                unsafe {
                    SetTimer(self.hwnd, TIMER_ID_SPRING, 16, None);
                }
            }
            for slot in &slots {
                if slot.account_index == self.account_index {
                    continue;
                }
                let sibling_x = all_bubbles
                    .iter()
                    .find(|b| b.account_index == slot.account_index)
                    .map(|b| b.x)
                    .unwrap_or(target_pos.x);
                crate::operating_systems::windows::bubble::NativeBubbleManager::global().dispatch(
                    crate::operating_systems::windows::bubble::NativeBubbleAction::AnimateTo {
                        account_index: slot.account_index,
                        target_x: sibling_x,
                        target_y: slot.target_y,
                        duration_ms: 180,
                    },
                );
            }
        }
    }
}
