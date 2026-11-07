pub trait BubbleDriver {
    fn show(&self, app: &tauri::AppHandle, account_index: u32, x: i32, y: i32, size: u32) -> Result<(), String>;
    fn hide(&self, app: &tauri::AppHandle, account_index: u32) -> Result<(), String>;
}
