use std::ffi::c_void;
#[allow(unused_imports)]
use super::ffi::*;
#[allow(unused_imports)]
use super::types::*;

pub struct MacOSPanelController;

/// Evaluates circular bubble and unread badge hit-test inside logical view bounds
pub fn is_point_in_bubble_hit_zone(
    local_x_pt: f64,
    local_y_pt: f64,
    canvas_w_pt: f64,
    canvas_h_pt: f64,
    bubble_size_pt: f64,
    unread_count: u32,
    is_left_edge: bool,
) -> bool {
    let cx = canvas_w_pt * 0.5;
    let cy = canvas_h_pt * 0.5;
    let radius = (bubble_size_pt * 0.5).max(12.0) + 2.0;
    let dx = local_x_pt - cx;
    let dy = local_y_pt - cy;
    if dx * dx + dy * dy <= radius * radius {
        return true;
    }
    if unread_count > 0 {
        let badge_cx = if is_left_edge { cx + radius * 0.72 } else { cx - radius * 0.72 };
        let badge_cy = cy + radius * 0.65;
        let bdx = local_x_pt - badge_cx;
        let bdy = local_y_pt - badge_cy;
        if bdx * bdx + bdy * bdy <= 14.0 * 14.0 {
            return true;
        }
    }
    false
}

#[cfg(target_os = "macos")]
fn register_bubble_view_class() -> *const objc::runtime::Class {
    use cocoa::base::{id, nil, BOOL, YES};
    use cocoa::foundation::{NSPoint as CocoaNSPoint, NSRect as CocoaNSRect};
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{class, msg_send, sel, sel_impl};
    static REGISTER_VIEW_ONCE: std::sync::Once = std::sync::Once::new();
    REGISTER_VIEW_ONCE.call_once(|| unsafe {
        let superclass = class!(NSView);
        if let Some(mut decl) = ClassDecl::new("DesktopBubbleChatView", superclass) {
            decl.add_ivar::<u32>("accountIndex");
            decl.add_ivar::<f64>("bubbleSizePt");
            decl.add_ivar::<u32>("unreadCount");
            decl.add_ivar::<u8>("isLeftEdge");
            extern "C" fn accepts_first_mouse(_this: &Object, _cmd: Sel, _event: id) -> BOOL {
                YES
            }
            extern "C" fn hit_test(this: &Object, _cmd: Sel, point: CocoaNSPoint) -> id {
                unsafe {
                    let superview: id = msg_send![this, superview];
                    let local: CocoaNSPoint = msg_send![this, convertPoint: point fromView: superview];
                    let bounds: CocoaNSRect = msg_send![this, bounds];
                    let bubble_pt = *this.get_ivar::<f64>("bubbleSizePt");
                    let unread = *this.get_ivar::<u32>("unreadCount");
                    let is_left = *this.get_ivar::<u8>("isLeftEdge") != 0;
                    if is_point_in_bubble_hit_zone(local.x, local.y, bounds.size.width, bounds.size.height, bubble_pt, unread, is_left) {
                        this as *const Object as id
                    } else {
                        nil
                    }
                }
            }
            extern "C" fn reset_cursor_rects(this: &Object, _cmd: Sel) {
                unsafe {
                    let bounds: CocoaNSRect = msg_send![this, bounds];
                    let cursor: id = msg_send![class!(NSCursor), pointingHandCursor];
                    let _: () = msg_send![this, addCursorRect: bounds cursor: cursor];
                }
            }
            extern "C" fn mouse_down(this: &Object, _cmd: Sel, _event: id) {
                unsafe {
                    let idx = *this.get_ivar::<u32>("accountIndex");
                    if let Some((cx, cy)) = crate::operating_systems::macos::cursor::MacOSCursorTracker::get_cursor_position() {
                        crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global().handle_mouse_down(idx, cx, cy);
                    }
                }
            }
            extern "C" fn right_mouse_down(this: &Object, _cmd: Sel, _event: id) {
                unsafe {
                    let idx = *this.get_ivar::<u32>("accountIndex");
                    crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global().handle_right_click(idx);
                }
            }
            extern "C" fn mouse_dragged(this: &Object, _cmd: Sel, _event: id) {
                unsafe {
                    let idx = *this.get_ivar::<u32>("accountIndex");
                    if let Some((cx, cy)) = crate::operating_systems::macos::cursor::MacOSCursorTracker::get_cursor_position() {
                        crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global().handle_mouse_move(idx, cx, cy);
                    }
                }
            }
            extern "C" fn mouse_up(this: &Object, _cmd: Sel, _event: id) {
                unsafe {
                    let idx = *this.get_ivar::<u32>("accountIndex");
                    let (cx, cy) = crate::operating_systems::macos::cursor::MacOSCursorTracker::get_cursor_position().unwrap_or((0, 0));
                    crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global().handle_mouse_up(idx, cx, cy);
                }
            }
            extern "C" fn mouse_entered(this: &Object, _cmd: Sel, _event: id) {
                unsafe {
                    let idx = *this.get_ivar::<u32>("accountIndex");
                    crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global().handle_mouse_enter(idx);
                }
            }
            extern "C" fn mouse_exited(this: &Object, _cmd: Sel, _event: id) {
                unsafe {
                    let idx = *this.get_ivar::<u32>("accountIndex");
                    crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global().handle_mouse_leave(idx);
                }
            }
            decl.add_method(sel!(acceptsFirstMouse:), accepts_first_mouse as extern "C" fn(&Object, Sel, id) -> BOOL);
            decl.add_method(sel!(hitTest:), hit_test as extern "C" fn(&Object, Sel, CocoaNSPoint) -> id);
            decl.add_method(sel!(resetCursorRects), reset_cursor_rects as extern "C" fn(&Object, Sel));
            decl.add_method(sel!(mouseDown:), mouse_down as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(rightMouseDown:), right_mouse_down as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(mouseDragged:), mouse_dragged as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(mouseUp:), mouse_up as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(mouseEntered:), mouse_entered as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(mouseExited:), mouse_exited as extern "C" fn(&Object, Sel, id));
            decl.register();
        }
    });
    Class::get("DesktopBubbleChatView").map(|c| c as *const Class).unwrap_or(std::ptr::null())
}

