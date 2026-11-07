use tauri::WebviewWindow;

static WAYLAND_CURSOR_POS: std::sync::RwLock<Option<(i32, i32)>> = std::sync::RwLock::new(None);
static WAYLAND_LBUTTON_DOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Updates cached Wayland cursor position and button state from window events
pub fn update_wayland_cursor(pos: Option<(i32, i32)>, lbutton_down: bool) {
    if let Ok(mut lock) = WAYLAND_CURSOR_POS.write() {
        *lock = pos;
    }
    WAYLAND_LBUTTON_DOWN.store(lbutton_down, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(target_os = "linux")]
fn query_x11_pointer() -> Option<((i32, i32), bool)> {
    unsafe {
        let display = x11::xlib::XOpenDisplay(std::ptr::null());
        if display.is_null() {
            return None;
        }
        let root = x11::xlib::XDefaultRootWindow(display);
        let mut root_return = 0;
        let mut child_return = 0;
        let mut root_x = 0;
        let mut root_y = 0;
        let mut win_x = 0;
        let mut win_y = 0;
        let mut mask_return = 0;
        let res = x11::xlib::XQueryPointer(
            display, root, &mut root_return, &mut child_return,
            &mut root_x, &mut root_y, &mut win_x, &mut win_y, &mut mask_return,
        );
        x11::xlib::XCloseDisplay(display);
        if res != 0 {
            // Button1Mask in X11 is (1 << 8)
            let is_btn1_down = (mask_return & (1 << 8)) != 0;
            Some(((root_x, root_y), is_btn1_down))
        } else {
            None
        }
    }
}

pub struct LinuxCursorTracker;

impl LinuxCursorTracker {
    /// Detects if current session runs under Wayland display server
    pub fn is_running_wayland() -> bool {
        std::env::var("WAYLAND_DISPLAY").map(|v| !v.trim().is_empty()).unwrap_or(false)
            || std::env::var("XDG_SESSION_TYPE").map(|v| v.to_lowercase() == "wayland").unwrap_or(false)
    }

    /// Queries global cursor position via X11 XQueryPointer or Wayland event cache
    pub fn get_cursor_position() -> Option<(i32, i32)> {
        #[cfg(target_os = "linux")]
        {
            if !Self::is_running_wayland() {
                if let Some((pos, _)) = query_x11_pointer() {
                    return Some(pos);
                }
            }
            if let Ok(guard) = WAYLAND_CURSOR_POS.read() {
                if let Some(pos) = *guard {
                    return Some(pos);
                }
            }
            None
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }

    /// Checks if mouse left button is currently pressed
    pub fn is_lbutton_down() -> bool {
        #[cfg(target_os = "linux")]
        {
            if !Self::is_running_wayland() {
                if let Some((_, btn1)) = query_x11_pointer() {
                    return btn1;
                }
            }
            WAYLAND_LBUTTON_DOWN.load(std::sync::atomic::Ordering::SeqCst)
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_detection_not_panicking() {
        let _ = LinuxCursorTracker::is_running_wayland();
    }

    #[test]
    fn test_wayland_cursor_cache_update() {
        update_wayland_cursor(Some((250, 400)), true);
        if let Ok(guard) = WAYLAND_CURSOR_POS.read() {
            assert_eq!(*guard, Some((250, 400)));
        }
        assert!(WAYLAND_LBUTTON_DOWN.load(std::sync::atomic::Ordering::SeqCst));
        update_wayland_cursor(None, false);
        if let Ok(guard) = WAYLAND_CURSOR_POS.read() {
            assert_eq!(*guard, None);
        }
        assert!(!WAYLAND_LBUTTON_DOWN.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_fallback_behavior() {
        #[cfg(not(target_os = "linux"))]
        {
            assert_eq!(LinuxCursorTracker::get_cursor_position(), None);
            assert_eq!(LinuxCursorTracker::is_lbutton_down(), false);
        }
    }
}
