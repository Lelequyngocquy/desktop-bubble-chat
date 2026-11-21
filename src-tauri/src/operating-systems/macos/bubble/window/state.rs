use std::ffi::c_void;
use super::panel::MacOSPanelController;
use super::types::*;
use tauri::{Manager, PhysicalPosition};
use crate::movement::BubbleMovementEngine;
use crate::operating_systems::macos::bubble::animator::SpringAnimationState;
use crate::operating_systems::macos::bubble::renderer::{render_bubble_bgra, BubbleRenderOptions};

pub struct MacOSBubbleWindowState {
    pub account_index: u32,
    pub panel: *mut c_void,
    pub x: i32,
    pub y: i32,
    pub bubble_size: u32,
    pub unread_count: u32,
    pub is_hovered: bool,
    pub is_left_edge: bool,
    pub is_always_on_top: bool,
    pub is_visible: bool,
    pub is_dragging: bool,
    pub has_dragged: bool,
    pub drag_start_mouse_x: i32,
    pub drag_start_mouse_y: i32,
    pub drag_start_bubble_x: i32,
    pub drag_start_bubble_y: i32,
    pub is_tucked: bool,
    pub anim_state: SpringAnimationState,
    pub app_handle: Option<tauri::AppHandle>,
}

unsafe impl Send for MacOSBubbleWindowState {}
unsafe impl Sync for MacOSBubbleWindowState {}

impl MacOSBubbleWindowState {
    pub fn new(account_index: u32, x: i32, y: i32, bubble_size: u32) -> Self {
        let canvas_w = bubble_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = bubble_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        let screen = crate::movement::get_target_dashboard_screen(None);
        let is_left_edge = x + (bubble_size as i32 / 2) < (screen.x + screen.width / 2);
        let cocoa_rect = MacOSPanelController::top_left_physical_to_cocoa_rect(
            x,
            y,
            canvas_w,
            canvas_h,
            screen.y,
            screen.height,
            screen.scale_factor,
        );
        let panel = if cfg!(test) {
            std::ptr::null_mut()
        } else {
            MacOSPanelController::create_floating_bubble_panel(
                account_index,
                cocoa_rect.origin.x,
                cocoa_rect.origin.y,
                cocoa_rect.size.width,
                cocoa_rect.size.height,
            )
        };
        let mut inst = Self {
            account_index,
            panel,
            x,
            y,
            bubble_size,
            unread_count: 0,
            is_hovered: false,
            is_left_edge,
            is_always_on_top: true,
            is_visible: false,
            is_dragging: false,
            has_dragged: false,
            drag_start_mouse_x: 0,
            drag_start_mouse_y: 0,
            drag_start_bubble_x: x,
            drag_start_bubble_y: y,
            is_tucked: false,
            anim_state: SpringAnimationState::default(),
            app_handle: None,
        };
        if !inst.panel.is_null() {
            inst.render_and_update();
        }
        inst
    }

    /// Updates dynamic left/right edge orientation based on screen midpoint
    pub fn refresh_edge_side(&mut self) -> bool {
        let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
        let mid_x = screen.x + screen.width / 2;
        let next_left = self.x + (self.bubble_size as i32 / 2) < mid_x;
        if self.is_left_edge != next_left {
            self.is_left_edge = next_left;
            return true;
        }
        false
    }

    /// Updates always-on-top NSPanel level
    pub fn set_always_on_top(&mut self, enabled: bool) {
        self.is_always_on_top = enabled;
        MacOSPanelController::set_panel_always_on_top(self.panel, enabled);
    }

    /// Renders tiny-skia pixel buffer and commits to CALayer
    pub fn render_and_update(&mut self) {
        self.refresh_edge_side();
        let canvas_w = self.bubble_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = self.bubble_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        let opts = BubbleRenderOptions {
            canvas_width: canvas_w,
            canvas_height: canvas_h,
            bubble_size: self.bubble_size,
            account_index: self.account_index,
            is_hovered: self.is_hovered,
            is_left_edge: self.is_left_edge,
            unread_count: self.unread_count,
        };
        if let Some(bgra_data) = render_bubble_bgra(&opts) {
            MacOSPanelController::update_panel_layer_contents(self.panel, &bgra_data, canvas_w as usize, canvas_h as usize);
        }
    }

