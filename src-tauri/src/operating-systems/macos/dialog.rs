pub struct MacOSDialogManager;

impl MacOSDialogManager {
    /// Shows native macOS modal alert dialog asking user to confirm exit
    pub fn show_confirm_exit(title: &str, text: &str) -> bool {
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::{id, nil};
            use cocoa::foundation::NSString;
            use objc::{class, msg_send, sel, sel_impl};
            super::bubble::window::ffi::with_autorelease_pool(|| unsafe {
                let alert_class = class!(NSAlert);
                let alloc_alert: id = msg_send![alert_class, alloc];
                let alert: id = msg_send![alloc_alert, init];
                if alert == nil {
                    return false;
                }
                let title_str = NSString::alloc(nil).init_str(title);
                let text_str = NSString::alloc(nil).init_str(text);
                let _: () = msg_send![alert, setMessageText: title_str];
                let _: () = msg_send![alert, setInformativeText: text_str];
                let yes_str = NSString::alloc(nil).init_str("Exit");
                let no_str = NSString::alloc(nil).init_str("Cancel");
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
    fn test_confirm_exit_fallback() {
        #[cfg(not(target_os = "macos"))]
        assert!(MacOSDialogManager::show_confirm_exit("Exit", "Are you sure?"));
    }
}
