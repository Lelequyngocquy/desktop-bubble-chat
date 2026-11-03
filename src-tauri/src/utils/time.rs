use std::time::{SystemTime, UNIX_EPOCH};

/// Lấy mốc thời gian hiện tại theo mili-giây (UNIX timestamp)
pub fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct SYSTEMTIME {
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
    fn GetLocalTime(lpSystemTime: *mut SYSTEMTIME);
}

/// Định dạng thời gian hiện tại thành chuỗi HH:MM:SS theo giờ địa phương
pub fn format_current_time() -> String {
    #[cfg(target_os = "windows")]
    unsafe {
        let mut st: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut st);
        format!("{:02}:{:02}:{:02}", st.w_hour, st.w_minute, st.w_second)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let secs = now_millis() / 1000;
        let s = secs % 60;
        let m = (secs / 60) % 60;
        let h = (secs / 3600) % 24;
        format!("{:02}:{:02}:{:02}", h, m, s)
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
