use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::state::ChatState;
use crate::utils::performance::{
    collect_performance_snapshot, trim_all_app_processes_memory, SystemPerformanceSnapshot,
};

/// IPC Command: Lấy snapshot tài nguyên thời gian thực của toàn bộ cây tiến trình DesktopBubbleChat
#[tauri::command]
pub fn get_performance_snapshot(
    app: AppHandle,
    state: State<'_, Arc<ChatState>>,
) -> Result<SystemPerformanceSnapshot, String> {
    Ok(collect_performance_snapshot(&app, &state))
}

/// IPC Command: Dọn dẹp bộ nhớ đệm (Working Set) của toàn bộ tiến trình ứng dụng
#[tauri::command]
pub fn trim_system_memory(app: AppHandle) -> Result<f64, String> {
    Ok(trim_all_app_processes_memory(&app))
}
