pub mod animator;
pub mod renderer;

#[cfg(target_os = "windows")]
pub mod manager;
#[cfg(target_os = "windows")]
pub mod window;

pub use animator::*;
pub use renderer::*;
#[cfg(target_os = "windows")]
pub use manager::*;
#[cfg(target_os = "windows")]
pub use window::*;
#[cfg(target_os = "windows")]
use tauri::Manager;

/// Helper bật/tắt một Bong bóng Native Win32 (ủy quyền cho Dedicated UI Thread)
#[cfg(target_os = "windows")]
pub fn toggle_native_bubble_prototype(
    app: Option<tauri::AppHandle>,
    account_index: u32,
    visible: bool,
) -> Result<bool, String> {
    if visible {
        let y_pos = 180 + ((account_index.max(1) - 1) as i32) * 72;
        if let Some(app_handle) = app {
            let config = crate::movement::get_bubble_physics_config(&app_handle);
            let monitor = app_handle
                .get_webview_window("main")
                .and_then(|m| m.current_monitor().ok().flatten())
                .or_else(|| app_handle.primary_monitor().ok().flatten());
            let scale = monitor.map(|m| m.scale_factor()).unwrap_or(1.0);
            let size = (config.size * scale).round() as u32;

            NativeBubbleManager::global().dispatch(NativeBubbleAction::Show {
                account_index,
                x: 30,
                y: y_pos,
                size,
                app: app_handle,
            });
        }
        Ok(true)
    } else {
        NativeBubbleManager::global().dispatch(NativeBubbleAction::Hide { account_index });
        Ok(false)
    }
}

/// Kiểm tra xem có Bong bóng Native nào đang được hiển thị hay không
#[cfg(target_os = "windows")]
pub fn has_native_bubble(account_index: u32) -> bool {
    NativeBubbleManager::global().has_bubble(account_index)
}

/// Cập nhật số lượng tin nhắn chưa đọc cho Native Bubble tương ứng
#[cfg(target_os = "windows")]
pub fn set_native_bubble_unread_count(account_index: u32, count: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetUnread { account_index, count });
}

/// Kích hoạt hiệu ứng sóng lan tỏa Pulse cho Native Bubble khi nhận tin nhắn mới
#[cfg(target_os = "windows")]
pub fn trigger_native_bubble_pulse(account_index: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::TriggerPulse { account_index });
}

/// Kiểm tra xem con trỏ chuột hiện tại có đang nằm trên Native Bubble hay không
#[cfg(target_os = "windows")]
pub fn is_cursor_over_native_bubble(account_index: u32) -> bool {
    NativeBubbleManager::global().is_cursor_over(account_index)
}

/// Kích hoạt hoạt ảnh lò xo 60 FPS thụt Bong bóng Native vào mép
#[cfg(target_os = "windows")]
pub fn tuck_native_bubble(app: &tauri::AppHandle, account_index: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::Tuck {
        account_index,
        app: app.clone(),
    });
}

/// Kích hoạt hoạt ảnh lò xo 60 FPS trượt Bong bóng Native ra ngoài mép
#[cfg(target_os = "windows")]
pub fn untuck_native_bubble(app: &tauri::AppHandle, account_index: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::Untuck {
        account_index,
        app: app.clone(),
    });
}

/// Áp dụng chế độ luôn hiển thị trên cùng (Always-on-top) cho tất cả Native Bubble
#[cfg(target_os = "windows")]
pub fn set_native_bubble_always_on_top(enabled: bool) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetAlwaysOnTop { enabled });
}

/// Thay đổi kích thước hiển thị cho Native Bubble tương ứng
#[cfg(target_os = "windows")]
pub fn set_native_bubble_size(account_index: u32, size: u32) {
    NativeBubbleManager::global().dispatch(NativeBubbleAction::SetSize { account_index, size });
}

#[cfg(not(target_os = "windows"))]
pub fn has_native_bubble(_account_index: u32) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn set_native_bubble_unread_count(_account_index: u32, _count: u32) {}

#[cfg(not(target_os = "windows"))]
pub fn trigger_native_bubble_pulse(_account_index: u32) {}

#[cfg(not(target_os = "windows"))]
pub fn is_cursor_over_native_bubble(_account_index: u32) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn tuck_native_bubble(_app: &tauri::AppHandle, _account_index: u32) {}

#[cfg(not(target_os = "windows"))]
pub fn untuck_native_bubble(_app: &tauri::AppHandle, _account_index: u32) {}

#[cfg(not(target_os = "windows"))]
pub fn set_native_bubble_always_on_top(_enabled: bool) {}

#[cfg(not(target_os = "windows"))]
pub fn set_native_bubble_size(_account_index: u32, _size: u32) {}
