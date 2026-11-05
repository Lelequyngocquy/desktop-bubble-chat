use std::sync::Arc;
use tauri::{AppHandle, Manager};

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

    fn IsWindow(hWnd: *mut std::ffi::c_void) -> i32;
}

#[cfg(target_os = "windows")]
const SWP_NOMOVE: u32 = 0x0002;
#[cfg(target_os = "windows")]
const SWP_NOZORDER: u32 = 0x0004;
#[cfg(target_os = "windows")]
const SWP_NOACTIVATE: u32 = 0x0010;
#[cfg(target_os = "windows")]
const SWP_FRAMECHANGED: u32 = 0x0020;

/// Resizes bubble window via Win32 SetWindowPos and Tauri set_size
pub fn resize_bubble_window(bubble: &tauri::WebviewWindow, size_px: f64) {
    #[cfg(target_os = "windows")]
    {
        let bubble_clone = bubble.clone();
        let _ = bubble.run_on_main_thread(move || {
            if let Ok(hwnd) = bubble_clone.hwnd() {
                let raw_hwnd = hwnd.0 as *mut std::ffi::c_void;
                let screen = crate::movement::get_target_dashboard_screen(None);
                let size = (size_px * screen.scale_factor) as i32;
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
        });
    }
    let _ = bubble.set_size(tauri::Size::Logical(tauri::LogicalSize {
        width: size_px,
        height: size_px,
    }));
}

/// Returns window label for bubble by account index
pub fn get_bubble_window_label(account_index: u32) -> String {
    format!("bubble-{:02}", account_index)
}

/// Finds or creates WebviewWindow for specified bubble account
pub fn get_or_create_bubble_window(
    app: &AppHandle,
    account_index: u32,
) -> Result<tauri::WebviewWindow, String> {
    let label = get_bubble_window_label(account_index);
    for attempt in 0..10 {
        if let Some(win) = app.get_webview_window(&label) {
            #[cfg(target_os = "windows")]
            {
                let is_alive = if let Ok(hwnd) = win.hwnd() {
                    unsafe { IsWindow(hwnd.0 as *mut std::ffi::c_void) != 0 }
                } else {
                    false
                };
                if is_alive {
                    return Ok(win);
                } else {
                    let _ = win.destroy();
                    if attempt < 9 {
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                    continue;
                }
            }
            #[cfg(not(target_os = "windows"))]
            return Ok(win);
        } else {
            break;
        }
    }
    let is_on_top = app
        .try_state::<Arc<crate::state::ChatState>>()
        .map(|s| s.is_always_on_top.load(std::sync::atomic::Ordering::SeqCst))
        .unwrap_or(true);
    let config = crate::movement::get_bubble_physics_config(app);
    let window = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title(format!("DesktopBubbleChat - Bubble {:02}", account_index))
    .inner_size(config.size, config.size)
    .min_inner_size(1.0, 1.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .always_on_top(is_on_top)
    .skip_taskbar(true)
    .shadow(false)
    .visible(false)
    .build()
    .map_err(|e| format!("Failed to create Bubble {:02}: {}", account_index, e))?;
    #[cfg(target_os = "windows")]
    {
        let config = crate::movement::get_bubble_physics_config(app);
        resize_bubble_window(&window, config.size);
    }
    Ok(window)
}
