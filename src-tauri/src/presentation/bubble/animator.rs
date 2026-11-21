use std::time::Instant;

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
}
