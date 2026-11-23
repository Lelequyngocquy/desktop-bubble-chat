pub struct MacOSMenuManager;

#[cfg(target_os = "macos")]
static MENU_APP_HANDLE: std::sync::Mutex<Option<(tauri::AppHandle, u32)>> = std::sync::Mutex::new(None);

#[cfg(target_os = "macos")]
fn register_bubble_menu_target_class() -> *const objc::runtime::Class {
    use cocoa::base::id;
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{class, sel, sel_impl};
    static REGISTER_MENU_TARGET: std::sync::Once = std::sync::Once::new();
    REGISTER_MENU_TARGET.call_once(|| unsafe {
        if let Some(mut decl) = ClassDecl::new("DesktopBubbleMenuTarget", class!(NSObject)) {
            extern "C" fn open_dashboard(_this: &Object, _cmd: Sel, _sender: id) {
                if let Ok(guard) = MENU_APP_HANDLE.lock()
                    && let Some((ref app, _)) = *guard {
                        let _ = crate::presentation::main_window::get_or_create_main_window(app);
                    }
            }
            extern "C" fn hide_bubble(_this: &Object, _cmd: Sel, _sender: id) {
                if let Ok(guard) = MENU_APP_HANDLE.lock()
                    && let Some((ref app, idx)) = *guard {
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            crate::commands::bubble::dismiss_bubble_from_drag(handle, idx).await;
                        });
                    }
            }
            extern "C" fn quit_app(_this: &Object, _cmd: Sel, _sender: id) {
                if let Ok(guard) = MENU_APP_HANDLE.lock()
                    && let Some((ref app, _)) = *guard {
                        crate::commands::system::app::exit_app(app.clone());
                    }
            }
            decl.add_method(sel!(openDashboard:), open_dashboard as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(hideBubble:), hide_bubble as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(quitApp:), quit_app as extern "C" fn(&Object, Sel, id));
            decl.register();
        }
    });
    Class::get("DesktopBubbleMenuTarget").map(|c| c as *const Class).unwrap_or(std::ptr::null())
}

impl MacOSMenuManager {
    /// Checks if native status bar menu host is available on macOS
    pub fn is_status_notifier_host_available() -> bool {
        true
    }

    /// Shows contextual popup menu for native bubble on right click wired to Tauri actions
    pub fn show_bubble_context_menu(app: &tauri::AppHandle, account_index: u32, x: f64, y: f64) {
        #[cfg(target_os = "macos")]
        {
            use cocoa::base::{id, nil, BOOL, NO};
            use cocoa::foundation::{NSPoint as CocoaNSPoint, NSString};
            use objc::{class, msg_send, sel, sel_impl};
            if let Ok(mut guard) = MENU_APP_HANDLE.lock() {
                *guard = Some((app.clone(), account_index));
            }
            super::bubble::window::ffi::run_on_macos_main_thread(move || unsafe {
                let target_cls = register_bubble_menu_target_class();
                if target_cls.is_null() {
                    return;
                }
                let target_alloc: id = msg_send![target_cls, alloc];
                let target: id = msg_send![target_alloc, init];
                if target == nil {
                    return;
                }
                let alloc_menu: id = msg_send![class!(NSMenu), alloc];
                let menu_title = NSString::alloc(nil).init_str("BubbleMenu");
                let menu: id = msg_send![alloc_menu, initWithTitle: menu_title];
                let _: () = msg_send![menu_title, release];
                if menu == nil {
                    let _: () = msg_send![target, release];
                    return;
                }
                let _: () = msg_send![menu, setAutoenablesItems: NO];
                let empty_key = NSString::alloc(nil).init_str("");
                let item1_title = NSString::alloc(nil).init_str("Open Dashboard");
                let item1: id = msg_send![menu, addItemWithTitle: item1_title action: sel!(openDashboard:) keyEquivalent: empty_key];
                let _: () = msg_send![item1, setTarget: target];
                let _: () = msg_send![item1_title, release];
                let item2_title = NSString::alloc(nil).init_str("Hide Bubble");
                let item2: id = msg_send![menu, addItemWithTitle: item2_title action: sel!(hideBubble:) keyEquivalent: empty_key];
                let _: () = msg_send![item2, setTarget: target];
                let _: () = msg_send![item2_title, release];
                let item3_title = NSString::alloc(nil).init_str("Quit DesktopBubbleChat");
                let item3: id = msg_send![menu, addItemWithTitle: item3_title action: sel!(quitApp:) keyEquivalent: empty_key];
                let _: () = msg_send![item3, setTarget: target];
                let _: () = msg_send![item3_title, release];
                let _: () = msg_send![empty_key, release];
                let point = CocoaNSPoint::new(x, y);
                let _: BOOL = msg_send![menu, popUpMenuPositioningItem: nil atLocation: point inView: nil];
                let _: () = msg_send![menu, release];
                let _: () = msg_send![target, release];
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
        assert!(MacOSMenuManager::is_status_notifier_host_available());
    }
}
