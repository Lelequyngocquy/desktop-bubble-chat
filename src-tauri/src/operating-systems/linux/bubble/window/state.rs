use std::ffi::c_void;
use super::gtk_window::LinuxGtkWindowController;
use super::types::*;
use tauri::{Manager, PhysicalPosition};
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
    pub has_dragged: bool,
    pub drag_start_mouse_x: i32,
    pub drag_start_mouse_y: i32,
    pub drag_start_bubble_x: i32,
    pub drag_start_bubble_y: i32,
    pub is_tucked: bool,
    pub anim_state: SpringAnimationState,
    pub spring_timer_id: u32,
    pub hover_leave_timer_id: u32,
    pub app_handle: Option<tauri::AppHandle>,
}

unsafe impl Send for LinuxBubbleWindowState {}
unsafe impl Sync for LinuxBubbleWindowState {}

impl LinuxBubbleWindowState {
    pub fn new(account_index: u32, x: i32, y: i32, bubble_size: u32) -> Self {
        let canvas_w = bubble_size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = bubble_size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        let window = if cfg!(test) {
            std::ptr::null_mut()
        } else {
            LinuxGtkWindowController::create_floating_bubble_window(account_index, x as f64, y as f64, canvas_w as f64, canvas_h as f64)
        };
        let mut inst = Self {
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
            has_dragged: false,
            drag_start_mouse_x: 0,
            drag_start_mouse_y: 0,
            drag_start_bubble_x: x,
            drag_start_bubble_y: y,
            is_tucked: false,
            anim_state: SpringAnimationState::default(),
            spring_timer_id: 0,
            hover_leave_timer_id: 0,
            app_handle: None,
        };
        if !inst.window.is_null() {
            inst.render_and_update();
        }
        inst
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

    /// Starts GLib 16ms spring animation timer
    pub fn start_spring_timer(&mut self) {
        self.stop_spring_timer();
        #[cfg(target_os = "linux")]
        {
            self.spring_timer_id = unsafe {
                crate::operating_systems::linux::bubble::window::ffi::g_timeout_add(
                    16,
                    Some(crate::operating_systems::linux::bubble::manager::spring_timer_callback),
                    self.account_index as usize as *mut c_void,
                )
            };
        }
    }

    /// Stops active spring animation timer
    pub fn stop_spring_timer(&mut self) {
        if self.spring_timer_id != 0 {
            #[cfg(target_os = "linux")]
            unsafe {
                crate::operating_systems::linux::bubble::window::ffi::g_source_remove(self.spring_timer_id);
            }
            self.spring_timer_id = 0;
        }
    }

    /// Starts GLib 160ms hover leave debounce timer
    pub fn start_hover_leave_timer(&mut self) {
        self.stop_hover_leave_timer();
        #[cfg(target_os = "linux")]
        {
            self.hover_leave_timer_id = unsafe {
                crate::operating_systems::linux::bubble::window::ffi::g_timeout_add(
                    160,
                    Some(crate::operating_systems::linux::bubble::manager::hover_leave_timer_callback),
                    self.account_index as usize as *mut c_void,
                )
            };
        }
    }

    /// Stops active hover leave debounce timer
    pub fn stop_hover_leave_timer(&mut self) {
        if self.hover_leave_timer_id != 0 {
            #[cfg(target_os = "linux")]
            unsafe {
                crate::operating_systems::linux::bubble::window::ffi::g_source_remove(self.hover_leave_timer_id);
            }
            self.hover_leave_timer_id = 0;
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

    /// Triggers spring animation to tuck into screen edge (180ms ease-out)
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
            self.start_spring_timer();
        }
        self.is_tucked = true;
    }

    /// Triggers spring animation to untuck from screen edge and cascades siblings
    pub fn untuck(&mut self, app: &tauri::AppHandle) {
        crate::utils::trim_controller::cancel_auto_trim(self.account_index);
        crate::utils::trim_controller::record_user_interaction(self.account_index);
        crate::platforms::messenger::windows::set_chat_window_memory_target(app, self.account_index, false);
        let config = crate::movement::get_bubble_physics_config(app);
        if config.is_free_position {
            return;
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
            self.start_spring_timer();
        }
        for slot in &slots {
            if slot.account_index != self.account_index {
                let sibling_x = all_bubbles.iter().find(|b| b.account_index == slot.account_index).map(|b| b.x).unwrap_or(target_pos.x);
                crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().dispatch(
                    crate::operating_systems::linux::bubble::manager::NativeBubbleAction::AnimateTo {
                        account_index: slot.account_index,
                        target_x: sibling_x,
                        target_y: slot.target_y,
                        duration_ms: 180,
                    },
                );
            }
        }
        self.is_tucked = false;
    }

    /// Animates bubble to target coordinates
    pub fn animate_to(&mut self, target_x: i32, target_y: i32, duration_ms: u64) {
        self.anim_state.start(self.x, self.y, target_x, target_y, duration_ms);
        self.start_spring_timer();
    }

    /// Returns current bubble bounds
    pub fn get_bounds(&self) -> (i32, i32, u32, u32) {
        (self.x, self.y, self.bubble_size, self.bubble_size)
    }

    /// Resizes bubble canvas
    pub fn resize(&mut self, size: u32) {
        self.bubble_size = size;
        let canvas_w = size + 2 * (BUBBLE_CANVAS_PAD_X as u32);
        let canvas_h = size + 2 * (BUBBLE_CANVAS_PAD_Y as u32);
        LinuxGtkWindowController::resize_window(self.window, canvas_w as i32, canvas_h as i32);
        self.render_and_update();
    }

    /// Triggers bubble pulse animation
    pub fn trigger_pulse(&mut self) {}

    /// Handles mouse button press
    pub fn on_mouse_down(&mut self, mouse_x: i32, mouse_y: i32) {
        self.stop_spring_timer();
        self.stop_hover_leave_timer();
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
                LinuxGtkWindowController::set_window_opacity(self.window, alpha_f64);
            }
        } else if !self.is_hovered {
            self.on_mouse_enter();
        }
    }

    /// Handles mouse button release for snap or click
    pub fn on_mouse_up(&mut self, _mouse_x: i32, _mouse_y: i32) {
        self.is_dragging = false;
        LinuxGtkWindowController::set_window_opacity(self.window, 1.0);
        let app_opt = self.app_handle.clone();
        if self.has_dragged {
            let (target_x, target_y, should_dismiss) = self.end_drag(app_opt.as_ref());
            if should_dismiss {
                self.set_visible(false);
                if let Some(app) = app_opt {
                    let idx = self.account_index;
                    tauri::async_runtime::spawn(async move {
                        let _ = crate::commands::bubble::toggle_bubble(app, false, Some(idx)).await;
                    });
                }
            } else {
                self.anim_state.start(self.x, self.y, target_x, target_y, 180);
                self.start_spring_timer();
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
                            crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().dispatch(
                                crate::operating_systems::linux::bubble::manager::NativeBubbleAction::AnimateTo {
                                    account_index: slot.account_index,
                                    target_x,
                                    target_y: slot.target_y,
                                    duration_ms: 180,
                                },
                            );
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
    }

    /// Handles mouse cursor enter with pre-warm and untuck
    pub fn on_mouse_enter(&mut self) {
        self.stop_hover_leave_timer();
        if !self.is_hovered {
            self.is_hovered = true;
            self.render_and_update();
            crate::utils::trim_controller::record_user_interaction(self.account_index);
            let app_opt = self.app_handle.clone();
            if let Some(app) = app_opt {
                crate::platforms::messenger::windows::set_chat_window_memory_target(&app, self.account_index, false);
                crate::utils::trim_controller::cancel_auto_trim(self.account_index);
                self.untuck(&app);
            }
        }
    }

    /// Handles mouse cursor leave by starting debounce timer
    pub fn on_mouse_leave(&mut self) {
        self.start_hover_leave_timer();
    }
}

impl Drop for LinuxBubbleWindowState {
    fn drop(&mut self) {
        self.stop_spring_timer();
        self.stop_hover_leave_timer();
        LinuxGtkWindowController::destroy_window(self.window);
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
