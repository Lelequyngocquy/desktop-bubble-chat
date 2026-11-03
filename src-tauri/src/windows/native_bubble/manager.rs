#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::mpsc::{channel, Sender};
#[cfg(target_os = "windows")]
use std::sync::{Arc, OnceLock, RwLock};
#[cfg(target_os = "windows")]
use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
use super::window::{ensure_window_class_registered, NativeBubbleWindow, POINT};

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MSG {
    pub hwnd: *mut std::ffi::c_void,
    pub message: u32,
    pub wparam: usize,
    pub lparam: isize,
    pub time: u32,
    pub pt: POINT,
    pub l_private: u32,
}

#[cfg(target_os = "windows")]
const WM_USER: u32 = 0x0400;
#[cfg(target_os = "windows")]
const WM_BUBBLE_DISPATCH: u32 = WM_USER + 200;

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn GetMessageW(
        lpMsg: *mut MSG,
        hWnd: *mut std::ffi::c_void,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
    ) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostThreadMessageW(
        idThread: u32,
        Msg: u32,
        wParam: usize,
        lParam: isize,
    ) -> i32;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentThreadId() -> u32;
}

/// Các lệnh điều khiển gửi tới luồng Win32 chuyên dụng của Bong bóng Native
#[cfg(target_os = "windows")]
pub enum NativeBubbleAction {
    Show {
        account_index: u32,
        x: i32,
        y: i32,
        size: u32,
        app: AppHandle,
    },
    Hide {
        account_index: u32,
    },
    SetPosition {
        account_index: u32,
        x: i32,
        y: i32,
    },
    AnimateTo {
        account_index: u32,
        target_x: i32,
        target_y: i32,
        duration_ms: u64,
    },
    SetUnread {
        account_index: u32,
        count: u32,
    },
    TriggerPulse {
        account_index: u32,
    },
    Tuck {
        account_index: u32,
        app: AppHandle,
    },
    Untuck {
        account_index: u32,
        app: AppHandle,
    },
    SetAlwaysOnTop {
        enabled: bool,
    },
    SetSize {
        account_index: u32,
        size: u32,
    },
}

/// Trình quản lý luồng Win32 chuyên trách (Dedicated Native Bubble UI Thread)
/// Đảm bảo tất cả HWND được tạo, vẽ, và hủy trên đúng 1 luồng OS có vòng lặp GetMessageW
#[cfg(target_os = "windows")]
pub struct NativeBubbleManager {
    thread_id: u32,
    sender: Sender<NativeBubbleAction>,
    bounds_cache: Arc<RwLock<HashMap<u32, (i32, i32, u32, u32)>>>,
}

#[cfg(target_os = "windows")]
static GLOBAL_MANAGER: OnceLock<NativeBubbleManager> = OnceLock::new();

