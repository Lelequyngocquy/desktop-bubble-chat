#[allow(unused_imports)]
use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GdkRectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[cfg(target_os = "linux")]
#[link(name = "gtk-3")]
unsafe extern "C" {
    pub fn gtk_init_check(argc: *mut i32, argv: *mut *mut *mut std::ffi::c_char) -> i32;
    pub fn gtk_window_new(type_: i32) -> *mut c_void;
    pub fn gtk_widget_set_app_paintable(widget: *mut c_void, app_paintable: i32);
    pub fn gtk_window_set_decorated(window: *mut c_void, setting: i32);
    pub fn gtk_window_set_skip_taskbar_hint(window: *mut c_void, setting: i32);
    pub fn gtk_window_set_skip_pager_hint(window: *mut c_void, setting: i32);
    pub fn gtk_window_set_keep_above(window: *mut c_void, setting: i32);
    pub fn gtk_window_stick(window: *mut c_void);
    pub fn gtk_window_move(window: *mut c_void, x: i32, y: i32);
    pub fn gtk_widget_show_all(widget: *mut c_void);
    pub fn gtk_widget_hide(widget: *mut c_void);
    pub fn gtk_widget_queue_draw(widget: *mut c_void);
    pub fn gtk_widget_get_screen(widget: *mut c_void) -> *mut c_void;
    pub fn gdk_screen_get_rgba_visual(screen: *mut c_void) -> *mut c_void;
    pub fn gtk_widget_set_visual(widget: *mut c_void, visual: *mut c_void);
    pub fn gtk_widget_add_events(widget: *mut c_void, events: i32);
    pub fn gtk_widget_set_size_request(widget: *mut c_void, width: i32, height: i32);
    pub fn gtk_window_resize(window: *mut c_void, width: i32, height: i32);
    pub fn gtk_window_set_type_hint(window: *mut c_void, hint: i32);
    pub fn gtk_window_set_accept_focus(window: *mut c_void, setting: i32);
    pub fn gtk_window_set_focus_on_map(window: *mut c_void, setting: i32);
    pub fn gtk_css_provider_new() -> *mut c_void;
    pub fn gtk_css_provider_load_from_data(css_provider: *mut c_void, data: *const std::ffi::c_char, length: isize, error: *mut *mut c_void) -> i32;
    pub fn gtk_widget_get_style_context(widget: *mut c_void) -> *mut c_void;
    pub fn gtk_style_context_add_provider(context: *mut c_void, provider: *mut c_void, priority: u32);
    pub fn gtk_widget_destroy(widget: *mut c_void);
}

#[cfg(target_os = "linux")]
#[link(name = "gobject-2.0")]
unsafe extern "C" {
    pub fn g_object_unref(object: *mut c_void);
    pub fn g_signal_connect_data(
        instance: *mut c_void,
        detailed_signal: *const std::ffi::c_char,
        c_handler: Option<unsafe extern "C" fn()>,
        data: *mut c_void,
        destroy_data: *mut c_void,
        connect_flags: i32,
    ) -> std::ffi::c_ulong;
}

#[cfg(target_os = "linux")]
#[link(name = "cairo")]
unsafe extern "C" {
    pub fn cairo_image_surface_create_for_data(
        data: *mut u8,
        format: i32,
        width: i32,
        height: i32,
        stride: i32,
    ) -> *mut c_void;
    pub fn cairo_surface_destroy(surface: *mut c_void);
    pub fn cairo_set_source_surface(cr: *mut c_void, surface: *mut c_void, x: f64, y: f64);
    pub fn cairo_set_operator(cr: *mut c_void, op: i32);
    pub fn cairo_paint(cr: *mut c_void);
}
