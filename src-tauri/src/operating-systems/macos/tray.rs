pub struct MacOSMenuManager;

impl MacOSMenuManager {
    /// Shows contextual popup menu for native bubble on right click
    pub fn show_bubble_context_menu(app: &tauri::AppHandle, account_index: u32, x: f64, y: f64) {
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::{id, nil};
            use cocoa::foundation::NSString;
            use objc::{class, msg_send, sel, sel_impl};
            unsafe {
                let menu_class = class!(NSMenu);
                let menu: id = msg_send![menu_class, alloc];
                let menu: id = msg_send![menu, initWithTitle: NSString::alloc(nil).init_str("BubbleMenu")];
                let _: () = msg_send![menu, setAutoenablesItems: false];
                let item1_title = NSString::alloc(nil).init_str("Open Dashboard");
                let item1: id = msg_send![menu, addItemWithTitle:item1_title action:sel!(openDashboard:) keyEquivalent:NSString::alloc(nil).init_str("")];
                let _: () = msg_send![item1, setTarget: nil];
                let item2_title = NSString::alloc(nil).init_str("Quit");
                let item2: id = msg_send![menu, addItemWithTitle:item2_title action:sel!(terminate:) keyEquivalent:NSString::alloc(nil).init_str("q")];
                let _: () = msg_send![item2, setTarget: nil];
                let point = super::bubble::window::ffi::NSPoint { x, y };
                let _: bool = msg_send![menu, popUpMenuPositioningItem:nil atLocation:point inView:nil];
            }
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
