use std::ffi::c_void;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tauri::AppHandle;

use crate::operating_systems::macos::bubble::window::panel::MacOSPanelController;
use crate::presentation::bubble::dock::{
    compute_action_dock_layout, render_action_dock_with_tooltip, ActionDockLayout, DockActionId,
    DockActionStateSnapshot, DockAnimationState, DockRenderOptions, DockVisibilityPhase,
    ScreenRect,
};
use super::panel::{
    dock_physical_to_cocoa_rect, hit_test_macos_dock_local_point, is_macos_dock_point_hittable,
    MacOSActionDockPanelController,
};

/// Shared lock-protected Hover Bridge screen rect for O(1) cursor queries on macOS
pub type MacOSDockHoverCorridorCache = Arc<RwLock<Option<ScreenRect>>>;

/// Singleton macOS NSPanel state driving the Liquid Glass ActionDock
pub struct MacOSActionDockState {
    pub panel: *mut c_void,
    pub account_index: u32,
    pub layout: ActionDockLayout,
    pub snapshot: DockActionStateSnapshot,
    pub animator: DockAnimationState,
    pub last_tick: Instant,
    pub is_visible: bool,
    pub is_always_on_top: bool,
    pub app_handle: Option<AppHandle>,
    pub hover_corridor_cache: MacOSDockHoverCorridorCache,
}

unsafe impl Send for MacOSActionDockState {}
unsafe impl Sync for MacOSActionDockState {}

