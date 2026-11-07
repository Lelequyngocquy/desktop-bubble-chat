use tauri::WebviewWindow;

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    fn GetAsyncKeyState(vKey: i32) -> i16;
}

pub struct WindowsCursorTracker;

impl WindowsCursorTracker {
    /// Returns current global cursor position
    pub fn get_cursor_position() -> Option<(i32, i32)> {
        let mut pt = POINT { x: 0, y: 0 };
        if unsafe { GetCursorPos(&mut pt) } != 0 {
            Some((pt.x, pt.y))
        } else {
            None
        }
    }
    /// Checks if mouse left button is currently pressed
    pub fn is_lbutton_down() -> bool {
        unsafe { (GetAsyncKeyState(0x01) as u16 & 0x8000) != 0 }
    }
    /// Checks if cursor is currently over given window
    pub fn is_cursor_over_window(window: &WebviewWindow) -> bool {
        let mut pt = POINT { x: 0, y: 0 };
        if unsafe { GetCursorPos(&mut pt) } == 0 {
            return false;
        }
        if let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) {
            pt.x >= pos.x && pt.x <= pos.x + size.width as i32 && pt.y >= pos.y && pt.y <= pos.y + size.height as i32
        } else {
            false
        }
    }
}
