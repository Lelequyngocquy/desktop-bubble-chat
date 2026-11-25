use std::ffi::c_void;
use std::sync::Arc;
use super::panel::MacOSPanelController;
use super::types::*;
use tauri::Manager;
use crate::movement::{BubbleDropPlan, BubbleInteractionFlags, BubbleMovementEngine, BubblePhysicsConfig, BubbleRect, ScreenMetrics, UntuckMode};
use crate::operating_systems::macos::bubble::animator::SpringAnimationState;
use crate::operating_systems::macos::bubble::renderer::{render_bubble_bgra, BubbleRenderOptions};
use crate::state::ChatState;

/// Evaluates drag-drop target coordinates for both Free Placement and Auto-Tuck modes
pub fn compute_bubble_drop_plan(
    account_index: u32,
    x: i32,
    y: i32,
    stay_untucked: bool,
    screen: &ScreenMetrics,
    config: &BubblePhysicsConfig,
    all_bubbles: &[BubbleRect],
) -> BubbleDropPlan {
    BubbleMovementEngine::compute_drop_plan(account_index, x, y, stay_untucked, screen, config, all_bubbles)
}

/// Evaluates whether a bubble should tuck after hover-leave debounce expires
pub fn should_tuck_after_hover_leave(
    is_dragging: bool,
    is_hovered: bool,
    is_chat_open: bool,
    is_chat_pinned: bool,
    is_free_position: bool,
    is_alert_active: bool,
) -> bool {
    BubbleMovementEngine::can_auto_tuck(BubbleInteractionFlags {
        is_hovered,
        is_dragging,
        is_chat_open,
        is_chat_pinned,
        is_free_position,
        is_alert_active,
    })
}

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
        let cocoa_rect = MacOSPanelController::top_left_physical_to_cocoa_rect(x, y, canvas_w, canvas_h, screen.y, screen.height, screen.scale_factor);
        let panel = if cfg!(test) {
            std::ptr::null_mut()
        } else {
            MacOSPanelController::create_floating_bubble_panel(account_index, cocoa_rect.origin.x, cocoa_rect.origin.y, cocoa_rect.size.width, cocoa_rect.size.height)
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

    /// Re-asserts floating level and orders panel front without stealing focus
    pub fn assert_topmost(&mut self) {
        MacOSPanelController::set_panel_always_on_top(self.panel, true);
        MacOSPanelController::order_panel_front_regardless(self.panel);
    }

    /// Renders tiny-skia pixel buffer, updates hit metadata, and commits to CALayer
    pub fn render_and_update(&mut self) {
        self.refresh_edge_side();
        let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
        let scale = screen.scale_factor.max(1.0);
        let canvas_w = self.bubble_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = self.bubble_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        MacOSPanelController::update_panel_hit_metadata(self.panel, (self.bubble_size as f64) / scale, self.unread_count, self.is_left_edge);
        let opts = BubbleRenderOptions {
            canvas_width: canvas_w,
            canvas_height: canvas_h,
            bubble_size: self.bubble_size,
            account_index: self.account_index,
            is_hovered: self.is_hovered,
            is_left_edge: self.is_left_edge,
            unread_count: self.unread_count,
            ..Default::default()
        };
        if let Some(bgra_data) = render_bubble_bgra(&opts) {
            MacOSPanelController::update_panel_layer_contents(self.panel, &bgra_data, canvas_w as usize, canvas_h as usize, scale);
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
        let cocoa_rect = MacOSPanelController::top_left_physical_to_cocoa_rect(x, y, canvas_w, canvas_h, screen.y, screen.height, screen.scale_factor);
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

    /// Ends dragging gesture and computes Free Placement or Mutual Edge Snap plan
    pub fn end_drag(&mut self, app: Option<&tauri::AppHandle>) -> BubbleDropPlan {
        self.is_dragging = false;
        let screen = crate::movement::get_target_dashboard_screen(app);
        let config = app.map(crate::movement::get_bubble_physics_config).unwrap_or_default();
        let all_bubbles = app.map(|a| crate::presentation::bubble::get_active_bubble_rects(a, None)).unwrap_or_default();
        let stay_untucked = app.map(|a| crate::presentation::bubble::is_bubble_stay_untucked(a, self.account_index)).unwrap_or(false);
        let plan = compute_bubble_drop_plan(self.account_index, self.x, self.y, stay_untucked, &screen, &config, &all_bubbles);
        if !plan.should_dismiss {
            self.anim_state.start(self.x, self.y, plan.target_x, plan.target_y, 160);
        }
        plan
    }

    /// Resets drag state if mouse capture is interrupted unexpectedly
    pub fn cancel_drag_if_interrupted(&mut self) -> Vec<(u32, i32, i32, u64)> {
        if !self.is_dragging {
            return Vec::new();
        }
        self.is_dragging = false;
        self.is_hovered = false;
        MacOSPanelController::set_panel_alpha(self.panel, 1.0);
        if self.has_dragged {
            self.finalize_drop_interaction()
        } else {
            Vec::new()
        }
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

    /// Collects current interaction and alert flags for tuck eligibility
    pub fn interaction_flags(&self, app_opt: Option<&tauri::AppHandle>) -> BubbleInteractionFlags {
        let (is_chat_open, is_chat_pinned, is_free_position, is_alert_active) = if let Some(app) = app_opt {
            let chat_open = crate::platforms::messenger::windows::get_messenger_webview(app, self.account_index)
                .and_then(|w| w.is_visible().ok())
                .unwrap_or(false);
            let (pinned, alert) = app
                .try_state::<Arc<ChatState>>()
                .map(|cs| (cs.is_chat_pinned(self.account_index), cs.is_account_alert_active(self.account_index)))
                .unwrap_or((false, false));
            let free = crate::movement::get_bubble_physics_config(app).is_free_position;
            (chat_open, pinned, free, alert)
        } else {
            (false, false, false, false)
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

    /// Checks whether bubble can safely tuck after hover-leave debounce
    pub fn can_tuck_on_hover_leave(&self) -> bool {
        let flags = self.interaction_flags(self.app_handle.as_ref());
        should_tuck_after_hover_leave(flags.is_dragging, flags.is_hovered, flags.is_chat_open, flags.is_chat_pinned, flags.is_free_position, flags.is_alert_active)
    }

    /// Triggers spring animation to tuck into screen edge and returns displaced siblings
    pub fn tuck(&mut self, app: &tauri::AppHandle) -> Vec<(u32, i32, i32, u64)> {
        let flags = self.interaction_flags(Some(app));
        if !flags.is_dragging && !flags.is_hovered && !flags.is_chat_open && !flags.is_chat_pinned {
            crate::utils::trim_controller::schedule_auto_trim_on_tuck(self.account_index, app.clone());
        }
        if !BubbleMovementEngine::can_auto_tuck(flags) {
            return Vec::new();
        }
        let config = crate::movement::get_bubble_physics_config(app);
        let screen = crate::movement::get_target_dashboard_screen(Some(app));
        let all_bubbles = crate::presentation::bubble::get_active_bubble_rects(app, None);
        let Some(plan) = BubbleMovementEngine::compute_edge_transition_plan(self.account_index, self.x, self.y, false, &screen, &config, &all_bubbles) else {
            return Vec::new();
        };
        if self.x != plan.target_x || self.y != plan.target_y {
            self.anim_state.start(self.x, self.y, plan.target_x, plan.target_y, 180);
        }
        self.is_tucked = true;
        plan.displaced_siblings.into_iter().map(|(sib_idx, sx, sy)| (sib_idx, sx, sy, 180)).collect()
    }

    /// Triggers spring animation to untuck from screen edge with explicit RAM-trim mode
    pub fn untuck_with_mode(&mut self, app: &tauri::AppHandle, mode: UntuckMode) -> Vec<(u32, i32, i32, u64)> {
        crate::utils::trim_controller::AutoTrimController::global().handle_untuck_mode(self.account_index, mode);
        if mode == UntuckMode::UserInteraction {
            crate::platforms::messenger::windows::set_chat_window_memory_target(app, self.account_index, false);
        }
        let config = crate::movement::get_bubble_physics_config(app);
        let screen = crate::movement::get_target_dashboard_screen(Some(app));
        let all_bubbles = crate::presentation::bubble::get_active_bubble_rects(app, None);
        let Some(plan) = BubbleMovementEngine::compute_edge_transition_plan(self.account_index, self.x, self.y, true, &screen, &config, &all_bubbles) else {
            return Vec::new();
        };
        if self.x != plan.target_x || self.y != plan.target_y {
            self.anim_state.start(self.x, self.y, plan.target_x, plan.target_y, 180);
        }
        self.is_tucked = false;
        plan.displaced_siblings.into_iter().map(|(sib_idx, sx, sy)| (sib_idx, sx, sy, 180)).collect()
    }

    /// Triggers spring animation to untuck from screen edge on user interaction
    pub fn untuck(&mut self, app: &tauri::AppHandle) -> Vec<(u32, i32, i32, u64)> {
        self.untuck_with_mode(app, UntuckMode::UserInteraction)
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
        let cocoa_rect = MacOSPanelController::top_left_physical_to_cocoa_rect(self.x, self.y, canvas_w, canvas_h, screen.y, screen.height, screen.scale_factor);
        MacOSPanelController::set_panel_frame(self.panel, cocoa_rect);
        self.render_and_update();
    }

    /// Triggers bubble pulse animation - no-op matching Windows
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

    fn finalize_drop_interaction(&mut self) -> Vec<(u32, i32, i32, u64)> {
        let app_opt = self.app_handle.clone();
        let plan = self.end_drag(app_opt.as_ref());
        if plan.should_dismiss {
            self.set_visible(false);
            if let Some(app) = app_opt {
                let idx = self.account_index;
                tauri::async_runtime::spawn(async move {
                    crate::commands::bubble::dismiss_bubble_from_drag(app, idx).await;
                });
            }
            return Vec::new();
        }
        let bubble_w = self.bubble_size as i32;
        if let Some(app) = &app_opt
            && let Some(cs) = app.try_state::<Arc<ChatState>>() {
                cs.set_bubble_rect(self.account_index, BubbleRect { account_index: self.account_index, x: plan.target_x, y: plan.target_y, width: bubble_w, height: bubble_w });
                for &(sib_idx, sx, sy) in &plan.displaced_siblings {
                    cs.set_bubble_rect(sib_idx, BubbleRect { account_index: sib_idx, x: sx, y: sy, width: bubble_w, height: bubble_w });
                }
            }
        plan.displaced_siblings.into_iter().map(|(sib_idx, sx, sy)| (sib_idx, sx, sy, 160)).collect()
    }

    /// Handles mouse button release for snap, free drop, or click
    pub fn on_mouse_up(&mut self, _mouse_x: i32, _mouse_y: i32) -> Vec<(u32, i32, i32, u64)> {
        self.is_dragging = false;
        MacOSPanelController::set_panel_alpha(self.panel, 1.0);
        let app_opt = self.app_handle.clone();
        if self.has_dragged {
            self.is_hovered = false;
            return self.finalize_drop_interaction();
        }
        if let Some(app) = app_opt {
            let idx = self.account_index;
            tauri::async_runtime::spawn(async move {
                if let Some(chat_state) = app.try_state::<Arc<ChatState>>() {
                    let _ = crate::platforms::messenger::services::ChatService::toggle_chat(&app, &chat_state, idx);
                }
            });
        }
        Vec::new()
    }

    /// Handles mouse cursor enter with pre-warm and untuck, returning sibling animations
    pub fn on_mouse_enter(&mut self) -> Vec<(u32, i32, i32, u64)> {
        if !self.is_hovered {
            self.is_hovered = true;
            self.render_and_update();
            crate::utils::trim_controller::record_user_interaction(self.account_index);
            if let Some(app) = self.app_handle.clone() {
                crate::platforms::messenger::windows::set_chat_window_memory_target(&app, self.account_index, false);
                crate::utils::trim_controller::cancel_auto_trim(self.account_index);
                return self.untuck(&app);
            }
        }
        Vec::new()
    }

    /// Handles mouse cursor leave while ignoring leave events during active drag
    pub fn on_mouse_leave(&mut self) -> bool {
        if !self.is_dragging && self.is_hovered {
            self.is_hovered = false;
            self.render_and_update();
            return true;
        }
        false
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
    #[cfg(target_os = "macos")]
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
    #[cfg(target_os = "macos")]
    fn test_drag_and_snap_lifecycle() {
        let mut state = MacOSBubbleWindowState::new(1, 100, 200, 56);
        assert!(!state.is_dragging);
        state.start_drag(120, 210);
        assert!(state.is_dragging);
        state.update_drag(150, 260);
        assert_eq!(state.x, 130);
        assert_eq!(state.y, 250);
        let plan = state.end_drag(None);
        assert!(!state.is_dragging);
        assert!(state.anim_state.is_animating);
        assert_eq!(state.anim_state.target_x, plan.target_x);
        assert_eq!(state.anim_state.target_y, plan.target_y);
    }

    #[test]
    fn test_drop_plan_respects_free_position_mode() {
        let screen = ScreenMetrics { x: 0, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
        let free_cfg = BubblePhysicsConfig { is_free_position: true, ..Default::default() };
        let plan = compute_bubble_drop_plan(1, 640, 420, false, &screen, &free_cfg, &[]);
        assert!(!plan.should_dismiss);
        assert_eq!(plan.target_x, 640);
        assert_eq!(plan.target_y, 420);
        assert!(plan.displaced_siblings.is_empty());
    }

    #[test]
    fn test_hover_leave_tuck_guard_matches_windows() {
        assert!(should_tuck_after_hover_leave(false, false, false, false, false, false));
        assert!(!should_tuck_after_hover_leave(true, false, false, false, false, false));
        assert!(!should_tuck_after_hover_leave(false, true, false, false, false, false));
        assert!(!should_tuck_after_hover_leave(false, false, true, false, false, false));
        assert!(!should_tuck_after_hover_leave(false, false, false, true, false, false));
        assert!(!should_tuck_after_hover_leave(false, false, false, false, true, false));
        assert!(!should_tuck_after_hover_leave(false, false, false, false, false, true));
    }
}
