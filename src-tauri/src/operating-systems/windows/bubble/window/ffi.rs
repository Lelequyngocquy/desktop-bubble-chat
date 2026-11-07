use super::types::*;

#[link(name = "user32")]
unsafe extern "system" {
    pub fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: *mut std::ffi::c_void,
        hMenu: *mut std::ffi::c_void,
        hInstance: *mut std::ffi::c_void,
        lpParam: *mut std::ffi::c_void,
    ) -> *mut std::ffi::c_void;
    pub fn DestroyWindow(hWnd: *mut std::ffi::c_void) -> i32;
    pub fn ShowWindow(hWnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;
    pub fn SetWindowPos(
        hWnd: *mut std::ffi::c_void,
        hWndInsertAfter: *mut std::ffi::c_void,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;
    pub fn IsWindowVisible(hWnd: *mut std::ffi::c_void) -> i32;
    pub fn DefWindowProcW(
        hWnd: *mut std::ffi::c_void,
        Msg: u32,
        wParam: usize,
        lParam: isize,
    ) -> isize;
    pub fn UpdateLayeredWindow(
        hWnd: *mut std::ffi::c_void,
        hdcDst: *mut std::ffi::c_void,
        pptDst: *const POINT,
        psize: *const SIZE,
        hdcSrc: *mut std::ffi::c_void,
        pptSrc: *const POINT,
        crKey: u32,
        pblend: *const BLENDFUNCTION,
        dwFlags: u32,
    ) -> i32;
    pub fn GetDC(hWnd: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    pub fn ReleaseDC(hWnd: *mut std::ffi::c_void, hDC: *mut std::ffi::c_void) -> i32;
    pub fn LoadCursorW(hInstance: *mut std::ffi::c_void, lpCursorName: *const u16) -> *mut std::ffi::c_void;
    pub fn SetCursor(hCursor: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    pub fn SetCapture(hWnd: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    pub fn ReleaseCapture() -> i32;
    pub fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    pub fn GetWindowLongPtrW(hWnd: *mut std::ffi::c_void, nIndex: i32) -> isize;
    pub fn SetWindowLongPtrW(hWnd: *mut std::ffi::c_void, nIndex: i32, dwNewLong: isize) -> isize;
    pub fn TrackMouseEvent(lpEventTrack: *mut TRACKMOUSEEVENT) -> i32;
    pub fn SetTimer(
        hWnd: *mut std::ffi::c_void,
        nIDEvent: usize,
        uElapse: u32,
        lpTimerFunc: Option<unsafe extern "system" fn(*mut std::ffi::c_void, u32, usize, u32)>,
    ) -> usize;
    pub fn KillTimer(hWnd: *mut std::ffi::c_void, uIDEvent: usize) -> i32;
    pub fn GetSystemMetrics(nIndex: i32) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn CreateCompatibleDC(hdc: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    pub fn DeleteDC(hdc: *mut std::ffi::c_void) -> i32;
    pub fn CreateDIBSection(
        hdc: *mut std::ffi::c_void,
        pbmi: *const BITMAPINFO,
        usage: u32,
        ppvBits: *mut *mut u8,
        hSection: *mut std::ffi::c_void,
        offset: u32,
    ) -> *mut std::ffi::c_void;
    pub fn SelectObject(hdc: *mut std::ffi::c_void, h: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    pub fn DeleteObject(ho: *mut std::ffi::c_void) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> *mut std::ffi::c_void;
}

/// Retrieves target screen metrics for bubbles, locked to Dashboard's current or last monitor
pub fn get_screen_metrics_for_hwnd(_hwnd: *mut std::ffi::c_void) -> crate::movement::ScreenMetrics {
    crate::movement::get_target_dashboard_screen(None)
}
