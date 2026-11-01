use std::sync::Arc;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

pub const TRAY_ID: &str = "main_tray";

/// Lấy danh sách các tài khoản đang có (có thư mục profile trên đĩa hoặc đã xác thực trong ChatState)
pub fn get_available_account_indices(app_handle: &AppHandle) -> Vec<u32> {
    let mut available = std::collections::BTreeSet::new();

    // 1. Quét các thư mục profile trên đĩa: profiles/messenger_01 .. profiles/messenger_05
    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        let profiles_dir = app_data_dir.join("profiles");
        for idx in 1..=5 {
            if profiles_dir.join(format!("messenger_{:02}", idx)).exists() {
                available.insert(idx);
            }
        }
    }

    // 2. Quét state xác thực trong ChatState
    if let Some(chat_state) = app_handle.try_state::<Arc<crate::state::ChatState>>() {
        if let Ok(map) = chat_state.account_auth.lock() {
            for (idx, info) in map.iter() {
                if info.connected {
                    available.insert(*idx);
                }
            }
        }
    }

    available.into_iter().collect()
}

/// Hiển thị hộp thoại Win32 xác nhận trước khi thoát để chống bấm nhầm
#[cfg(target_os = "windows")]
pub fn confirm_exit_dialog() -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let text_wide: Vec<u16> = OsStr::new(
        "Bạn có chắc chắn muốn thoát hoàn toàn DesktopBubbleChat không?\n\n• Toàn bộ các Bong bóng nổi sẽ đóng lại.\n• Các cửa sổ chat Messenger sẽ ngừng hoạt động."
    )
    .encode_wide()
    .chain(std::iter::once(0))
    .collect();

    let title_wide: Vec<u16> = OsStr::new("Xác nhận thoát DesktopBubbleChat")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(
            hWnd: *mut std::ffi::c_void,
            lpText: *const u16,
            lpCaption: *const u16,
            uType: u32,
        ) -> i32;
    }

    const MB_YESNO: u32 = 0x00000004;
    const MB_ICONQUESTION: u32 = 0x00000020;
    const MB_DEFBUTTON2: u32 = 0x00000100; // Mặc định chọn nút "Không" (No) để chống bấm nhầm
    const MB_SYSTEMMODAL: u32 = 0x00001000; // Đưa hộp thoại lên trên cùng màn hình
    const IDYES: i32 = 6;

    let result = unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text_wide.as_ptr(),
            title_wide.as_ptr(),
            MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2 | MB_SYSTEMMODAL,
        )
    };

    result == IDYES
}

#[cfg(not(target_os = "windows"))]
pub fn confirm_exit_dialog() -> bool {
    true
}

/// Helper điều khiển bật/tắt riêng một bong bóng chỉ định từ Tray Icon
pub fn toggle_single_bubble_from_tray(app: &AppHandle, account_index: u32) {
    let bubble_label = format!("bubble-{:02}", account_index);
    let is_visible = app
        .get_webview_window(&bubble_label)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);

    let next_visible = !is_visible;
    let _ = crate::commands::bubble::toggle_bubble(app.clone(), next_visible, Some(account_index));
}

/// Dựng menu động cho System Tray: Mở Trang chủ, Bật/Tắt tất cả, list từng Bong bóng kèm checkmark
pub fn build_tray_menu(app_handle: &AppHandle) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let menu = Menu::new(app_handle)?;

    let show_home = MenuItem::with_id(app_handle, "show_home", "Mở Trang chủ", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app_handle)?;
    menu.append(&show_home)?;
    menu.append(&sep1)?;

    let available_accounts = get_available_account_indices(app_handle);

    if !available_accounts.is_empty() {
        // Kiểm tra xem có bất kỳ bong bóng nào đang hiển thị không
        let mut any_visible = false;
        for idx in &available_accounts {
            let label = format!("bubble-{:02}", idx);
            if let Some(bw) = app_handle.get_webview_window(&label) {
                if bw.is_visible().unwrap_or(false) {
                    any_visible = true;
                    break;
                }
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
            let label = format!("bubble-{:02}", idx);
            let is_visible = app_handle
                .get_webview_window(&label)
                .and_then(|w| w.is_visible().ok())
                .unwrap_or(false);

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

/// Khởi tạo System Tray Icon & Menu điều khiển
pub fn setup_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let menu = build_tray_menu(&app.handle())?;
    let icon = app.default_window_icon().cloned().ok_or("Failed to get app icon")?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app_handle, event| {
            let id_str = event.id.as_ref();
            match id_str {
                "show_home" => {
                    if let Some(main_win) = app_handle.get_webview_window("main") {
                        let _ = main_win.show();
                        let _ = main_win.set_focus();
                    }
                }
                "toggle_all_bubbles" | "toggle_bubble_item" => {
                    let handle = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        crate::commands::bubble::toggle_all_bubbles_impl(&handle, None, None).await;
                        let _ = update_tray_menu(&handle);
                    });
                }
                "quit" => {
                    if confirm_exit_dialog() {
                        app_handle.exit(0);
                    }
                }
                id if id.starts_with("toggle_bubble_") => {
                    if let Ok(idx) = id.trim_start_matches("toggle_bubble_").parse::<u32>() {
                        let handle = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            toggle_single_bubble_from_tray(&handle, idx);
                            let _ = update_tray_menu(&handle);
                        });
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app_handle = tray.app_handle();
                if let Some(main_win) = app_handle.get_webview_window("main") {
                    let _ = main_win.show();
                    let _ = main_win.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
