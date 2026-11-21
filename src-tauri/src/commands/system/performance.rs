use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::state::ChatState;
use crate::utils::performance::{
    collect_performance_snapshot, trim_all_app_processes_memory, SystemPerformanceSnapshot,
};

/// IPC Command: Retrieves real-time performance snapshot of the process tree
#[tauri::command]
pub async fn get_performance_snapshot(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
) -> Result<SystemPerformanceSnapshot, String> {
    let state_arc = Arc::clone(&state);
    tokio::task::spawn_blocking(move || collect_performance_snapshot(&app, &state_arc))
        .await
        .map_err(|e| format!("Performance snapshot task failed: {}", e))
}

/// IPC Command: Trims working set memory of all application processes
#[tauri::command]
pub async fn trim_system_memory(app: AppHandle) -> Result<f64, String> {
    tokio::task::spawn_blocking(move || trim_all_app_processes_memory(&app))
        .await
        .map_err(|e| format!("Memory trim task failed: {}", e))
}
