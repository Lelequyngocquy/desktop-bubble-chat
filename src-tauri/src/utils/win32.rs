use tauri::WebviewWindow;

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    fn GetAsyncKeyState(vKey: i32) -> i16;
}

/// Checks if the cursor is currently over a given WebviewWindow
#[cfg(target_os = "windows")]
pub fn is_cursor_over_window(window: &WebviewWindow) -> bool {
    let mut pt = POINT { x: 0, y: 0 };
    unsafe {
        if GetCursorPos(&mut pt) == 0 {
            return false;
        }
    }
    if let Ok(pos) = window.outer_position() {
        if let Ok(size) = window.outer_size() {
            return pt.x >= pos.x
                && pt.x <= pos.x + size.width as i32
                && pt.y >= pos.y
                && pt.y <= pos.y + size.height as i32;
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
pub fn is_cursor_over_window(_window: &WebviewWindow) -> bool {
    false
}

/// Checks if the cursor is currently over the bubble window
pub fn is_cursor_over_bubble(bubble_window: &WebviewWindow) -> bool {
    is_cursor_over_window(bubble_window)
}

/// Checks if the left mouse button is currently held down
#[cfg(target_os = "windows")]
pub fn is_lbutton_down() -> bool {
    unsafe { (GetAsyncKeyState(0x01) as u16 & 0x8000) != 0 }
}

#[cfg(not(target_os = "windows"))]
pub fn is_lbutton_down() -> bool {
    false
}
