use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

use super::bubble::window::{
    describe_window_fullscreen_state, is_window_fullscreen, AllowSetForegroundWindow,
    AttachThreadInput, BringWindowToTop, GetCurrentThreadId, GetForegroundWindow, GetGUIThreadInfo,
    GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible, MapVirtualKeyW, SendInput,
    SetForegroundWindow, SetWindowPos, ShowWindow, ASFW_ANY, GUITHREADINFO, INPUT, INPUT_KEYBOARD,
    KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC, SWP_HIDEWINDOW,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_HIDE, SW_SHOWNA,
    VK_LMENU,
};

#[repr(C)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[repr(C)]
struct ITaskbarList2Vtbl {
    _query_interface: unsafe extern "system" fn(*mut ITaskbarList2, *const Guid, *mut *mut std::ffi::c_void) -> i32,
    _add_ref: unsafe extern "system" fn(*mut ITaskbarList2) -> u32,
    release: unsafe extern "system" fn(*mut ITaskbarList2) -> u32,
    hr_init: unsafe extern "system" fn(*mut ITaskbarList2) -> i32,
    _add_tab: unsafe extern "system" fn(*mut ITaskbarList2, *mut std::ffi::c_void) -> i32,
    _delete_tab: unsafe extern "system" fn(*mut ITaskbarList2, *mut std::ffi::c_void) -> i32,
    _activate_tab: unsafe extern "system" fn(*mut ITaskbarList2, *mut std::ffi::c_void) -> i32,
    _set_active_alt: unsafe extern "system" fn(*mut ITaskbarList2, *mut std::ffi::c_void) -> i32,
    mark_fullscreen_window: unsafe extern "system" fn(*mut ITaskbarList2, *mut std::ffi::c_void, i32) -> i32,
}

#[repr(C)]
struct ITaskbarList2 {
    vtbl: *const ITaskbarList2Vtbl,
}

const CLSID_TASKBAR_LIST: Guid = Guid {
    data1: 0x56FDF344,
    data2: 0xFD6D,
    data3: 0x11D0,
    data4: [0x95, 0x8A, 0x00, 0x60, 0x97, 0xC9, 0xA0, 0x90],
};

const IID_ITASKBAR_LIST2: Guid = Guid {
    data1: 0x602D4995,
    data2: 0xB13A,
    data3: 0x429B,
    data4: [0xA6, 0x6E, 0x19, 0x35, 0xE4, 0x4F, 0x43, 0x17],
};

const COINIT_APARTMENTTHREADED: u32 = 0x2;
const CLSCTX_ALL: u32 = 0x17;
const S_OK: i32 = 0;
const S_FALSE: i32 = 1;
const RPC_E_CHANGED_MODE: i32 = 0x80010106_u32 as i32;

#[link(name = "ole32")]
unsafe extern "system" {
    fn CoInitializeEx(pvReserved: *mut std::ffi::c_void, dwCoInit: u32) -> i32;
    fn CoUninitialize();
    fn CoCreateInstance(
        rclsid: *const Guid,
        pUnkOuter: *mut std::ffi::c_void,
        dwClsContext: u32,
        riid: *const Guid,
        ppv: *mut *mut std::ffi::c_void,
    ) -> i32;
}

static MARKED_FULLSCREEN_HWNDS: OnceLock<Mutex<HashSet<isize>>> = OnceLock::new();
static OVERLAY_SESSIONS: OnceLock<Mutex<HashMap<isize, isize>>> = OnceLock::new();

fn marked_hwnds() -> &'static Mutex<HashSet<isize>> {
    MARKED_FULLSCREEN_HWNDS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn overlay_sessions() -> &'static Mutex<HashMap<isize, isize>> {
    OVERLAY_SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Sends extended Alt keystroke tap to unlock Windows foreground lock only when needed
