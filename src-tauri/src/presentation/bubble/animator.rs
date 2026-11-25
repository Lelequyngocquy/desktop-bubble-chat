use std::time::Instant;

/// Total duration of one biological heartbeat cycle (lub-dub beat + quiet rest)
pub const HEARTBEAT_CYCLE_MS: u64 = 2000;
/// Active beat duration containing both lub-dub peaks and ripple fade-out
pub const HEARTBEAT_ACTIVE_MS: u64 = 650;
/// Quiet rest duration where rendering timer sleeps to save CPU/GPU
pub const HEARTBEAT_REST_MS: u64 = HEARTBEAT_CYCLE_MS - HEARTBEAT_ACTIVE_MS;
/// Frame interval (~60 FPS) used only during the active 650ms beat phase
pub const HEARTBEAT_FRAME_INTERVAL_MS: u64 = 16;

/// Single sampled visual frame of the heartbeat alert animation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeartbeatFrame {
    pub beat_scale: f32,
    pub ripple_progress: f32,
    pub ripple_alpha: u8,
    pub in_active_beat: bool,
}

impl Default for HeartbeatFrame {
    fn default() -> Self {
        Self { beat_scale: 1.0, ripple_progress: 0.0, ripple_alpha: 0, in_active_beat: false }
    }
}

/// Samples the biological double-beat (lub-dub + rest) curve at a cycle offset in milliseconds
pub fn sample_heartbeat_frame(cycle_elapsed_ms: u64) -> HeartbeatFrame {
    let t = cycle_elapsed_ms % HEARTBEAT_CYCLE_MS;
    if t >= HEARTBEAT_ACTIVE_MS {
        return HeartbeatFrame::default();
    }
    let beat_scale = if t < 110 {
        let p = t as f64 / 110.0;
        1.0 + 0.045 * ease_out_quad(p)
    } else if t < 210 {
        let p = (t - 110) as f64 / 100.0;
        1.045 - 0.033 * p
    } else if t < 330 {
        let p = (t - 210) as f64 / 120.0;
        1.012 + 0.058 * ease_out_quad(p)
    } else {
        let p = (t - 330) as f64 / 320.0;
        1.070 - 0.070 * ease_out_cubic(p)
    };
    let (ripple_progress, ripple_alpha) = if t >= 210 {
        let rp = ((t - 210) as f64 / 440.0).clamp(0.0, 1.0);
        let alpha = ((1.0 - rp).powi(2) * 210.0).round().clamp(0.0, 255.0) as u8;
        (rp as f32, alpha)
    } else {
        (0.0, 0)
    };
    HeartbeatFrame {
        beat_scale: beat_scale as f32,
        ripple_progress,
        ripple_alpha,
        in_active_beat: true,
    }
}

/// Computes next timer delay in ms for two-speed scheduling (16ms beat / sleep across rest)
pub fn heartbeat_next_delay_for_elapsed(cycle_elapsed_ms: u64) -> u64 {
    let t = cycle_elapsed_ms % HEARTBEAT_CYCLE_MS;
    if t < HEARTBEAT_ACTIVE_MS {
        HEARTBEAT_FRAME_INTERVAL_MS
    } else {
        (HEARTBEAT_CYCLE_MS - t).max(HEARTBEAT_FRAME_INTERVAL_MS)
    }
}

/// State machine managing lub-dub heartbeat cycle and two-speed timer scheduling
#[derive(Debug, Clone)]
pub struct HeartbeatAnimationState {
    pub is_alert_active: bool,
    pub is_paused: bool,
    pub cycle_start: Instant,
}

impl Default for HeartbeatAnimationState {
    fn default() -> Self {
        Self { is_alert_active: false, is_paused: false, cycle_start: Instant::now() }
    }
}

impl HeartbeatAnimationState {
    /// Updates unread alert eligibility and starts a fresh cycle on activation
    pub fn set_alert_active(&mut self, active: bool) -> bool {
        if self.is_alert_active == active {
            return false;
        }
        self.is_alert_active = active;
        if active {
            self.cycle_start = Instant::now();
        }
        true
    }

