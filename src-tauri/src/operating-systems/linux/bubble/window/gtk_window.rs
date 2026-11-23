#[cfg(target_os = "linux")]
use std::collections::HashMap;
use std::ffi::c_void;
#[cfg(target_os = "linux")]
use std::sync::{LazyLock, Mutex};
#[allow(unused_imports)]
use super::ffi::*;
#[allow(unused_imports)]
use super::types::*;

pub struct LinuxGtkWindowController;

/// Converts physical pixel coordinate to GDK logical coordinate
pub fn physical_coord_to_logical(val_px: i32, scale_factor: f64) -> i32 {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 { scale_factor } else { 1.0 };
    ((val_px as f64) / scale).round() as i32
}

/// Converts physical pixel dimension to GDK logical dimension (minimum 1)
pub fn physical_size_to_logical(size_px: u32, scale_factor: f64) -> i32 {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 { scale_factor } else { 1.0 };
    (((size_px as f64) / scale).round() as i32).max(1)
}

/// Converts GDK logical root coordinate to physical pixel coordinate
pub fn logical_root_to_physical(val_logical: f64, scale_factor: f64) -> i32 {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 { scale_factor } else { 1.0 };
    (val_logical * scale).round() as i32
}

/// Builds circular scanline rectangles plus optional badge box for GTK input shape click-through
pub fn compute_bubble_input_rects(
    canvas_w_logical: i32,
    canvas_h_logical: i32,
    bubble_size_logical: i32,
    unread_count: u32,
    is_left_edge: bool,
) -> Vec<CairoRectangleInt> {
    let mut rects = Vec::new();
    let cx = (canvas_w_logical as f64) * 0.5;
    let cy = (canvas_h_logical as f64) * 0.5;
    let r = ((bubble_size_logical as f64) * 0.5).max(12.0) + 2.0;
    let min_y = ((cy - r).floor() as i32).max(0);
    let max_y = ((cy + r).ceil() as i32).min(canvas_h_logical.saturating_sub(1));
    let step = 2i32;
    let mut y = min_y;
    while y <= max_y {
        let dy = (y as f64 + 0.5) - cy;
        let span_sq = r * r - dy * dy;
        if span_sq > 0.0 {
            let half_w = span_sq.sqrt();
            let x0 = ((cx - half_w).floor() as i32).max(0);
            let x1 = ((cx + half_w).ceil() as i32).min(canvas_w_logical);
            if x1 > x0 {
                let h = (max_y - y + 1).min(step);
                rects.push(CairoRectangleInt { x: x0, y, width: x1 - x0, height: h });
            }
        }
        y += step;
    }
    if unread_count > 0 {
        let badge_w = 26;
        let badge_h = 22;
        let bx = if is_left_edge { (cx + r * 0.45) as i32 } else { (cx - r * 0.45 - badge_w as f64) as i32 };
        let by = (cy - r * 0.85) as i32;
        rects.push(CairoRectangleInt { x: bx.max(0), y: by.max(0), width: badge_w, height: badge_h });
    }
    rects
}

#[cfg(target_os = "linux")]
type SurfaceBufferEntry = (Vec<u8>, usize, usize, f64);
#[cfg(target_os = "linux")]
static SURFACE_CACHE: LazyLock<Mutex<HashMap<usize, SurfaceBufferEntry>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_draw(widget: *mut c_void, cr: *mut c_void, _user_data: *mut c_void) -> i32 {
    let key = widget as usize;
    if let Ok(cache) = SURFACE_CACHE.lock() {
        if let Some((bgra, width, height, scale)) = cache.get(&key) {
            unsafe {
                let surface = cairo_image_surface_create_for_data(bgra.as_ptr() as *mut u8, CAIRO_FORMAT_ARGB32, *width as i32, *height as i32, (*width * 4) as i32);
                if !surface.is_null() {
                    let s = if scale.is_finite() && *scale > 0.0 { *scale } else { 1.0 };
                    cairo_surface_set_device_scale(surface, s, s);
                    cairo_set_operator(cr, CAIRO_OPERATOR_SOURCE);
                    cairo_set_source_surface(cr, surface, 0.0, 0.0);
                    cairo_paint(cr);
                    cairo_surface_destroy(surface);
                }
            }
        } else {
            unsafe {
                cairo_set_operator(cr, CAIRO_OPERATOR_CLEAR);
                cairo_paint(cr);
            }
        }
    }
    0
}

