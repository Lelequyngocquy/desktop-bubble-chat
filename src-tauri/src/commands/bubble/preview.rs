use tauri::AppHandle;

/// Command to preview/test native Rust bubble
#[tauri::command]
pub fn toggle_native_bubble_preview(
    app: AppHandle,
    account_index: Option<u32>,
    visible: Option<bool>,
) -> Result<bool, String> {
    let idx = account_index.unwrap_or(1);
    let vis = visible.unwrap_or(true);
    crate::operating_systems::current::bubble::toggle_native_bubble_prototype(Some(app), idx, vis)
}

/// Command to test native bubble pulse animation
#[tauri::command]
pub fn trigger_native_bubble_pulse_preview(
    account_index: Option<u32>,
) -> Result<bool, String> {
    let idx = account_index.unwrap_or(1);
    crate::operating_systems::current::bubble::trigger_native_bubble_pulse(idx);
    Ok(true)
}

/// Command to test setting native bubble unread badge count
#[tauri::command]
pub fn set_native_bubble_badge_preview(
    account_index: Option<u32>,
    count: u32,
) -> Result<bool, String> {
    let idx = account_index.unwrap_or(1);
    crate::operating_systems::current::bubble::set_native_bubble_unread_count(idx, count);
    Ok(true)
}

/// Command to test native bubble tuck animation
#[tauri::command]
pub fn tuck_native_bubble_preview(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<bool, String> {
    let idx = account_index.unwrap_or(1);
    crate::operating_systems::current::bubble::tuck_native_bubble(&app, idx);
    Ok(true)
}

/// Command to test native bubble untuck animation
#[tauri::command]
pub fn untuck_native_bubble_preview(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<bool, String> {
    let idx = account_index.unwrap_or(1);
    crate::operating_systems::current::bubble::untuck_native_bubble(&app, idx);
    Ok(true)
}
