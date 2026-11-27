use std::ffi::c_void;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tauri::AppHandle;

use crate::operating_systems::linux::bubble::window::gtk_window::LinuxGtkWindowController;
use crate::presentation::bubble::dock::{
    compute_action_dock_layout, render_action_dock_with_tooltip, ActionDockLayout, DockActionId,
    DockActionStateSnapshot, DockAnimationState, DockRenderOptions, DockVisibilityPhase,
    ScreenRect,
};
use super::gtk_window::{hit_test_linux_dock_local_logical, LinuxActionDockGtkController};

/// Shared lock-protected Hover Bridge screen rect for O(1) cursor queries on Linux
pub type LinuxDockHoverCorridorCache = Arc<RwLock<Option<ScreenRect>>>;

/// Singleton Linux GtkWindow state driving the Liquid Glass ActionDock
pub struct LinuxActionDockState {
    pub window: *mut c_void,
    pub account_index: u32,
    pub layout: ActionDockLayout,
    pub snapshot: DockActionStateSnapshot,
    pub animator: DockAnimationState,
    pub last_tick: Instant,
    pub anim_timer_id: u32,
    pub is_visible: bool,
    pub is_always_on_top: bool,
    pub app_handle: Option<AppHandle>,
    pub hover_corridor_cache: LinuxDockHoverCorridorCache,
}

unsafe impl Send for LinuxActionDockState {}
unsafe impl Sync for LinuxActionDockState {}

