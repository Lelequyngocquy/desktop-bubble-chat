use super::model::{DockActionId, DockDirection, DOCK_ACTIONS, DOCK_ACTION_COUNT};

pub const DOCK_APPEAR_MS: f32 = 160.0;
pub const DOCK_STAGGER_MS: f32 = 16.0;
pub const DOCK_HIDE_MS: f32 = 90.0;
pub const DOCK_INTERACT_MS: f32 = 80.0;
pub const DOCK_HOVER_SCALE: f32 = 1.08;
pub const DOCK_PRESS_SCALE: f32 = 0.93;

/// Lifecycle phase of the ActionDock overlay animation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockVisibilityPhase {
    Hidden,
    Appearing,
    IdleVisible,
    Hiding,
}

/// Per-droplet interpolated frame parameters passed to the Skia renderer
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropletFrameState {
    pub alpha: f32,
    pub scale: f32,
    pub offset_y_norm: f32,
    pub hover_t: f32,
    pub specular_boost: f32,
}

impl Default for DropletFrameState {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            scale: 1.0,
            offset_y_norm: 0.0,
            hover_t: 0.0,
            specular_boost: 0.0,
        }
    }
}

/// Pure animation controller driving staggered drop-in, hover/press bounce, and zero-CPU idle sleep
#[derive(Debug, Clone)]
pub struct DockAnimationState {
    pub phase: DockVisibilityPhase,
    pub phase_elapsed_ms: f32,
    pub hovered_action: Option<DockActionId>,
    pub pressed_action: Option<DockActionId>,
    hover_t: [f32; DOCK_ACTION_COUNT],
    press_t: [f32; DOCK_ACTION_COUNT],
}

impl Default for DockAnimationState {
    fn default() -> Self {
        Self {
            phase: DockVisibilityPhase::Hidden,
            phase_elapsed_ms: 0.0,
            hovered_action: None,
            pressed_action: None,
            hover_t: [0.0; DOCK_ACTION_COUNT],
            press_t: [0.0; DOCK_ACTION_COUNT],
        }
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let u = 1.0 - t.clamp(0.0, 1.0);
    1.0 - u * u * u
}

fn approach(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else if target > current {
        current + max_delta
    } else {
        current - max_delta
    }
}

impl DockAnimationState {
    /// Starts staggered appear animation from bubble anchor
    pub fn start_appear(&mut self) {
        self.phase = DockVisibilityPhase::Appearing;
        self.phase_elapsed_ms = 0.0;
        self.hovered_action = None;
        self.pressed_action = None;
        self.hover_t = [0.0; DOCK_ACTION_COUNT];
        self.press_t = [0.0; DOCK_ACTION_COUNT];
    }

    /// Starts fast retract fade-out animation
    pub fn start_hide(&mut self) {
        if self.phase == DockVisibilityPhase::Hidden || self.phase == DockVisibilityPhase::Hiding {
            return;
        }
        self.phase = DockVisibilityPhase::Hiding;
        self.phase_elapsed_ms = 0.0;
        self.hovered_action = None;
        self.pressed_action = None;
    }

    /// Resets immediately to hidden with zero active animation
    pub fn hide_immediate(&mut self) {
        self.phase = DockVisibilityPhase::Hidden;
        self.phase_elapsed_ms = 0.0;
        self.hovered_action = None;
        self.pressed_action = None;
        self.hover_t = [0.0; DOCK_ACTION_COUNT];
        self.press_t = [0.0; DOCK_ACTION_COUNT];
    }

    /// Updates hovered droplet target and reports whether a redraw/animation is required
    pub fn set_hovered(&mut self, action: Option<DockActionId>) -> bool {
        if self.hovered_action == action {
            return false;
        }
        self.hovered_action = action;
        true
    }

    /// Updates pressed droplet target and reports whether a redraw/animation is required
    pub fn set_pressed(&mut self, action: Option<DockActionId>) -> bool {
        if self.pressed_action == action {
            return false;
        }
        self.pressed_action = action;
        true
    }

    /// Returns true while any visibility, hover, or press animation is still progressing
    pub fn needs_frame(&self) -> bool {
        if matches!(self.phase, DockVisibilityPhase::Appearing | DockVisibilityPhase::Hiding) {
            return true;
        }
        if self.phase == DockVisibilityPhase::Hidden {
            return false;
        }
        for &action in &DOCK_ACTIONS {
            let i = action.index();
            let target_h = if self.hovered_action == Some(action) { 1.0 } else { 0.0 };
            let target_p = if self.pressed_action == Some(action) { 1.0 } else { 0.0 };
            if (self.hover_t[i] - target_h).abs() > 0.005 || (self.press_t[i] - target_p).abs() > 0.005 {
                return true;
            }
        }
        false
    }

