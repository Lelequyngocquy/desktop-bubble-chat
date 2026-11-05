use tauri::AppHandle;

/// Command to preview/test native Rust bubble
#[tauri::command]
pub fn toggle_native_bubble_preview(
    app: AppHandle,
    account_index: Option<u32>,
    visible: Option<bool>,
) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let idx = account_index.unwrap_or(1);
        let vis = visible.unwrap_or(true);
        crate::windows::native_bubble::toggle_native_bubble_prototype(Some(app), idx, vis)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Err("Native bubble only supported on Windows".into())
    }
}

/// Command to test native bubble pulse animation
#[tauri::command]
pub fn trigger_native_bubble_pulse_preview(
    account_index: Option<u32>,
) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let idx = account_index.unwrap_or(1);
        crate::windows::native_bubble::trigger_native_bubble_pulse(idx);
        Ok(true)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = account_index;
        Err("Native bubble only supported on Windows".into())
    }
}

/// Command to test setting native bubble unread badge count
#[tauri::command]
pub fn set_native_bubble_badge_preview(
    account_index: Option<u32>,
    count: u32,
) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let idx = account_index.unwrap_or(1);
        crate::windows::native_bubble::set_native_bubble_unread_count(idx, count);
        Ok(true)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (account_index, count);
        Err("Native bubble only supported on Windows".into())
    }
}

/// Command to test native bubble tuck animation
#[tauri::command]
pub fn tuck_native_bubble_preview(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let idx = account_index.unwrap_or(1);
        crate::windows::native_bubble::tuck_native_bubble(&app, idx);
        Ok(true)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, account_index);
        Err("Native bubble only supported on Windows".into())
    }
}

/// Command to test native bubble untuck animation
#[tauri::command]
pub fn untuck_native_bubble_preview(
    app: AppHandle,
    account_index: Option<u32>,
) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let idx = account_index.unwrap_or(1);
        crate::windows::native_bubble::untuck_native_bubble(&app, idx);
        Ok(true)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, account_index);
        Err("Native bubble only supported on Windows".into())
    }
}
