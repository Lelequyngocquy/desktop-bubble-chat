use tauri::WebviewWindow;

pub struct CursorTracker;
impl CursorTracker {
    pub fn get_cursor_position() -> Option<(i32, i32)> { None }
    pub fn is_lbutton_down() -> bool { false }
    pub fn is_cursor_over_window(_window: &WebviewWindow) -> bool { false }
}

pub struct MemoryOptimizer;
impl MemoryOptimizer {
    pub fn trim_single_process_working_set(_pid: u32) {}
    pub fn trim_current_process_working_set() {}
    pub fn set_webview_memory_target_level(_window: &WebviewWindow, _is_low: bool) {}
    pub fn is_webview_process(name: &str) -> bool {
        let n = name.to_lowercase();
        n.contains("com.apple.webkit.webcontent") || n.contains("com.apple.webkit.networking")
    }
    pub fn terminate_profile_process(needle: &str) {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        for (_pid, process) in system.processes() {
            let cmd = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ");
            if cmd.contains(needle) {
                let _ = process.kill_with(sysinfo::Signal::Term);
            }
        }
    }
}

pub struct AutostartManager;
impl AutostartManager {
    pub fn get_windows_display_name(_username: &str) -> Option<String> { None }
    pub fn toggle_autostart_registry(enabled: bool) -> Result<bool, String> { Ok(enabled) }
    pub fn is_autostart_registry_enabled() -> bool { false }
}

pub struct SoundPlayer;
impl SoundPlayer {
    pub fn play_wav(_wav: &[u8]) {}
}

pub struct DialogManager;
impl DialogManager {
    pub fn show_confirm_exit(_title: &str, _text: &str) -> bool { true }
}

pub struct ThemeManager;
impl ThemeManager {
    pub fn apply_window_theme_color(_window: &WebviewWindow, _hex: &str, _border_hex: &str) {}
}

pub mod bubble {
    pub fn toggle_native_bubble_prototype(_app: Option<tauri::AppHandle>, _account_index: u32, _visible: bool) -> Result<bool, String> { Ok(false) }
    pub fn has_native_bubble(_account_index: u32) -> bool { false }
    pub fn set_native_bubble_unread_count(_account_index: u32, _count: u32) {}
    pub fn trigger_native_bubble_pulse(_account_index: u32) {}
    pub fn is_cursor_over_native_bubble(_account_index: u32) -> bool { false }
    pub fn tuck_native_bubble(_app: &tauri::AppHandle, _account_index: u32) {}
    pub fn untuck_native_bubble(_app: &tauri::AppHandle, _account_index: u32) {}
    pub fn set_native_bubble_always_on_top(_enabled: bool) {}
    pub fn set_native_bubble_size(_account_index: u32, _size: u32) {}
}
pub use bubble::*;
