use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const MESSENGER_INJECT_SCRIPT: &str = include_str!("scripts/messenger_inject.js");

// Trạng thái theo dõi thời điểm ẩn cửa sổ để tránh xung đột race-condition giữa Blur và Click
pub struct ChatState {
    pub last_blur_time: AtomicI64,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            last_blur_time: AtomicI64::new(0),
        }
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

// Trích xuất số unread từ tiêu đề cửa sổ (ví dụ: "(3) Messenger", "(99+) Messenger")
fn parse_unread_from_title(title: &str) -> u32 {
    if let (Some(start), Some(end)) = (title.find('('), title.find(')')) {
        if start < end {
            let inner = &title[start + 1..end];
            let digits: String = inner.chars().filter(|c| c.is_ascii_digit()).collect();
            return digits.parse::<u32>().unwrap_or(0);
        }
    }
    0
}

// Command nhận số unread từ injected script và phát event cho toàn bộ app
#[tauri::command]
fn update_unread_badge(app: AppHandle, count: u32) -> Result<(), String> {
    let _ = app.emit("badge-updated", count);
    Ok(())
}

// Command giả lập / thiết lập trực tiếp số badge (tiện cho test và điều khiển)
#[tauri::command]
fn set_badge_count(app: AppHandle, count: u32) -> Result<(), String> {
    let _ = app.emit("badge-updated", count);
    Ok(())
}

// Command điều khiển ẩn/hiện cửa sổ bong bóng nổi
#[tauri::command]
fn toggle_bubble(app: AppHandle, visible: bool) -> Result<(), String> {
    if let Some(bubble_window) = app.get_webview_window("bubble") {
        if visible {
            bubble_window.show().map_err(|e| e.to_string())?;
        } else {
            bubble_window.hide().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// Command bật/tắt khởi động cùng Windows
#[tauri::command]
fn toggle_autostart(enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let exe_path = current_exe.to_string_lossy().to_string();

        if enabled {
            let _ = std::process::Command::new("reg")
                .args(&[
                    "add",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "DesktopBubbleChat",
                    "/t",
                    "REG_SZ",
                    "/d",
                    &format!("\"{}\"", exe_path),
                    "/f",
                ])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .output();
        } else {
            let _ = std::process::Command::new("reg")
                .args(&[
                    "delete",
                    "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                    "/v",
                    "DesktopBubbleChat",
                    "/f",
                ])
                .creation_flags(0x08000000)
                .output();
        }
    }
    Ok(enabled)
}

// Command kiểm tra xem app đã bật khởi động cùng Windows chưa
#[tauri::command]
fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("reg")
            .args(&[
                "query",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run",
                "/v",
                "DesktopBubbleChat",
            ])
            .creation_flags(0x08000000)
            .output();

        if let Ok(out) = output {
            return out.status.success();
        }
    }
    false
}

// Command thoát ứng dụng hoàn toàn
#[tauri::command]
fn exit_app(app: AppHandle) {
    app.exit(0);
}

// Command tự động hít vào mép màn hình (Snap to Edge) & Dismiss đáy
#[tauri::command]
fn snap_bubble(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("bubble") {
        if let Ok(Some(monitor)) = window.current_monitor() {
            let win_pos = window.outer_position().map_err(|e| e.to_string())?;
            let monitor_pos = monitor.position();
            let monitor_size = monitor.size();
            let scale = monitor.scale_factor();

            let bubble_width = (64.0 * scale) as i32;
            let margin = (12.0 * scale) as i32;
            let taskbar_margin = (60.0 * scale) as i32;

            // Kiểm tra nếu kéo xuống sát đáy màn hình -> Tự động ẩn vào Tray
            let bottom_threshold = monitor_pos.y + monitor_size.height as i32 - (90.0 * scale) as i32;
            if win_pos.y >= bottom_threshold {
                let _ = window.hide();
                return Ok(());
            }

            // Tính toán hít mép trái hoặc mép phải
            let mid_x = monitor_pos.x + (monitor_size.width as i32) / 2;
            let target_x = if win_pos.x + bubble_width / 2 < mid_x {
                monitor_pos.x + margin
            } else {
                monitor_pos.x + monitor_size.width as i32 - bubble_width - margin
            };

            // Giới hạn trục Y không lọt ra ngoài mép trên và thanh Taskbar
            let min_y = monitor_pos.y + margin;
            let max_y = monitor_pos.y + monitor_size.height as i32 - bubble_width - taskbar_margin;
            let target_y = win_pos.y.clamp(min_y, max_y);

            window
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: target_x,
                    y: target_y,
                }))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// Command bật/tắt khung chat Messenger với thuật toán định vị thông minh cạnh bong bóng