impl MacOSPanelController {
    /// Converts top-left physical bubble coordinates into Cocoa bottom-left logical NSRect using primary display height
    pub fn top_left_physical_to_cocoa_rect_with_primary_height(
        x_px: i32,
        y_px: i32,
        canvas_w_px: u32,
        canvas_h_px: u32,
        primary_display_height_pt: f64,
        scale_factor: f64,
    ) -> NSRect {
        let scale = if scale_factor.is_finite() && scale_factor > 0.0 { scale_factor } else { 1.0 };
        let win_x_px = (x_px - BUBBLE_CANVAS_PAD_X) as f64;
        let win_y_px = (y_px - BUBBLE_CANVAS_PAD_Y) as f64;
        let logical_w = (canvas_w_px as f64) / scale;
        let logical_h = (canvas_h_px as f64) / scale;
        let logical_x = win_x_px / scale;
        let logical_top_y = win_y_px / scale;
        let logical_y = primary_display_height_pt - logical_top_y - logical_h;
        NSRect::new(logical_x, logical_y, logical_w, logical_h)
    }

    /// Converts top-left physical bubble coordinates into Cocoa bottom-left logical NSRect
    pub fn top_left_physical_to_cocoa_rect(
        x_px: i32,
        y_px: i32,
        canvas_w_px: u32,
        canvas_h_px: u32,
        screen_y_px: i32,
        screen_h_px: i32,
        scale_factor: f64,
    ) -> NSRect {
        let scale = if scale_factor.is_finite() && scale_factor > 0.0 { scale_factor } else { 1.0 };
        let primary_height_pt = get_primary_display_height_points()
            .unwrap_or_else(|| (screen_y_px + screen_h_px) as f64 / scale);
        Self::top_left_physical_to_cocoa_rect_with_primary_height(
            x_px,
            y_px,
            canvas_w_px,
            canvas_h_px,
            primary_height_pt,
            scale,
        )
    }

