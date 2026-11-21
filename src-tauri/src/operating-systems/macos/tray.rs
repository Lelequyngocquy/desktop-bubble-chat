pub struct MacOSMenuManager;

impl MacOSMenuManager {
    /// Checks if native status bar menu host is available on macOS
    pub fn is_status_notifier_host_available() -> bool {
        true
    }

    /// Shows contextual popup menu for native bubble on right click
    pub fn show_bubble_context_menu(app: &tauri::AppHandle, account_index: u32, x: f64, y: f64) {
        #[cfg(target_os = "macos")]
        {
            let _ = (app, account_index);
            use cocoa::base::{id, nil, BOOL, NO};
            use cocoa::foundation::NSString;
            use objc::{class, msg_send, sel, sel_impl};
            super::bubble::window::ffi::run_on_macos_main_thread(move || unsafe {
                let menu_class = class!(NSMenu);
                let alloc_menu: id = msg_send![menu_class, alloc];
                let menu_title = NSString::alloc(nil).init_str("BubbleMenu");
                let menu: id = msg_send![alloc_menu, initWithTitle: menu_title];
                let _: () = msg_send![menu_title, release];
                if menu == nil {
                    return;
                }
                let _: () = msg_send![menu, setAutoenablesItems: NO];
                let item1_title = NSString::alloc(nil).init_str("Open Dashboard");
                let empty_key = NSString::alloc(nil).init_str("");
                let item1: id = msg_send![menu, addItemWithTitle: item1_title action: sel!(openDashboard:) keyEquivalent: empty_key];
                let _: () = msg_send![item1, setTarget: nil];
                let _: () = msg_send![empty_key, release];
                let _: () = msg_send![item1_title, release];
                let item2_title = NSString::alloc(nil).init_str("Quit");
                let quit_key = NSString::alloc(nil).init_str("q");
                let item2: id = msg_send![menu, addItemWithTitle: item2_title action: sel!(terminate:) keyEquivalent: quit_key];
                let _: () = msg_send![item2, setTarget: nil];
                let _: () = msg_send![quit_key, release];
                let _: () = msg_send![item2_title, release];
                let point = super::bubble::window::ffi::NSPoint { x, y };
                let _: BOOL = msg_send![menu, popUpMenuPositioningItem: nil atLocation: point inView: nil];
                let _: () = msg_send![menu, release];
            });
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (app, account_index, x, y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_manager_safety() {
        let _ = MacOSMenuManager;
    }
}
