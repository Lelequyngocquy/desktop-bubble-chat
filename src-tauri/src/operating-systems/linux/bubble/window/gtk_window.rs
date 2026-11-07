use std::ffi::c_void;
#[allow(unused_imports)]
use super::ffi::*;
#[allow(unused_imports)]
use super::types::*;

pub struct LinuxGtkWindowController;

impl LinuxGtkWindowController {
    /// Creates a transparent, borderless, floating GTK window sticky on all workspaces
    #[cfg(target_os = "linux")]
    pub fn create_floating_bubble_window(x: f64, y: f64, _width: f64, _height: f64) -> *mut c_void {
        unsafe {
            let window = gtk_window_new(GTK_WINDOW_POPUP);
            if window.is_null() {
                return std::ptr::null_mut();
            }
            gtk_widget_set_app_paintable(window, 1);
            gtk_window_set_decorated(window, 0);
            gtk_window_set_skip_taskbar_hint(window, 1);
            gtk_window_set_skip_pager_hint(window, 1);
            gtk_window_set_keep_above(window, 1);
            gtk_window_stick(window);
            let screen = gtk_widget_get_screen(window);
            if !screen.is_null() {
                let visual = gdk_screen_get_rgba_visual(screen);
                if !visual.is_null() {
                    gtk_widget_set_visual(window, visual);
                }
            }
            gtk_window_move(window, x as i32, y as i32);
            window
        }
    }

    /// Fallback for non-Linux compilation
    #[cfg(not(target_os = "linux"))]
    pub fn create_floating_bubble_window(_x: f64, _y: f64, _width: f64, _height: f64) -> *mut c_void {
        std::ptr::null_mut()
    }

    /// Updates window Cairo surface with tiny-skia rendered BGRA/ARGB buffer
    #[cfg(target_os = "linux")]
    pub fn update_window_surface(window: *mut c_void, bgra_data: &[u8], width: usize, height: usize) {
        if window.is_null() || bgra_data.is_empty() || width == 0 || height == 0 {
            return;
        }
        unsafe {
            let surface = cairo_image_surface_create_for_data(
                bgra_data.as_ptr() as *mut u8,
                CAIRO_FORMAT_ARGB32,
                width as i32,
                height as i32,
                (width * 4) as i32,
            );
            if !surface.is_null() {
                gtk_widget_queue_draw(window);
                cairo_surface_destroy(surface);
            }
        }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn update_window_surface(_window: *mut c_void, _bgra_data: &[u8], _width: usize, _height: usize) {}

    /// Sets window top-left position coordinates
    #[cfg(target_os = "linux")]
    pub fn set_window_position(window: *mut c_void, x: i32, y: i32) {
        if window.is_null() {
            return;
        }
        unsafe {
            gtk_window_move(window, x, y);
        }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn set_window_position(_window: *mut c_void, _x: i32, _y: i32) {}

    /// Shows or hides GTK window
    #[cfg(target_os = "linux")]
    pub fn set_window_visible(window: *mut c_void, visible: bool) {
        if window.is_null() {
            return;
        }
        unsafe {
            if visible {
                gtk_widget_show_all(window);
            } else {
                gtk_widget_hide(window);
            }
        }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn set_window_visible(_window: *mut c_void, _visible: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtk_window_controller_fallback_safety() {
        let ptr = LinuxGtkWindowController::create_floating_bubble_window(100.0, 100.0, 56.0, 56.0);
        #[cfg(not(target_os = "linux"))]
        {
            assert!(ptr.is_null());
            LinuxGtkWindowController::update_window_surface(ptr, &[], 0, 0);
            LinuxGtkWindowController::set_window_position(ptr, 0, 0);
            LinuxGtkWindowController::set_window_visible(ptr, true);
        }
    }
}
