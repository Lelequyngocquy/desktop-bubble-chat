use tauri::{AppHandle, Emitter, Manager};

use crate::movement::{BubbleMovementEngine, BubblePhysicsConfig, BubbleRect, ScreenMetrics};

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

/// Cấu hình tất cả các cửa sổ bong bóng (bubble-01 -> bubble-05)
#[cfg(target_os = "windows")]
pub fn setup_bubble_window(app: &AppHandle) {
    for idx in 1..=5 {
        let label = format!("bubble-{:02}", idx);
        if let Some(bubble) = app.get_webview_window(&label) {
            setup_bubble_subclass(&bubble);
        }
    }
}

/// Trả về label định danh cửa sổ Bubble theo số thứ tự tài khoản
pub fn get_bubble_window_label(account_index: u32) -> String {
    format!("bubble-{:02}", account_index)
}

/// Tìm hoặc khởi tạo WebviewWindow nổi độc lập cho Bubble của tài khoản chỉ định
pub fn get_or_create_bubble_window(
    app: &AppHandle,
    account_index: u32,
) -> Result<tauri::WebviewWindow, String> {
    let label = get_bubble_window_label(account_index);

    // 1. Kiểm tra cửa sổ đã tồn tại
    if let Some(win) = app.get_webview_window(&label) {
        return Ok(win);
    }

    // 2. Khởi tạo WebviewWindow Bubble dự phòng nếu chưa có
    let url_str = format!("index.html?window=bubble&account={}", account_index);
    let window = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::App(url_str.into()),
    )
    .title(format!("DesktopBubbleChat - Bubble {:02}", account_index))
    .inner_size(64.0, 64.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .shadow(false)
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    setup_bubble_subclass(&window);

    Ok(window)
}

/// Lấy danh sách tọa độ của tất cả Bubble đang hiển thị dưới dạng BubbleRect
pub fn get_active_bubble_rects(
    app: &AppHandle,
    exclude_label: Option<&str>,
) -> Vec<BubbleRect> {
    let mut rects = Vec::new();

    for idx in 1..=5 {
        let label = get_bubble_window_label(idx);
        if let Some(exclude) = exclude_label {
            if label == exclude {
                continue;
            }
        }
        let win = app.get_webview_window(&label);

        if let Some(w) = win {
            if w.is_visible().unwrap_or(false) {
                if let (Ok(pos), Ok(size)) = (w.outer_position(), w.outer_size()) {
                    rects.push(BubbleRect {
                        account_index: idx,
                        x: pos.x,
                        y: pos.y,
                        width: size.width as i32,
                        height: size.height as i32,
                    });
                }
            }
        }
    }

    rects
}

/// Hiển thị và định vị Bubble cho tài khoản chỉ định với cơ chế xếp chống va chạm
pub fn ensure_bubble_positioned_and_visible(app: &AppHandle, account_index: u32) -> Result<(), String> {
    let bubble_window = get_or_create_bubble_window(app, account_index)?;
    let win_label = bubble_window.label().to_string();

    let monitor = app
        .get_webview_window("main")
        .and_then(|m| {
            if m.is_visible().unwrap_or(false) {
                m.current_monitor().ok().flatten()
            } else {
                None
            }
        })
        .or_else(|| bubble_window.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());

    if let Some(monitor) = monitor {
        let win_pos = bubble_window.outer_position().unwrap_or_default();
        let screen = ScreenMetrics::from_monitor(&monitor);
        let config = BubblePhysicsConfig::default();
        let bubble_w = (config.size * screen.scale_factor) as i32;

        let is_visible = bubble_window.is_visible().unwrap_or(false);
        let is_out_of_bounds = win_pos.x < screen.x - bubble_w
            || win_pos.x > screen.x + screen.width
            || (win_pos.x == 0 && win_pos.y == 0);

        if !is_visible || is_out_of_bounds {
            let other_bubbles = get_active_bubble_rects(app, Some(&win_label));
            let target_pos = BubbleMovementEngine::calculate_stacked_pos(&screen, &config, &other_bubbles);
            let _ = bubble_window.set_position(tauri::Position::Physical(target_pos));
        }
    }
    let _ = bubble_window.show();
    let _ = app.emit("bubble-visibility-changed", serde_json::json!({
        "accountIndex": account_index,
        "visible": true
    }));
    Ok(())
}

/// Tự động thụt các bong bóng nổi vào mép màn hình (tuck_ratio)
pub fn tuck_bubble(app: &AppHandle, account_index: Option<u32>) {
    let indices: Vec<u32> = match account_index {
        Some(idx) => vec![idx],
        None => (1..=5).collect(),
    };

    let config = BubblePhysicsConfig::default();

    for idx in indices {
        let label = get_bubble_window_label(idx);
        let win = app.get_webview_window(&label);

        if let Some(bubble_window) = win {
            if let Ok(Some(monitor)) = bubble_window.current_monitor() {
                let screen = ScreenMetrics::from_monitor(&monitor);
                let win_pos = bubble_window.outer_position().unwrap_or_default();
                let tucked_pos = BubbleMovementEngine::calculate_hover_pos(
                    &screen,
                    &config,
                    win_pos.x,
                    win_pos.y,
                    false,
                );
                let _ = bubble_window.set_position(tauri::Position::Physical(tucked_pos));
            }
        }
    }
}
