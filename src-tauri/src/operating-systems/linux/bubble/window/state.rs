use std::ffi::c_void;
use super::gtk_window::LinuxGtkWindowController;
use super::types::*;
use tauri::PhysicalPosition;
use crate::movement::BubbleMovementEngine;
use crate::operating_systems::linux::bubble::animator::SpringAnimationState;
use crate::operating_systems::linux::bubble::renderer::{render_bubble_bgra, BubbleRenderOptions};

pub struct LinuxBubbleWindowState {
    pub account_index: u32,
    pub window: *mut c_void,
    pub x: i32,
    pub y: i32,
    pub bubble_size: u32,
    pub unread_count: u32,
    pub is_hovered: bool,
    pub is_left_edge: bool,
    pub is_visible: bool,
    pub is_dragging: bool,
    pub drag_start_mouse_x: i32,
    pub drag_start_mouse_y: i32,
    pub drag_start_bubble_x: i32,
    pub drag_start_bubble_y: i32,
    pub is_tucked: bool,
    pub anim_state: SpringAnimationState,
}

unsafe impl Send for LinuxBubbleWindowState {}
unsafe impl Sync for LinuxBubbleWindowState {}

impl LinuxBubbleWindowState {
    pub fn new(account_index: u32, x: i32, y: i32, bubble_size: u32) -> Self {
        let canvas_w = bubble_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = bubble_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        let window = LinuxGtkWindowController::create_floating_bubble_window(x as f64, y as f64, canvas_w as f64, canvas_h as f64);
        Self {
            account_index,
            window,
            x,
            y,
            bubble_size,
            unread_count: 0,
            is_hovered: false,
            is_left_edge: true,
            is_visible: false,
            is_dragging: false,
            drag_start_mouse_x: 0,
            drag_start_mouse_y: 0,
            drag_start_bubble_x: x,
            drag_start_bubble_y: y,
            is_tucked: false,
            anim_state: SpringAnimationState::default(),
        }
    }

    /// Renders tiny-skia pixel buffer and commits to Cairo/GTK surface
    pub fn render_and_update(&mut self) {
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
            LinuxGtkWindowController::update_window_surface(self.window, &bgra_data, canvas_w as usize, canvas_h as usize);
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
        LinuxGtkWindowController::set_window_visible(self.window, visible);
        if visible {
            self.render_and_update();
        }
    }

    /// Updates position
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
        LinuxGtkWindowController::set_window_position(self.window, x, y);
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
        self.anim_state.start(self.x, self.y, target_pos.x, target_pos.y, 180);
        self.is_tucked = true;
    }

    /// Triggers spring animation to untuck from screen edge
    pub fn untuck(&mut self, app: &tauri::AppHandle) {
        crate::utils::trim_controller::cancel_auto_trim(self.account_index);
        crate::utils::trim_controller::record_user_interaction(self.account_index);
        let config = crate::movement::get_bubble_physics_config(app);
        if config.is_free_position {
            return;
        }
        let screen = crate::movement::get_target_dashboard_screen(Some(app));
        let target_pos = BubbleMovementEngine::calculate_hover_pos(&screen, &config, self.x, self.y, true);
        self.anim_state.start(self.x, self.y, target_pos.x, target_pos.y, 180);
        self.is_tucked = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_bubble_window_state_creation() {
        let mut state = LinuxBubbleWindowState::new(1, 100, 150, 56);
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
        let mut state = LinuxBubbleWindowState::new(1, 100, 200, 56);
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
