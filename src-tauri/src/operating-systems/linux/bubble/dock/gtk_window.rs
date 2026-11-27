#[cfg(target_os = "linux")]
use std::collections::HashMap;
use std::ffi::c_void;
#[cfg(target_os = "linux")]
use std::sync::{LazyLock, Mutex};

#[allow(unused_imports)]
use crate::operating_systems::linux::bubble::window::ffi::*;
#[cfg(target_os = "linux")]
use crate::operating_systems::linux::bubble::window::gtk_window::{
    logical_root_to_physical, physical_coord_to_logical,
};
use crate::operating_systems::linux::bubble::window::gtk_window::physical_size_to_logical;
#[allow(unused_imports)]
use crate::operating_systems::linux::bubble::window::types::*;
use crate::presentation::bubble::dock::{
    hit_test_dock_droplet, ActionDockLayout, DockActionId, DockTooltipPillRect,
};

pub struct LinuxActionDockGtkController;

/// Builds 6 circular scanline rectangle clusters for GTK input shape click-through across gaps
pub fn compute_dock_input_rects(
    layout: &ActionDockLayout,
    is_visible: bool,
) -> Vec<CairoRectangleInt> {
    if !is_visible {
        return Vec::new();
    }
    let scale = if layout.scale_factor.is_finite() && layout.scale_factor > 0.0 { layout.scale_factor } else { 1.0 };
    let logical_w = physical_size_to_logical(layout.canvas_width, scale);
    let logical_h = physical_size_to_logical(layout.canvas_height, scale);
    let mut rects = Vec::new();
    let step = 2i32;
    for slot in &layout.slots {
        let cx = (slot.local_cx as f64) / scale;
        let cy = (slot.local_cy as f64) / scale;
        let r = ((slot.radius as f64) / scale) + 1.5;
        let min_y = ((cy - r).floor() as i32).max(0);
        let max_y = ((cy + r).ceil() as i32).min(logical_h.saturating_sub(1));
        let mut y = min_y;
        while y <= max_y {
            let dy = (y as f64 + 0.5) - cy;
            let span_sq = r * r - dy * dy;
            if span_sq > 0.0 {
                let half_w = span_sq.sqrt();
                let x0 = ((cx - half_w).floor() as i32).max(0);
                let x1 = ((cx + half_w).ceil() as i32).min(logical_w);
                if x1 > x0 {
                    let h = (max_y - y + 1).min(step);
                    rects.push(CairoRectangleInt { x: x0, y, width: x1 - x0, height: h });
                }
            }
            y += step;
        }
    }
    rects
}

/// Evaluates which ActionDock droplet is hit by GDK top-down local logical coordinates
pub fn hit_test_linux_dock_local_logical(
    local_x: f64,
    local_y: f64,
    layout: &ActionDockLayout,
    is_visible: bool,
) -> Option<DockActionId> {
    if !is_visible {
        return None;
    }
    let scale = if layout.scale_factor.is_finite() && layout.scale_factor > 0.0 { layout.scale_factor } else { 1.0 };
    let px = (local_x * scale) as f32;
    let py = (local_y * scale) as f32;
    hit_test_dock_droplet(px, py, layout)
}