unsafe fn unlock_foreground_with_alt_tap() {
    unsafe {
        let alt_sc = MapVirtualKeyW(u32::from(VK_LMENU), MAPVK_VK_TO_VSC) as u16;
        let mut inputs: [INPUT; 2] = std::mem::zeroed();
        inputs[0].r#type = INPUT_KEYBOARD;
        inputs[0].anonymous.ki = KEYBDINPUT { w_vk: VK_LMENU, w_scan: alt_sc, dw_flags: KEYEVENTF_EXTENDEDKEY, time: 0, dw_extra_info: 0 };
        inputs[1].r#type = INPUT_KEYBOARD;
        inputs[1].anonymous.ki = KEYBDINPUT { w_vk: VK_LMENU, w_scan: alt_sc, dw_flags: KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, time: 0, dw_extra_info: 0 };
        SendInput(2, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
    }
}

/// Transfers foreground focus to target fullscreen HWND BEFORE hiding chat_hwnd
unsafe fn handoff_foreground_then_hide(
    chat_hwnd: *mut std::ffi::c_void,
    saved_fg: *mut std::ffi::c_void,
) {
    unsafe {
        let cur_fg = GetForegroundWindow();
        let our_tid = GetCurrentThreadId();
        let mut gui: GUITHREADINFO = std::mem::zeroed();
        gui.cb_size = std::mem::size_of::<GUITHREADINFO>() as u32;
        let gui_ok = GetGUIThreadInfo(0, &mut gui) != 0;
        let focus_hwnd = if gui_ok { gui.hwnd_focus } else { std::ptr::null_mut() };
        let fg_tid = if !cur_fg.is_null() { GetWindowThreadProcessId(cur_fg, std::ptr::null_mut()) } else { 0 };
        let focus_tid = if !focus_hwnd.is_null() { GetWindowThreadProcessId(focus_hwnd, std::ptr::null_mut()) } else { 0 };
        let target_tid = GetWindowThreadProcessId(saved_fg, std::ptr::null_mut());
        let mut attached_tids = [0u32; 3];
        let mut attached_count = 0usize;
        for tid in [fg_tid, focus_tid, target_tid] {
            if tid != 0 && tid != our_tid && !attached_tids[..attached_count].contains(&tid) && AttachThreadInput(our_tid, tid, 1) != 0 {
                attached_tids[attached_count] = tid;
                attached_count += 1;
            }
        }
        AllowSetForegroundWindow(ASFW_ANY);
        let first_ok = SetForegroundWindow(saved_fg) != 0;
        let mut fg_after = GetForegroundWindow();
        let mut fallback_used = false;
        if !first_ok || fg_after != saved_fg {
            fallback_used = true;
            unlock_foreground_with_alt_tap();
            let _ = SetForegroundWindow(saved_fg);
            let _ = BringWindowToTop(saved_fg);
            fg_after = GetForegroundWindow();
        }
        for &tid in &attached_tids[..attached_count] {
            AttachThreadInput(our_tid, tid, 0);
        }
        SetWindowPos(
            chat_hwnd,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_HIDEWINDOW | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
        let post_hide_fg = GetForegroundWindow();
        println!(
            "[FullscreenOverlay] Handoff: chat={:?} saved_fg={:?} cur_fg={:?} focus={:?} first_ok={} fallback={} fg_after={:?} post_hide_fg={:?}",
            chat_hwnd, saved_fg, cur_fg, focus_hwnd, first_ok, fallback_used, fg_after, post_hide_fg
        );
    }
}

/// Calls ITaskbarList2::MarkFullscreenWindow on the given HWND via COM
fn mark_hwnd_fullscreen_for_taskbar(hwnd: *mut std::ffi::c_void, is_fullscreen: bool) -> bool {
    if hwnd.is_null() {
        return false;
    }
    unsafe {
        let init_hr = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED);
        let should_uninit = init_hr == S_OK || init_hr == S_FALSE;
        if !should_uninit && init_hr != RPC_E_CHANGED_MODE {
            return false;
        }
        let mut raw_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let create_hr = CoCreateInstance(
            &CLSID_TASKBAR_LIST,
            std::ptr::null_mut(),
            CLSCTX_ALL,
            &IID_ITASKBAR_LIST2,
            &mut raw_ptr,
        );
        let mut ok = false;
        if create_hr >= 0 && !raw_ptr.is_null() {
            let taskbar = raw_ptr as *mut ITaskbarList2;
            let vtbl = (*taskbar).vtbl;
            if ((*vtbl).hr_init)(taskbar) >= 0 {
                let flag = if is_fullscreen { 1 } else { 0 };
                ok = ((*vtbl).mark_fullscreen_window)(taskbar, hwnd, flag) >= 0;
            }
            ((*vtbl).release)(taskbar);
        }
        if should_uninit {
            CoUninitialize();
        }
        ok
    }
}

/// Clears fullscreen taskbar flag if this HWND was previously marked
fn clear_hwnd_fullscreen_mark(hwnd: *mut std::ffi::c_void) {
    let key = hwnd as isize;
    let was_marked = marked_hwnds().lock().map(|mut set| set.remove(&key)).unwrap_or(false);
    if was_marked {
        mark_hwnd_fullscreen_for_taskbar(hwnd, false);
    }
}

/// Shows chat window without activating or popping the Taskbar when a fullscreen app is active
pub fn open_window_if_fullscreen(window: &tauri::WebviewWindow) -> bool {
    let Ok(raw_hwnd) = window.hwnd() else { return false; };
    let hwnd = raw_hwnd.0;
    let fg_hwnd = unsafe { GetForegroundWindow() };
    let is_fg_fullscreen = fg_hwnd != hwnd && unsafe { is_window_fullscreen(fg_hwnd) };
    if !is_fg_fullscreen {
        if let Ok(mut map) = overlay_sessions().lock() {
            map.remove(&(hwnd as isize));
        }
        let hwnd_val = hwnd as usize;
        let _ = window.run_on_main_thread(move || {
            clear_hwnd_fullscreen_mark(hwnd_val as *mut std::ffi::c_void);
        });
        return false;
    }
    if let Ok(mut map) = overlay_sessions().lock() {
        map.insert(hwnd as isize, fg_hwnd as isize);
    }
    let hwnd_val = hwnd as usize;
    let fg_val = fg_hwnd as usize;
    let win_clone = window.clone();
    let _ = window.run_on_main_thread(move || {
        let hwnd_ptr = hwnd_val as *mut std::ffi::c_void;
        let fg_ptr = fg_val as *mut std::ffi::c_void;
        unsafe {
            ShowWindow(hwnd_ptr, SW_SHOWNA);
            SetWindowPos(
                hwnd_ptr,
                -1_isize as *mut std::ffi::c_void,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
        }
        let marked_ok = mark_hwnd_fullscreen_for_taskbar(hwnd_ptr, true);
        if marked_ok
            && let Ok(mut set) = marked_hwnds().lock() {
                set.insert(hwnd_ptr as isize);
            }
        let _ = win_clone.with_webview(|wv| unsafe {
            let _ = wv.controller().SetIsVisible(true);
        });
        println!("[FullscreenOverlay] Opened overlay chat={:?} over fg={:?} taskbar_mark={}", hwnd_ptr, fg_ptr, marked_ok);
    });
    true
}

/// Hides chat window on main UI thread and hands focus back cleanly before hiding
pub fn hide_window_with_fullscreen_cleanup(window: &tauri::WebviewWindow) {
    let Ok(raw_hwnd) = window.hwnd() else {
        let _ = window.hide();
        return;
    };
    let hwnd_raw = raw_hwnd.0 as isize;
    let saved_fg_opt = overlay_sessions().lock().ok().and_then(|mut map| map.remove(&hwnd_raw));
    if let Some(saved_fg_raw) = saved_fg_opt {
        let _ = window.run_on_main_thread(move || unsafe {
            let hwnd = hwnd_raw as *mut std::ffi::c_void;
            let saved_fg = saved_fg_raw as *mut std::ffi::c_void;
            let current_fg = GetForegroundWindow();
            let saved_alive = !saved_fg.is_null() && IsWindow(saved_fg) != 0 && IsWindowVisible(saved_fg) != 0 && IsIconic(saved_fg) == 0;
            let saved_still_fs = saved_alive && is_window_fullscreen(saved_fg);
            if saved_alive && current_fg == saved_fg {
                SetWindowPos(hwnd, std::ptr::null_mut(), 0, 0, 0, 0, SWP_HIDEWINDOW | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
                if !saved_still_fs {
                    clear_hwnd_fullscreen_mark(hwnd);
                }
                println!("[FullscreenOverlay] Silent hide: saved_fg={:?} already active fs={}", saved_fg, saved_still_fs);
            } else if saved_alive && (saved_still_fs || current_fg == hwnd) {
                if !saved_still_fs {
                    println!("[FullscreenOverlay] Handoff with inactive-fs drift: saved_fg={:?} {}", saved_fg, describe_window_fullscreen_state(saved_fg));
                    clear_hwnd_fullscreen_mark(hwnd);
                }
                handoff_foreground_then_hide(hwnd, saved_fg);
            } else {
                println!("[FullscreenOverlay] Hide fallback (no handoff): chat={:?} saved_fg={:?} cur_fg={:?} alive={} fs={} {}", hwnd, saved_fg, current_fg, saved_alive, saved_still_fs, describe_window_fullscreen_state(saved_fg));
                ShowWindow(hwnd, SW_HIDE);
                clear_hwnd_fullscreen_mark(hwnd);
            }
        });
        return;
    }
    println!("[FullscreenOverlay] Hide without saved session: chat={:?} cur_fg={:?}", raw_hwnd.0, unsafe { GetForegroundWindow() });
    let _ = window.hide();
    if !unsafe { is_window_fullscreen(GetForegroundWindow()) } {
        clear_hwnd_fullscreen_mark(raw_hwnd.0);
    }
    if unsafe { IsWindowVisible(raw_hwnd.0) != 0 } {
        let win_clone = window.clone();
        let _ = window.run_on_main_thread(move || {
            if let Ok(h) = win_clone.hwnd()
                && unsafe { IsWindowVisible(h.0) != 0 } {
                    unsafe { ShowWindow(h.0, SW_HIDE); }
                }
        });
    }
}

/// Clears tracked fullscreen overlay state before destroying a window
pub fn clear_window_fullscreen_overlay(window: &tauri::WebviewWindow) {
    if let Ok(raw_hwnd) = window.hwnd() {
        if let Ok(mut map) = overlay_sessions().lock() {
            map.remove(&(raw_hwnd.0 as isize));
        }
        clear_hwnd_fullscreen_mark(raw_hwnd.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const E_NOTIMPL: i32 = 0x80004001_u32 as i32;

    #[test]
    fn test_win32_input_and_guithreadinfo_sizes() {
        assert_eq!(std::mem::size_of::<GUITHREADINFO>(), if cfg!(target_pointer_width = "64") { 72 } else { 48 });
        assert_eq!(std::mem::size_of::<INPUT>(), if cfg!(target_pointer_width = "64") { 40 } else { 28 });
    }

    #[test]
    fn test_overlay_session_hwnd_tracking() {
        let fake_chat_hwnd = 0x1234_isize;
        let fake_video_hwnd = 0x5678_isize;
        assert!(overlay_sessions().lock().unwrap().insert(fake_chat_hwnd, fake_video_hwnd).is_none());
        assert_eq!(overlay_sessions().lock().unwrap().remove(&fake_chat_hwnd), Some(fake_video_hwnd));
        assert!(overlay_sessions().lock().unwrap().remove(&fake_chat_hwnd).is_none());
    }

    #[test]
    fn test_taskbar_list2_com_initialization() {
        unsafe {
            let init_hr = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED);
            let should_uninit = init_hr == S_OK || init_hr == S_FALSE;
            assert!(should_uninit || init_hr == RPC_E_CHANGED_MODE);
            let mut raw_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            let create_hr = CoCreateInstance(
                &CLSID_TASKBAR_LIST,
                std::ptr::null_mut(),
                CLSCTX_ALL,
                &IID_ITASKBAR_LIST2,
                &mut raw_ptr,
            );
            assert!(create_hr >= 0 && !raw_ptr.is_null());
            let taskbar = raw_ptr as *mut ITaskbarList2;
            let vtbl = (*taskbar).vtbl;
            let hr_init = ((*vtbl).hr_init)(taskbar);
            ((*vtbl).release)(taskbar);
            if should_uninit {
                CoUninitialize();
            }
            assert!(hr_init >= 0 || hr_init == E_NOTIMPL, "Unexpected HrInit HRESULT 0x{:08X}", hr_init as u32);
        }
    }
}
