use tauri::WebviewWindow;
use crate::core::traits::MemoryOptimizerTrait;

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn malloc_trim(pad: usize) -> i32;
}

pub struct LinuxMemoryOptimizer;

impl MemoryOptimizerTrait for LinuxMemoryOptimizer {
    fn trim_single_process_working_set(pid: u32) {
        Self::trim_single_process_working_set(pid);
    }
    fn trim_current_process_working_set() {
        Self::trim_current_process_working_set();
    }
    fn set_webview_memory_target_level(window: &WebviewWindow, is_low: bool) {
        Self::set_webview_memory_target_level(window, is_low);
    }
    fn is_webview_process(name: &str) -> bool {
        Self::is_webview_process(name)
    }
    fn terminate_profile_process(needle: &str) {
        Self::terminate_profile_process(needle);
    }
}

impl LinuxMemoryOptimizer {
    /// Trims memory working set of target PID if it matches current host process
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

    /// Adjusts WebKit memory target level without clearing persistent HTTP/avatar cache
    pub fn set_webview_memory_target_level(window: &WebviewWindow, is_low: bool) {
        #[cfg(target_os = "linux")]
        {
            use webkit2gtk::{WebContextExt, WebViewExt};
            let _ = window.with_webview(move |webview| {
                let inner = webview.inner();
                if let Some(context) = inner.web_context() {
                    let model = if is_low { webkit2gtk::CacheModel::DocumentViewer } else { webkit2gtk::CacheModel::WebBrowser };
                    context.set_cache_model(model);
                }
            });
            if is_low {
                let _ = window.eval("if(window.performance&&window.performance.clearResourceTimings){window.performance.clearResourceTimings();}");
                Self::trim_current_process_working_set();
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (window, is_low);
        }
    }

    /// Sets native WebKitGTK audio mute state for a Messenger WebviewWindow
    pub fn set_webview_muted(window: &WebviewWindow, muted: bool) {
        #[cfg(target_os = "linux")]
        {
            use webkit2gtk::WebViewExt;
            let _ = window.with_webview(move |webview| {
                webview.inner().set_is_muted(muted);
            });
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (window, muted);
        }
    }

    /// Checks whether a proc path or command string references the profile needle
    pub fn matches_profile_needle(haystack: &str, needle_lower: &str) -> bool {
        !needle_lower.is_empty() && haystack.to_lowercase().contains(needle_lower)
    }

    /// Checks if process name belongs to WebKitGTK helper processes
    pub fn is_webview_process(name: &str) -> bool {
        let n = name.to_lowercase();
        n.contains("webkitweb") || n.contains("webkitnetwork") || n.contains("webkitgpu")
    }

    /// Terminates WebKit processes matching non-empty needle profile path
    pub fn terminate_profile_process(needle: &str) {
        let trimmed = needle.trim();
        if trimmed.is_empty() {
            return;
        }
        let needle_lower = trimmed.to_lowercase();
        let system = sysinfo::System::new_with_specifics(
            sysinfo::RefreshKind::nothing().with_processes(
                sysinfo::ProcessRefreshKind::nothing().with_cmd(sysinfo::UpdateKind::Always),
            ),
        );
        for (pid, process) in system.processes() {
            if process.thread_kind().is_some() {
                continue;
            }
            let name = process.name().to_string_lossy().to_lowercase();
            let cmd = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ").to_lowercase();
            if Self::is_webview_process(&name) || Self::is_webview_process(&cmd) {
                #[allow(unused_mut)]
                let mut matched = Self::matches_profile_needle(&cmd, &needle_lower);
                #[cfg(target_os = "linux")]
                if !matched
                    && let Ok(file) = std::fs::File::open(format!("/proc/{}/maps", pid.as_u32())) {
                        use std::io::BufRead;
                        let reader = std::io::BufReader::new(file);
                        for line in reader.lines().map_while(Result::ok) {
                            if Self::matches_profile_needle(&line, &needle_lower) {
                                matched = true;
                                break;
                            }
                        }
                    }
                #[cfg(target_os = "linux")]
                if !matched
                    && let Ok(entries) = std::fs::read_dir(format!("/proc/{}/fd", pid.as_u32())) {
                        for entry in entries.flatten() {
                            if let Ok(target) = std::fs::read_link(entry.path())
                                && Self::matches_profile_needle(&target.to_string_lossy(), &needle_lower) {
                                    matched = true;
                                    break;
                                }
                        }
                    }
                #[cfg(not(target_os = "linux"))]
                let _ = pid;
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
