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
}