#[cfg(target_os = "windows")]
impl NativeBubbleManager {
    /// Lấy hoặc khởi tạo instance quản lý luồng bong bóng duy nhất của toàn hệ thống
    pub fn global() -> &'static Self {
        GLOBAL_MANAGER.get_or_init(|| {
            let (init_tx, init_rx) = channel::<u32>();
            let (cmd_tx, cmd_rx) = channel::<NativeBubbleAction>();
            let bounds_cache = Arc::new(RwLock::new(HashMap::new()));
            let bounds_clone = Arc::clone(&bounds_cache);

            std::thread::Builder::new()
                .name("native-bubble-worker".into())
                .spawn(move || {
                    let thread_id = unsafe { GetCurrentThreadId() };
                    ensure_window_class_registered();

                    // Khởi tạo hàng đợi thông điệp Win32 cho luồng
                    let mut dummy_pt = POINT { x: 0, y: 0 };
                    unsafe { GetCursorPos(&mut dummy_pt); }

                    let _ = init_tx.send(thread_id);

                    let mut bubbles: HashMap<u32, NativeBubbleWindow> = HashMap::new();
                    let mut msg: MSG = unsafe { std::mem::zeroed() };

                    while unsafe { GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) } > 0 {
                        if msg.message == WM_BUBBLE_DISPATCH {
                            // Xử lý toàn bộ lệnh gửi từ các luồng khác
                            while let Ok(action) = cmd_rx.try_recv() {
                                match action {
                                    NativeBubbleAction::Show { account_index, x, y, size, app } => {
                                        if let Some(win) = bubbles.get_mut(&account_index) {
                                            win.set_position(x, y);
                                            win.resize(size);
                                            win.show();
                                        } else {
                                            match NativeBubbleWindow::new(Some(app), account_index, x, y, size) {
                                                Ok(win) => {
                                                    win.show();
                                                    bubbles.insert(account_index, win);
                                                }
                                                Err(e) => {
                                                    eprintln!("[NativeBubble ERROR] Không thể tạo cửa sổ cho account {}: {}", account_index, e);
                                                }
                                            }
                                        }
                                        if let Ok(mut cache) = bounds_clone.write() {
                                            cache.insert(account_index, (x, y, size, size));
                                        }
                                    }
                                    NativeBubbleAction::Hide { account_index } => {
                                        if let Some(win) = bubbles.remove(&account_index) {
                                            win.hide();
                                            // win tự động DestroyWindow an toàn trên đúng luồng này qua Drop
                                        }
                                        if let Ok(mut cache) = bounds_clone.write() {
                                            cache.remove(&account_index);
                                        }
                                    }
                                    NativeBubbleAction::SetPosition { account_index, x, y } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            win.set_position(x, y);
                                            if let Ok(mut cache) = bounds_clone.write() {
                                                if let Some(b) = cache.get_mut(&account_index) {
                                                    b.0 = x;
                                                    b.1 = y;
                                                }
                                            }
                                        }
                                    }
                                    NativeBubbleAction::AnimateTo { account_index, target_x, target_y, duration_ms } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            win.animate_to(target_x, target_y, duration_ms);
                                        }
                                    }
                                    NativeBubbleAction::SetUnread { account_index, count } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            win.set_unread_count(count);
                                        }
                                    }
                                    NativeBubbleAction::TriggerPulse { account_index } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            win.trigger_pulse();
                                        }
                                    }
                                    NativeBubbleAction::Tuck { account_index, app } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            let monitor = app
                                                .get_webview_window("main")
                                                .and_then(|m| m.current_monitor().ok().flatten())
                                                .or_else(|| app.primary_monitor().ok().flatten());

                                            if let Some(mon) = monitor {
                                                let screen = crate::movement::ScreenMetrics::from_monitor(&mon);
                                                let config = crate::movement::get_bubble_physics_config(&app);
                                                let (cur_x, cur_y, _, _) = win.get_bounds();

                                                let target_pos = crate::movement::BubbleMovementEngine::calculate_hover_pos(
                                                    &screen,
                                                    &config,
                                                    cur_x,
                                                    cur_y,
                                                    false,
                                                );
                                                win.animate_to(target_pos.x, target_pos.y, 180);
                                            }
                                        }
                                    }
                                    NativeBubbleAction::Untuck { account_index, app } => {
                                        if let Some(win) = bubbles.get(&account_index) {
                                            let monitor = app
                                                .get_webview_window("main")
                                                .and_then(|m| m.current_monitor().ok().flatten())
                                                .or_else(|| app.primary_monitor().ok().flatten());

                                            if let Some(mon) = monitor {
                                                let screen = crate::movement::ScreenMetrics::from_monitor(&mon);
                                                let config = crate::movement::get_bubble_physics_config(&app);
                                                let (cur_x, cur_y, _, _) = win.get_bounds();

                                                let target_pos = crate::movement::BubbleMovementEngine::calculate_hover_pos(
                                                    &screen,
                                                    &config,
                                                    cur_x,
                                                    cur_y,
                                                    true,
                                                );
                                                win.animate_to(target_pos.x, target_pos.y, 180);
                                            }
                                        }
                                    }
                                    NativeBubbleAction::SetAlwaysOnTop { enabled } => {
                                        for win in bubbles.values() {
                                            win.set_always_on_top(enabled);
                                        }
                                    }
                                    NativeBubbleAction::SetSize { account_index, size } => {
                                        if let Some(win) = bubbles.get_mut(&account_index) {
                                            win.resize(size);
                                        }
                                        if let Ok(mut cache) = bounds_clone.write() {
                                            if let Some(b) = cache.get_mut(&account_index) {
                                                b.2 = size;
                                                b.3 = size;
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Xử lý các thông điệp chuột, bàn phím, timer cho các HWND thuộc luồng này
                            unsafe {
                                TranslateMessage(&msg);
                                DispatchMessageW(&msg);
                            }
                        }
                    }
                })
                .expect("Không thể tạo luồng Native Bubble Worker");

            let thread_id = init_rx.recv().expect("Không thể nhận thread_id từ Native Bubble Worker");

            Self {
                thread_id,
                sender: cmd_tx,
                bounds_cache,
            }
        })
    }

    /// Gửi một hành động tới luồng chuyên dụng và đánh thức vòng lặp GetMessageW
    pub fn dispatch(&self, action: NativeBubbleAction) {
        if self.sender.send(action).is_ok() {
            unsafe {
                PostThreadMessageW(self.thread_id, WM_BUBBLE_DISPATCH, 0, 0);
            }
        }
    }

    /// Kiểm tra xem tài khoản có Bong bóng Native nào đang hiển thị hay không
    pub fn has_bubble(&self, account_index: u32) -> bool {
        if let Ok(cache) = self.bounds_cache.read() {
            return cache.contains_key(&account_index);
        }
        false
    }

    /// Lấy tọa độ và kích thước hiện tại của Bong bóng Native
    pub fn get_bounds(&self, account_index: u32) -> Option<(i32, i32, u32, u32)> {
        if let Ok(cache) = self.bounds_cache.read() {
            return cache.get(&account_index).copied();
        }
        None
    }

    /// Cập nhật tọa độ trong bộ nhớ đệm (gọi từ window.rs khi kéo thả chuột)
    pub fn update_cached_bounds(&self, account_index: u32, x: i32, y: i32, w: u32, h: u32) {
        if let Ok(mut cache) = self.bounds_cache.write() {
            cache.insert(account_index, (x, y, w, h));
        }
    }

    /// Kiểm tra xem con trỏ chuột có đang nằm trên Bong bóng Native hay không
    pub fn is_cursor_over(&self, account_index: u32) -> bool {
        if let Some((x, y, w, h)) = self.get_bounds(account_index) {
            let mut pt = POINT { x: 0, y: 0 };
            unsafe {
                if GetCursorPos(&mut pt) != 0 {
                    return pt.x >= x && pt.x <= x + w as i32 && pt.y >= y && pt.y <= y + h as i32;
                }
            }
        }
        false
    }
}

/// Cập nhật tọa độ bộ nhớ đệm nhanh (hàm tiện ích gọi từ window.rs)
#[cfg(target_os = "windows")]
pub fn update_cached_bubble_bounds(account_index: u32, x: i32, y: i32, w: u32, h: u32) {
    NativeBubbleManager::global().update_cached_bounds(account_index, x, y, w, h);
}
