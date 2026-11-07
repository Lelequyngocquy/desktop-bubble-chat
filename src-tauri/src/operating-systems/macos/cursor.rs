use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventCreate(source: *const std::ffi::c_void) -> *mut std::ffi::c_void;
    fn CGEventGetLocation(event: *mut std::ffi::c_void) -> CGPoint;
    fn CGEventSourceButtonState(state_id: u32, button: u32) -> bool;
    fn CFRelease(cf: *const std::ffi::c_void);
    fn CGMainDisplayID() -> u32;
    fn CGDisplayPixelsHigh(display: u32) -> usize;
}

pub struct MacOSCursorTracker;

impl MacOSCursorTracker {
    /// Returns current global cursor position with macOS coordinate flipped to top-left origin
    #[cfg(target_os = "macos")]
    pub fn get_cursor_position() -> Option<(i32, i32)> {
        unsafe {
            let event = CGEventCreate(std::ptr::null());
            if event.is_null() {
                return None;
            }
            let loc = CGEventGetLocation(event);
            CFRelease(event);
            let main_disp = CGMainDisplayID();
            let screen_h = CGDisplayPixelsHigh(main_disp) as f64;
            let top_left_y = Self::flip_y_coordinate(loc.y, screen_h);
            Some((loc.x.round() as i32, top_left_y))
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
        unsafe {
            // state_id 0 corresponds to kCGEventSourceStateCombinedSessionState, button 0 is left button
            CGEventSourceButtonState(0, 0)
        }
    }

    /// Fallback for non-macOS compilation
    #[cfg(not(target_os = "macos"))]
    pub fn is_lbutton_down() -> bool {
        false
    }

    /// Checks if cursor is currently over given window
    pub fn is_cursor_over_window(window: &WebviewWindow) -> bool {
        let (cx, cy) = match Self::get_cursor_position() {
            Some(pos) => pos,
            None => return false,
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
    fn test_fallback_button_state() {
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(MacOSCursorTracker::is_lbutton_down(), false);
            assert_eq!(MacOSCursorTracker::get_cursor_position(), None);
        }
    }
}
