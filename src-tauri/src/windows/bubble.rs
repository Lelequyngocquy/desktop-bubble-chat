use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
#[link(name = "comctl32")]
unsafe extern "system" {
    fn SetWindowSubclass(
        hWnd: *mut std::ffi::c_void,
        pfnSubclass: unsafe extern "system" fn(
            *mut std::ffi::c_void,
            u32,
            usize,
            isize,
            usize,
            usize,
        ) -> isize,
        uIdSubclass: usize,
        dwRefData: usize,
    ) -> i32;

    fn DefSubclassProc(
        hWnd: *mut std::ffi::c_void,
        uMsg: u32,
        wParam: usize,
        lParam: isize,
    ) -> isize;
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn SetWindowPos(
        hWnd: *mut std::ffi::c_void,
        hWndInsertAfter: *mut std::ffi::c_void,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> i32;
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct MINMAXINFO {
    pt_reserved: POINT,
    pt_max_size: POINT,
    pt_max_position: POINT,
    pt_min_track_size: POINT,
    pt_max_track_size: POINT,
}

#[cfg(target_os = "windows")]
const WM_GETMINMAXINFO: u32 = 0x0024;
#[cfg(target_os = "windows")]
const SWP_NOMOVE: u32 = 0x0002;
#[cfg(target_os = "windows")]
const SWP_NOZORDER: u32 = 0x0004;
#[cfg(target_os = "windows")]
const SWP_NOACTIVATE: u32 = 0x0010;
#[cfg(target_os = "windows")]
const SWP_FRAMECHANGED: u32 = 0x0020;

#[cfg(target_os = "windows")]
unsafe extern "system" fn bubble_subclass_proc(
    hwnd: *mut std::ffi::c_void,
    msg: u32,
    wparam: usize,
    lparam: isize,
    _id: usize,
    _data: usize,
) -> isize {
    // Cho phép kích thước cửa sổ thu nhỏ tự do dưới SM_CXMINTRACK (136px) của Windows
    if msg == WM_GETMINMAXINFO && lparam != 0 {
        unsafe {
            let mmi = &mut *(lparam as *mut MINMAXINFO);
            mmi.pt_min_track_size.x = 1;
            mmi.pt_min_track_size.y = 1;
        }
        return 0;
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

/// Cấu hình sự kiện cho cửa sổ chính (Home Page): Chặn thoát app khi ấn nút X, ẩn về tray
pub fn setup_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        let main_clone = main_window.clone();
        main_window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = main_clone.hide();
            }
        });
    }
}

/// Cấu hình cửa sổ bong bóng: Cho phép kích thước nhỏ hơn SM_CXMINTRACK (136px) của Windows
#[cfg(target_os = "windows")]
pub fn setup_bubble_subclass(bubble: &tauri::WebviewWindow) {
    if let Ok(hwnd) = bubble.hwnd() {
        let raw_hwnd = hwnd.0 as *mut std::ffi::c_void;
        unsafe {
            SetWindowSubclass(
                raw_hwnd,
                bubble_subclass_proc,
                101, // Subclass ID
                0,
            );
        }
        if let Ok(Some(monitor)) = bubble.current_monitor() {
            let scale = monitor.scale_factor();
            let size = (64.0 * scale) as i32;
            unsafe {
                SetWindowPos(
                    raw_hwnd,
                    std::ptr::null_mut(),
                    0,
                    0,
                    size,
                    size,
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
            }
        }
    }
}

/// Cấu hình cửa sổ bong bóng: Cho phép kích thước nhỏ hơn SM_CXMINTRACK (136px) của Windows
#[cfg(target_os = "windows")]
pub fn setup_bubble_window(app: &AppHandle) {
    for idx in 1..=5 {
        let label = format!("bubble-{:02}", idx);
        if let Some(bubble) = app.get_webview_window(&label) {
            setup_bubble_subclass(&bubble);
        }
    }
}
