use std::ffi::c_void;

use crate::operating_systems::macos::bubble::window::ffi::{
    get_primary_display_height_points, NSRect,
};
#[cfg(target_os = "macos")]
use crate::operating_systems::macos::bubble::window::ffi::with_autorelease_pool;
#[cfg(target_os = "macos")]
use crate::operating_systems::macos::bubble::window::types::{
    NS_FLOATING_WINDOW_LEVEL, NS_NORMAL_WINDOW_LEVEL,
    NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES,
    NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY,
    NS_WINDOW_STYLE_MASK_BORDERLESS, NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL,
};
use crate::presentation::bubble::dock::{hit_test_dock_droplet, ActionDockLayout, DockActionId};

pub struct MacOSActionDockPanelController;

/// Converts top-left physical ActionDock canvas rect into Cocoa bottom-left logical NSRect
pub fn dock_physical_to_cocoa_rect_with_primary_height(
    layout: &ActionDockLayout,
    primary_display_height_pt: f64,
) -> NSRect {
    let scale = if layout.scale_factor.is_finite() && layout.scale_factor > 0.0 { layout.scale_factor } else { 1.0 };
    let logical_w = (layout.canvas_width as f64) / scale;
    let logical_h = (layout.canvas_height as f64) / scale;
    let logical_x = (layout.window_x as f64) / scale;
    let logical_top_y = (layout.window_y as f64) / scale;
    let logical_y = primary_display_height_pt - logical_top_y - logical_h;
    NSRect::new(logical_x, logical_y, logical_w, logical_h)
}

/// Converts top-left physical ActionDock canvas rect to Cocoa coordinates using primary display height
pub fn dock_physical_to_cocoa_rect(
    layout: &ActionDockLayout,
    screen_y_px: i32,
    screen_h_px: i32,
) -> NSRect {
    let scale = if layout.scale_factor.is_finite() && layout.scale_factor > 0.0 { layout.scale_factor } else { 1.0 };
    let primary_height_pt = get_primary_display_height_points()
        .unwrap_or_else(|| (screen_y_px + screen_h_px) as f64 / scale);
    dock_physical_to_cocoa_rect_with_primary_height(layout, primary_height_pt)
}

/// Converts unflipped Cocoa bottom-left view coordinates into top-down physical dock coordinates
pub fn cocoa_local_point_to_top_down_physical(
    local_x_pt: f64,
    local_y_pt: f64,
    canvas_h_pt: f64,
    scale_factor: f64,
) -> (f32, f32) {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 { scale_factor } else { 1.0 };
    let px = (local_x_pt * scale) as f32;
    let py = ((canvas_h_pt - local_y_pt) * scale) as f32;
    (px, py)
}

/// Evaluates which ActionDock droplet is hit by a Cocoa bottom-left local NSView point
pub fn hit_test_macos_dock_local_point(
    local_x_pt: f64,
    local_y_pt: f64,
    layout: &ActionDockLayout,
    is_visible: bool,
) -> Option<DockActionId> {
    if !is_visible {
        return None;
    }
    let scale = if layout.scale_factor.is_finite() && layout.scale_factor > 0.0 { layout.scale_factor } else { 1.0 };
    let canvas_h_pt = (layout.canvas_height as f64) / scale;
    let (px, py) = cocoa_local_point_to_top_down_physical(local_x_pt, local_y_pt, canvas_h_pt, scale);
    hit_test_dock_droplet(px, py, layout)
}

/// Returns whether Cocoa NSView hitTest: should claim the point or pass clicks through gaps
pub fn is_macos_dock_point_hittable(
    local_x_pt: f64,
    local_y_pt: f64,
    layout: &ActionDockLayout,
    is_visible: bool,
    is_pressed: bool,
) -> bool {
    if !is_visible {
        return false;
    }
    if is_pressed {
        return true;
    }
    hit_test_macos_dock_local_point(local_x_pt, local_y_pt, layout, is_visible).is_some()
}

