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
    pub fn ValidateRect(hWnd: *mut std::ffi::c_void, lpRect: *const RECT) -> i32;
    pub fn IsWindow(hWnd: *mut std::ffi::c_void) -> i32;
    pub fn GetWindowRect(hWnd: *mut std::ffi::c_void, lpRect: *mut RECT) -> i32;
    pub fn MonitorFromWindow(hwnd: *mut std::ffi::c_void, dwFlags: u32) -> *mut std::ffi::c_void;
    pub fn GetMonitorInfoW(hMonitor: *mut std::ffi::c_void, lpmi: *mut MONITORINFO) -> i32;
    pub fn GetDesktopWindow() -> *mut std::ffi::c_void;
    pub fn GetShellWindow() -> *mut std::ffi::c_void;
    pub fn GetForegroundWindow() -> *mut std::ffi::c_void;
    pub fn GetClassNameW(hWnd: *mut std::ffi::c_void, lpClassName: *mut u16, nMaxCount: i32) -> i32;
    pub fn SetWinEventHook(
        eventMin: u32,
        eventMax: u32,
        hmodWinEventProc: *mut std::ffi::c_void,
        pfnWinEventProc: Option<WinEventProc>,
        idProcess: u32,
        idThread: u32,
        dwFlags: u32,
    ) -> *mut std::ffi::c_void;
    pub fn UnhookWinEvent(hWinEventHook: *mut std::ffi::c_void) -> i32;
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

/// Checks if a window class belongs to Windows Desktop or Taskbar shell
pub fn is_shell_window_class(class_name: &str) -> bool {
    matches!(class_name, "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd")
}

/// Pure predicate determining whether window style and geometry represent a true fullscreen window
pub fn is_fullscreen_geometry(style: u32, win_rect: RECT, mon_rect: RECT) -> bool {
    let has_caption = (style & WS_CAPTION) == WS_CAPTION;
    let has_thickframe = (style & WS_THICKFRAME) != 0;
    if has_caption || has_thickframe {
        return false;
    }
    win_rect.left <= mon_rect.left
        && win_rect.top <= mon_rect.top
        && win_rect.right >= mon_rect.right
        && win_rect.bottom >= mon_rect.bottom
}

/// Checks if a given Win32 HWND is a true fullscreen application window (excluding Desktop/Shell)
///
/// # Safety
/// Caller should pass a valid or null Win32 HWND.
pub unsafe fn is_window_fullscreen(hwnd: *mut std::ffi::c_void) -> bool {
    unsafe {
        if hwnd.is_null() || IsWindow(hwnd) == 0 {
            return false;
        }
        if hwnd == GetDesktopWindow() || hwnd == GetShellWindow() {
            return false;
        }
        let mut class_buf = [0u16; 64];
        let class_len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), class_buf.len() as i32);
        if class_len > 0 {
            let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);
            if is_shell_window_class(&class_name) {
                return false;
            }
        }
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let mut win_rect = RECT::default();
        if GetWindowRect(hwnd, &mut win_rect) == 0 {
            return false;
        }
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if hmon.is_null() {
            return false;
        }
        let mut mon_info = MONITORINFO { cb_size: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if GetMonitorInfoW(hmon, &mut mon_info) == 0 {
            return false;
        }
        is_fullscreen_geometry(style, win_rect, mon_info.rc_monitor)
    }
}

/// Retrieves target screen metrics for bubbles, locked to Dashboard's current or last monitor
pub fn get_screen_metrics_for_hwnd(_hwnd: *mut std::ffi::c_void) -> crate::movement::ScreenMetrics {
    crate::movement::get_target_dashboard_screen(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fullscreen_geometry_detection() {
        let mon = RECT { left: 0, top: 0, right: 1920, bottom: 1080 };
        let normal_chrome = RECT { left: 100, top: 100, right: 1400, bottom: 900 };
        assert!(!is_fullscreen_geometry(WS_CAPTION | WS_THICKFRAME, normal_chrome, mon));
        let max_chrome_autohide = RECT { left: -8, top: -8, right: 1928, bottom: 1088 };
        assert!(!is_fullscreen_geometry(WS_CAPTION | WS_THICKFRAME, max_chrome_autohide, mon));
        let borderless_resizable = RECT { left: 0, top: 0, right: 1920, bottom: 1080 };
        assert!(!is_fullscreen_geometry(WS_THICKFRAME, borderless_resizable, mon));
        let f11_chrome = RECT { left: 0, top: 0, right: 1920, bottom: 1080 };
        assert!(is_fullscreen_geometry(WS_POPUP, f11_chrome, mon));
        let small_popup = RECT { left: 400, top: 300, right: 800, bottom: 600 };
        assert!(!is_fullscreen_geometry(WS_POPUP, small_popup, mon));
    }

    #[test]
    fn test_shell_window_class_filtering() {
        assert!(is_shell_window_class("Progman"));
        assert!(is_shell_window_class("WorkerW"));
        assert!(is_shell_window_class("Shell_TrayWnd"));
        assert!(is_shell_window_class("Shell_SecondaryTrayWnd"));
        assert!(!is_shell_window_class("Chrome_WidgetWin_1"));
    }
}