impl LinuxActionDockState {
    /// Creates a hidden singleton Linux ActionDock GTK window state
    pub fn new(is_always_on_top: bool, hover_corridor_cache: LinuxDockHoverCorridorCache) -> Self {
        let default_screen = crate::movement::ScreenMetrics { x: 0, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
        let init_layout = compute_action_dock_layout(12, 120, 56, &default_screen, 12.0, 60.0);
        let window = LinuxActionDockGtkController::create_action_dock_window(&init_layout, is_always_on_top);
        Self {
            window,
            account_index: 1,
            layout: init_layout,
            snapshot: DockActionStateSnapshot::default(),
            animator: DockAnimationState::default(),
            last_tick: Instant::now(),
            anim_timer_id: 0,
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

    /// Stops GLib 16ms animation timer immediately
    pub fn stop_anim_timer(&mut self) {
        if self.anim_timer_id != 0 {
            #[cfg(target_os = "linux")]
            unsafe { crate::operating_systems::linux::bubble::window::ffi::g_source_remove(self.anim_timer_id); }
            self.anim_timer_id = 0;
        }
    }

    /// Starts or disarms self-stopping GLib 16ms timer to guarantee 0% CPU when idle
    pub fn sync_anim_timer(&mut self) {
        if self.animator.needs_frame() {
            if self.anim_timer_id == 0 {
                self.last_tick = Instant::now();
                #[cfg(target_os = "linux")]
                {
                    self.anim_timer_id = unsafe {
                        crate::operating_systems::linux::bubble::window::ffi::g_timeout_add(
                            16,
                            Some(crate::operating_systems::linux::bubble::manager::dock_anim_timer_callback),
                            std::ptr::null_mut(),
                        )
                    };
                }
            }
        } else {
            self.stop_anim_timer();
        }
    }

    /// Renders Liquid Glass droplets plus active hover tooltip into BGRA and pushes to Cairo cache
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
        if let Some((bgra, pill_opt)) = render_action_dock_with_tooltip(&opts) {
            let tooltip_overlay = pill_opt.zip(tooltip_label).map(|(p, lbl)| (lbl.to_string(), p));
            LinuxActionDockGtkController::update_dock_surface(
                self.window,
                &bgra,
                self.layout.canvas_width as usize,
                self.layout.canvas_height as usize,
                self.layout.scale_factor,
                tooltip_overlay,
            );
        }
    }

    /// Shows or repositions ActionDock for an account bubble with 6-circle input mask
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
        LinuxActionDockGtkController::reposition_and_resize_dock(self.window, &self.layout);
        LinuxActionDockGtkController::update_dock_input_shape(self.window, &self.layout, true);
        LinuxGtkWindowController::set_window_keep_above(self.window, self.is_always_on_top);
        self.sync_hover_corridor_cache();
        self.render_and_update();
        LinuxGtkWindowController::set_window_visible(self.window, true);
        if self.is_always_on_top {
            LinuxGtkWindowController::raise_window(self.window);
        }
        self.sync_anim_timer();
    }

    /// Syncs SSOT toggle snapshot and optional layout without restarting appear animation
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
            LinuxActionDockGtkController::reposition_and_resize_dock(self.window, &self.layout);
            LinuxActionDockGtkController::update_dock_input_shape(self.window, &self.layout, true);
            self.sync_hover_corridor_cache();
        }
        self.render_and_update();
        self.sync_anim_timer();
    }

    /// Hides ActionDock immediately or initiates fast 90ms fade retract
    pub fn hide(&mut self, immediate: bool) {
        if immediate || !self.is_visible {
            self.animator.hide_immediate();
            self.is_visible = false;
            self.sync_hover_corridor_cache();
            self.stop_anim_timer();
            LinuxActionDockGtkController::update_dock_input_shape(self.window, &self.layout, false);
            LinuxGtkWindowController::set_window_visible(self.window, false);
        } else {
            self.animator.start_hide();
            self.sync_hover_corridor_cache();
            self.render_and_update();
            self.sync_anim_timer();
        }
    }

    /// Advances animation on 16ms GLib tick and returns false to auto-remove timer when idle
    pub fn on_anim_tick(&mut self) -> bool {
        let now = Instant::now();
        let dt_ms = now.duration_since(self.last_tick).as_secs_f32() * 1000.0;
        self.last_tick = now;
        let finished_hide = self.animator.step(dt_ms);
        if finished_hide {
            self.is_visible = false;
            self.anim_timer_id = 0;
            self.sync_hover_corridor_cache();
            LinuxActionDockGtkController::update_dock_input_shape(self.window, &self.layout, false);
            LinuxGtkWindowController::set_window_visible(self.window, false);
            false
        } else {
            self.render_and_update();
            if self.animator.needs_frame() {
                true
            } else {
                self.anim_timer_id = 0;
                false
            }
        }
    }

    /// Updates GTK keep-above hint without stealing focus
    pub fn set_always_on_top(&mut self, enabled: bool) {
        self.is_always_on_top = enabled;
        LinuxGtkWindowController::set_window_keep_above(self.window, enabled);
    }

    /// Raises ActionDock above sibling bubbles when visible
    pub fn assert_topmost(&self) {
        if self.is_visible {
            LinuxGtkWindowController::raise_window(self.window);
        }
    }

    /// Updates hovered droplet on GTK motion-notify-event
    pub fn on_mouse_move(&mut self, local_x: f64, local_y: f64) {
        let hit = hit_test_linux_dock_local_logical(local_x, local_y, &self.layout, self.is_visible);
        if self.animator.set_hovered(hit) {
            self.render_and_update();
            self.sync_anim_timer();
        }
    }

    /// Clears hover/press visual states on GTK leave-notify-event
    pub fn on_mouse_leave(&mut self) {
        let changed_h = self.animator.set_hovered(None);
        let changed_p = self.animator.set_pressed(None);
        if changed_h || changed_p {
            self.render_and_update();
            self.sync_anim_timer();
        }
    }

    /// Captures droplet press state on GTK button-press-event
    pub fn on_mouse_down(&mut self, local_x: f64, local_y: f64) {
        let hit = hit_test_linux_dock_local_logical(local_x, local_y, &self.layout, self.is_visible);
        if hit.is_some() {
            self.animator.set_hovered(hit);
            if self.animator.set_pressed(hit) {
                self.render_and_update();
                self.sync_anim_timer();
            }
        }
    }

    /// Finalizes click on GTK button-release-event and returns triggered SSOT action payload
    pub fn on_mouse_up(&mut self, local_x: f64, local_y: f64) -> Option<(Option<AppHandle>, u32, DockActionId)> {
        let pressed = self.animator.pressed_action;
        let hit = hit_test_linux_dock_local_logical(local_x, local_y, &self.layout, self.is_visible);
        let triggered = if pressed.is_some() && pressed == hit { pressed } else { None };
        self.animator.set_pressed(None);
        self.animator.set_hovered(hit);
        self.render_and_update();
        self.sync_anim_timer();
        triggered.map(|action| (self.app_handle.clone(), self.account_index, action))
    }
}

impl Drop for LinuxActionDockState {
    fn drop(&mut self) {
        self.stop_anim_timer();
        self.is_visible = false;
        self.sync_hover_corridor_cache();
        LinuxActionDockGtkController::destroy_dock(self.window);
        self.window = std::ptr::null_mut();
    }
}
