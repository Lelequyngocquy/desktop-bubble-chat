use std::sync::Arc;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Manager};

use super::helpers::{get_available_account_indices, is_bubble_visible, is_main_window_open};
use super::TRAY_ID;

/// Dựng menu động cho System Tray: Mở Trang chủ, Bật/Tắt tất cả, list từng Bong bóng kèm checkmark
pub fn build_tray_menu(app_handle: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let menu = Menu::new(app_handle)?;

    let is_open = is_main_window_open(app_handle);
    let home_title = if is_open {
        "Ẩn Trang chủ"
    } else {
        "Mở Trang chủ"
    };
    let show_home = MenuItem::with_id(app_handle, "show_home", home_title, true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app_handle)?;
    menu.append(&show_home)?;
    menu.append(&sep1)?;

    let available_accounts = get_available_account_indices(app_handle);

    if !available_accounts.is_empty() {
        // Kiểm tra xem có bất kỳ bong bóng nào đang hiển thị không
        let mut any_visible = false;
        for idx in &available_accounts {
            if is_bubble_visible(app_handle, *idx) {
                any_visible = true;
                break;
            }
        }

        // 1. Mục Master: Bật tất cả / Ẩn tất cả bong bóng
        let toggle_all_text = if any_visible {
            "Ẩn tất cả bong bóng"
        } else {
            "Bật tất cả bong bóng"
        };
        let toggle_all_item = MenuItem::with_id(
            app_handle,
            "toggle_all_bubbles",
            toggle_all_text,
            true,
            None::<&str>,
        )?;
        menu.append(&toggle_all_item)?;

        let sep2 = PredefinedMenuItem::separator(app_handle)?;
        menu.append(&sep2)?;

        // 2. Danh sách từng Bong bóng đang có (CheckMenuItem: ✓ khi đang bật)
        for idx in &available_accounts {
            let is_visible = is_bubble_visible(app_handle, *idx);

            let display_name = if let Some(chat_state) = app_handle.try_state::<Arc<crate::state::ChatState>>() {
                if let Some(auth) = chat_state.get_account_auth(*idx) {
                    let trimmed = auth.name.trim();
                    if !trimmed.is_empty() && trimmed != format!("Messenger {:02}", idx) {
                        format!("Messenger {:02} ({})", idx, trimmed)
                    } else {
                        format!("Messenger {:02}", idx)
                    }
                } else {
                    format!("Messenger {:02}", idx)
                }
            } else {
                format!("Messenger {:02}", idx)
            };

            let item_id = format!("toggle_bubble_{}", idx);
            let check_item = CheckMenuItem::with_id(
                app_handle,
                &item_id,
                display_name,
                true,
                is_visible,
                None::<&str>,
            )?;
            menu.append(&check_item)?;
        }

        let sep3 = PredefinedMenuItem::separator(app_handle)?;
        menu.append(&sep3)?;
    }

    let quit = MenuItem::with_id(app_handle, "quit", "Thoát ứng dụng...", true, None::<&str>)?;
    menu.append(&quit)?;

    Ok(menu)
}

/// Cập nhật menu của System Tray theo trạng thái thực tế
pub fn update_tray_menu(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tray) = app_handle.tray_by_id(TRAY_ID) {
        let menu = build_tray_menu(app_handle)?;
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}