    /// Updates unread badge count
    pub fn set_unread_count(&mut self, count: u32) {
        if self.unread_count != count {
            self.unread_count = count;
            self.render_and_update();
        }
    }

    /// Sets visibility state
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
        MacOSPanelController::set_panel_visible(self.panel, visible);
        if visible {
            MacOSPanelController::set_panel_always_on_top(self.panel, self.is_always_on_top);
            self.render_and_update();
        }
    }

    /// Updates physical position and converts to Cocoa bottom-left logical coordinates
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        let canvas_w = self.bubble_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = self.bubble_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
        let cocoa_rect = MacOSPanelController::top_left_physical_to_cocoa_rect(
            x,
            y,
            canvas_w,
            canvas_h,
            screen.y,
            screen.height,
            screen.scale_factor,
        );
        MacOSPanelController::set_panel_position(self.panel, cocoa_rect.origin.x, cocoa_rect.origin.y);
        if self.refresh_edge_side() && self.unread_count > 0 && !self.panel.is_null() {
            self.render_and_update();
        }
    }

    /// Begins dragging gesture
    pub fn start_drag(&mut self, mouse_x: i32, mouse_y: i32) {
        self.is_dragging = true;
        self.drag_start_mouse_x = mouse_x;
        self.drag_start_mouse_y = mouse_y;
        self.drag_start_bubble_x = self.x;
        self.drag_start_bubble_y = self.y;
        self.is_tucked = false;
        self.anim_state.stop();
    }

    /// Updates drag offset
    pub fn update_drag(&mut self, mouse_x: i32, mouse_y: i32) {
        if !self.is_dragging {
            return;
        }
        let dx = mouse_x - self.drag_start_mouse_x;
        let dy = mouse_y - self.drag_start_mouse_y;
        self.set_position(self.drag_start_bubble_x + dx, self.drag_start_bubble_y + dy);
    }

    /// Ends dragging gesture and calculates screen snap physics
    pub fn end_drag(&mut self, app: Option<&tauri::AppHandle>) -> (i32, i32, bool) {
        self.is_dragging = false;
        let screen = crate::movement::get_target_dashboard_screen(app);
        let config = match app {
            Some(a) => crate::movement::get_bubble_physics_config(a),
            None => crate::movement::BubblePhysicsConfig::default(),
        };
        let all_bubbles = match app {
            Some(a) => crate::presentation::bubble::get_active_bubble_rects(a, None),
            None => Vec::new(),
        };
        let snap = BubbleMovementEngine::calculate_snap(&screen, &config, PhysicalPosition { x: self.x, y: self.y }, self.account_index, &all_bubbles);
        self.anim_state.start(self.x, self.y, snap.target_x, snap.target_y, 180);
        (snap.target_x, snap.target_y, snap.should_dismiss)
    }

    /// Steps spring physics animation
    pub fn step_animation(&mut self) -> Option<(i32, i32, bool)> {
        if let Some((new_x, new_y, finished)) = self.anim_state.step() {
            self.set_position(new_x, new_y);
            Some((new_x, new_y, finished))
        } else {
            None
        }
    }

    /// Triggers spring animation to tuck into screen edge
    pub fn tuck(&mut self, app: &tauri::AppHandle) {
        crate::utils::trim_controller::schedule_auto_trim_on_tuck(self.account_index, app.clone());
        let config = crate::movement::get_bubble_physics_config(app);
        if config.is_free_position {
            return;
        }
        let screen = crate::movement::get_target_dashboard_screen(Some(app));
        let target_pos = BubbleMovementEngine::calculate_hover_pos(&screen, &config, self.x, self.y, false);
        if self.x != target_pos.x || self.y != target_pos.y {
            self.anim_state.start(self.x, self.y, target_pos.x, target_pos.y, 180);
        }
        self.is_tucked = true;
    }

    /// Triggers spring animation to untuck from screen edge and returns sibling animations
    pub fn untuck(&mut self, app: &tauri::AppHandle) -> Vec<(u32, i32, i32, u64)> {
        crate::utils::trim_controller::cancel_auto_trim(self.account_index);
        crate::utils::trim_controller::record_user_interaction(self.account_index);
        crate::platforms::messenger::windows::set_chat_window_memory_target(app, self.account_index, false);
        let config = crate::movement::get_bubble_physics_config(app);
        if config.is_free_position {
            return Vec::new();
        }
        let screen = crate::movement::get_target_dashboard_screen(Some(app));
        let target_pos = BubbleMovementEngine::calculate_hover_pos(&screen, &config, self.x, self.y, true);
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
        let resolved_y = slots.iter().find(|s| s.account_index == self.account_index).map(|s| s.target_y).unwrap_or(self.y);
        if self.x != target_pos.x || self.y != resolved_y {
            self.anim_state.start(self.x, self.y, target_pos.x, resolved_y, 180);
        }
        let mut sibling_moves = Vec::new();
        for slot in &slots {
            if slot.account_index != self.account_index {
                let sibling_x = all_bubbles.iter().find(|b| b.account_index == slot.account_index).map(|b| b.x).unwrap_or(target_pos.x);
                sibling_moves.push((slot.account_index, sibling_x, slot.target_y, 180));
            }
        }
        self.is_tucked = false;
        sibling_moves
    }

    /// Animates bubble to target coordinates
    pub fn animate_to(&mut self, target_x: i32, target_y: i32, duration_ms: u64) {
        self.anim_state.start(self.x, self.y, target_x, target_y, duration_ms);
    }

    /// Returns current bubble bounds
    pub fn get_bounds(&self) -> (i32, i32, u32, u32) {
        (self.x, self.y, self.bubble_size, self.bubble_size)
    }

    /// Resizes bubble canvas and updates Cocoa panel frame
    pub fn resize(&mut self, size: u32) {
        self.bubble_size = size;
        let canvas_w = size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
        let cocoa_rect = MacOSPanelController::top_left_physical_to_cocoa_rect(
            self.x,
            self.y,
            canvas_w,
            canvas_h,
            screen.y,
            screen.height,
            screen.scale_factor,
        );
        MacOSPanelController::set_panel_frame(self.panel, cocoa_rect);
        self.render_and_update();
    }

    /// Triggers bubble pulse animation
    pub fn trigger_pulse(&mut self) {}

    /// Handles mouse button press
    pub fn on_mouse_down(&mut self, mouse_x: i32, mouse_y: i32) {
        self.start_drag(mouse_x, mouse_y);
        self.has_dragged = false;
        crate::utils::trim_controller::record_user_interaction(self.account_index);
        if let Some(app) = &self.app_handle {
            crate::platforms::messenger::windows::set_chat_window_memory_target(app, self.account_index, false);
            crate::utils::trim_controller::cancel_auto_trim(self.account_index);
        }
    }

    /// Handles mouse motion during drag with bottom dismiss opacity fade
    pub fn on_mouse_move(&mut self, mouse_x: i32, mouse_y: i32) {
        if self.is_dragging {
            let dx = mouse_x - self.drag_start_mouse_x;
            let dy = mouse_y - self.drag_start_mouse_y;
            if dx * dx + dy * dy >= 16 {
                self.has_dragged = true;
            }
            if self.has_dragged {
                let new_x = self.drag_start_bubble_x + dx;
                let new_y = self.drag_start_bubble_y + dy;
                self.set_position(new_x, new_y);
                let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
                let bottom_edge = screen.y + screen.height;
                let dist = bottom_edge - new_y;
                let start_fade_dist = (340.0 * screen.scale_factor) as i32;
                let max_fade_dist = (140.0 * screen.scale_factor) as i32;
                let alpha_f64 = if dist >= start_fade_dist {
                    1.0
                } else if dist <= max_fade_dist {
                    0.25
                } else {
                    let ratio = (start_fade_dist - dist) as f64 / (start_fade_dist - max_fade_dist) as f64;
                    (1.0 - ratio * 0.75).clamp(0.25, 1.0)
                };
                MacOSPanelController::set_panel_alpha(self.panel, alpha_f64);
            }
        } else if !self.is_hovered {
            let _ = self.on_mouse_enter();
        }
    }

    /// Handles mouse button release for snap or click and returns sibling animations
    pub fn on_mouse_up(&mut self, _mouse_x: i32, _mouse_y: i32) -> Vec<(u32, i32, i32, u64)> {
        self.is_dragging = false;
        MacOSPanelController::set_panel_alpha(self.panel, 1.0);
        let app_opt = self.app_handle.clone();
        let mut sibling_moves = Vec::new();
        if self.has_dragged {
            self.is_hovered = false;
            let (target_x, target_y, should_dismiss) = self.end_drag(app_opt.as_ref());
            if should_dismiss {
                self.set_visible(false);
                if let Some(app) = app_opt {
                    let idx = self.account_index;
                    tauri::async_runtime::spawn(async move {
                        crate::commands::bubble::dismiss_bubble_from_drag(app, idx).await;
                    });
                }
            } else {
                self.anim_state.start(self.x, self.y, target_x, target_y, 180);
                if let Some(app) = &app_opt {
                    let all_bubbles = crate::presentation::bubble::get_active_bubble_rects(app, None);
                    let screen = crate::movement::get_target_dashboard_screen(Some(app));
                    let config = crate::movement::get_bubble_physics_config(app);
                    let mid_x = screen.x + screen.width / 2;
                    let is_left_edge = target_x + (self.bubble_size as i32 / 2) < mid_x;
                    let slots = crate::movement::collision::CollisionResolver::resolve_mutual_column_layout(
                        self.account_index,
                        target_y,
                        is_left_edge,
                        &screen,
                        &config,
                        &all_bubbles,
                    );
                    for slot in slots {
                        if slot.account_index != self.account_index {
                            sibling_moves.push((slot.account_index, target_x, slot.target_y, 180));
                        }
                    }
                }
            }
        } else if let Some(app) = app_opt {
            let idx = self.account_index;
            tauri::async_runtime::spawn(async move {
                if let Some(chat_state) = app.try_state::<std::sync::Arc<crate::state::ChatState>>() {
                    let _ = crate::platforms::messenger::services::ChatService::toggle_chat(&app, &chat_state, idx);
                }
            });
        }
        sibling_moves
    }

    /// Handles mouse cursor enter with pre-warm and untuck, returning sibling animations
    pub fn on_mouse_enter(&mut self) -> Vec<(u32, i32, i32, u64)> {
        if !self.is_hovered {
            self.is_hovered = true;
            self.render_and_update();
            crate::utils::trim_controller::record_user_interaction(self.account_index);
            let app_opt = self.app_handle.clone();
            if let Some(app) = app_opt {
                crate::platforms::messenger::windows::set_chat_window_memory_target(&app, self.account_index, false);
                crate::utils::trim_controller::cancel_auto_trim(self.account_index);
                return self.untuck(&app);
            }
        }
        Vec::new()
    }

    /// Handles mouse cursor leave
    pub fn on_mouse_leave(&mut self) {
        if self.is_hovered {
            self.is_hovered = false;
            self.render_and_update();
        }
    }
}

