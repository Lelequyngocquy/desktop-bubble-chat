/// Displays Win32 confirmation dialog before quitting to prevent accidental exits.
#[cfg(target_os = "windows")]
pub fn confirm_exit_dialog(locale: &str) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    let tray_i18n = crate::utils::i18n::get_tray_locale(locale);
    let title = &tray_i18n.confirm_exit_title;
    let text = &tray_i18n.confirm_exit_text;
    let text_wide: Vec<u16> = OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect();
    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(std::iter::once(0)).collect();
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
    const MB_DEFBUTTON2: u32 = 0x00000100;
    const MB_SYSTEMMODAL: u32 = 0x00001000;
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
pub fn confirm_exit_dialog(_locale: &str) -> bool {
    true
}