    /// Creates a transparent, borderless, non-activating floating NSPanel with custom interactive NSView
    #[cfg(target_os = "macos")]
    pub fn create_floating_bubble_panel(account_index: u32, x: f64, y: f64, width: f64, height: f64) -> *mut c_void {
        use cocoa::base::{id, nil, NO, YES};
        use objc::runtime::Object;
        use objc::{class, msg_send, sel, sel_impl};
        with_autorelease_pool(|| unsafe {
            let style_mask = NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL | NS_WINDOW_STYLE_MASK_BORDERLESS;
            let rect = NSRect::new(x, y, width, height);
            let panel_class = class!(NSPanel);
            let alloc_panel: id = msg_send![panel_class, alloc];
            let panel: id = msg_send![alloc_panel, initWithContentRect: rect styleMask: style_mask backing: 2usize defer: NO];
            if panel == nil {
                return std::ptr::null_mut();
            }
            let _: () = msg_send![panel, setReleasedWhenClosed: NO];
            let _: () = msg_send![panel, setHidesOnDeactivate: NO];
            let _: () = msg_send![panel, setLevel: NS_FLOATING_WINDOW_LEVEL];
            let collection_behavior = NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY;
            let _: () = msg_send![panel, setCollectionBehavior: collection_behavior];
            let _: () = msg_send![panel, setOpaque: NO];
            let clear_color: id = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![panel, setBackgroundColor: clear_color];
            let _: () = msg_send![panel, setHasShadow: NO];
            let _: () = msg_send![panel, setAcceptsMouseMovedEvents: YES];
            let view_cls = register_bubble_view_class();
            if !view_cls.is_null() {
                let view_alloc: id = msg_send![view_cls, alloc];
                let view_frame = NSRect::new(0.0, 0.0, width, height);
                let custom_view: id = msg_send![view_alloc, initWithFrame: view_frame];
                if custom_view != nil {
                    let obj = &mut *(custom_view as *mut Object);
                    obj.set_ivar::<u32>("accountIndex", account_index);
                    obj.set_ivar::<f64>("bubbleSizePt", (height - (2 * BUBBLE_CANVAS_PAD_Y) as f64).max(24.0));
                    obj.set_ivar::<u32>("unreadCount", 0);
                    obj.set_ivar::<u8>("isLeftEdge", 1);
                    let _: () = msg_send![custom_view, setWantsLayer: YES];
                    let tracking_opts: usize = 0x01 | 0x80 | 0x200;
                    let ta_alloc: id = msg_send![class!(NSTrackingArea), alloc];
                    let ta: id = msg_send![ta_alloc, initWithRect: view_frame options: tracking_opts owner: custom_view userInfo: nil];
                    if ta != nil {
                        let _: () = msg_send![custom_view, addTrackingArea: ta];
                        let _: () = msg_send![ta, release];
                    }
                    let _: () = msg_send![panel, setContentView: custom_view];
                    let _: () = msg_send![custom_view, release];
                }
            } else {
                let content_view: id = msg_send![panel, contentView];
                let _: () = msg_send![content_view, setWantsLayer: YES];
            }
            panel as *mut c_void
        })
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn create_floating_bubble_panel(_account_index: u32, _x: f64, _y: f64, _width: f64, _height: f64) -> *mut c_void {
        std::ptr::null_mut()
    }

    /// Updates custom NSView hit-test metadata ivars for circular click-through
    #[cfg(target_os = "macos")]
    pub fn update_panel_hit_metadata(panel: *mut c_void, bubble_size_pt: f64, unread_count: u32, is_left_edge: bool) {
        use cocoa::base::{id, nil};
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        with_autorelease_pool(|| unsafe {
            let content_view: id = msg_send![panel as id, contentView];
            if content_view != nil {
                let obj = &mut *(content_view as *mut Object);
                obj.set_ivar::<f64>("bubbleSizePt", bubble_size_pt);
                obj.set_ivar::<u32>("unreadCount", unread_count);
                obj.set_ivar::<u8>("isLeftEdge", if is_left_edge { 1 } else { 0 });
            }
        });
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn update_panel_hit_metadata(_panel: *mut c_void, _bubble_size_pt: f64, _unread_count: u32, _is_left_edge: bool) {}

    /// Updates panel CALayer.contents with owned BGRA buffer and Retina contentsScale
    #[cfg(target_os = "macos")]
    pub fn update_panel_layer_contents(panel: *mut c_void, bgra_data: &[u8], width: usize, height: usize, scale_factor: f64) {
        use cocoa::base::{id, nil};
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() || bgra_data.is_empty() || width == 0 || height == 0 {
            return;
        }
        with_autorelease_pool(|| unsafe {
            let boxed = Box::new(bgra_data.to_vec());
            let data_ptr = boxed.as_ptr() as *const c_void;
            let data_len = boxed.len();
            let info_ptr = Box::into_raw(boxed) as *mut c_void;
            let provider = CGDataProviderCreateWithData(info_ptr, data_ptr, data_len, Some(release_boxed_bgra_data));
            if provider.is_null() {
                let _ = Box::from_raw(info_ptr as *mut Vec<u8>);
                return;
            }
            let color_space = CGColorSpaceCreateDeviceRGB();
            let bitmap_info = K_CG_BITMAP_BYTE_ORDER_32_LITTLE | K_CG_IMAGE_ALPHA_PREMULTIPLIED_FIRST;
            let cg_image = CGImageCreate(width, height, 8, 32, width * 4, color_space, bitmap_info, provider, std::ptr::null(), false, K_CG_RENDERING_INTENT_DEFAULT);
            let panel_id = panel as id;
            let content_view: id = msg_send![panel_id, contentView];
            let layer: id = msg_send![content_view, layer];
            if layer != nil && !cg_image.is_null() {
                let scale = if scale_factor.is_finite() && scale_factor > 0.0 { scale_factor } else { 1.0 };
                let _: () = msg_send![layer, setContentsScale: scale];
                let _: () = msg_send![layer, setContents: cg_image as id];
            }
            if !cg_image.is_null() {
                CGImageRelease(cg_image);
            }
            if !color_space.is_null() {
                CGColorSpaceRelease(color_space);
            }
            CGDataProviderRelease(provider);
        });
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn update_panel_layer_contents(_panel: *mut c_void, _bgra_data: &[u8], _width: usize, _height: usize, _scale_factor: f64) {}

    /// Sets panel frame origin in Cocoa logical coordinates
    #[cfg(target_os = "macos")]
    pub fn set_panel_position(panel: *mut c_void, x: f64, y: f64) {
        use cocoa::base::id;
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        with_autorelease_pool(|| unsafe {
            let point = NSPoint { x, y };
            let _: () = msg_send![panel as id, setFrameOrigin: point];
        });
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn set_panel_position(_panel: *mut c_void, _x: f64, _y: f64) {}

    /// Resizes and repositions NSPanel frame in Cocoa logical coordinates
    #[cfg(target_os = "macos")]
    pub fn set_panel_frame(panel: *mut c_void, rect: NSRect) {
        use cocoa::base::{id, YES};
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        with_autorelease_pool(|| unsafe {
            let _: () = msg_send![panel as id, setFrame: rect display: YES];
        });
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn set_panel_frame(_panel: *mut c_void, _rect: NSRect) {}

    /// Sets NSPanel alpha opacity value (0.0 to 1.0)
    #[cfg(target_os = "macos")]
    pub fn set_panel_alpha(panel: *mut c_void, alpha: f64) {
        use cocoa::base::id;
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        with_autorelease_pool(|| unsafe {
            let _: () = msg_send![panel as id, setAlphaValue: alpha];
        });
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn set_panel_alpha(_panel: *mut c_void, _alpha: f64) {}

    /// Updates NSPanel window level for always-on-top mode
    #[cfg(target_os = "macos")]
    pub fn set_panel_always_on_top(panel: *mut c_void, enabled: bool) {
        use cocoa::base::id;
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        with_autorelease_pool(|| unsafe {
            let level = if enabled { NS_FLOATING_WINDOW_LEVEL } else { NS_NORMAL_WINDOW_LEVEL };
            let _: () = msg_send![panel as id, setLevel: level];
        });
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn set_panel_always_on_top(_panel: *mut c_void, _enabled: bool) {}

    /// Re-asserts floating panel in front without activating the app
    #[cfg(target_os = "macos")]
    pub fn order_panel_front_regardless(panel: *mut c_void) {
        use cocoa::base::id;
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        with_autorelease_pool(|| unsafe {
            let _: () = msg_send![panel as id, orderFrontRegardless];
        });
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn order_panel_front_regardless(_panel: *mut c_void) {}

    /// Shows or hides NSPanel
    #[cfg(target_os = "macos")]
    pub fn set_panel_visible(panel: *mut c_void, visible: bool) {
        use cocoa::base::{id, nil};
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        with_autorelease_pool(|| unsafe {
            let panel_id = panel as id;
            if visible {
                let _: () = msg_send![panel_id, orderFrontRegardless];
            } else {
                let _: () = msg_send![panel_id, orderOut: nil];
            }
        });
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn set_panel_visible(_panel: *mut c_void, _visible: bool) {}

    /// Closes and releases NSPanel
    #[cfg(target_os = "macos")]
    pub fn destroy_panel(panel: *mut c_void) {
        use cocoa::base::{id, nil};
        use objc::{msg_send, sel, sel_impl};
        if panel.is_null() {
            return;
        }
        with_autorelease_pool(|| unsafe {
            let panel_id = panel as id;
            let _: () = msg_send![panel_id, orderOut: nil];
            let _: () = msg_send![panel_id, close];
            let _: () = msg_send![panel_id, release];
        });
    }

    /// Fallback for non-macOS target
    #[cfg(not(target_os = "macos"))]
    pub fn destroy_panel(_panel: *mut c_void) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_controller_fallback_safety() {
        let ptr = MacOSPanelController::create_floating_bubble_panel(1, 100.0, 100.0, 56.0, 56.0);
        #[cfg(not(target_os = "macos"))]
        {
            assert!(ptr.is_null());
            MacOSPanelController::update_panel_layer_contents(ptr, &[], 0, 0, 2.0);
            MacOSPanelController::set_panel_position(ptr, 0.0, 0.0);
            MacOSPanelController::set_panel_visible(ptr, true);
            MacOSPanelController::set_panel_always_on_top(ptr, true);
            MacOSPanelController::order_panel_front_regardless(ptr);
            MacOSPanelController::destroy_panel(ptr);
        }
    }

    #[test]
    fn test_top_left_physical_to_cocoa_rect_retina_with_dock_exclusion() {
        // 1440x900pt @2x Retina primary display with 74px menu bar and 140px Dock
        let rect = MacOSPanelController::top_left_physical_to_cocoa_rect_with_primary_height(60, 74, 172, 136, 900.0, 2.0);
        assert_eq!(rect.origin.x, 15.0);
        assert_eq!(rect.origin.y, 900.0 - 31.0 - 68.0);
        assert_eq!(rect.size.width, 86.0);
        assert_eq!(rect.size.height, 68.0);
    }

    #[test]
    fn test_circular_bubble_hit_zone() {
        assert!(is_point_in_bubble_hit_zone(58.0, 40.0, 116.0, 80.0, 56.0, 0, true));
        assert!(!is_point_in_bubble_hit_zone(5.0, 5.0, 116.0, 80.0, 56.0, 0, true));
        assert!(is_point_in_bubble_hit_zone(83.0, 61.0, 116.0, 80.0, 56.0, 3, true));
        assert!(!is_point_in_bubble_hit_zone(83.0, 61.0, 116.0, 80.0, 56.0, 0, true));
    }
}
