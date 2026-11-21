use std::collections::HashSet;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, UpdateKind};
use tauri::{AppHandle, Manager};

use super::{detect_account_index_for_process, get_performance_system, is_webview_process_name};

/// Trims working set memory of all application processes, returns reclaimed MB
pub fn trim_all_app_processes_memory(app: &AppHandle) -> f64 {
    for idx in 1..=5 {
        crate::platforms::messenger::windows::set_chat_window_memory_target(app, idx, true);
    }
    if let Some(dash_win) = app.get_webview_window("main") {
        crate::operating_systems::current::MemoryOptimizer::set_webview_memory_target_level(&dash_win, true);
    }
    let host_pid = std::process::id();
    let target_pids: Vec<u32> = {
        let sys_mutex = get_performance_system();
        let Ok(mut sys) = sys_mutex.lock() else {
            return 0.0;
        };
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_memory()
                .with_cmd(UpdateKind::Always),
        );
        let mut app_pids = HashSet::new();
        app_pids.insert(host_pid);
        let mut pids = vec![host_pid];
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
                    pids.push(pid_u32);
                    added = true;
                }
            }
        }
        pids
    };
    let ram_before: u64 = {
        let sys_mutex = get_performance_system();
        let Ok(sys) = sys_mutex.lock() else {
            return 0.0;
        };
        target_pids
            .iter()
            .filter_map(|p| sys.processes().get(&Pid::from_u32(*p)))
            .map(|p| p.memory())
            .sum()
    };
    for pid in &target_pids {
        trim_single_process_working_set(*pid);
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
    let pid_list: Vec<Pid> = target_pids.iter().map(|p| Pid::from_u32(*p)).collect();
    let ram_after: u64 = {
        let sys_mutex = get_performance_system();
        let Ok(mut sys) = sys_mutex.lock() else {
            return 0.0;
        };
        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&pid_list),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        pid_list
            .iter()
            .filter_map(|p| sys.processes().get(p))
            .map(|p| p.memory())
            .sum()
    };
    let diff_bytes = ram_before.saturating_sub(ram_after);
    let diff_mb = (diff_bytes as f64) / (1024.0 * 1024.0);
    println!(
        "\x1b[32m[Performance] Trim Working Set completed for {} processes. Reclaimed: {:.2} MB RAM.\x1b[0m",
        target_pids.len(),
        diff_mb
    );
    (diff_mb * 100.0).round() / 100.0
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
                if let Some(parent) = proc.parent()
                    && pids.contains(&parent.as_u32()) {
                        pids.insert(pid_u32);
                        added = true;
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
