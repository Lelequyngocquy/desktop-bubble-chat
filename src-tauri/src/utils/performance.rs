use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};

use crate::state::ChatState;
use crate::tray::helpers::get_available_account_indices;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetricItem {
    pub pid: u32,
    pub name: String,
    pub category: String, // "host", "dashboard", "messenger", "gpu", "utility"
    pub account_index: Option<u32>,
    pub account_name: Option<String>,
    pub working_set_mb: f64,
    pub cpu_percent: f64,
    pub is_suspended: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemPerformanceSnapshot {
    pub timestamp: i64,
    pub total_ram_mb: f64,
    pub total_cpu_percent: f64,
    pub host_ram_mb: f64,
    pub dashboard_ram_mb: f64,
    pub active_accounts_count: u32,
    pub suspended_accounts_count: u32,
    pub estimated_saved_ram_mb: f64,
    pub cpu_cores: u32,
    pub items: Vec<ProcessMetricItem>,
}

/// System singleton for CPU calculation
static PERFORMANCE_SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();

fn get_performance_system() -> &'static Mutex<System> {
    PERFORMANCE_SYSTEM.get_or_init(|| {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing().with_processes(
                ProcessRefreshKind::nothing()
                    .with_cpu()
                    .with_memory()
                    .with_cmd(UpdateKind::Always),
            ),
        );
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .with_cmd(UpdateKind::Always),
        );
        Mutex::new(sys)
    })
}

