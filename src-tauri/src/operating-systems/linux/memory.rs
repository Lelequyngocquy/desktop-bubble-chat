use tauri::WebviewWindow;

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn malloc_trim(pad: usize) -> i32;
}

pub struct LinuxMemoryOptimizer;

impl LinuxMemoryOptimizer {
    /// Trims memory working set of target PID to standby list
    pub fn trim_single_process_working_set(pid: u32) {
        if pid == std::process::id() {
            Self::trim_current_process_working_set();
        }
    }

    /// Trims current application process working set using Glibc malloc_trim
    pub fn trim_current_process_working_set() {
        #[cfg(target_os = "linux")]
        unsafe {
            let _ = malloc_trim(0);
        }
    }

    /// Adjusts WebKit memory target level by requesting garbage collection and memory trim
    pub fn set_webview_memory_target_level(window: &WebviewWindow, is_low: bool) {
        if is_low {
            let _ = window.eval("if (window.gc) { window.gc(); }");
            Self::trim_current_process_working_set();
        }
    }

    /// Checks if process name belongs to WebKitGTK helper processes
    pub fn is_webview_process(name: &str) -> bool {
        let n = name.to_lowercase();
        n.contains("webkitweb") || n.contains("webkitnetwork") || n.contains("webkitgpu")
    }

    /// Terminates WebKit processes matching needle profile path
    pub fn terminate_profile_process(needle: &str) {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        for (pid, process) in system.processes() {
            if process.thread_kind().is_some() {
                continue;
            }
            let name = process.name().to_string_lossy().to_lowercase();
            let cmd = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ").to_lowercase();
            if Self::is_webview_process(&name) || Self::is_webview_process(&cmd) {
                let mut matched = cmd.contains(needle);
                #[cfg(target_os = "linux")]
                if !matched {
                    if let Ok(file) = std::fs::File::open(format!("/proc/{}/maps", pid.as_u32())) {
                        use std::io::BufRead;
                        let reader = std::io::BufReader::new(file);
                        for line in reader.lines().flatten() {
                            if line.contains(needle) {
                                matched = true;
                                break;
                            }
                        }
                    }
                }
                if matched {
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
        assert!(LinuxMemoryOptimizer::is_webview_process("WebKitWebProcess"));
        assert!(LinuxMemoryOptimizer::is_webview_process("WebKitWebProces"));
        assert!(LinuxMemoryOptimizer::is_webview_process("WebKitNetworkProcess"));
        assert!(LinuxMemoryOptimizer::is_webview_process("WebKitNetworkPr"));
        assert!(LinuxMemoryOptimizer::is_webview_process("WebKitGPUProcess"));
        assert!(LinuxMemoryOptimizer::is_webview_process("WebKitGPUProces"));
        assert!(!LinuxMemoryOptimizer::is_webview_process("desktop-bubble-chat"));
        assert!(!LinuxMemoryOptimizer::is_webview_process("gnome-shell"));
    }

    #[test]
    fn test_trim_working_set_safety() {
        LinuxMemoryOptimizer::trim_current_process_working_set();
        LinuxMemoryOptimizer::trim_single_process_working_set(std::process::id());
        LinuxMemoryOptimizer::trim_single_process_working_set(999999);
    }
}