    /// Pauses or resumes beat motion (e.g., on hover, drag, or open chat)
    pub fn set_paused(&mut self, paused: bool) -> bool {
        if self.is_paused == paused {
            return false;
        }
        self.is_paused = paused;
        if !paused && self.is_alert_active {
            self.cycle_start = Instant::now();
        }
        true
    }

    /// Restarts the beat cycle from t=0 immediately (e.g., on a newly arrived message)
    pub fn reset_cycle(&mut self) {
        self.cycle_start = Instant::now();
    }

    /// Returns true when the heartbeat timer should actively tick
    pub fn should_animate(&self) -> bool {
        self.is_alert_active && !self.is_paused
    }

    /// Samples current render frame, returning static identity frame when inactive or paused
    pub fn current_frame(&self) -> HeartbeatFrame {
        if !self.should_animate() {
            return HeartbeatFrame::default();
        }
        sample_heartbeat_frame(self.cycle_start.elapsed().as_millis() as u64)
    }

    /// Returns next timer interval in milliseconds, or None when timer should stop
    pub fn next_delay_ms(&self) -> Option<u64> {
        if !self.should_animate() {
            return None;
        }
        Some(heartbeat_next_delay_for_elapsed(self.cycle_start.elapsed().as_millis() as u64))
    }
}

/// Ease-out cubic curve: f(p) = 1 - (1 - p)^3
pub fn ease_out_cubic(progress: f64) -> f64 {
    let p = progress.clamp(0.0, 1.0);
    1.0 - (1.0 - p).powi(3)
}

/// Ease-out quadratic curve: f(p) = 1 - (1 - p)^2
pub fn ease_out_quad(progress: f64) -> f64 {
    let p = progress.clamp(0.0, 1.0);
    1.0 - (1.0 - p).powi(2)
}

/// Interpolates 2D position based on easing progress
pub fn interpolate_position(
    start: (i32, i32),
    target: (i32, i32),
    progress: f64,
) -> (i32, i32) {
    let ease = ease_out_cubic(progress);
    let cur_x = (start.0 as f64 + (target.0 - start.0) as f64 * ease).round() as i32;
    let cur_y = (start.1 as f64 + (target.1 - start.1) as f64 * ease).round() as i32;
    (cur_x, cur_y)
}

/// Spring animation state for 60 FPS interpolation
#[derive(Debug, Clone)]
pub struct SpringAnimationState {
    pub is_animating: bool,
    pub start_x: i32,
    pub start_y: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub start_time: Instant,
    pub duration_ms: u64,
}

impl Default for SpringAnimationState {
    fn default() -> Self {
        Self {
            is_animating: false,
            start_x: 0,
            start_y: 0,
            target_x: 0,
            target_y: 0,
            start_time: Instant::now(),
            duration_ms: 180,
        }
    }
}

impl SpringAnimationState {
    /// Starts a new animation toward target coordinates
    pub fn start(&mut self, current_x: i32, current_y: i32, target_x: i32, target_y: i32, duration_ms: u64) {
        self.is_animating = true;
        self.start_x = current_x;
        self.start_y = current_y;
        self.target_x = target_x;
        self.target_y = target_y;
        self.start_time = Instant::now();
        self.duration_ms = duration_ms.max(40);
    }

    /// Stops animation immediately
    pub fn stop(&mut self) {
        self.is_animating = false;
    }