/// Collects real-time performance snapshot for the application process tree
pub fn collect_performance_snapshot(
    app: &AppHandle,
    chat_state: &Arc<ChatState>,
) -> SystemPerformanceSnapshot {
    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    let sys_mutex = get_performance_system();
    let mut sys = sys_mutex.lock().unwrap();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_cpu()
            .with_memory()
            .with_cmd(UpdateKind::Always),
    );
    let host_pid = std::process::id();
    let cpu_cores = sys.cpus().len() as u32;
    let mut app_pids = HashSet::new();
    app_pids.insert(host_pid);
    let mut added_more = true;
    while added_more {
        added_more = false;
        for (pid, process) in sys.processes() {
            let pid_u32 = pid.as_u32();
            if app_pids.contains(&pid_u32) {
                continue;
            }
            let is_child_of_app = process
                .parent()
                .map(|p| app_pids.contains(&p.as_u32()))
                .unwrap_or(false);
            if is_child_of_app {
                app_pids.insert(pid_u32);
                added_more = true;
                continue;
            }
            let p_name = process.name().to_string_lossy().to_lowercase();
            if p_name.contains("msedgewebview2") || p_name.contains("desktopbubblechat") {
                let cmd_joined = process
                    .cmd()
                    .iter()
                    .map(|c| c.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .to_lowercase();
                if cmd_joined.contains("com.desktopbubblechat.app")
                    || cmd_joined.contains("desktopbubblechat")
                {
                    app_pids.insert(pid_u32);
                    added_more = true;
                }
            }
        }
    }
    let mut items = Vec::new();
    let mut host_ram_mb = 0.0;
    let mut dashboard_ram_mb = 0.0;
    let mut total_ram_mb = 0.0;
    let mut total_cpu_percent = 0.0;
    let mut active_accounts = HashSet::new();
    for pid_u32 in &app_pids {
        let pid_obj = Pid::from_u32(*pid_u32);
        if let Some(proc) = sys.processes().get(&pid_obj) {
            let working_set_mb = (proc.memory() as f64) / (1024.0 * 1024.0);
            let cpu_percent = proc.cpu_usage() as f64;
            total_ram_mb += working_set_mb;
            total_cpu_percent += cpu_percent;
            if *pid_u32 == host_pid {
                host_ram_mb += working_set_mb;
                items.push(ProcessMetricItem {
                    pid: *pid_u32,
                    name: "Rust Core (Host & Native Bubbles)".to_string(),
                    category: "host".to_string(),
                    account_index: None,
                    account_name: None,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
                continue;
            }
            let cmd_str = proc
                .cmd()
                .iter()
                .map(|c| c.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            let mut detected_account_index = None;
            for idx in 1..=5 {
                let pattern_padded = format!("messenger_{:02}", idx);
                let pattern_single = format!("messenger_{}", idx);
                if cmd_str.contains(&pattern_padded) || cmd_str.contains(&pattern_single) {
                    detected_account_index = Some(idx);
                    break;
                }
            }
            if let Some(acc_idx) = detected_account_index {
                active_accounts.insert(acc_idx);
                let acc_name = chat_state.get_account_auth(acc_idx).map(|info| {
                    if info.name.trim().is_empty() {
                        format!("Account {:02}", acc_idx)
                    } else {
                        info.name
                    }
                });
                let sub_type = if cmd_str.contains("--type=renderer") {
                    "Renderer"
                } else if cmd_str.contains("--type=utility") {
                    "Utility"
                } else {
                    "Browser"
                };
                items.push(ProcessMetricItem {
                    pid: *pid_u32,
                    name: format!("Messenger {:02} ({})", acc_idx, sub_type),
                    category: "messenger".to_string(),
                    account_index: Some(acc_idx),
                    account_name: acc_name,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
            } else if cmd_str.contains("--type=gpu-process") {
                items.push(ProcessMetricItem {
                    pid: *pid_u32,
                    name: "GPU Acceleration (Direct3D)".to_string(),
                    category: "gpu".to_string(),
                    account_index: None,
                    account_name: None,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
            } else if cmd_str.contains("--type=utility") {
                let name = if cmd_str.contains("network") {
                    "Utility: Network Service"
                } else if cmd_str.contains("audio") {
                    "Utility: Audio Service"
                } else if cmd_str.contains("storage") {
                    "Utility: Storage Service"
                } else {
                    "Utility Broker Service"
                };
                items.push(ProcessMetricItem {
                    pid: *pid_u32,
                    name: name.to_string(),
                    category: "utility".to_string(),
                    account_index: None,
                    account_name: None,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
            } else if cmd_str.contains("--type=crashpad-handler") {
                items.push(ProcessMetricItem {
                    pid: *pid_u32,
                    name: "Crashpad Handler".to_string(),
                    category: "utility".to_string(),
                    account_index: None,
                    account_name: None,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
            } else {
                dashboard_ram_mb += working_set_mb;
                let name = if cmd_str.contains("--type=renderer") {
                    "Dashboard Web Renderer"
                } else {
                    "Dashboard Main Window"
                };
                items.push(ProcessMetricItem {
                    pid: *pid_u32,
                    name: name.to_string(),
                    category: "dashboard".to_string(),
                    account_index: None,
                    account_name: None,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
            }
        }
    }
    let available_indices = get_available_account_indices(app);
    let mut suspended_accounts_count = 0;
    for idx in available_indices {
        let is_running = active_accounts.contains(&idx);
        let is_suspended = chat_state.is_account_suspended(idx) || !is_running;
        if is_suspended && !is_running {
            suspended_accounts_count += 1;
            let acc_name = chat_state.get_account_auth(idx).map(|info| {
                if info.name.trim().is_empty() {
                    format!("Account {:02}", idx)
                } else {
                    info.name
                }
            });
            items.push(ProcessMetricItem {
                pid: 0,
                name: format!("Messenger {:02} (Suspended)", idx),
                category: "messenger".to_string(),
                account_index: Some(idx),
                account_name: acc_name,
                working_set_mb: 0.0,
                cpu_percent: 0.0,
                is_suspended: true,
            });
        }
    }
    let estimated_saved_ram_mb = (suspended_accounts_count as f64) * 700.0;
    items.sort_by(|a, b| {
        let order = |cat: &str| match cat {
            "host" => 0,
            "dashboard" => 1,
            "messenger" => 2,
            "gpu" => 3,
            "utility" => 4,
            _ => 5,
        };
        let ord_a = order(&a.category);
        let ord_b = order(&b.category);
        if ord_a != ord_b {
            ord_a.cmp(&ord_b)
        } else {
            a.account_index
                .cmp(&b.account_index)
                .then(b.working_set_mb.partial_cmp(&a.working_set_mb).unwrap())
        }
    });
    SystemPerformanceSnapshot {
        timestamp: now_ts,
        total_ram_mb: (total_ram_mb * 100.0).round() / 100.0,
        total_cpu_percent: (total_cpu_percent * 10.0).round() / 10.0,
        host_ram_mb: (host_ram_mb * 100.0).round() / 100.0,
        dashboard_ram_mb: (dashboard_ram_mb * 100.0).round() / 100.0,
        active_accounts_count: active_accounts.len() as u32,
        suspended_accounts_count,
        estimated_saved_ram_mb,
        cpu_cores,
        items,
    }
}

/// Trims working set memory of all application processes, returns reclaimed MB
pub fn trim_all_app_processes_memory(_app: &AppHandle) -> f64 {
    let host_pid = std::process::id();
    let mut target_pids = Vec::new();
    target_pids.push(host_pid);
    let sys_mutex = get_performance_system();
    if let Ok(mut sys) = sys_mutex.lock() {
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        let ram_before: u64 = sys
            .processes()
            .values()
            .map(|p| p.memory())
            .sum();
        let mut app_pids = HashSet::new();
        app_pids.insert(host_pid);
        let mut added = true;
        while added {
            added = false;
            for (pid, process) in sys.processes() {
                let pid_u32 = pid.as_u32();
                if app_pids.contains(&pid_u32) {
                    continue;
                }
                if process
                    .parent()
                    .map(|p| app_pids.contains(&p.as_u32()))
                    .unwrap_or(false)
                {
                    app_pids.insert(pid_u32);
                    target_pids.push(pid_u32);
                    added = true;
                }
            }
        }
        for pid in &target_pids {
            trim_single_process_working_set(*pid);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        let ram_after: u64 = sys
            .processes()
            .values()
            .map(|p| p.memory())
            .sum();
        let diff_bytes = ram_before.saturating_sub(ram_after);
        let diff_mb = (diff_bytes as f64) / (1024.0 * 1024.0);
        println!(
            "\x1b[32m[Performance] Trim Working Set completed for {} processes. Reclaimed: {:.2} MB RAM.\x1b[0m",
            target_pids.len(),
            diff_mb
        );
        (diff_mb * 100.0).round() / 100.0
    } else {
        0.0
    }
}

/// Returns all process IDs belonging to a specific account index
pub fn get_account_process_pids(account_index: u32) -> Vec<u32> {
    let mut pids = HashSet::new();
    let pattern_padded = format!("messenger_{:02}", account_index);
    let pattern_single = format!("messenger_{}", account_index);
    let sys_mutex = get_performance_system();
    if let Ok(mut sys) = sys_mutex.lock() {
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cmd(UpdateKind::OnlyIfNotSet),
        );
        for (pid, proc) in sys.processes() {
            let cmd_str = proc
                .cmd()
                .iter()
                .map(|c| c.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            if cmd_str.contains(&pattern_padded) || cmd_str.contains(&pattern_single) {
                pids.insert(pid.as_u32());
            }
        }
        let mut added = true;
        while added {
            added = false;
            for (pid, proc) in sys.processes() {
                let pid_u32 = pid.as_u32();
                if pids.contains(&pid_u32) {
                    continue;
                }
                if let Some(parent) = proc.parent() {
                    if pids.contains(&parent.as_u32()) {
                        pids.insert(pid_u32);
                        added = true;
                    }
                }
            }
        }
    }
    pids.into_iter().collect()
}

#[cfg(target_os = "windows")]
pub fn trim_single_process_working_set(pid: u32) {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(
            dwDesiredAccess: u32,
            bInheritHandle: i32,
            dwProcessId: u32,
        ) -> *mut std::ffi::c_void;
        fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
        fn SetProcessWorkingSetSize(
            hProcess: *mut std::ffi::c_void,
            dwMinimumWorkingSetSize: usize,
            dwMaximumWorkingSetSize: usize,
        ) -> i32;
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
    }
    const PROCESS_SET_QUOTA: u32 = 0x0100;
    const PROCESS_QUERY_INFORMATION: u32 = 0x0400;
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

#[cfg(not(target_os = "windows"))]
pub fn trim_single_process_working_set(_pid: u32) {}