impl Drop for MacOSBubbleWindowState {
    fn drop(&mut self) {
        MacOSPanelController::destroy_panel(self.panel);
        self.panel = std::ptr::null_mut();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_bubble_window_state_creation() {
        let mut state = MacOSBubbleWindowState::new(1, 100, 150, 56);
        assert_eq!(state.account_index, 1);
        assert_eq!(state.x, 100);
        assert_eq!(state.y, 150);
        assert_eq!(state.bubble_size, 56);
        assert_eq!(state.unread_count, 0);
        state.set_unread_count(5);
        assert_eq!(state.unread_count, 5);
        state.set_position(200, 250);
        assert_eq!(state.x, 200);
        assert_eq!(state.y, 250);
    }

    #[test]
    fn test_drag_and_snap_lifecycle() {
        let mut state = MacOSBubbleWindowState::new(1, 100, 200, 56);
        assert!(!state.is_dragging);
        state.start_drag(120, 210);
        assert!(state.is_dragging);
        assert_eq!(state.drag_start_mouse_x, 120);
        assert_eq!(state.drag_start_mouse_y, 210);
        state.update_drag(150, 260);
        assert_eq!(state.x, 130);
        assert_eq!(state.y, 250);
        let (target_x, target_y, _) = state.end_drag(None);
        assert!(!state.is_dragging);
        assert!(state.anim_state.is_animating);
        assert_eq!(state.anim_state.target_x, target_x);
        assert_eq!(state.anim_state.target_y, target_y);
    }
}
