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
