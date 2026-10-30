use tauri::Monitor;

#[derive(Debug, Clone, Copy)]
pub struct ScreenMetrics {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub scale_factor: f64,
}

impl ScreenMetrics {
    pub fn from_monitor(monitor: &Monitor) -> Self {
        let pos = monitor.position();
        let size = monitor.size();
        let scale = monitor.scale_factor();
        Self {
            x: pos.x,
            y: pos.y,
            width: size.width as i32,
            height: size.height as i32,
            scale_factor: scale,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SnapResult {
    pub target_x: i32,
    pub target_y: i32,
    pub should_dismiss: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct BubblePhysicsConfig {
    pub size: f64,
    pub gap: f64,
    pub tuck_ratio: f64,
    pub appear_ratio: f64,
    pub margin: f64,
    pub taskbar_margin: f64,
    pub dismiss_margin: f64,
    pub chat_width: f64,
    pub chat_height: f64,
    pub chat_gap: f64,
}

impl Default for BubblePhysicsConfig {
    fn default() -> Self {
        Self {
            size: 64.0,
            gap: 12.0,
            tuck_ratio: 0.8,
            appear_ratio: 0.0,
            margin: 12.0,
            taskbar_margin: 60.0,
            dismiss_margin: 80.0,
            chat_width: 460.0,
            chat_height: 620.0,
            chat_gap: 8.0,
        }
    }
}
