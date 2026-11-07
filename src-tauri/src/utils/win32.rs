use tauri::WebviewWindow;

/// Checks if cursor is currently over given WebviewWindow
pub fn is_cursor_over_window(window: &WebviewWindow) -> bool {
    crate::operating_systems::current::CursorTracker::is_cursor_over_window(window)
}

/// Checks if cursor is currently over bubble window
pub fn is_cursor_over_bubble(bubble_window: &WebviewWindow) -> bool {
    is_cursor_over_window(bubble_window)
}

/// Checks if mouse left button is currently held down
pub fn is_lbutton_down() -> bool {
    crate::operating_systems::current::CursorTracker::is_lbutton_down()
}
