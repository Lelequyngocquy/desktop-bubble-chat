use std::sync::Arc;
use tauri::{AppHandle, Manager};

use super::ffi::*;
use super::types::*;
use crate::movement::{BubbleInteractionFlags, BubbleMovementEngine, UntuckMode};
use crate::operating_systems::windows::bubble::renderer::{render_bubble_bgra, BubbleRenderOptions};
use crate::state::ChatState;

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
    pub is_chat_open: bool,
    pub drag_alpha: u8,

    // 60 FPS spring and two-speed heartbeat animation states
    pub anim_state: crate::operating_systems::windows::bubble::animator::SpringAnimationState,
    pub heartbeat: crate::operating_systems::windows::bubble::animator::HeartbeatAnimationState,
    pub pulse_timer_interval_ms: u32,

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
        let canvas_w = new_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = new_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        let allocated = unsafe {
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
            if new_hbitmap.is_null() || p_bits.is_null() {
                if !new_hbitmap.is_null() {
                    DeleteObject(new_hbitmap);
                }
                false
            } else {
                SelectObject(self.hdc_mem, self.old_bitmap);
                DeleteObject(self.hbitmap);
                self.hbitmap = new_hbitmap;
                self.old_bitmap = SelectObject(self.hdc_mem, new_hbitmap);
                self.p_bits = p_bits;
                true
            }
        };
        if !allocated {
            return;
        }
        self.bubble_size = new_size;
        self.width = canvas_w;
        self.height = canvas_h;
        unsafe {
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
        let hb = self.heartbeat.current_frame();
        let opts = BubbleRenderOptions {
            canvas_width: self.width,
            canvas_height: self.height,
            bubble_size: self.bubble_size,
            account_index: self.account_index,
            is_hovered: self.is_hovered,
            is_left_edge: self.is_at_left_edge(),
            unread_count: self.unread_count,
            has_alert_ring: self.heartbeat.is_alert_active,
            beat_scale: hb.beat_scale,
            ripple_progress: hb.ripple_progress,
            ripple_alpha: hb.ripple_alpha,
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

    /// Synchronizes heartbeat paused flag from hover, drag, and chat-open state without rendering.
    pub fn sync_heartbeat_pause_flag(&mut self) -> bool {
        let paused = self.is_hovered || self.is_dragging || self.is_chat_open;
        self.heartbeat.set_paused(paused)
    }

    /// Synchronizes heartbeat pause state, timer scheduling, and redraw when pause state changes.
    pub fn sync_heartbeat_pause(&mut self) {
        let changed = self.sync_heartbeat_pause_flag();
        if changed || (self.heartbeat.should_animate() && self.pulse_timer_interval_ms == 0) {
            self.sync_heartbeat_timer_and_render();
        }
    }

    /// Stops active Win32 heartbeat timer immediately.
    pub fn stop_heartbeat_timer(&mut self) {
        if self.pulse_timer_interval_ms != 0 {
            unsafe {
                KillTimer(self.hwnd, TIMER_ID_PULSE);
            }
            self.pulse_timer_interval_ms = 0;
        }
    }

    /// Arms or disarms the two-speed Win32 heartbeat timer (16ms beat / sleep rest) and redraws.
    pub fn sync_heartbeat_timer_and_render(&mut self) {
        if let Some(delay_ms) = self.heartbeat.next_delay_ms() {
            let target_ms = delay_ms as u32;
            if self.pulse_timer_interval_ms != target_ms {
                unsafe {
                    SetTimer(self.hwnd, TIMER_ID_PULSE, target_ms, None);
                }
                self.pulse_timer_interval_ms = target_ms;
            }
        } else {
            self.stop_heartbeat_timer();
        }
        self.update_layered_buffer();
    }

    /// Collects current interaction, chat, and alert flags for tuck eligibility checks.
    pub fn interaction_flags(&self) -> BubbleInteractionFlags {
        let (is_chat_open, is_chat_pinned, is_free_position, is_alert_active) = if let Some(app) = &self.app_handle {
            let chat_open = self.is_chat_open
                || crate::platforms::messenger::windows::get_messenger_webview(app, self.account_index)
                    .and_then(|w| w.is_visible().ok())
                    .unwrap_or(false);
            let (pinned, alert) = app
                .try_state::<Arc<ChatState>>()
                .map(|cs| (cs.is_chat_pinned(self.account_index), cs.is_account_alert_active(self.account_index)))
                .unwrap_or((false, false));
            let free = crate::movement::get_bubble_physics_config(app).is_free_position;
            (chat_open, pinned, free, alert || self.heartbeat.is_alert_active)
        } else {
            (self.is_chat_open, false, false, self.heartbeat.is_alert_active)
        };
        BubbleInteractionFlags {
            is_hovered: self.is_hovered,
            is_dragging: self.is_dragging,
            is_chat_open,
            is_chat_pinned,
            is_free_position,
            is_alert_active,
        }
    }

    /// Executes shared edge transition plan and syncs bounds cache consistently.
    fn apply_edge_transition(&mut self, is_untucked: bool) {
        let Some(app) = self.app_handle.clone() else {
            return;
        };
        let config = crate::movement::get_bubble_physics_config(&app);
        let screen = crate::movement::get_target_dashboard_screen(Some(&app));
        let all_bubbles = crate::presentation::bubble::get_active_bubble_rects(&app, None);
        let Some(plan) = BubbleMovementEngine::compute_edge_transition_plan(
            self.account_index,
            self.x,
            self.y,
            is_untucked,
            &screen,
            &config,
            &all_bubbles,
        ) else {
            return;
        };
        if self.x != plan.target_x || self.y != plan.target_y {
            crate::operating_systems::windows::bubble::manager::update_cached_bubble_bounds(
                self.account_index,
                plan.target_x,
                plan.target_y,
                self.bubble_size,
                self.bubble_size,
            );
            self.anim_state.start(self.x, self.y, plan.target_x, plan.target_y, 180);
            unsafe {
                SetTimer(self.hwnd, TIMER_ID_SPRING, 16, None);
            }
        }
        for (sib_idx, sib_x, sib_y) in plan.displaced_siblings {
            crate::operating_systems::windows::bubble::manager::update_cached_bubble_bounds(
                sib_idx,
                sib_x,
                sib_y,
                self.bubble_size,
                self.bubble_size,
            );
            crate::operating_systems::windows::bubble::NativeBubbleManager::global().dispatch(
                crate::operating_systems::windows::bubble::NativeBubbleAction::AnimateTo {
                    account_index: sib_idx,
                    target_x: sib_x,
                    target_y: sib_y,
                    duration_ms: 180,
                },
            );
        }
    }

    /// Triggers spring animation to untuck from edge with explicit RAM-trim mode.
    pub fn untuck_with_mode(&mut self, mode: UntuckMode) {
        crate::utils::trim_controller::AutoTrimController::global().handle_untuck_mode(self.account_index, mode);
        if mode == UntuckMode::UserInteraction
            && let Some(app) = &self.app_handle {
                crate::platforms::messenger::windows::set_chat_window_memory_target(app, self.account_index, false);
            }
        self.apply_edge_transition(true);
    }

    /// Triggers spring animation to untuck from edge on user interaction.
    pub fn untuck(&mut self) {
        self.untuck_with_mode(UntuckMode::UserInteraction);
    }

    /// Schedules idle RAM trim and physically tucks bubble into edge when eligible.
    pub fn tuck(&mut self) {
        let flags = self.interaction_flags();
        if !flags.is_dragging && !flags.is_hovered && !flags.is_chat_open && !flags.is_chat_pinned
            && let Some(app) = &self.app_handle {
                crate::utils::trim_controller::schedule_auto_trim_on_tuck(self.account_index, app.clone());
            }
        if !BubbleMovementEngine::can_auto_tuck(flags) {
            return;
        }
        self.apply_edge_transition(false);
    }
}