#[cfg(target_os = "linux")]
fn active_scale_for_widget(widget: *mut c_void) -> f64 {
    let widget_scale = if !widget.is_null() { unsafe { gtk_widget_get_scale_factor(widget) } } else { 0 };
    if widget_scale >= 1 {
        widget_scale as f64
    } else {
        crate::movement::get_target_dashboard_screen(None).scale_factor.max(1.0)
    }
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_button_press(widget: *mut c_void, event: *mut GdkEventButton, user_data: *mut c_void) -> i32 {
    let account_index = user_data as usize as u32;
    if !event.is_null() {
        let scale = active_scale_for_widget(widget);
        let (rx, ry) = unsafe { (logical_root_to_physical((*event).x_root, scale), logical_root_to_physical((*event).y_root, scale)) };
        let button = unsafe { (*event).button };
        if button == 1 {
            crate::operating_systems::linux::cursor::update_wayland_cursor(Some((rx, ry)), true);
            crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().handle_mouse_down(account_index, rx, ry);
        } else if button == 3 {
            crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().handle_right_click(account_index, rx as f64, ry as f64);
        }
    }
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_motion_notify(widget: *mut c_void, event: *mut GdkEventMotion, user_data: *mut c_void) -> i32 {
    let account_index = user_data as usize as u32;
    if !event.is_null() {
        let scale = active_scale_for_widget(widget);
        let (rx, ry) = unsafe { (logical_root_to_physical((*event).x_root, scale), logical_root_to_physical((*event).y_root, scale)) };
        let lbtn_down = unsafe { ((*event).state & (1 << 8)) != 0 };
        crate::operating_systems::linux::cursor::update_wayland_cursor(Some((rx, ry)), lbtn_down);
        crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global().handle_mouse_move(account_index, rx, ry);
    }
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_button_release(widget: *mut c_void, event: *mut GdkEventButton, user_data: *mut c_void) -> i32 {
    let account_index = user_data as usize as u32;
    if !event.is_null() && unsafe { (*event).button } == 1 {
        let scale = active_scale_for_widget(widget);
        let (rx, ry) = unsafe { (logical_root_to_physical((*event).x_root, scale), logical_root_to_physical((*event).y_root, scale)) };
        crate::operating_systems::linux::cursor::update_wayland_cursor(Some((rx, ry)), false);
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

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl LinuxGtkWindowController {
    /// Creates a transparent, borderless, floating GTK window scaled properly for HiDPI
    #[cfg(target_os = "linux")]
    pub fn create_floating_bubble_window(account_index: u32, x_px: i32, y_px: i32, width_px: u32, height_px: u32, scale_factor: f64) -> *mut c_void {
        unsafe {
            if gtk_init_check(std::ptr::null_mut(), std::ptr::null_mut()) == 0 {
                return std::ptr::null_mut();
            }
            let window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
            if window.is_null() {
                return std::ptr::null_mut();
            }
            gtk_widget_set_app_paintable(window, 1);
            gtk_window_set_decorated(window, 0);
            gtk_window_set_type_hint(window, GDK_WINDOW_TYPE_HINT_UTILITY);
            gtk_window_set_accept_focus(window, 0);
            gtk_window_set_focus_on_map(window, 0);
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
            let css_provider = gtk_css_provider_new();
            if !css_provider.is_null() {
                let css = b"window { background-color: transparent; background: transparent; border: none; }\0";
                gtk_css_provider_load_from_data(css_provider, css.as_ptr() as *const _, -1, std::ptr::null_mut());
                let style_ctx = gtk_widget_get_style_context(window);
                if !style_ctx.is_null() {
                    gtk_style_context_add_provider(style_ctx, css_provider, GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);
                }
                g_object_unref(css_provider);
            }
            let logical_w = physical_size_to_logical(width_px, scale_factor);
            let logical_h = physical_size_to_logical(height_px, scale_factor);
            gtk_widget_set_size_request(window, logical_w, logical_h);
            gtk_window_resize(window, logical_w, logical_h);
            gtk_widget_realize(window);
            let gdk_win = gtk_widget_get_window(window);
            if !gdk_win.is_null() {
                let display = gdk_display_get_default();
                if !display.is_null() {
                    let cursor = gdk_cursor_new_from_name(display, c"pointer".as_ptr());
                    if !cursor.is_null() {
                        gdk_window_set_cursor(gdk_win, cursor);
                        g_object_unref(cursor);
                    }
                }
            }
            gtk_widget_add_events(window, GDK_BUTTON_PRESS_MASK | GDK_BUTTON_RELEASE_MASK | GDK_POINTER_MOTION_MASK | GDK_ENTER_NOTIFY_MASK | GDK_LEAVE_NOTIFY_MASK);
            let user_data = account_index as usize as *mut c_void;
            g_signal_connect_data(window, c"draw".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_draw as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, c"button-press-event".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_button_press as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, c"motion-notify-event".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_motion_notify as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, c"button-release-event".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_button_release as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, c"enter-notify-event".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_enter_notify as *const ())), user_data, std::ptr::null_mut(), 0);
            g_signal_connect_data(window, c"leave-notify-event".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_leave_notify as *const ())), user_data, std::ptr::null_mut(), 0);
            let logical_x = physical_coord_to_logical(x_px - BUBBLE_CANVAS_PAD_X, scale_factor);
            let logical_y = physical_coord_to_logical(y_px - BUBBLE_CANVAS_PAD_Y, scale_factor);
            gtk_window_move(window, logical_x, logical_y);
            window
        }
    }

    /// Fallback for non-Linux compilation
    #[cfg(not(target_os = "linux"))]
    pub fn create_floating_bubble_window(_account_index: u32, _x_px: i32, _y_px: i32, _width_px: u32, _height_px: u32, _scale_factor: f64) -> *mut c_void {
        std::ptr::null_mut()
    }

    /// Applies circular input shape mask so transparent canvas padding passes clicks through
    #[cfg(target_os = "linux")]
    pub fn update_window_input_shape(window: *mut c_void, canvas_w_px: u32, canvas_h_px: u32, bubble_size_px: u32, unread_count: u32, is_left_edge: bool, scale_factor: f64) {
        if window.is_null() {
            return;
        }
        unsafe {
            let gdk_win = gtk_widget_get_window(window);
            if gdk_win.is_null() {
                return;
            }
            let logical_w = physical_size_to_logical(canvas_w_px, scale_factor);
            let logical_h = physical_size_to_logical(canvas_h_px, scale_factor);
            let logical_bubble = physical_size_to_logical(bubble_size_px, scale_factor);
            let rects = compute_bubble_input_rects(logical_w, logical_h, logical_bubble, unread_count, is_left_edge);
            let region = cairo_region_create();
            if !region.is_null() {
                for r in &rects {
                    cairo_region_union_rectangle(region, r as *const CairoRectangleInt);
                }
                gdk_window_input_shape_combine_region(gdk_win, region, 0, 0);
                cairo_region_destroy(region);
            }
        }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn update_window_input_shape(_window: *mut c_void, _canvas_w_px: u32, _canvas_h_px: u32, _bubble_size_px: u32, _unread_count: u32, _is_left_edge: bool, _scale_factor: f64) {}

    /// Updates window Cairo surface with tiny-skia rendered BGRA/ARGB buffer and HiDPI device scale
    #[cfg(target_os = "linux")]
    pub fn update_window_surface(window: *mut c_void, bgra_data: &[u8], width: usize, height: usize, scale_factor: f64) {
        if window.is_null() || bgra_data.is_empty() || width == 0 || height == 0 {
            return;
        }
        let key = window as usize;
        if let Ok(mut cache) = SURFACE_CACHE.lock() {
            cache.insert(key, (bgra_data.to_vec(), width, height, scale_factor));
        }
        unsafe { gtk_widget_queue_draw(window); }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn update_window_surface(_window: *mut c_void, _bgra_data: &[u8], _width: usize, _height: usize, _scale_factor: f64) {}

    /// Resizes GTK window canvas dimensions in logical pixels derived from physical pixels
    #[cfg(target_os = "linux")]
    pub fn resize_window(window: *mut c_void, width_px: u32, height_px: u32, scale_factor: f64) {
        if window.is_null() {
            return;
        }
        let logical_w = physical_size_to_logical(width_px, scale_factor);
        let logical_h = physical_size_to_logical(height_px, scale_factor);
        unsafe {
            gtk_widget_set_size_request(window, logical_w, logical_h);
            gtk_window_resize(window, logical_w, logical_h);
            gtk_widget_queue_draw(window);
        }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn resize_window(_window: *mut c_void, _width_px: u32, _height_px: u32, _scale_factor: f64) {}

    /// Sets window top-left position coordinates converted from physical pixels to logical GDK coords
    #[cfg(target_os = "linux")]
    pub fn set_window_position(window: *mut c_void, x_px: i32, y_px: i32, scale_factor: f64) {
        if window.is_null() {
            return;
        }
        let logical_x = physical_coord_to_logical(x_px - BUBBLE_CANVAS_PAD_X, scale_factor);
        let logical_y = physical_coord_to_logical(y_px - BUBBLE_CANVAS_PAD_Y, scale_factor);
        unsafe { gtk_window_move(window, logical_x, logical_y); }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn set_window_position(_window: *mut c_void, _x_px: i32, _y_px: i32, _scale_factor: f64) {}

    /// Sets window opacity (0.0 to 1.0)
    #[cfg(target_os = "linux")]
    pub fn set_window_opacity(window: *mut c_void, opacity: f64) {
        if window.is_null() {
            return;
        }
        unsafe { gtk_widget_set_opacity(window, opacity); }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn set_window_opacity(_window: *mut c_void, _opacity: f64) {}

    /// Shows or hides GTK window
    #[cfg(target_os = "linux")]
    pub fn set_window_visible(window: *mut c_void, visible: bool) {
        if window.is_null() {
            return;
        }
        unsafe {
            if visible {
                gtk_widget_show_all(window);
                gtk_widget_queue_draw(window);
            } else {
                gtk_widget_hide(window);
            }
        }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn set_window_visible(_window: *mut c_void, _visible: bool) {}

    /// Sets GTK window always-on-top hint
    #[cfg(target_os = "linux")]
    pub fn set_window_keep_above(window: *mut c_void, enabled: bool) {
        if window.is_null() {
            return;
        }
        unsafe { gtk_window_set_keep_above(window, if enabled { 1 } else { 0 }); }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn set_window_keep_above(_window: *mut c_void, _enabled: bool) {}

    /// Raises GTK window above sibling windows without stealing input focus
    #[cfg(target_os = "linux")]
    pub fn raise_window(window: *mut c_void) {
        if window.is_null() {
            return;
        }
        unsafe {
            gtk_window_set_keep_above(window, 1);
            let gdk_win = gtk_widget_get_window(window);
            if !gdk_win.is_null() {
                gdk_window_raise(gdk_win);
            }
        }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn raise_window(_window: *mut c_void) {}

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
        unsafe { gtk_widget_destroy(window); }
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
        LinuxGtkWindowController::update_window_surface(ptr, &[], 0, 0, 2.0);
        LinuxGtkWindowController::update_window_input_shape(ptr, 172, 136, 112, 3, true, 2.0);
        LinuxGtkWindowController::set_window_position(ptr, 0, 0, 2.0);
        LinuxGtkWindowController::resize_window(ptr, 84, 84, 2.0);
        LinuxGtkWindowController::set_window_visible(ptr, true);
        LinuxGtkWindowController::raise_window(ptr);
        LinuxGtkWindowController::destroy_window(ptr);
    }

    #[test]
    fn test_hidpi_physical_logical_roundtrip() {
        assert_eq!(physical_size_to_logical(112, 2.0), 56);
        assert_eq!(physical_coord_to_logical(200, 2.0), 100);
        assert_eq!(logical_root_to_physical(100.0, 2.0), 200);
        assert_eq!(physical_size_to_logical(84, 1.5), 56);
        assert_eq!(logical_root_to_physical(100.0, 1.5), 150);
    }

    #[test]
    fn test_compute_bubble_input_rects_circular_shape() {
        let no_badge = compute_bubble_input_rects(116, 80, 56, 0, true);
        let with_badge = compute_bubble_input_rects(116, 80, 56, 5, true);
        assert!(!no_badge.is_empty());
        assert_eq!(with_badge.len(), no_badge.len() + 1);
        assert!(no_badge.iter().all(|r| r.x >= 0 && r.width <= 62));
    }
}
