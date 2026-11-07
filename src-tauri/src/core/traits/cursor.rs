pub trait CursorProvider {
    fn get_cursor_position(&self) -> Option<(i32, i32)>;
    fn is_lbutton_down(&self) -> bool;
}