    /// Advances animation state by dt_ms; returns true when hiding just finished into Hidden
    pub fn step(&mut self, dt_ms: f32) -> bool {
        let dt = dt_ms.clamp(0.0, 100.0);
        let mut finished_hide = false;
        let total_appear = DOCK_APPEAR_MS + (DOCK_ACTIONS.len().saturating_sub(1) as f32) * DOCK_STAGGER_MS;
        match self.phase {
            DockVisibilityPhase::Appearing => {
                self.phase_elapsed_ms += dt;
                if self.phase_elapsed_ms >= total_appear {
                    self.phase = DockVisibilityPhase::IdleVisible;
                    self.phase_elapsed_ms = total_appear;
                }
            }
            DockVisibilityPhase::Hiding => {
                self.phase_elapsed_ms += dt;
                if self.phase_elapsed_ms >= DOCK_HIDE_MS {
                    self.hide_immediate();
                    finished_hide = true;
                }
            }
            DockVisibilityPhase::IdleVisible | DockVisibilityPhase::Hidden => {}
        }
        let interact_step = dt / DOCK_INTERACT_MS;
        for &action in &DOCK_ACTIONS {
            let i = action.index();
            let target_h = if self.hovered_action == Some(action) { 1.0 } else { 0.0 };
            let target_p = if self.pressed_action == Some(action) { 1.0 } else { 0.0 };
            self.hover_t[i] = approach(self.hover_t[i], target_h, interact_step);
            self.press_t[i] = approach(self.press_t[i], target_p, interact_step * 1.4);
        }
        finished_hide
    }

    /// Samples interpolated visual parameters for all 5 droplets
    pub fn sample_droplets(&self, direction: DockDirection) -> [DropletFrameState; DOCK_ACTION_COUNT] {
        let mut out = [DropletFrameState::default(); DOCK_ACTION_COUNT];
        let dir_sign = match direction {
            DockDirection::Down => -1.0,
            DockDirection::Up => 1.0,
        };
        for &action in &DOCK_ACTIONS {
            let i = action.index();
            let (vis_alpha, vis_scale, vis_offset) = match self.phase {
                DockVisibilityPhase::Hidden => (0.0, 0.72, dir_sign),
                DockVisibilityPhase::IdleVisible => (1.0, 1.0, 0.0),
                DockVisibilityPhase::Appearing => {
                    let local_t = ((self.phase_elapsed_ms - i as f32 * DOCK_STAGGER_MS) / DOCK_APPEAR_MS).clamp(0.0, 1.0);
                    let eased = ease_out_cubic(local_t);
                    (eased, 0.72 + 0.28 * eased, dir_sign * (1.0 - eased))
                }
                DockVisibilityPhase::Hiding => {
                    let p = (self.phase_elapsed_ms / DOCK_HIDE_MS).clamp(0.0, 1.0);
                    let rem = 1.0 - ease_out_cubic(p);
                    (rem, 0.86 + 0.14 * rem, dir_sign * 0.5 * (1.0 - rem))
                }
            };
            let h = self.hover_t[i];
            let p = self.press_t[i];
            let interact_scale = (1.0 + (DOCK_HOVER_SCALE - 1.0) * h) * (1.0 - (1.0 - DOCK_PRESS_SCALE) * p);
            out[i] = DropletFrameState {
                alpha: vis_alpha,
                scale: vis_scale * interact_scale,
                offset_y_norm: vis_offset,
                hover_t: h,
                specular_boost: (h * 0.55).clamp(0.0, 1.0),
            };
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staggered_appear_settles_to_zero_cpu_idle() {
        let mut anim = DockAnimationState::default();
        assert!(!anim.needs_frame());
        anim.start_appear();
        assert!(anim.needs_frame());
        anim.step(32.0);
        let mid = anim.sample_droplets(DockDirection::Down);
        assert!(mid[0].alpha > mid[4].alpha);
        for _ in 0..20 {
            anim.step(16.0);
        }
        assert_eq!(anim.phase, DockVisibilityPhase::IdleVisible);
        assert!(!anim.needs_frame());
    }

    #[test]
    fn test_hover_press_and_hide_lifecycle() {
        let mut anim = DockAnimationState::default();
        anim.start_appear();
        for _ in 0..20 {
            anim.step(16.0);
        }
        assert!(anim.set_hovered(Some(DockActionId::Pin)));
        assert!(anim.needs_frame());
        for _ in 0..8 {
            anim.step(16.0);
        }
        let hovered = anim.sample_droplets(DockDirection::Down);
        assert!((hovered[0].scale - DOCK_HOVER_SCALE).abs() < 0.01);
        assert!(!anim.needs_frame());
        anim.set_pressed(Some(DockActionId::Pin));
        for _ in 0..8 {
            anim.step(16.0);
        }
        let pressed = anim.sample_droplets(DockDirection::Down);
        assert!(pressed[0].scale < 1.02);
        anim.start_hide();
        let mut hid = false;
        for _ in 0..10 {
            if anim.step(16.0) {
                hid = true;
            }
        }
        assert!(hid);
        assert_eq!(anim.phase, DockVisibilityPhase::Hidden);
        assert!(!anim.needs_frame());
    }
}
