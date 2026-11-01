use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use super::dto::TransitionLockGuard;
use crate::movement::BubbleRect;
use crate::platforms::messenger::windows::get_messenger_window_label;
use crate::state::ChatState;
use crate::windows::bubble::{
    ensure_bubble_positioned_and_visible, ensure_bubble_positioned_and_visible_with_extra,
    get_bubble_window_label,
};

/// Helper shutdown: Hủy cửa sổ chat webview, giải phóng RAM, terminate process nhưng bảo toàn cache trên đĩa
pub fn suspend_account_bubble_and_chat(app: &AppHandle, account_index: u32) {
    let idx = account_index;
    let bubble_label = get_bubble_window_label(idx);
    let chat_label = get_messenger_window_label(idx);

    if let Some(state) = app.try_state::<Arc<ChatState>>() {
        state.set_bypass_close_guard(&chat_label, true);
        state.set_account_suspended(idx, true);
        state.set_chat_ready(idx, false);
        state.remove_bubble_rect(idx);
    }

    // 1. Hủy cửa sổ chat Messenger (giải phóng WebView2 COM/DOM trong RAM)
    let win = app
        .get_webview_window(&chat_label)
        .or_else(|| if idx == 1 { app.get_webview_window("chat-messenger") } else { None });

    if let Some(cw) = win {
        let _ = cw.hide();
        let _ = cw.close();
        let _ = cw.destroy();
    }

    // 2. Ẩn và tiêu hủy cửa sổ Bubble (Dynamic Destruction giải phóng Renderer RAM của Bong bóng)
    #[cfg(target_os = "windows")]
    crate::windows::native_bubble::NativeBubbleManager::global().dispatch(
        crate::windows::native_bubble::NativeBubbleAction::Hide { account_index: idx }
    );

    if let Some(bw) = app.get_webview_window(&bubble_label) {
        let _ = bw.hide();
        let _ = bw.destroy();
    }

    #[cfg(target_os = "windows")]
    crate::windows::main_window::trim_current_process_working_set();

    // 4. Phát event thông báo đã ẩn
    let _ = app.emit("bubble-visibility-changed", serde_json::json!({
        "accountIndex": idx,
        "visible": false
    }));
}

/// Helper resume: Tái sử dụng flow của Bubble 05 (nạp lại webview từ cache đĩa và đặt vị trí snap)
pub fn resume_account_bubble_and_chat(
    app: &AppHandle,
    account_index: u32,
    committed_rects: Option<&[BubbleRect]>,
) -> Result<BubbleRect, String> {
    let idx = account_index;

    // 1. Đánh dấu gỡ trạng thái ngủ đông trong state và reset cờ chat ready
    if let Some(state) = app.try_state::<Arc<ChatState>>() {
        state.set_account_suspended(idx, false);
        state.set_chat_ready(idx, false);
    }

    // 2. Định vị và hiển thị Bubble lên màn hình ngay lập tức (Hàm chuẩn của Bubble 05).
    let rect = if let Some(committed) = committed_rects {
        ensure_bubble_positioned_and_visible_with_extra(app, idx, committed)?
    } else {
        ensure_bubble_positioned_and_visible(app, idx)?
    };

    // 3. Phát event thông báo
    let _ = app.emit("bubble-spawned", serde_json::json!({
        "accountIndex": idx,
        "x": rect.x,
        "y": rect.y
    }));
    let _ = app.emit("bubble-visibility-changed", serde_json::json!({
        "accountIndex": idx,
        "visible": true
    }));

    Ok(rect)
}

/// Command điều khiển ẩn/hiện cửa sổ bong bóng nổi cho tài khoản chỉ định
#[tauri::command]
pub async fn toggle_bubble(
    app: AppHandle,
    visible: bool,
    account_index: Option<u32>,
) -> Result<(), String> {
    let idx = account_index.unwrap_or(1);

    let state = app.try_state::<Arc<ChatState>>().map(|s| Arc::clone(&s));
    if let Some(s) = &state {
        if !s.try_acquire_transition_lock(idx) {
            println!("[toggle_bubble] Account {} is transitioning, skipping duplicate toggle", idx);
            return Ok(());
        }
    }
    let guard = TransitionLockGuard::new(state, idx);

    if visible {
        match resume_account_bubble_and_chat(&app, idx, None) {
            Ok(_) => println!("[toggle_bubble] Account {} bubble resumed successfully", idx),
            Err(e) => eprintln!("[toggle_bubble ERROR] Failed to resume bubble for account {}: {}", idx, e),
        }
    } else {
        suspend_account_bubble_and_chat(&app, idx);
        println!("[toggle_bubble] Account {} bubble suspended successfully", idx);
    }

    drop(guard);

    let _ = crate::tray::update_tray_menu(&app);

    Ok(())
}

/// Helper & command bật/tắt toàn bộ bong bóng nổi tuần tự (Async Sequential Pipeline & Committed Slots)
pub async fn toggle_all_bubbles_impl(
    app: &AppHandle,
    force_visible: Option<bool>,
    account_indices: Option<Vec<u32>>,
) -> bool {
    let mut visible_indices: Vec<u32> = Vec::new();
    for idx in 1..=5 {
        if crate::tray::is_bubble_visible(app, idx) {
            visible_indices.push(idx);
        }
    }

    let any_visible = !visible_indices.is_empty();

    let next_visible = match force_visible {
        Some(v) => v,
        None => !any_visible,
    };

    if next_visible {
        let targets = if let Some(indices) = account_indices {
            indices
        } else {
            let available = crate::tray::get_available_account_indices(app);
            if available.is_empty() {
                vec![1]
            } else {
                available
            }
        };

        let mut committed_rects: Vec<BubbleRect> = Vec::new();

        for (i, idx) in targets.iter().enumerate() {
            match resume_account_bubble_and_chat(app, *idx, Some(&committed_rects)) {
                Ok(rect) => {
                    println!("[toggle_all_bubbles] Account {} bubble resumed successfully", idx);
                    committed_rects.push(rect);
                }
                Err(err) => {
                    eprintln!("[toggle_all_bubbles ERROR] Failed to resume bubble for account {}: {}", idx, err);
                }
            }
            // Giãn cách 70ms giữa các bubble để Windows DWM vẽ và tạo hiệu ứng thác nước mượt mà
            if i + 1 < targets.len() {
                tokio::time::sleep(std::time::Duration::from_millis(70)).await;
            }
        }
    } else {
        // Ghi nhớ danh sách các Bubble đang mở trước khi ẩn
        if let Some(chat_state) = app.try_state::<Arc<ChatState>>() {
            chat_state.save_previously_visible_bubbles(visible_indices);
        }

        for idx in 1..=5 {
            suspend_account_bubble_and_chat(app, idx);
        }
    }

    let _ = crate::tray::update_tray_menu(app);

    next_visible
}

#[tauri::command]
pub async fn toggle_all_bubbles(
    app: AppHandle,
    visible: Option<bool>,
    account_indices: Option<Vec<u32>>,
) -> Result<bool, String> {
    Ok(toggle_all_bubbles_impl(&app, visible, account_indices).await)
}
