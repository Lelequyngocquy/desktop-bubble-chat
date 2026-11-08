use std::collections::{HashMap, HashSet};
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

/// Checks if process name belongs to webview engine on any platform
pub fn is_webview_process_name(name: &str) -> bool {
    crate::operating_systems::current::MemoryOptimizer::is_webview_process(name)
        || {
            let n = name.to_lowercase();
            n.contains("msedgewebview2")
                || n.contains("webkitweb")
                || n.contains("webkitnetwork")
                || n.contains("webkitgpu")
                || n.contains("com.apple.webkit")
        }
}

/// Detects account profile index for process using cmdline and proc maps
pub fn detect_account_index_for_process(_pid: u32, cmd_joined: &str) -> u32 {
    for idx in 1..=5 {
        if cmd_joined.contains(&format!("messenger_{:02}", idx)) || cmd_joined.contains(&format!("messenger_{}", idx)) {
            return idx;
        }
    }
    #[cfg(target_os = "linux")]
    {
        use std::io::BufRead;
        if let Ok(file) = std::fs::File::open(format!("/proc/{}/maps", _pid)) {
            let reader = std::io::BufReader::new(file);
            for line in reader.lines().flatten() {
                for idx in 1..=5 {
                    if line.contains(&format!("messenger_{:02}", idx)) || line.contains(&format!("messenger_{}", idx)) {
                        return idx;
                    }
                }
            }
        }
    }
    0
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
    let cpu_cores = sys.cpus().len().max(1) as u32;
    let cores_f64 = cpu_cores as f64;
    let mut process_account_map: HashMap<u32, u32> = HashMap::new();
    for (pid, process) in sys.processes() {
        if process.thread_kind().is_some() {
            continue;
        }
        let pid_u32 = pid.as_u32();
        if pid_u32 == host_pid {
            continue;
        }
        let p_name = process.name().to_string_lossy().to_lowercase();
        let cmd_joined = process.cmd().iter().map(|c| c.to_string_lossy()).collect::<Vec<_>>().join(" ").to_lowercase();
        if !is_webview_process_name(&p_name) && !is_webview_process_name(&cmd_joined) {
            continue;
        }
        let is_direct_child = process.parent().map(|p| p.as_u32() == host_pid).unwrap_or(false);
        let detected_acc = detect_account_index_for_process(pid_u32, &cmd_joined);
        let belongs_to_app = is_direct_child || detected_acc > 0 || cmd_joined.contains("com.desktopbubblechat.app");
        if !belongs_to_app {
            continue;
        }
        let is_broker_or_child = is_direct_child || (!cmd_joined.contains("--type=") && (cmd_joined.contains("--embedded-browser-webview=1") || cmd_joined.contains("com.desktopbubblechat.app")));
        if is_broker_or_child || detected_acc > 0 {
            process_account_map.insert(pid_u32, detected_acc);
        }
    }
    let mut added_more = true;
    while added_more {
        added_more = false;
        for (pid, process) in sys.processes() {
            if process.thread_kind().is_some() {
                continue;
            }
            let pid_u32 = pid.as_u32();
            if pid_u32 == host_pid || process_account_map.contains_key(&pid_u32) {
                continue;
            }
            let p_name = process.name().to_string_lossy().to_lowercase();
            let cmd_joined = process.cmd().iter().map(|c| c.to_string_lossy()).collect::<Vec<_>>().join(" ").to_lowercase();
            if !is_webview_process_name(&p_name) && !is_webview_process_name(&cmd_joined) {
                continue;
            }
            if let Some(parent) = process.parent() {
                let parent_u32 = parent.as_u32();
                if let Some(&acc) = process_account_map.get(&parent_u32) {
                    process_account_map.insert(pid_u32, acc);
                    added_more = true;
                    continue;
                }
            }
            let detected_acc = detect_account_index_for_process(pid_u32, &cmd_joined);
            let belongs_to_app = detected_acc > 0 || cmd_joined.contains("com.desktopbubblechat.app");
            if belongs_to_app {
                process_account_map.insert(pid_u32, detected_acc);
                added_more = true;
            }
        }
    }
    let mut items = Vec::new();
    let mut host_ram_mb = 0.0;
    let mut dashboard_ram_mb = 0.0;
    let mut total_ram_mb = 0.0;
    let mut total_cpu_percent = 0.0;
    let mut active_accounts = HashSet::new();
    if let Some(proc) = sys.processes().get(&Pid::from_u32(host_pid)) {
        let working_set_mb = (proc.memory() as f64) / (1024.0 * 1024.0);
        let cpu_percent = (proc.cpu_usage() as f64) / cores_f64;
        total_ram_mb += working_set_mb;
        total_cpu_percent += cpu_percent;
        host_ram_mb += working_set_mb;
        items.push(ProcessMetricItem {
            pid: host_pid,
            name: "Rust Core (Host & Native Bubbles)".to_string(),
            category: "host".to_string(),
            account_index: None,
            account_name: None,
            working_set_mb: (working_set_mb * 100.0).round() / 100.0,
            cpu_percent: (cpu_percent * 10.0).round() / 10.0,
            is_suspended: false,
        });
    }
    for (&pid_u32, &acc_idx) in &process_account_map {
        let pid_obj = Pid::from_u32(pid_u32);
        if let Some(proc) = sys.processes().get(&pid_obj) {
            let working_set_mb = (proc.memory() as f64) / (1024.0 * 1024.0);
            let cpu_percent = (proc.cpu_usage() as f64) / cores_f64;
            total_ram_mb += working_set_mb;
            total_cpu_percent += cpu_percent;
            let cmd_str = proc.cmd().iter().map(|c| c.to_string_lossy()).collect::<Vec<_>>().join(" ").to_lowercase();
            let p_name = proc.name().to_string_lossy().to_lowercase();
            let is_net = p_name.contains("webkitnetwork") || cmd_str.contains("webkitnetwork") || p_name.contains("com.apple.webkit.networking");
            let is_gpu = p_name.contains("webkitgpu") || cmd_str.contains("webkitgpu") || cmd_str.contains("--type=gpu-process");
            let is_web = p_name.contains("webkitweb") || cmd_str.contains("webkitweb") || p_name.contains("com.apple.webkit.webcontent");
            if is_net {
                if acc_idx > 0 {
                    active_accounts.insert(acc_idx);
                }
                let name = if acc_idx > 0 {
                    format!("Messenger {:02} (Network Service)", acc_idx)
                } else {
                    "Dashboard Network Service".to_string()
                };
                items.push(ProcessMetricItem {
                    pid: pid_u32,
                    name,
                    category: "utility".to_string(),
                    account_index: if acc_idx > 0 { Some(acc_idx) } else { None },
                    account_name: None,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
            } else if is_gpu {
                let name = if p_name.contains("webkit") || cmd_str.contains("webkit") { "GPU Acceleration (WebKit/Cairo)" } else { "GPU Acceleration (Direct3D)" };
                items.push(ProcessMetricItem {
                    pid: pid_u32,
                    name: name.to_string(),
                    category: "gpu".to_string(),
                    account_index: if acc_idx > 0 { Some(acc_idx) } else { None },
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
                    pid: pid_u32,
                    name: name.to_string(),
                    category: "utility".to_string(),
                    account_index: if acc_idx > 0 { Some(acc_idx) } else { None },
                    account_name: None,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
            } else if cmd_str.contains("--type=crashpad-handler") {
                items.push(ProcessMetricItem {
                    pid: pid_u32,
                    name: "Crashpad Handler".to_string(),
                    category: "utility".to_string(),
                    account_index: if acc_idx > 0 { Some(acc_idx) } else { None },
                    account_name: None,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
            } else if acc_idx > 0 {
                active_accounts.insert(acc_idx);
                let acc_name = chat_state.get_account_auth(acc_idx).map(|info| {
                    if info.name.trim().is_empty() {
                        format!("Account {:02}", acc_idx)
                    } else {
                        info.name
                    }
                });
                let sub_type = if is_web {
                    "WebKit Webview"
                } else {
                    let is_renderer = cmd_str.contains("--type=renderer") || proc.parent().map(|p| p.as_u32() != host_pid).unwrap_or(false);
                    if is_renderer { "Renderer" } else { "Broker" }
                };
                items.push(ProcessMetricItem {
                    pid: pid_u32,
                    name: format!("Messenger {:02} ({})", acc_idx, sub_type),
                    category: "messenger".to_string(),
                    account_index: Some(acc_idx),
                    account_name: acc_name,
                    working_set_mb: (working_set_mb * 100.0).round() / 100.0,
                    cpu_percent: (cpu_percent * 10.0).round() / 10.0,
                    is_suspended: false,
                });
            } else {
                dashboard_ram_mb += working_set_mb;
                let name = if is_web {
                    "Dashboard Webview (WebKit)"
                } else {
                    let is_renderer = cmd_str.contains("--type=renderer") || proc.parent().map(|p| p.as_u32() != host_pid).unwrap_or(false);
                    if is_renderer {
                        "Dashboard Web Renderer"
                    } else {
                        "Dashboard Main Window"
                    }
                };
                items.push(ProcessMetricItem {
                    pid: pid_u32,
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
        total_cpu_percent: (total_cpu_percent.min(100.0) * 10.0).round() / 10.0,
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
        let mut app_pids = HashSet::new();
        app_pids.insert(host_pid);
        let mut added = true;
        while added {
            added = false;
            for (pid, process) in sys.processes() {
                if process.thread_kind().is_some() {
                    continue;
                }
                let pid_u32 = pid.as_u32();
                if app_pids.contains(&pid_u32) {
                    continue;
                }
                let p_name = process.name().to_string_lossy().to_lowercase();
                let cmd_str = process.cmd().iter().map(|c| c.to_string_lossy()).collect::<Vec<_>>().join(" ").to_lowercase();
                let is_wv = is_webview_process_name(&p_name) || is_webview_process_name(&cmd_str);
                if !is_wv && !p_name.contains("desktopbubblechat") && !p_name.contains("desktop-bubble-chat") {
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
        let ram_before: u64 = target_pids
            .iter()
            .filter_map(|p| sys.processes().get(&Pid::from_u32(*p)))
            .map(|p| p.memory())
            .sum();
        for pid in &target_pids {
            trim_single_process_working_set(*pid);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        let ram_after: u64 = target_pids
            .iter()
            .filter_map(|p| sys.processes().get(&Pid::from_u32(*p)))
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
    let sys_mutex = get_performance_system();
    if let Ok(mut sys) = sys_mutex.lock() {
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_cmd(UpdateKind::OnlyIfNotSet),
        );
        for (pid, proc) in sys.processes() {
            if proc.thread_kind().is_some() {
                continue;
            }
            let p_name = proc.name().to_string_lossy().to_lowercase();
            let cmd_str = proc.cmd().iter().map(|c| c.to_string_lossy()).collect::<Vec<_>>().join(" ").to_lowercase();
            if !is_webview_process_name(&p_name) && !is_webview_process_name(&cmd_str) {
                continue;
            }
            let pid_u32 = pid.as_u32();
            if detect_account_index_for_process(pid_u32, &cmd_str) == account_index {
                pids.insert(pid_u32);
            }
        }
        let mut added = true;
        while added {
            added = false;
            for (pid, proc) in sys.processes() {
                if proc.thread_kind().is_some() {
                    continue;
                }
                let pid_u32 = pid.as_u32();
                if pids.contains(&pid_u32) {
                    continue;
                }
                let p_name = proc.name().to_string_lossy().to_lowercase();
                let cmd_str = proc.cmd().iter().map(|c| c.to_string_lossy()).collect::<Vec<_>>().join(" ").to_lowercase();
                if !is_webview_process_name(&p_name) && !is_webview_process_name(&cmd_str) {
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

/// Trims memory working set of target process
pub fn trim_single_process_working_set(pid: u32) {
    crate::operating_systems::current::MemoryOptimizer::trim_single_process_working_set(pid);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_webview_process_name_matches() {
        assert!(is_webview_process_name("msedgewebview2.exe"));
        assert!(is_webview_process_name("WebKitWebProcess"));
        assert!(is_webview_process_name("WebKitWebProces"));
        assert!(is_webview_process_name("WebKitNetworkProcess"));
        assert!(is_webview_process_name("WebKitNetworkPr"));
        assert!(is_webview_process_name("WebKitGPUProcess"));
        assert!(is_webview_process_name("WebKitGPUProces"));
        assert!(is_webview_process_name("com.apple.WebKit.WebContent"));
        assert!(!is_webview_process_name("desktop-bubble-chat"));
        assert!(!is_webview_process_name("bash"));
    }

    #[test]
    fn test_detect_account_index_for_process_cmdline() {
        assert_eq!(detect_account_index_for_process(9999, "--user-data-dir=profiles/messenger_01"), 1);
        assert_eq!(detect_account_index_for_process(9999, "--user-data-dir=profiles/messenger_04"), 4);
        assert_eq!(detect_account_index_for_process(9999, "dashboard main process"), 0);
    }

    #[test]
    fn test_thread_kind_filtering() {
        let mut sys = System::new_all();
        sys.refresh_all();
        let main_procs = sys.processes().values().filter(|p| p.thread_kind().is_none()).count();
        assert!(main_procs > 0);
    }
}
