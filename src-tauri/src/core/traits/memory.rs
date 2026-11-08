use tauri::WebviewWindow;

/// Abstract interface for platform-specific memory optimization operations
pub trait MemoryOptimizerTrait {
    fn trim_single_process_working_set(pid: u32);
    fn trim_current_process_working_set();
    fn set_webview_memory_target_level(window: &WebviewWindow, is_low: bool);
    fn is_webview_process(name: &str) -> bool;
    fn terminate_profile_process(needle: &str);
}
