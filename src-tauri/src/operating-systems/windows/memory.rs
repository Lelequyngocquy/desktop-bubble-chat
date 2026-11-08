use tauri::WebviewWindow;
use crate::core::traits::MemoryOptimizerTrait;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut std::ffi::c_void;
    fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    fn SetProcessWorkingSetSize(hProcess: *mut std::ffi::c_void, dwMinimumWorkingSetSize: usize, dwMaximumWorkingSetSize: usize) -> i32;
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
}

const PROCESS_SET_QUOTA: u32 = 0x0100;
const PROCESS_QUERY_INFORMATION: u32 = 0x0400;

pub struct WindowsMemoryOptimizer;

impl MemoryOptimizerTrait for WindowsMemoryOptimizer {
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

impl WindowsMemoryOptimizer {
    /// Trims memory working set of target PID to standby list
    pub fn trim_single_process_working_set(pid: u32) {
        unsafe {
            if pid == std::process::id() {
                SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
            } else {
                let handle = OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION, 0, pid);
                if !handle.is_null() {
                    SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX);
                    CloseHandle(handle);
                }
            }
        }
    }
    /// Trims current application process working set to zero pages
    pub fn trim_current_process_working_set() {
        unsafe {
            SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
        }
    }
    /// Adjusts WebView2 target memory usage level via COM interface
    pub fn set_webview_memory_target_level(window: &WebviewWindow, is_low: bool) {
        let _ = window.with_webview(move |webview| {
            unsafe {
                use webview2_com::Microsoft::Web::WebView2::Win32::{
                    ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
                    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
                };
                use windows_core::Interface;
                let controller = webview.controller();
                if let Ok(core_webview) = controller.CoreWebView2() {
                    if let Ok(webview_19) = core_webview.cast::<ICoreWebView2_19>() {
                        let level = if is_low { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW } else { COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL };
                        let _ = webview_19.SetMemoryUsageTargetLevel(level);
                    }
                }
            }
        });
    }
    /// Checks if process name belongs to WebView2 renderer
    pub fn is_webview_process(name: &str) -> bool {
        name.to_lowercase().contains("msedgewebview2")
    }
    /// Terminates WebView2 processes matching needle without spawning PowerShell
    pub fn terminate_profile_process(needle: &str) {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        for (_pid, process) in system.processes() {
            let name = process.name().to_string_lossy().to_lowercase();
            if name.contains("msedgewebview2") {
                let cmd = process.cmd().iter().map(|s| s.to_string_lossy()).collect::<Vec<_>>().join(" ");
                if cmd.contains(needle) {
                    let _ = process.kill_with(sysinfo::Signal::Term);
                }
            }
        }
    }
}