#[cfg(target_os = "macos")]
fn register_action_dock_view_class() -> *const objc::runtime::Class {
    use cocoa::base::{id, nil, BOOL, YES};
    use cocoa::foundation::{NSPoint as CocoaNSPoint, NSRect as CocoaNSRect};
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};
    use objc::{class, msg_send, sel, sel_impl};
    static REGISTER_DOCK_VIEW_ONCE: std::sync::Once = std::sync::Once::new();
    REGISTER_DOCK_VIEW_ONCE.call_once(|| unsafe {
        let superclass = class!(NSView);
        if let Some(mut decl) = ClassDecl::new("DesktopBubbleChatActionDockView", superclass) {
            extern "C" fn accepts_first_mouse(_this: &Object, _cmd: Sel, _event: id) -> BOOL {
                YES
            }
            extern "C" fn hit_test(this: &Object, _cmd: Sel, point: CocoaNSPoint) -> id {
                unsafe {
                    let superview: id = msg_send![this, superview];
                    let local: CocoaNSPoint = msg_send![this, convertPoint: point fromView: superview];
                    if crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global()
                        .is_action_dock_local_point_hittable(local.x, local.y) {
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
            extern "C" fn mouse_moved(this: &Object, _cmd: Sel, event: id) {
                unsafe {
                    let win_pt: CocoaNSPoint = msg_send![event, locationInWindow];
                    let local: CocoaNSPoint = msg_send![this, convertPoint: win_pt fromView: nil];
                    crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global()
                        .handle_action_dock_mouse_move(local.x, local.y);
                }
            }
            extern "C" fn mouse_dragged(this: &Object, _cmd: Sel, event: id) {
                unsafe {
                    let win_pt: CocoaNSPoint = msg_send![event, locationInWindow];
                    let local: CocoaNSPoint = msg_send![this, convertPoint: win_pt fromView: nil];
                    crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global()
                        .handle_action_dock_mouse_move(local.x, local.y);
                }
            }
            extern "C" fn mouse_down(this: &Object, _cmd: Sel, event: id) {
                unsafe {
                    let win_pt: CocoaNSPoint = msg_send![event, locationInWindow];
                    let local: CocoaNSPoint = msg_send![this, convertPoint: win_pt fromView: nil];
                    crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global()
                        .handle_action_dock_mouse_down(local.x, local.y);
                }
            }
            extern "C" fn mouse_up(this: &Object, _cmd: Sel, event: id) {
                unsafe {
                    let win_pt: CocoaNSPoint = msg_send![event, locationInWindow];
                    let local: CocoaNSPoint = msg_send![this, convertPoint: win_pt fromView: nil];
                    crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global()
                        .handle_action_dock_mouse_up(local.x, local.y);
                }
            }
            extern "C" fn mouse_exited(_this: &Object, _cmd: Sel, _event: id) {
                crate::operating_systems::macos::bubble::manager::NativeBubbleManager::global()
                    .handle_action_dock_mouse_leave();
            }
            decl.add_method(sel!(acceptsFirstMouse:), accepts_first_mouse as extern "C" fn(&Object, Sel, id) -> BOOL);
            decl.add_method(sel!(hitTest:), hit_test as extern "C" fn(&Object, Sel, CocoaNSPoint) -> id);
            decl.add_method(sel!(resetCursorRects), reset_cursor_rects as extern "C" fn(&Object, Sel));
            decl.add_method(sel!(mouseMoved:), mouse_moved as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(mouseDragged:), mouse_dragged as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(mouseDown:), mouse_down as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(mouseUp:), mouse_up as extern "C" fn(&Object, Sel, id));
            decl.add_method(sel!(mouseExited:), mouse_exited as extern "C" fn(&Object, Sel, id));
            decl.register();
        }
    });
    Class::get("DesktopBubbleChatActionDockView").map(|c| c as *const Class).unwrap_or(std::ptr::null())
}

impl MacOSActionDockPanelController {
    /// Creates a non-activating borderless NSPanel hosting the custom Liquid Glass ActionDock NSView
    #[cfg(target_os = "macos")]
    pub fn create_action_dock_panel(rect: NSRect, is_always_on_top: bool) -> *mut c_void {
        use cocoa::base::{id, nil, NO, YES};
        use cocoa::foundation::{NSPoint as CocoaNSPoint, NSRect as CocoaNSRect, NSSize as CocoaNSSize};
        use objc::{class, msg_send, sel, sel_impl};
        with_autorelease_pool(|| unsafe {
            let style_mask = NS_WINDOW_STYLE_MASK_NONACTIVATING_PANEL | NS_WINDOW_STYLE_MASK_BORDERLESS;
            let cocoa_rect = CocoaNSRect::new(CocoaNSPoint::new(rect.origin.x, rect.origin.y), CocoaNSSize::new(rect.size.width, rect.size.height));
            let alloc_panel: id = msg_send![class!(NSPanel), alloc];
            let panel: id = msg_send![alloc_panel, initWithContentRect: cocoa_rect styleMask: style_mask backing: 2usize defer: NO];
            if panel == nil {
                return std::ptr::null_mut();
            }
            let _: () = msg_send![panel, setReleasedWhenClosed: NO];
            let _: () = msg_send![panel, setHidesOnDeactivate: NO];
            let _: () = msg_send![panel, setIgnoresMouseEvents: NO];
            let level = if is_always_on_top { NS_FLOATING_WINDOW_LEVEL } else { NS_NORMAL_WINDOW_LEVEL };
            let _: () = msg_send![panel, setLevel: level];
            let behavior = NS_WINDOW_COLLECTION_BEHAVIOR_CAN_JOIN_ALL_SPACES | NS_WINDOW_COLLECTION_BEHAVIOR_FULL_SCREEN_AUXILIARY;
            let _: () = msg_send![panel, setCollectionBehavior: behavior];
            let _: () = msg_send![panel, setOpaque: NO];
            let clear_color: id = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![panel, setBackgroundColor: clear_color];
            let _: () = msg_send![panel, setHasShadow: NO];
            let _: () = msg_send![panel, setAcceptsMouseMovedEvents: YES];
            let view_cls = register_action_dock_view_class();
            if !view_cls.is_null() {
                let view_alloc: id = msg_send![view_cls, alloc];
                let view_frame = CocoaNSRect::new(CocoaNSPoint::new(0.0, 0.0), CocoaNSSize::new(rect.size.width, rect.size.height));
                let custom_view: id = msg_send![view_alloc, initWithFrame: view_frame];
                if custom_view != nil {
                    let _: () = msg_send![custom_view, setWantsLayer: YES];
                    // NSTrackingMouseEnteredAndExited | NSTrackingMouseMoved | NSTrackingActiveAlways | NSTrackingInVisibleRect
                    let tracking_opts: usize = 0x01 | 0x02 | 0x80 | 0x200;
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

    /// Fallback for non-macOS compilation
    #[cfg(not(target_os = "macos"))]
    pub fn create_action_dock_panel(_rect: NSRect, _is_always_on_top: bool) -> *mut c_void {
        std::ptr::null_mut()
    }
}
