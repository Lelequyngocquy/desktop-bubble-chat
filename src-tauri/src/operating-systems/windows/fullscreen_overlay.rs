use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use super::bubble::window::{
    is_window_fullscreen, GetForegroundWindow, SetWindowPos, ShowWindow, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_SHOWNA,
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

fn marked_hwnds() -> &'static Mutex<HashSet<isize>> {
    MARKED_FULLSCREEN_HWNDS.get_or_init(|| Mutex::new(HashSet::new()))
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
    let is_fg_fullscreen = unsafe { is_window_fullscreen(GetForegroundWindow()) };
    if !is_fg_fullscreen {
        clear_hwnd_fullscreen_mark(hwnd);
        return false;
    }
    unsafe {
        ShowWindow(hwnd, SW_SHOWNA);
        SetWindowPos(
            hwnd,
            -1_isize as *mut std::ffi::c_void,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
    if mark_hwnd_fullscreen_for_taskbar(hwnd, true)
        && let Ok(mut set) = marked_hwnds().lock() {
            set.insert(hwnd as isize);
        }
    let _ = window.with_webview(|wv| unsafe {
        let _ = wv.controller().SetIsVisible(true);
    });
    true
}

/// Clears any temporary fullscreen taskbar marker after hiding the chat window
pub fn clear_window_fullscreen_overlay(window: &tauri::WebviewWindow) {
    if let Ok(raw_hwnd) = window.hwnd() {
        clear_hwnd_fullscreen_mark(raw_hwnd.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const E_NOTIMPL: i32 = 0x80004001_u32 as i32;

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
