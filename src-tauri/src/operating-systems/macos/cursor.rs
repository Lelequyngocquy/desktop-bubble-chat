use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(cf: *const std::ffi::c_void);
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventCreate(source: *const std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CGEventGetLocation(event: *mut std::ffi::c_void) -> CGPoint;
    fn CGEventSourceButtonState(state_id: u32, button: u32) -> bool;
    fn CGGetDisplaysWithPoint(point: CGPoint, max_displays: u32, displays: *mut u32, matching_display_count: *mut u32) -> i32;
    fn CGDisplayCopyDisplayMode(display: u32) -> *mut std::ffi::c_void;
    fn CGDisplayModeGetPixelWidth(mode: *mut std::ffi::c_void) -> usize;
    fn CGDisplayModeGetWidth(mode: *mut std::ffi::c_void) -> usize;
    fn CGDisplayModeRelease(mode: *mut std::ffi::c_void);
}

pub struct MacOSCursorTracker;

impl MacOSCursorTracker {
    /// Converts logical Quartz point coordinates into physical top-left pixels
    pub fn quartz_point_to_physical(x_pt: f64, y_pt: f64, scale_factor: f64) -> (i32, i32) {
        let scale = if scale_factor.is_finite() && scale_factor > 0.0 { scale_factor } else { 1.0 };
        ((x_pt * scale).round() as i32, (y_pt * scale).round() as i32)
    }

    /// Resolves backing scale factor of the display containing a given Quartz point
    #[cfg(target_os = "macos")]
    fn display_scale_at_point(point: CGPoint) -> Option<f64> {
        unsafe {
            let mut display_id: u32 = 0;
            let mut count: u32 = 0;
            if CGGetDisplaysWithPoint(point, 1, &mut display_id, &mut count) == 0 && count > 0 {
                let mode = CGDisplayCopyDisplayMode(display_id);
                if !mode.is_null() {
                    let pixel_w = CGDisplayModeGetPixelWidth(mode);
                    let point_w = CGDisplayModeGetWidth(mode);
                    CGDisplayModeRelease(mode);
                    if point_w > 0 && pixel_w > 0 {
                        return Some((pixel_w as f64) / (point_w as f64));
                    }
                }
            }
            None
        }
    }

    /// Returns current global cursor position in physical top-left pixels using active display scale
    #[cfg(target_os = "macos")]
    pub fn get_cursor_position() -> Option<(i32, i32)> {
        unsafe {
            let event = CGEventCreate(std::ptr::null());
            if event.is_null() {
                return None;
            }
            let loc = CGEventGetLocation(event);
            CFRelease(event);
            let scale = Self::display_scale_at_point(loc)
                .unwrap_or_else(|| crate::movement::get_target_dashboard_screen(None).scale_factor.max(1.0));
            Some(Self::quartz_point_to_physical(loc.x, loc.y, scale))
        }
    }

    /// Fallback for non-macOS compilation
    #[cfg(not(target_os = "macos"))]
    pub fn get_cursor_position() -> Option<(i32, i32)> {
        None
    }

    /// Checks if mouse left button is currently pressed
    #[cfg(target_os = "macos")]
    pub fn is_lbutton_down() -> bool {
        unsafe { CGEventSourceButtonState(0, 0) }
    }

    /// Fallback for non-macOS compilation
    #[cfg(not(target_os = "macos"))]
    pub fn is_lbutton_down() -> bool {
        false
    }

    /// Checks if cursor is currently over given window
    pub fn is_cursor_over_window(window: &WebviewWindow) -> bool {
        let Some((cx, cy)) = Self::get_cursor_position() else {
            return false;
        };
        if let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) {
            cx >= pos.x && cx <= pos.x + size.width as i32 && cy >= pos.y && cy <= pos.y + size.height as i32
        } else {
            false
        }
    }

    /// Converts bottom-left origin y coordinate into top-left origin
    pub fn flip_y_coordinate(raw_y: f64, screen_height: f64) -> i32 {
        (screen_height - raw_y).round() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_flip_formula() {
        let screen_height = 1080.0;
        assert_eq!(MacOSCursorTracker::flip_y_coordinate(1080.0, screen_height), 0);
        assert_eq!(MacOSCursorTracker::flip_y_coordinate(0.0, screen_height), 1080);
        assert_eq!(MacOSCursorTracker::flip_y_coordinate(280.0, screen_height), 800);
    }

    #[test]
    fn test_quartz_point_to_physical_multi_dpi() {
        assert_eq!(MacOSCursorTracker::quartz_point_to_physical(500.0, 300.0, 2.0), (1000, 600));
        assert_eq!(MacOSCursorTracker::quartz_point_to_physical(1920.0, 400.0, 1.0), (1920, 400));
        assert_eq!(MacOSCursorTracker::quartz_point_to_physical(100.0, 50.0, 0.0), (100, 50));
    }

    #[test]
    fn test_fallback_button_state() {
        #[cfg(not(target_os = "macos"))]
        {
            assert!(!MacOSCursorTracker::is_lbutton_down());
            assert_eq!(MacOSCursorTracker::get_cursor_position(), None);
        }
    }
}
