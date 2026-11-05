#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct SIZE {
    pub cx: i32,
    pub cy: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct MONITORINFO {
    pub cb_size: u32,
    pub rc_monitor: RECT,
    pub rc_work: RECT,
    pub dw_flags: u32,
}

pub const MONITOR_DEFAULTTONEAREST: u32 = 2;
pub const SM_CXSCREEN: i32 = 0;
pub const SM_CYSCREEN: i32 = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BLENDFUNCTION {
    pub blend_op: u8,
    pub blend_flags: u8,
    pub source_constant_alpha: u8,
    pub alpha_format: u8,
}

#[repr(C)]
pub struct TRACKMOUSEEVENT {
    pub cb_size: u32,
    pub dw_flags: u32,
    pub hwnd_track: *mut std::ffi::c_void,
    pub dw_hover_time: u32,
}

#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub bi_size: u32,
    pub bi_width: i32,
    pub bi_height: i32,
    pub bi_planes: u16,
    pub bi_bit_count: u16,
    pub bi_compression: u32,
    pub bi_size_image: u32,
    pub bi_x_pels_per_meter: i32,
    pub bi_y_pels_per_meter: i32,
    pub bi_clr_used: u32,
    pub bi_clr_important: u32,
}

#[repr(C)]
pub struct RGBQUAD {
    pub rgb_blue: u8,
    pub rgb_green: u8,
    pub rgb_red: u8,
    pub rgb_reserved: u8,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub bmi_header: BITMAPINFOHEADER,
    pub bmi_colors: [RGBQUAD; 1],
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cb_size: u32,
    pub style: u32,
    pub lpfn_wnd_proc: Option<unsafe extern "system" fn(*mut std::ffi::c_void, u32, usize, isize) -> isize>,
    pub cb_cls_extra: i32,
    pub cb_wnd_extra: i32,
    pub h_instance: *mut std::ffi::c_void,
    pub h_icon: *mut std::ffi::c_void,
    pub h_cursor: *mut std::ffi::c_void,
    pub hbr_background: *mut std::ffi::c_void,
    pub lpsz_menu_name: *const u16,
    pub lpsz_class_name: *const u16,
    pub h_icon_sm: *mut std::ffi::c_void,
}

// Win32 Window Styles & Flags
pub const WS_EX_LAYERED: u32 = 0x00080000;
pub const WS_EX_TOPMOST: u32 = 0x00000008;
pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
pub const WS_POPUP: u32 = 0x80000000;

pub const CS_HREDRAW: u32 = 0x0002;
pub const CS_VREDRAW: u32 = 0x0001;

pub const SW_SHOWNOACTIVATE: i32 = 4;
pub const SW_HIDE: i32 = 0;

pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;

pub const ULW_ALPHA: u32 = 0x00000002;
pub const AC_SRC_OVER: u8 = 0x00;
pub const AC_SRC_ALPHA: u8 = 0x01;
pub const DIB_RGB_COLORS: u32 = 0;
pub const BI_RGB: u32 = 0;

pub const IDC_ARROW: usize = 32512;
pub const IDC_HAND: usize = 32649;

pub const GWLP_USERDATA: i32 = -21;

pub const WM_DESTROY: u32 = 0x0002;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_ERASEBKGND: u32 = 0x0014;
pub const WM_SETCURSOR: u32 = 0x0020;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_NCDESTROY: u32 = 0x0082;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_CAPTURECHANGED: u32 = 0x0215;
pub const WM_MOUSELEAVE: u32 = 0x02A3;

pub const TIMER_ID_PULSE: usize = 1001;
pub const TIMER_ID_SPRING: usize = 1002;
pub const TIMER_ID_HOVER_LEAVE: usize = 1003;

pub const TME_LEAVE: u32 = 0x00000002;
pub const CLASS_NAME: &str = "DesktopBubbleChat_NativeBubble\0";

// Transparent canvas padding extending HWND window dimensions.
// Allows unread badge to sit outside bubble body without clipping.
pub const BUBBLE_CANVAS_PAD_X: i32 = 30;
pub const BUBBLE_CANVAS_PAD_Y: i32 = 12;
