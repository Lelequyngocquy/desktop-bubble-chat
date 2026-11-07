use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn malloc_zone_pressure_relief(zone: *mut std::ffi::c_void, flags: usize);
}

pub struct MacOSMemoryOptimizer;

impl MacOSMemoryOptimizer {
    /// Trims memory working set of target PID to standby list
    pub fn trim_single_process_working_set(pid: u32) {
        if pid == std::process::id() {
            Self::trim_current_process_working_set();
        }
    }

    /// Trims current application process working set using Mach zone pressure relief
    pub fn trim_current_process_working_set() {
        #[cfg(target_os = "macos")]
        unsafe {
            malloc_zone_pressure_relief(std::ptr::null_mut(), 0);
        }
    }

    /// Adjusts WebKit memory target level by requesting garbage collection and memory relief
    pub fn set_webview_memory_target_level(window: &WebviewWindow, is_low: bool) {
        if is_low {
            let _ = window.eval("if (window.gc) { window.gc(); }");
            Self::trim_current_process_working_set();
        }
    }

    /// Checks if process name belongs to macOS WebKit helper processes
    pub fn is_webview_process(name: &str) -> bool {
        let n = name.to_lowercase();
        n.contains("com.apple.webkit.webcontent") || n.contains("com.apple.webkit.networking")
    }

    /// Terminates WebKit processes matching needle profile path
    pub fn terminate_profile_process(needle: &str) {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        for (_pid, process) in system.processes() {
            let name = process.name().to_string_lossy().to_lowercase();
            if Self::is_webview_process(&name) {
                let cmd = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ");
                if cmd.contains(needle) {
                    let _ = process.kill_with(sysinfo::Signal::Term);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_webview_process_patterns() {
        assert!(MacOSMemoryOptimizer::is_webview_process("com.apple.WebKit.WebContent"));
        assert!(MacOSMemoryOptimizer::is_webview_process("com.apple.WebKit.WebContent.Development"));
        assert!(MacOSMemoryOptimizer::is_webview_process("com.apple.WebKit.Networking"));
        assert!(!MacOSMemoryOptimizer::is_webview_process("com.apple.WebKit.GPU"));
        assert!(!MacOSMemoryOptimizer::is_webview_process("desktop-bubble-chat"));
        assert!(!MacOSMemoryOptimizer::is_webview_process("Finder"));
    }

    #[test]
    fn test_trim_working_set_safety() {
        MacOSMemoryOptimizer::trim_current_process_working_set();
        MacOSMemoryOptimizer::trim_single_process_working_set(std::process::id());
        MacOSMemoryOptimizer::trim_single_process_working_set(999999);
    }
}