impl MacOSActionDockState {
    /// Creates a hidden singleton macOS ActionDock panel state
    pub fn new(is_always_on_top: bool, hover_corridor_cache: MacOSDockHoverCorridorCache) -> Self {
        let default_screen = crate::movement::ScreenMetrics { x: 0, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
        let init_layout = compute_action_dock_layout(12, 120, 56, &default_screen, 12.0, 60.0);
        let init_rect = dock_physical_to_cocoa_rect(&init_layout, default_screen.y, default_screen.height);
        let panel = MacOSActionDockPanelController::create_action_dock_panel(init_rect, is_always_on_top);
        Self {
            panel,
            account_index: 1,
            layout: init_layout,
            snapshot: DockActionStateSnapshot::default(),
            animator: DockAnimationState::default(),
            last_tick: Instant::now(),
            is_visible: false,
            is_always_on_top,
            app_handle: None,
            hover_corridor_cache,
        }
    }

    /// Publishes or clears the shared Hover Bridge screen rect based on visibility and phase
    pub fn sync_hover_corridor_cache(&self) {
        if let Ok(mut guard) = self.hover_corridor_cache.write() {
            *guard = if self.is_visible && self.animator.phase != DockVisibilityPhase::Hiding {
                Some(self.layout.hover_corridor)
            } else {
                None
            };
        }
    }

    /// Renders Liquid Glass droplets plus hover tooltip into BGRA and uploads via CGImage
    pub fn render_and_update(&mut self) {
        let frames = self.animator.sample_droplets(self.layout.direction);
        let tooltip_label = self.animator.hovered_action.map(|act| self.snapshot.tooltip_label(act));
        let opts = DockRenderOptions {
            layout: self.layout.clone(),
            state: self.snapshot,
            frames,
            hovered_action: self.animator.hovered_action,
            tooltip_text_width_px: None,
        };
        if let Some((mut bgra, pill_opt)) = render_action_dock_with_tooltip(&opts) {
            if let (Some(pill), Some(label)) = (pill_opt, tooltip_label) {
                MacOSActionDockPanelController::composite_tooltip_text_bgra(
                    &mut bgra,
                    self.layout.canvas_width,
                    self.layout.canvas_height,
                    &pill,
                    label,
                    self.layout.scale_factor,
                );
            }
            MacOSPanelController::update_panel_layer_contents(
                self.panel,
                &bgra,
                self.layout.canvas_width as usize,
                self.layout.canvas_height as usize,
                self.layout.scale_factor,
            );
        }
    }

    /// Shows or repositions ActionDock for an account bubble and starts staggered drop-in when needed
    pub fn show_for_bubble(
        &mut self,
        account_index: u32,
        layout: ActionDockLayout,
        snapshot: DockActionStateSnapshot,
        app: Option<AppHandle>,
    ) {
        let should_appear = !self.is_visible
            || self.account_index != account_index
            || self.animator.phase == DockVisibilityPhase::Hiding
            || self.animator.phase == DockVisibilityPhase::Hidden;
        self.account_index = account_index;
        self.layout = layout;
        self.snapshot = snapshot;
        self.app_handle = app;
        self.is_visible = true;
        if should_appear {
            self.animator.start_appear();
            self.last_tick = Instant::now();
        }
        let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
        let cocoa_rect = dock_physical_to_cocoa_rect(&self.layout, screen.y, screen.height);
        MacOSPanelController::set_panel_frame(self.panel, cocoa_rect);
        MacOSPanelController::set_panel_always_on_top(self.panel, self.is_always_on_top);
        self.sync_hover_corridor_cache();
        self.render_and_update();
        MacOSPanelController::set_panel_visible(self.panel, true);
    }

    /// Updates SSOT toggle snapshot and optional layout without restarting appear animation
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
            let screen = crate::movement::get_target_dashboard_screen(self.app_handle.as_ref());
            let cocoa_rect = dock_physical_to_cocoa_rect(&self.layout, screen.y, screen.height);
            MacOSPanelController::set_panel_frame(self.panel, cocoa_rect);
            self.sync_hover_corridor_cache();
        }
        self.render_and_update();
    }

    /// Hides ActionDock immediately or initiates fast 90ms fade retract
    pub fn hide(&mut self, immediate: bool) {
        if immediate || !self.is_visible {
            self.animator.hide_immediate();
            self.is_visible = false;
            self.sync_hover_corridor_cache();
            MacOSPanelController::set_panel_visible(self.panel, false);
        } else {
            self.animator.start_hide();
            self.last_tick = Instant::now();
            self.sync_hover_corridor_cache();
            self.render_and_update();
        }
    }

    /// Steps dock animation on the shared adaptive driver, returning None when idle for 0% CPU
    pub fn step_animation(&mut self) -> Option<u64> {
        if !self.animator.needs_frame() {
            return None;
        }
        let now = Instant::now();
        let dt_ms = now.duration_since(self.last_tick).as_secs_f32() * 1000.0;
        self.last_tick = now;
        let finished_hide = self.animator.step(dt_ms);
        if finished_hide {
            self.is_visible = false;
            self.sync_hover_corridor_cache();
            MacOSPanelController::set_panel_visible(self.panel, false);
            None
        } else {
            self.render_and_update();
            if self.animator.needs_frame() { Some(16) } else { None }
        }
    }

    /// Updates floating panel always-on-top level without stealing activation
    pub fn set_always_on_top(&mut self, enabled: bool) {
        self.is_always_on_top = enabled;
        MacOSPanelController::set_panel_always_on_top(self.panel, enabled);
    }

    /// Re-asserts ActionDock above all sibling bubbles when visible
    pub fn assert_topmost(&self) {
        if self.is_visible {
            MacOSPanelController::order_panel_front_regardless(self.panel);
        }
    }

    /// Evaluates whether a Cocoa local point hits a droplet (false makes gaps click-through)
    pub fn is_local_point_hittable(&self, local_x_pt: f64, local_y_pt: f64) -> bool {
        is_macos_dock_point_hittable(
            local_x_pt,
            local_y_pt,
            &self.layout,
            self.is_visible,
            self.animator.pressed_action.is_some(),
        )
    }

    /// Updates hovered droplet on Cocoa mouseMoved:/mouseDragged: and returns whether driver must wake
    pub fn on_mouse_move(&mut self, local_x_pt: f64, local_y_pt: f64) -> bool {
        let hit = hit_test_macos_dock_local_point(local_x_pt, local_y_pt, &self.layout, self.is_visible);
        if self.animator.set_hovered(hit) {
            self.last_tick = Instant::now();
            self.render_and_update();
            return self.animator.needs_frame();
        }
        false
    }

    /// Clears hover/press visual states on Cocoa mouseExited: and returns whether driver must wake
    pub fn on_mouse_leave(&mut self) -> bool {
        let changed_h = self.animator.set_hovered(None);
        let changed_p = self.animator.set_pressed(None);
        if changed_h || changed_p {
            self.last_tick = Instant::now();
            self.render_and_update();
            return self.animator.needs_frame();
        }
        false
    }

    /// Captures droplet press state on Cocoa mouseDown: and returns whether driver must wake
    pub fn on_mouse_down(&mut self, local_x_pt: f64, local_y_pt: f64) -> bool {
        let hit = hit_test_macos_dock_local_point(local_x_pt, local_y_pt, &self.layout, self.is_visible);
        if hit.is_some() {
            self.animator.set_hovered(hit);
            if self.animator.set_pressed(hit) {
                self.last_tick = Instant::now();
                self.render_and_update();
                return self.animator.needs_frame();
            }
        }
        false
    }

    /// Finalizes click on Cocoa mouseUp:, returning triggered action payload and animation wake flag
    pub fn on_mouse_up(
        &mut self,
        local_x_pt: f64,
        local_y_pt: f64,
    ) -> (Option<(Option<AppHandle>, u32, DockActionId)>, bool) {
        let pressed = self.animator.pressed_action;
        let hit = hit_test_macos_dock_local_point(local_x_pt, local_y_pt, &self.layout, self.is_visible);
        let triggered = if pressed.is_some() && pressed == hit { pressed } else { None };
        self.animator.set_pressed(None);
        self.animator.set_hovered(hit);
        self.last_tick = Instant::now();
        self.render_and_update();
        let payload = triggered.map(|action| (self.app_handle.clone(), self.account_index, action));
        (payload, self.animator.needs_frame())
    }
}

impl Drop for MacOSActionDockState {
    fn drop(&mut self) {
        self.is_visible = false;
        self.sync_hover_corridor_cache();
        MacOSPanelController::destroy_panel(self.panel);
        self.panel = std::ptr::null_mut();
    }
}
