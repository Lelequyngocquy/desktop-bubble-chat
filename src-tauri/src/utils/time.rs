use std::time::{SystemTime, UNIX_EPOCH};

/// Current timestamp in milliseconds (UNIX epoch)
pub fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct Win32SystemTime {
    w_year: u16,
    w_month: u16,
    w_day_of_week: u16,
    w_day: u16,
    w_hour: u16,
    w_minute: u16,
    w_second: u16,
    w_milliseconds: u16,
}

#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLocalTime(lpSystemTime: *mut Win32SystemTime);
}

#[cfg(not(target_os = "windows"))]
#[repr(C)]
struct PosixTm {
    tm_sec: std::ffi::c_int,
    tm_min: std::ffi::c_int,
    tm_hour: std::ffi::c_int,
    tm_mday: std::ffi::c_int,
    tm_mon: std::ffi::c_int,
    tm_year: std::ffi::c_int,
    tm_wday: std::ffi::c_int,
    tm_yday: std::ffi::c_int,
    tm_isdst: std::ffi::c_int,
    tm_gmtoff: std::ffi::c_long,
    tm_zone: *const std::ffi::c_char,
}

#[cfg(not(target_os = "windows"))]
unsafe extern "C" {
    fn localtime_r(timep: *const std::ffi::c_long, result: *mut PosixTm) -> *mut PosixTm;
}

/// Formats current local time as HH:MM:SS
pub fn format_current_time() -> String {
    #[cfg(target_os = "windows")]
    unsafe {
        let mut st: Win32SystemTime = std::mem::zeroed();
        GetLocalTime(&mut st);
        format!("{:02}:{:02}:{:02}", st.w_hour, st.w_minute, st.w_second)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let secs = (now_millis() / 1000) as std::ffi::c_long;
        let mut tm: PosixTm = unsafe { std::mem::zeroed() };
        let res = unsafe { localtime_r(&secs, &mut tm) };
        if !res.is_null() {
            format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
        } else {
            let s = secs % 60;
            let m = (secs / 60) % 60;
            let h = (secs / 3600) % 24;
            format!("{:02}:{:02}:{:02}", h, m, s)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_current_time() {
        let t = format_current_time();
        assert_eq!(t.len(), 8);
        assert_eq!(&t[2..3], ":");
        assert_eq!(&t[5..6], ":");
    }
}
