use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::state::ChatState;
use crate::utils::performance::{
    collect_performance_snapshot, trim_all_app_processes_memory, SystemPerformanceSnapshot,
};

/// IPC Command: Retrieves real-time performance snapshot of the process tree
#[tauri::command]
pub fn get_performance_snapshot(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
) -> Result<SystemPerformanceSnapshot, String> {
    Ok(collect_performance_snapshot(&app, &state))
}

/// IPC Command: Trims working set memory of all application processes
#[tauri::command]
pub fn trim_system_memory(app: AppHandle) -> Result<f64, String> {
    Ok(trim_all_app_processes_memory(&app))
}