#[tauri::command]
fn toggle_chat(app: AppHandle, state: tauri::State<'_, Arc<ChatState>>) -> Result<bool, String> {
    let chat_window = match app.get_webview_window("chat-messenger") {
        Some(w) => w,
        None => return Err("Chat window 'chat-messenger' not found".into()),
    };

    let is_visible = chat_window.is_visible().unwrap_or(false);

    if is_visible {
        state.last_blur_time.store(now_millis(), Ordering::SeqCst);
        chat_window.hide().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        // Tránh lỗi nảy cửa sổ: Nếu vừa bị ẩn do mất focus (blur) trong vòng 250ms -> Không mở lại
        let last_blur = state.last_blur_time.load(Ordering::SeqCst);
        if now_millis() - last_blur < 250 {
            return Ok(false);
        }

        // Nạp observer & responsive CSS vào webview của Messenger
        let _ = chat_window.eval(MESSENGER_INJECT_SCRIPT);

        // Kiểm tra tiêu đề cửa sổ hiện tại để cập nhật ngay số badge
        if let Ok(title) = chat_window.title() {
            let count = parse_unread_from_title(&title);
            let _ = app.emit("badge-updated", count);
        }

        // Định vị thông minh áp sát bong bóng nổi
        if let Some(bubble_window) = app.get_webview_window("bubble") {
            if let Ok(bubble_pos) = bubble_window.outer_position() {
                if let Ok(Some(monitor)) = bubble_window.current_monitor() {
                    let monitor_pos = monitor.position();
                    let monitor_size = monitor.size();
                    let scale = monitor.scale_factor();

                    let chat_w = (420.0 * scale) as i32;
                    let chat_h = (620.0 * scale) as i32;
                    let bubble_w = (64.0 * scale) as i32;
                    let gap = (8.0 * scale) as i32;

                    // Nếu bong bóng ở nửa phải màn hình -> bung khung chat về bên trái, và ngược lại
                    let mid_x = monitor_pos.x + (monitor_size.width as i32) / 2;
                    let target_x = if bubble_pos.x + bubble_w / 2 >= mid_x {
                        bubble_pos.x - chat_w - gap
                    } else {
                        bubble_pos.x + bubble_w + gap
                    };

                    // Căn giữa theo trục Y của bong bóng, clamp không cho tràn màn hình
                    let target_y_raw = bubble_pos.y - chat_h / 2 + bubble_w / 2;
                    let min_y = monitor_pos.y + (12.0 * scale) as i32;
                    let max_y = monitor_pos.y + monitor_size.height as i32 - chat_h - (60.0 * scale) as i32;
                    let target_y = target_y_raw.clamp(min_y, max_y);

                    let _ = chat_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x: target_x,
                        y: target_y,
                    }));
                }
            }
        }

        chat_window.show().map_err(|e| e.to_string())?;
        chat_window.set_focus().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

// Khởi tạo System Tray Icon & Menu điều khiển
fn setup_system_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_home = MenuItem::with_id(app, "show_home", "Mở Trang chủ", true, None::<&str>)?;
    let toggle_bubble_item = MenuItem::with_id(app, "toggle_bubble_item", "Bật/Tắt Bong bóng nổi", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Thoát hoàn toàn", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_home, &toggle_bubble_item, &separator, &quit])?;

    let icon = app.default_window_icon().cloned().ok_or("Failed to get app icon")?;

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app_handle, event| {
            match event.id.as_ref() {
                "show_home" => {
                    if let Some(main_win) = app_handle.get_webview_window("main") {
                        let _ = main_win.show();
                        let _ = main_win.set_focus();
                    }
                }
                "toggle_bubble_item" => {
                    if let Some(bubble_win) = app_handle.get_webview_window("bubble") {
                        let is_vis = bubble_win.is_visible().unwrap_or(false);
                        if is_vis {
                            let _ = bubble_win.hide();
                        } else {
                            let _ = bubble_win.show();
                        }
                    }
                }
                "quit" => {
                    app_handle.exit(0);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let chat_state = Arc::new(ChatState::default());
    let chat_state_clone = Arc::clone(&chat_state);

    tauri::Builder::default()
        .manage(chat_state)
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            toggle_bubble,
            snap_bubble,
            toggle_chat,
            update_unread_badge,
            set_badge_count,
            toggle_autostart,
            is_autostart_enabled,
            exit_app
        ])
        .setup(move |app| {
            // 1. Cấu hình System Tray Menu & Icon
            if let Err(err) = setup_system_tray(app) {
                eprintln!("[Tray Error] Failed to setup system tray: {}", err);
            }

            // 2. Chặn việc đóng cửa sổ chính (Home Page) -> Ẩn về khay hệ thống, giữ chat heads chạy ngầm
            if let Some(main_window) = app.get_webview_window("main") {
                let main_clone = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = main_clone.hide();
                    }
                });
            }

            // 3. Lắng nghe sự kiện mất focus (Focused: false) trên cửa sổ chat -> Tự động ẩn
            if let Some(chat_window) = app.get_webview_window("chat-messenger") {
                let chat_clone = chat_window.clone();
                let state_clone = Arc::clone(&chat_state_clone);

                chat_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        state_clone.last_blur_time.store(now_millis(), Ordering::SeqCst);
                        let _ = chat_clone.hide();
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
