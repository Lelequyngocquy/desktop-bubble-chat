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
    pub is_left_edge: bool,
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
    pub is_free_position: bool,
}

impl Default for BubblePhysicsConfig {
    fn default() -> Self {
        Self {
            size: 56.0,
            gap: 12.0,
            tuck_ratio: 0.75,
            appear_ratio: 0.0,
            margin: 12.0,
            taskbar_margin: 60.0,
            dismiss_margin: 160.0,
            chat_width: 404.0,
            chat_height: 550.0,
            chat_gap: 8.0,
            is_free_position: false,
        }
    }
}

pub struct BubbleConfigState(pub std::sync::RwLock<BubblePhysicsConfig>);

impl Default for BubbleConfigState {
    fn default() -> Self {
        Self(std::sync::RwLock::new(BubblePhysicsConfig::default()))
    }
}

pub fn get_bubble_physics_config(app: &tauri::AppHandle) -> BubblePhysicsConfig {
    use tauri::Manager;
    if let Some(state) = app.try_state::<BubbleConfigState>() {
        if let Ok(guard) = state.0.read() {
            return *guard;
        }
    }
    BubblePhysicsConfig::default()
}

static TARGET_DASHBOARD_SCREEN: std::sync::RwLock<Option<ScreenMetrics>> = std::sync::RwLock::new(None);

pub fn update_target_dashboard_screen(metrics: ScreenMetrics) -> Option<(ScreenMetrics, ScreenMetrics)> {
    if let Ok(mut lock) = TARGET_DASHBOARD_SCREEN.write() {
        if let Some(old) = *lock {
            if old.x != metrics.x
                || old.y != metrics.y
                || old.width != metrics.width
                || old.height != metrics.height
                || (old.scale_factor - metrics.scale_factor).abs() > 0.001
            {
                *lock = Some(metrics);
                return Some((old, metrics));
            }
        } else {
            *lock = Some(metrics);
        }
    }
    None
}

pub fn get_target_dashboard_screen(app: Option<&tauri::AppHandle>) -> ScreenMetrics {
    if let Ok(lock) = TARGET_DASHBOARD_SCREEN.read() {
        if let Some(metrics) = *lock {
            return metrics;
        }
    }
    if let Some(app_handle) = app {
        use tauri::Manager;
        if let Some(main_win) = app_handle.get_webview_window("main") {
            if let Ok(Some(mon)) = main_win.current_monitor() {
                let screen = ScreenMetrics::from_monitor(&mon);
                update_target_dashboard_screen(screen);
                return screen;
            }
        }
        if let Ok(Some(mon)) = app_handle.primary_monitor() {
            let screen = ScreenMetrics::from_monitor(&mon);
            update_target_dashboard_screen(screen);
            return screen;
        }
    }
    #[cfg(target_os = "windows")]
    {
        use crate::operating_systems::windows::bubble::window::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
        let cx = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let cy = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: if cx > 0 { cx } else { 1920 },
            height: if cy > 0 { cy } else { 1080 },
            scale_factor: 1.0,
        };
        update_target_dashboard_screen(screen);
        screen
    }
    #[cfg(not(target_os = "windows"))]
    {
        let screen = ScreenMetrics {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        };
        update_target_dashboard_screen(screen);
        screen
    }
}
