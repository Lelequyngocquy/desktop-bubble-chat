use std::ffi::c_void;
#[allow(unused_imports)]
use super::ffi::*;
#[allow(unused_imports)]
use super::types::*;

pub struct MacOSPanelController;

impl MacOSPanelController {
    /// Creates a transparent, borderless, non-activating floating NSPanel
    #[cfg(target_os = "macos")]
    pub fn create_floating_bubble_panel(x: f64, y: f64, width: f64, height: f64) -> *mut c_void {
        use cocoa::base::{id, nil};
        use objc::{class, msg_send, sel, sel_impl};
        unsafe {
            let style_mask = NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL | NS_WINDOW_STYLE_MASK_BORDERLESS;
            let rect = NSRect::new(x, y, width, height);
            let panel_class = class!(NSPanel);
            let alloc_panel: id = msg_send![panel_class, alloc];
            let panel: id = msg_send![
                alloc_panel,
                initWithContentRect: rect
                styleMask: style_mask
                backing: 2usize
                defer: false
            ];
            if panel == nil {
                return std::ptr::null_mut();
            }
            let _: () = msg_send![panel, setLevel: NS_FLOATING_WINDOW_LEVEL];
            let collection_behavior = NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY;
            let _: () = msg_send![panel, setCollectionBehavior: collection_behavior];
            let _: () = msg_send![panel, setOpaque: false];
            let clear_color: id = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![panel, setBackgroundColor: clear_color];
            let _: () = msg_send![panel, setHasShadow: false];
            let content_view: id = msg_send![panel, contentView];
            let _: () = msg_send![content_view, setWantsLayer: true];
            panel as *mut c_void
        }
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn create_floating_bubble_panel(_x: f64, _y: f64, _width: f64, _height: f64) -> *mut c_void {
        std::ptr::null_mut()
    }

    /// Updates panel CALayer.contents with tiny-skia rendered BGRA buffer
    #[cfg(target_os = "macos")]
    pub fn update_panel_layer_contents(panel: *mut c_void, bgra_data: &[u8], width: usize, height: usize) {
        use cocoa::base::{id, nil};
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() || bgra_data.is_empty() || width == 0 || height == 0 {
            return;
        }
        unsafe {
            let provider = CGDataProviderCreateWithData(
                std::ptr::null_mut(),
                bgra_data.as_ptr() as *const c_void,
                bgra_data.len(),
                None,
            );
            let color_space = CGColorSpaceCreateDeviceRGB();
            let bitmap_info = K_CG_BITMAP_BYTE_ORDER_32_LITTLE | K_CG_IMAGE_ALPHA_PREMULTIPLIED_FIRST;
            let cg_image = CGImageCreate(
                width,
                height,
                8,
                32,
                width * 4,
                color_space,
                bitmap_info,
                provider,
                std::ptr::null(),
                false,
                K_CG_RENDERING_INTENT_DEFAULT,
            );
            let panel_id = panel as id;
            let content_view: id = msg_send![panel_id, contentView];
            let layer: id = msg_send![content_view, layer];
            if layer != nil && !cg_image.is_null() {
                let _: () = msg_send![layer, setContents: cg_image as id];
            }
            if !cg_image.is_null() {
                CGImageRelease(cg_image);
            }
            if !color_space.is_null() {
                CGColorSpaceRelease(color_space);
            }
            if !provider.is_null() {
                CGDataProviderRelease(provider);
            }
        }
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn update_panel_layer_contents(_panel: *mut c_void, _bgra_data: &[u8], _width: usize, _height: usize) {}

    /// Sets panel origin position
    #[cfg(target_os = "macos")]
    pub fn set_panel_position(panel: *mut c_void, x: f64, y: f64) {
        use cocoa::base::id;
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        unsafe {
            let point = NSPoint { x, y };
            let _: () = msg_send![panel as id, setFrameOrigin: point];
        }
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn set_panel_position(_panel: *mut c_void, _x: f64, _y: f64) {}

    /// Shows or hides NSPanel
    #[cfg(target_os = "macos")]
    pub fn set_panel_visible(panel: *mut c_void, visible: bool) {
        use cocoa::base::{id, nil};
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        unsafe {
            let panel_id = panel as id;
            if visible {
                let _: () = msg_send![panel_id, orderFront: nil];
            } else {
                let _: () = msg_send![panel_id, orderOut: nil];
            }
        }
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn set_panel_visible(_panel: *mut c_void, _visible: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_controller_fallback_safety() {
        let ptr = MacOSPanelController::create_floating_bubble_panel(100.0, 100.0, 56.0, 56.0);
        #[cfg(not(target_os = "macos"))]
        {
            assert!(ptr.is_null());
            MacOSPanelController::update_panel_layer_contents(ptr, &[], 0, 0);
            MacOSPanelController::set_panel_position(ptr, 0.0, 0.0);
            MacOSPanelController::set_panel_visible(ptr, true);
        }
    }
}
