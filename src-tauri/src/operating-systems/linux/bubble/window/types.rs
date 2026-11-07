pub const BUBBLE_CANVAS_PAD_X: i32 = 18;
pub const BUBBLE_CANVAS_PAD_Y: i32 = 18;

pub const GTK_WINDOW_TOPLEVEL: i32 = 0;
pub const GTK_WINDOW_POPUP: i32 = 1;

pub const CAIRO_FORMAT_ARGB32: i32 = 0;
pub const CAIRO_OPERATOR_SOURCE: i32 = 1;

pub const GDK_POINTER_MOTION_MASK: i32 = 1 << 2;
pub const GDK_BUTTON_PRESS_MASK: i32 = 1 << 8;
pub const GDK_BUTTON_RELEASE_MASK: i32 = 1 << 9;
pub const GDK_ENTER_NOTIFY_MASK: i32 = 1 << 12;
pub const GDK_LEAVE_NOTIFY_MASK: i32 = 1 << 13;

#[repr(C)]
pub struct GdkEventButton {
    pub type_: i32,
    pub window: *mut std::ffi::c_void,
    pub send_event: i8,
    pub time: u32,
    pub x: f64,
    pub y: f64,
    pub axes: *mut f64,
    pub state: u32,
    pub button: u32,
    pub device: *mut std::ffi::c_void,
    pub x_root: f64,
    pub y_root: f64,
}

#[repr(C)]
pub struct GdkEventMotion {
    pub type_: i32,
    pub window: *mut std::ffi::c_void,
    pub send_event: i8,
    pub time: u32,
    pub x: f64,
    pub y: f64,
    pub axes: *mut f64,
    pub state: u32,
    pub is_hint: i16,
    pub device: *mut std::ffi::c_void,
    pub x_root: f64,
    pub y_root: f64,
}