    /// Calculates position at current time, returns (x, y, is_finished)
    pub fn step(&mut self) -> Option<(i32, i32, bool)> {
        if !self.is_animating {
            return None;
        }
        let elapsed = self.start_time.elapsed().as_millis() as f64;
        let progress = (elapsed / self.duration_ms as f64).clamp(0.0, 1.0);
        let (cur_x, cur_y) = interpolate_position(
            (self.start_x, self.start_y),
            (self.target_x, self.target_y),
            progress,
        );
        let is_finished = progress >= 1.0;
        if is_finished {
            self.is_animating = false;
        }
        Some((cur_x, cur_y, is_finished))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ease_out_cubic_boundaries() {
        assert_eq!(ease_out_cubic(0.0), 0.0);
        assert_eq!(ease_out_cubic(1.0), 1.0);
        let mid = ease_out_cubic(0.5);
        assert!((mid - 0.875).abs() < 1e-6);
    }

    #[test]
    fn test_interpolate_position() {
        let start = (0, 100);
        let target = (200, 300);
        let (x0, y0) = interpolate_position(start, target, 0.0);
        assert_eq!(x0, 0);
        assert_eq!(y0, 100);
        let (x1, y1) = interpolate_position(start, target, 1.0);
        assert_eq!(x1, 200);
        assert_eq!(y1, 300);
    }

    #[test]
    fn test_spring_animation_state_lifecycle() {
        let mut anim = SpringAnimationState::default();
        assert!(!anim.is_animating);
        assert_eq!(anim.step(), None);
        anim.start(10, 20, 100, 200, 180);
        assert!(anim.is_animating);
        let step_res = anim.step();
        assert!(step_res.is_some());
        let (x, y, finished) = step_res.unwrap();
        assert!(x >= 10 && x <= 100);
        assert!(y >= 20 && y <= 200);
        assert!(!finished);
        anim.stop();
        assert!(!anim.is_animating);
    }

    #[test]
    fn test_heartbeat_curve_keyframes_and_two_speed_schedule() {
        let f0 = sample_heartbeat_frame(0);
        assert!(f0.in_active_beat);
        assert!((f0.beat_scale - 1.0).abs() < 1e-4);
        assert_eq!(f0.ripple_alpha, 0);
        let f_lub = sample_heartbeat_frame(110);
        assert!(f_lub.in_active_beat);
        assert!((f_lub.beat_scale - 1.045).abs() < 1e-3);
        let f_valley = sample_heartbeat_frame(210);
        assert!((f_valley.beat_scale - 1.012).abs() < 1e-3);
        assert_eq!(f_valley.ripple_alpha, 210);
        let f_dub = sample_heartbeat_frame(330);
        assert!((f_dub.beat_scale - 1.070).abs() < 1e-3);
        assert!(f_dub.ripple_progress > 0.2 && f_dub.ripple_alpha > 0);
        let f_rest = sample_heartbeat_frame(650);
        assert!(!f_rest.in_active_beat);
        assert_eq!(f_rest.beat_scale, 1.0);
        assert_eq!(f_rest.ripple_alpha, 0);
        let f_mid_rest = sample_heartbeat_frame(1200);
        assert!(!f_mid_rest.in_active_beat);
        assert_eq!(f_mid_rest.beat_scale, 1.0);
        let f_wrap = sample_heartbeat_frame(2000);
        assert!(f_wrap.in_active_beat);
        assert_eq!(heartbeat_next_delay_for_elapsed(0), HEARTBEAT_FRAME_INTERVAL_MS);
        assert_eq!(heartbeat_next_delay_for_elapsed(330), HEARTBEAT_FRAME_INTERVAL_MS);
        assert_eq!(heartbeat_next_delay_for_elapsed(650), HEARTBEAT_REST_MS);
        assert_eq!(heartbeat_next_delay_for_elapsed(1200), 800);
    }

    #[test]
    fn test_heartbeat_state_pause_and_activation_lifecycle() {
        let mut hb = HeartbeatAnimationState::default();
        assert!(!hb.should_animate());
        assert_eq!(hb.next_delay_ms(), None);
        assert!(hb.set_alert_active(true));
        assert!(!hb.set_alert_active(true));
        assert!(hb.should_animate());
        assert!(hb.next_delay_ms().is_some());
        assert!(hb.set_paused(true));
        assert!(!hb.should_animate());
        assert_eq!(hb.next_delay_ms(), None);
        assert_eq!(hb.current_frame(), HeartbeatFrame::default());
        assert!(hb.set_paused(false));
        assert!(hb.should_animate());
        assert!(hb.set_alert_active(false));
        assert!(!hb.should_animate());
    }
}
