use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{LazyLock, Mutex};
#[allow(unused_imports)]
use super::ffi::*;
#[allow(unused_imports)]
use super::types::*;

pub struct LinuxGtkWindowController;

static SURFACE_CACHE: LazyLock<Mutex<HashMap<usize, (Vec<u8>, usize, usize)>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_draw(widget: *mut c_void, cr: *mut c_void, _user_data: *mut c_void) -> i32 {
    let key = widget as usize;
    if let Ok(cache) = SURFACE_CACHE.lock() {
        if let Some((bgra, width, height)) = cache.get(&key) {
            unsafe {
                let surface = cairo_image_surface_create_for_data(bgra.as_ptr() as *mut u8, CAIRO_FORMAT_ARGB32, *width as i32, *height as i32, (*width * 4) as i32);
                if !surface.is_null() {
                    cairo_set_operator(cr, CAIRO_OPERATOR_SOURCE);
                    cairo_set_source_surface(cr, surface, 0.0, 0.0);
                    cairo_paint(cr);
                    cairo_surface_destroy(surface);
                }
            }
        }
    }
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_button_press(_widget: *mut c_void, event: *mut GdkEventButton, user_data: *mut c_void) -> i32 {
    let account_index = user_data as usize as u32;
    if !event.is_null() && unsafe { (*event).button } == 1 {
        let (rx, ry) = unsafe { ((*event).x_root as i32, (*event).y_root as i32) };
        crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().handle_mouse_down(account_index, rx, ry);
    }
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_motion_notify(_widget: *mut c_void, event: *mut GdkEventMotion, user_data: *mut c_void) -> i32 {
    let account_index = user_data as usize as u32;
    if !event.is_null() {
        let (rx, ry) = unsafe { ((*event).x_root as i32, (*event).y_root as i32) };
        crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().handle_mouse_move(account_index, rx, ry);
    }
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_button_release(_widget: *mut c_void, event: *mut GdkEventButton, user_data: *mut c_void) -> i32 {
    let account_index = user_data as usize as u32;
    if !event.is_null() && unsafe { (*event).button } == 1 {
        let (rx, ry) = unsafe { ((*event).x_root as i32, (*event).y_root as i32) };
        crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().handle_mouse_up(account_index, rx, ry);
    }
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_enter_notify(_widget: *mut c_void, _event: *mut c_void, user_data: *mut c_void) -> i32 {
    let account_index = user_data as usize as u32;
    crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().handle_mouse_enter(account_index);
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_leave_notify(_widget: *mut c_void, _event: *mut c_void, user_data: *mut c_void) -> i32 {
    let account_index = user_data as usize as u32;
    crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().handle_mouse_leave(account_index);
    0
}

impl LinuxGtkWindowController {
    /// Creates a transparent, borderless, floating GTK window sticky on all workspaces
    #[cfg(target_os = "linux")]
    pub fn create_floating_bubble_window(account_index: u32, x: f64, y: f64, _width: f64, _height: f64) -> *mut c_void {
        unsafe {
            if gtk_init_check(std::ptr::null_mut(), std::ptr::null_mut()) == 0 {
                return std::ptr::null_mut();
            }
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
            gtk_widget_add_events(window, GDK_BUTTON_PRESS_MASK | GDK_BUTTON_RELEASE_MASK | GDK_POINTER_MOTION_MASK | GDK_ENTER_NOTIFY_MASK | GDK_LEAVE_NOTIFY_MASK);
            let user_data = account_index as usize as *mut c_void;
            g_signal_connect_data(window, b"draw\0".as_ptr() as *const _, Some(std::mem::transmute(on_draw as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, b"button-press-event\0".as_ptr() as *const _, Some(std::mem::transmute(on_button_press as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, b"motion-notify-event\0".as_ptr() as *const _, Some(std::mem::transmute(on_motion_notify as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, b"button-release-event\0".as_ptr() as *const _, Some(std::mem::transmute(on_button_release as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, b"enter-notify-event\0".as_ptr() as *const _, Some(std::mem::transmute(on_enter_notify as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, b"leave-notify-event\0".as_ptr() as *const _, Some(std::mem::transmute(on_leave_notify as *const ())), user_data, std::ptr::null_mut(), 0);
            gtk_window_move(window, x as i32, y as i32);
            window
        }
    }

    /// Fallback for non-Linux compilation
    #[cfg(not(target_os = "linux"))]
    pub fn create_floating_bubble_window(_account_index: u32, _x: f64, _y: f64, _width: f64, _height: f64) -> *mut c_void {
        std::ptr::null_mut()
    }

    /// Updates window Cairo surface with tiny-skia rendered BGRA/ARGB buffer
    #[cfg(target_os = "linux")]
    pub fn update_window_surface(window: *mut c_void, bgra_data: &[u8], width: usize, height: usize) {
        if window.is_null() || bgra_data.is_empty() || width == 0 || height == 0 {
            return;
        }
        let key = window as usize;
        if let Ok(mut cache) = SURFACE_CACHE.lock() {
            cache.insert(key, (bgra_data.to_vec(), width, height));
        }
        unsafe {
            gtk_widget_queue_draw(window);
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

    /// Destroys GTK window and cleans up surface cache
    #[cfg(target_os = "linux")]
    pub fn destroy_window(window: *mut c_void) {
        if window.is_null() {
            return;
        }
        let key = window as usize;
        if let Ok(mut cache) = SURFACE_CACHE.lock() {
            cache.remove(&key);
        }
        unsafe {
            gtk_widget_destroy(window);
        }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn destroy_window(_window: *mut c_void) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gtk_window_controller_fallback_safety() {
        let ptr = std::ptr::null_mut();
        LinuxGtkWindowController::update_window_surface(ptr, &[], 0, 0);
        LinuxGtkWindowController::set_window_position(ptr, 0, 0);
        LinuxGtkWindowController::set_window_visible(ptr, true);
        LinuxGtkWindowController::destroy_window(ptr);
    }
}