#[cfg(target_os = "linux")]
type DockTooltipOverlay = Option<(String, DockTooltipPillRect)>;
#[cfg(target_os = "linux")]
type DockSurfaceBufferEntry = (Vec<u8>, usize, usize, f64, DockTooltipOverlay);
#[cfg(target_os = "linux")]
static DOCK_SURFACE_CACHE: LazyLock<Mutex<HashMap<usize, DockSurfaceBufferEntry>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_dock_draw(widget: *mut c_void, cr: *mut c_void, _user_data: *mut c_void) -> i32 {
    let key = widget as usize;
    if let Ok(cache) = DOCK_SURFACE_CACHE.lock() {
        if let Some((bgra, width, height, scale, tooltip_opt)) = cache.get(&key) {
            unsafe {
                let surface = cairo_image_surface_create_for_data(bgra.as_ptr() as *mut u8, CAIRO_FORMAT_ARGB32, *width as i32, *height as i32, (*width * 4) as i32);
                if !surface.is_null() {
                    let s = if scale.is_finite() && *scale > 0.0 { *scale } else { 1.0 };
                    cairo_surface_set_device_scale(surface, s, s);
                    cairo_set_operator(cr, CAIRO_OPERATOR_SOURCE);
                    cairo_set_source_surface(cr, surface, 0.0, 0.0);
                    cairo_paint(cr);
                    cairo_surface_destroy(surface);
                    if let Some((label, pill)) = tooltip_opt
                        && let Ok(c_text) = std::ffi::CString::new(label.as_str()) {
                            cairo_set_operator(cr, 2);
                            cairo_select_font_face(cr, c"Sans".as_ptr(), 0, 0);
                            cairo_set_font_size(cr, 11.5);
                            let mut ext = CairoTextExtents::default();
                            cairo_text_extents(cr, c_text.as_ptr(), &mut ext);
                            let lx = (pill.x as f64) / s + ((pill.width as f64) / s - ext.width) * 0.5 - ext.x_bearing;
                            let ly = (pill.y as f64) / s + ((pill.height as f64) / s - ext.height) * 0.5 - ext.y_bearing;
                            cairo_set_source_rgba(cr, 1.0, 1.0, 1.0, pill.alpha as f64);
                            cairo_move_to(cr, lx, ly);
                            cairo_show_text(cr, c_text.as_ptr());
                        }
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
fn active_scale_for_dock_widget(widget: *mut c_void) -> f64 {
    let widget_scale = if !widget.is_null() { unsafe { gtk_widget_get_scale_factor(widget) } } else { 0 };
    if widget_scale >= 1 {
        widget_scale as f64
    } else {
        crate::movement::get_target_dashboard_screen(None).scale_factor.max(1.0)
    }
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_dock_motion_notify(widget: *mut c_void, event: *mut GdkEventMotion, _user_data: *mut c_void) -> i32 {
    if !event.is_null() {
        let scale = active_scale_for_dock_widget(widget);
        let (rx, ry, lx, ly, lbtn_down) = unsafe {
            (
                logical_root_to_physical((*event).x_root, scale),
                logical_root_to_physical((*event).y_root, scale),
                (*event).x,
                (*event).y,
                ((*event).state & (1 << 8)) != 0,
            )
        };
        crate::operating_systems::linux::cursor::update_wayland_cursor(Some((rx, ry)), lbtn_down);
        crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global()
            .handle_action_dock_mouse_move(lx, ly);
    }
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_dock_leave_notify(_widget: *mut c_void, _event: *mut c_void, _user_data: *mut c_void) -> i32 {
    crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global()
        .handle_action_dock_mouse_leave();
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_dock_button_press(widget: *mut c_void, event: *mut GdkEventButton, _user_data: *mut c_void) -> i32 {
    if !event.is_null() && unsafe { (*event).button } == 1 {
        let scale = active_scale_for_dock_widget(widget);
        let (rx, ry, lx, ly) = unsafe {
            (
                logical_root_to_physical((*event).x_root, scale),
                logical_root_to_physical((*event).y_root, scale),
                (*event).x,
                (*event).y,
            )
        };
        crate::operating_systems::linux::cursor::update_wayland_cursor(Some((rx, ry)), true);
        crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global()
            .handle_action_dock_mouse_down(lx, ly);
    }
    0
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn on_dock_button_release(widget: *mut c_void, event: *mut GdkEventButton, _user_data: *mut c_void) -> i32 {
    if !event.is_null() && unsafe { (*event).button } == 1 {
        let scale = active_scale_for_dock_widget(widget);
        let (rx, ry, lx, ly) = unsafe {
            (
                logical_root_to_physical((*event).x_root, scale),
                logical_root_to_physical((*event).y_root, scale),
                (*event).x,
                (*event).y,
            )
        };
        crate::operating_systems::linux::cursor::update_wayland_cursor(Some((rx, ry)), false);
        crate::operating_systems::linux::bubble::manager::NativeBubbleManager::global()
            .handle_action_dock_mouse_up(lx, ly);
    }
    0
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl LinuxActionDockGtkController {
    /// Creates a non-activating, transparent RGBA GTK popup window for the singleton ActionDock
    #[cfg(target_os = "linux")]
    pub fn create_action_dock_window(layout: &ActionDockLayout, is_always_on_top: bool) -> *mut c_void {
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
            gtk_window_set_type_hint(window, GDK_WINDOW_TYPE_HINT_UTILITY);
            gtk_window_set_accept_focus(window, 0);
            gtk_window_set_focus_on_map(window, 0);
            gtk_window_set_skip_taskbar_hint(window, 1);
            gtk_window_set_skip_pager_hint(window, 1);
            gtk_window_set_keep_above(window, if is_always_on_top { 1 } else { 0 });
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
            let scale = layout.scale_factor.max(1.0);
            let logical_w = physical_size_to_logical(layout.canvas_width, scale);
            let logical_h = physical_size_to_logical(layout.canvas_height, scale);
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
            gtk_widget_add_events(window, GDK_BUTTON_PRESS_MASK | GDK_BUTTON_RELEASE_MASK | GDK_POINTER_MOTION_MASK | GDK_LEAVE_NOTIFY_MASK);
            g_signal_connect_data(window, c"draw".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_dock_draw as *const ())), std::ptr::null_mut(), std::ptr::null_mut(), 0);
            g_signal_connect_data(window, c"motion-notify-event".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_dock_motion_notify as *const ())), std::ptr::null_mut(), std::ptr::null_mut(), 0);
            g_signal_connect_data(window, c"leave-notify-event".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_dock_leave_notify as *const ())), std::ptr::null_mut(), std::ptr::null_mut(), 0);
            g_signal_connect_data(window, c"button-press-event".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_dock_button_press as *const ())), std::ptr::null_mut(), std::ptr::null_mut(), 0);
            g_signal_connect_data(window, c"button-release-event".as_ptr(), Some(std::mem::transmute::<*const (), unsafe extern "C" fn()>(on_dock_button_release as *const ())), std::ptr::null_mut(), std::ptr::null_mut(), 0);
            let logical_x = physical_coord_to_logical(layout.window_x, scale);
            let logical_y = physical_coord_to_logical(layout.window_y, scale);
            gtk_window_move(window, logical_x, logical_y);
            window
        }
    }

    /// Fallback for non-Linux compilation
    #[cfg(not(target_os = "linux"))]
    pub fn create_action_dock_window(_layout: &ActionDockLayout, _is_always_on_top: bool) -> *mut c_void {
        std::ptr::null_mut()
    }

    /// Repositions and resizes ActionDock GTK window in logical GDK coordinates
    #[cfg(target_os = "linux")]
    pub fn reposition_and_resize_dock(window: *mut c_void, layout: &ActionDockLayout) {
        if window.is_null() {
            return;
        }
        let scale = layout.scale_factor.max(1.0);
        let logical_w = physical_size_to_logical(layout.canvas_width, scale);
        let logical_h = physical_size_to_logical(layout.canvas_height, scale);
        let logical_x = physical_coord_to_logical(layout.window_x, scale);
        let logical_y = physical_coord_to_logical(layout.window_y, scale);
        unsafe {
            gtk_widget_set_size_request(window, logical_w, logical_h);
            gtk_window_resize(window, logical_w, logical_h);
            gtk_window_move(window, logical_x, logical_y);
        }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn reposition_and_resize_dock(_window: *mut c_void, _layout: &ActionDockLayout) {}

    /// Applies 6-circle input shape mask so inter-droplet gaps remain click-through
    #[cfg(target_os = "linux")]
    pub fn update_dock_input_shape(window: *mut c_void, layout: &ActionDockLayout, is_visible: bool) {
        if window.is_null() {
            return;
        }
        unsafe {
            let gdk_win = gtk_widget_get_window(window);
            if gdk_win.is_null() {
                return;
            }
            let rects = compute_dock_input_rects(layout, is_visible);
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
    pub fn update_dock_input_shape(_window: *mut c_void, _layout: &ActionDockLayout, _is_visible: bool) {}

    /// Uploads BGRA pixel buffer plus optional tooltip label to Cairo cache and schedules GTK redraw
    #[cfg(target_os = "linux")]
    pub fn update_dock_surface(
        window: *mut c_void,
        bgra_data: &[u8],
        width: usize,
        height: usize,
        scale_factor: f64,
        tooltip: Option<(String, DockTooltipPillRect)>,
    ) {
        if window.is_null() || bgra_data.is_empty() || width == 0 || height == 0 {
            return;
        }
        let key = window as usize;
        if let Ok(mut cache) = DOCK_SURFACE_CACHE.lock() {
            cache.insert(key, (bgra_data.to_vec(), width, height, scale_factor, tooltip));
        }
        unsafe { gtk_widget_queue_draw(window); }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn update_dock_surface(
        _window: *mut c_void,
        _bgra_data: &[u8],
        _width: usize,
        _height: usize,
        _scale_factor: f64,
        _tooltip: Option<(String, DockTooltipPillRect)>,
    ) {}

    /// Destroys ActionDock GTK window and removes cached Cairo surface bytes
    #[cfg(target_os = "linux")]
    pub fn destroy_dock(window: *mut c_void) {
        if window.is_null() {
            return;
        }
        let key = window as usize;
        if let Ok(mut cache) = DOCK_SURFACE_CACHE.lock() {
            cache.remove(&key);
        }
        unsafe { gtk_widget_destroy(window); }
    }

    /// Fallback for non-Linux target
    #[cfg(not(target_os = "linux"))]
    pub fn destroy_dock(_window: *mut c_void) {}
}
