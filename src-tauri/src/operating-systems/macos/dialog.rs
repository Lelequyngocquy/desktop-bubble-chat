pub struct MacOSDialogManager;

impl MacOSDialogManager {
    /// Resolves localized Exit and Cancel labels based on dialog title or message text
    pub fn resolve_confirm_button_labels(title: &str, text: &str) -> (&'static str, &'static str) {
        let combined = format!("{} {}", title, text).to_lowercase();
        if combined.contains("thoát") || combined.contains("xác nhận") || combined.contains("ứng dụng") {
            ("Thoát", "Hủy")
        } else {
            ("Exit", "Cancel")
        }
    }

    /// Shows native macOS modal alert dialog safely on AppKit main thread
    pub fn show_confirm_exit(title: &str, text: &str) -> bool {
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::{id, nil, YES};
            use cocoa::foundation::NSString;
            use objc::{class, msg_send, sel, sel_impl};
            let title_owned = title.to_string();
            let text_owned = text.to_string();
            let (yes_label, no_label) = Self::resolve_confirm_button_labels(title, text);
            super::bubble::window::ffi::run_on_macos_main_thread_sync(move || unsafe {
                let app: id = msg_send![class!(NSApplication), sharedApplication];
                if app != nil {
                    let _: () = msg_send![app, activateIgnoringOtherApps: YES];
                }
                let alert_class = class!(NSAlert);
                let alloc_alert: id = msg_send![alert_class, alloc];
                let alert: id = msg_send![alloc_alert, init];
                if alert == nil {
                    return false;
                }
                let title_str = NSString::alloc(nil).init_str(&title_owned);
                let text_str = NSString::alloc(nil).init_str(&text_owned);
                let _: () = msg_send![alert, setMessageText: title_str];
                let _: () = msg_send![alert, setInformativeText: text_str];
                let yes_str = NSString::alloc(nil).init_str(yes_label);
                let no_str = NSString::alloc(nil).init_str(no_label);
                let _: id = msg_send![alert, addButtonWithTitle: yes_str];
                let _: id = msg_send![alert, addButtonWithTitle: no_str];
                let response: isize = msg_send![alert, runModal];
                let _: () = msg_send![no_str, release];
                let _: () = msg_send![yes_str, release];
                let _: () = msg_send![text_str, release];
                let _: () = msg_send![title_str, release];
                let _: () = msg_send![alert, release];
                response == 1000
            })
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (title, text);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_button_labels_localization() {
        assert_eq!(MacOSDialogManager::resolve_confirm_button_labels("Xác nhận thoát", "Bạn có chắc?"), ("Thoát", "Hủy"));
        assert_eq!(MacOSDialogManager::resolve_confirm_button_labels("Confirm Exit", "Are you sure?"), ("Exit", "Cancel"));
    }

    #[test]
    fn test_confirm_exit_fallback() {
        #[cfg(not(target_os = "macos"))]
        assert!(MacOSDialogManager::show_confirm_exit("Exit", "Are you sure?"));
    }
}
