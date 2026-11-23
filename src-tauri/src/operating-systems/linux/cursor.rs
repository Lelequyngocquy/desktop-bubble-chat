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
static X11_DISPLAY: std::sync::Mutex<usize> = std::sync::Mutex::new(0);
#[cfg(target_os = "linux")]
static X11_INIT_ONCE: std::sync::Once = std::sync::Once::new();

/// Initializes X11 thread support before GTK or Tauri creates any X11 display connection
pub fn init_x11_threads() {
    #[cfg(target_os = "linux")]
    X11_INIT_ONCE.call_once(|| unsafe {
        x11::xlib::XInitThreads();
    });
}

#[cfg(target_os = "linux")]
fn with_x11_display<R, F: FnOnce(*mut x11::xlib::Display) -> R>(f: F) -> Option<R> {
    init_x11_threads();
    let Ok(mut disp_guard) = X11_DISPLAY.lock() else {
        return None;
    };
    unsafe {
        if *disp_guard == 0 {
            let opened = x11::xlib::XOpenDisplay(std::ptr::null());
            if opened.is_null() {
                return None;
            }
            *disp_guard = opened as usize;
        }
        Some(f(*disp_guard as *mut x11::xlib::Display))
    }
}

#[cfg(target_os = "linux")]
fn query_x11_pointer() -> Option<((i32, i32), bool)> {
    with_x11_display(|display| unsafe {
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
        if res != 0 {
            let is_btn1_down = (mask_return & (1 << 8)) != 0;
            Some(((root_x, root_y), is_btn1_down))
        } else {
            None
        }
    }).flatten()
}

pub struct LinuxCursorTracker;

impl LinuxCursorTracker {
    /// Classifies a GDK display name as native Wayland (true) vs X11/XWayland (false)
    pub fn classify_gdk_display_name(name: &str) -> bool {
        let lower = name.trim().to_lowercase();
        lower.contains("wayland")
    }

    /// Detects if current session environment indicates Wayland display server
    pub fn is_running_wayland() -> bool {
        std::env::var("WAYLAND_DISPLAY").map(|v| !v.trim().is_empty()).unwrap_or(false)
            || std::env::var("XDG_SESSION_TYPE").map(|v| v.to_lowercase() == "wayland").unwrap_or(false)
    }

    /// Detects whether active GDK backend is actually Wayland (false under XWayland/X11)
    pub fn is_active_gdk_wayland() -> bool {
        #[cfg(target_os = "linux")]
        unsafe {
            let display = crate::operating_systems::linux::bubble::window::ffi::gdk_display_get_default();
            if !display.is_null() {
                let name_ptr = crate::operating_systems::linux::bubble::window::ffi::gdk_display_get_name(display);
                if !name_ptr.is_null() {
                    let name = std::ffi::CStr::from_ptr(name_ptr).to_string_lossy();
                    return Self::classify_gdk_display_name(&name);
                }
            }
            Self::is_running_wayland()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Checks whether the active X11 top-level window currently has _NET_WM_STATE_FULLSCREEN
    pub fn is_active_x11_window_fullscreen() -> bool {
        #[cfg(target_os = "linux")]
        {
            if Self::is_active_gdk_wayland() {
                return false;
            }
            with_x11_display(|display| unsafe {
                let active_atom = x11::xlib::XInternAtom(display, c"_NET_ACTIVE_WINDOW".as_ptr(), 1);
                let state_atom = x11::xlib::XInternAtom(display, c"_NET_WM_STATE".as_ptr(), 1);
                let fs_atom = x11::xlib::XInternAtom(display, c"_NET_WM_STATE_FULLSCREEN".as_ptr(), 1);
                if active_atom == 0 || state_atom == 0 || fs_atom == 0 {
                    return false;
                }
                let root = x11::xlib::XDefaultRootWindow(display);
                let mut actual_type: std::ffi::c_ulong = 0;
                let mut actual_format: std::ffi::c_int = 0;
                let mut nitems: std::ffi::c_ulong = 0;
                let mut bytes_after: std::ffi::c_ulong = 0;
                let mut prop_ptr: *mut std::ffi::c_uchar = std::ptr::null_mut();
                let status = x11::xlib::XGetWindowProperty(
                    display, root, active_atom, 0, 1, 0, x11::xlib::AnyPropertyType as std::ffi::c_ulong,
                    &mut actual_type, &mut actual_format, &mut nitems, &mut bytes_after, &mut prop_ptr,
                );
                if status != x11::xlib::Success as std::ffi::c_int || prop_ptr.is_null() || nitems == 0 {
                    if !prop_ptr.is_null() {
                        x11::xlib::XFree(prop_ptr as *mut std::ffi::c_void);
                    }
                    return false;
                }
                let active_win = *(prop_ptr as *const std::ffi::c_ulong);
                x11::xlib::XFree(prop_ptr as *mut std::ffi::c_void);
                if active_win == 0 {
                    return false;
                }
                let mut st_type: std::ffi::c_ulong = 0;
                let mut st_format: std::ffi::c_int = 0;
                let mut st_items: std::ffi::c_ulong = 0;
                let mut st_after: std::ffi::c_ulong = 0;
                let mut st_ptr: *mut std::ffi::c_uchar = std::ptr::null_mut();
                let st_status = x11::xlib::XGetWindowProperty(
                    display, active_win, state_atom, 0, 64, 0, x11::xlib::AnyPropertyType as std::ffi::c_ulong,
                    &mut st_type, &mut st_format, &mut st_items, &mut st_after, &mut st_ptr,
                );
                let mut is_fs = false;
                if st_status == x11::xlib::Success as std::ffi::c_int && !st_ptr.is_null() && st_items > 0 {
                    let atoms = std::slice::from_raw_parts(st_ptr as *const std::ffi::c_ulong, st_items as usize);
                    is_fs = atoms.contains(&fs_atom);
                }
                if !st_ptr.is_null() {
                    x11::xlib::XFree(st_ptr as *mut std::ffi::c_void);
                }
                is_fs
            }).unwrap_or(false)
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Queries global cursor position via X11 XQueryPointer or Wayland event cache
    pub fn get_cursor_position() -> Option<(i32, i32)> {
        #[cfg(target_os = "linux")]
        {
            if !Self::is_active_gdk_wayland()
                && let Some((pos, _)) = query_x11_pointer() {
                    return Some(pos);
                }
            if let Ok(guard) = WAYLAND_CURSOR_POS.read()
                && let Some(pos) = *guard {
                    return Some(pos);
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
            if !Self::is_active_gdk_wayland()
                && let Some((_, btn1)) = query_x11_pointer() {
                    return btn1;
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
            assert!(!LinuxCursorTracker::is_lbutton_down());
        }
    }
}
