//! Cross-OS parity contract tests compiled and executed on every host OS.
use crate::commands::system::app::global_shortcuts_for_os;
use crate::movement::{BubbleMovementEngine, BubblePhysicsConfig, BubbleRect, CollisionResolver, ScreenMetrics};
use crate::operating_systems::linux::autostart::LinuxAutostartManager;
use crate::operating_systems::linux::bubble::animator::SpringAnimationState as LinuxSpringAnim;
use crate::operating_systems::linux::bubble::dock::{
    compute_dock_input_rects, hit_test_linux_dock_local_logical,
};
use crate::operating_systems::linux::bubble::window::gtk_window::{
    compute_bubble_input_rects, logical_root_to_physical, physical_coord_to_logical, physical_size_to_logical,
};
use crate::operating_systems::linux::bubble::window::state::{
    compute_linux_bubble_drop_plan, should_tuck_linux_bubble_on_leave,
};
use crate::operating_systems::linux::cursor::LinuxCursorTracker;
use crate::operating_systems::linux::memory::LinuxMemoryOptimizer;
use crate::operating_systems::linux::theme::LinuxThemeManager;
use crate::operating_systems::macos::bubble::animator::SpringAnimationState as MacOSSpringAnim;
use crate::operating_systems::macos::bubble::dock::{
    dock_physical_to_cocoa_rect_with_primary_height, hit_test_macos_dock_local_point,
    is_macos_dock_point_hittable,
};
use crate::operating_systems::macos::bubble::window::panel::{
    is_point_in_bubble_hit_zone, MacOSPanelController,
};
use crate::operating_systems::macos::bubble::window::state::{
    compute_bubble_drop_plan, should_tuck_after_hover_leave,
};
use crate::operating_systems::macos::cursor::MacOSCursorTracker;
use crate::operating_systems::macos::dialog::MacOSDialogManager;
use crate::operating_systems::macos::memory::MacOSMemoryOptimizer;
use crate::operating_systems::macos::theme::MacOSThemeManager;
use crate::platforms::messenger::windows::chat_window::{
    data_store_identifier_for_account, platform_user_agent,
};
use crate::utils::performance::extract_account_index_from_text;
use crate::utils::update_notifier::is_stale_update_artifact_name;
use std::collections::HashSet;
use tauri::PhysicalPosition;

fn sample_screen(scale: f64) -> ScreenMetrics {
    ScreenMetrics { x: 0, y: 0, width: (1920.0 * scale) as i32, height: (1080.0 * scale) as i32, scale_factor: scale }
}

#[test]
fn parity_drag_drop_free_position_matches_windows_collision_resolver() {
    let screen = sample_screen(1.0);
    let cfg = BubblePhysicsConfig { is_free_position: true, ..Default::default() };
    let sibling = BubbleRect { account_index: 2, x: 500, y: 400, width: 56, height: 56 };
    let existing = vec![sibling];
    let (win_x, win_y) = CollisionResolver::resolve_2d_free_collision(1, 502, 402, &screen, &cfg, &existing);
    let mac_plan = compute_bubble_drop_plan(1, 502, 402, false, &screen, &cfg, &existing);
    let linux_plan = compute_linux_bubble_drop_plan(1, 502, 402, false, &screen, &cfg, &existing);
    assert!(!mac_plan.should_dismiss);
    assert!(!linux_plan.should_dismiss);
    assert_eq!((mac_plan.target_x, mac_plan.target_y), (win_x, win_y));
    assert_eq!((linux_plan.target_x, linux_plan.target_y), (win_x, win_y));
    assert!(mac_plan.displaced_siblings.is_empty());
    assert!(linux_plan.displaced_siblings.is_empty());
}

#[test]
fn parity_drag_drop_mutual_snap_and_dismiss_match_across_all_oses() {
    let screen = sample_screen(1.0);
    let cfg = BubblePhysicsConfig { is_free_position: false, ..Default::default() };
    let sibling = BubbleRect { account_index: 2, x: 0, y: 300, width: 56, height: 56 };
    let existing = vec![sibling];
    let win_snap = BubbleMovementEngine::calculate_mutual_snap(&screen, &cfg, PhysicalPosition { x: 40, y: 300 }, 1, &existing);
    let mac_plan = compute_bubble_drop_plan(1, 40, 300, false, &screen, &cfg, &existing);
    let linux_plan = compute_linux_bubble_drop_plan(1, 40, 300, false, &screen, &cfg, &existing);
    assert_eq!((mac_plan.target_x, mac_plan.target_y), (win_snap.target_x, win_snap.target_y));
    assert_eq!((linux_plan.target_x, linux_plan.target_y), (win_snap.target_x, win_snap.target_y));
    assert_eq!(mac_plan.displaced_siblings, linux_plan.displaced_siblings);
    assert_eq!(mac_plan.displaced_siblings[0].1, 0);
    let mac_alert_plan = compute_bubble_drop_plan(1, 40, 300, true, &screen, &cfg, &existing);
    let linux_alert_plan = compute_linux_bubble_drop_plan(1, 40, 300, true, &screen, &cfg, &existing);
    assert_eq!(mac_alert_plan.target_x, 0);
    assert_eq!(linux_alert_plan.target_x, 0);
    let mac_dismiss = compute_bubble_drop_plan(1, 40, 1040, false, &screen, &cfg, &existing);
    let linux_dismiss = compute_linux_bubble_drop_plan(1, 40, 1040, false, &screen, &cfg, &existing);
    assert!(mac_dismiss.should_dismiss);
    assert!(linux_dismiss.should_dismiss);
}

#[test]
fn parity_hover_leave_tuck_guards_match_across_macos_and_linux() {
    for is_dragging in [false, true] {
        for is_hovered in [false, true] {
            for is_chat_open in [false, true] {
                for is_chat_pinned in [false, true] {
                    for is_free_position in [false, true] {
                        for is_alert_active in [false, true] {
                            let expected = !is_dragging && !is_hovered && !is_chat_open && !is_chat_pinned && !is_free_position && !is_alert_active;
                            assert_eq!(should_tuck_after_hover_leave(is_dragging, is_hovered, is_chat_open, is_chat_pinned, is_free_position, is_alert_active), expected);
                            assert_eq!(should_tuck_linux_bubble_on_leave(is_dragging, is_hovered, is_chat_open, is_chat_pinned, is_free_position, is_alert_active), expected);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn macos_cocoa_y_flip_hit_test_and_cursor_scaling_contracts() {
    let rect = MacOSPanelController::top_left_physical_to_cocoa_rect_with_primary_height(200, 100, 172, 160, 900.0, 2.0);
    assert!((rect.size.width - 86.0).abs() < 1e-6);
    assert!((rect.size.height - 80.0).abs() < 1e-6);
    assert!((rect.origin.x - 85.0).abs() < 1e-6);
    assert!((rect.origin.y - 782.0).abs() < 1e-6);
    assert!(is_point_in_bubble_hit_zone(58.0, 52.0, 116.0, 104.0, 56.0, 0, true));
    assert!(!is_point_in_bubble_hit_zone(2.0, 2.0, 116.0, 104.0, 56.0, 0, true));
    assert!(!is_point_in_bubble_hit_zone(58.0, 88.0, 116.0, 104.0, 56.0, 3, true));
    assert!(!is_point_in_bubble_hit_zone(83.0, 73.0, 116.0, 104.0, 56.0, 0, true));
    assert!(is_point_in_bubble_hit_zone(83.0, 73.0, 116.0, 104.0, 56.0, 3, true));
    assert_eq!(MacOSCursorTracker::quartz_point_to_physical(500.0, 300.0, 2.0), (1000, 600));
    assert_eq!(MacOSDialogManager::resolve_confirm_button_labels("Xác nhận thoát", "Bạn có chắc"), ("Thoát", "Hủy"));
    assert_eq!(MacOSDialogManager::resolve_confirm_button_labels("Confirm Exit", "Are you sure?"), ("Exit", "Cancel"));
    assert!(MacOSMemoryOptimizer::is_webview_process("com.apple.WebKit.GPU"));
    let rgb = MacOSThemeManager::parse_hex_rgb("#112233").expect("valid hex");
    assert!(MacOSThemeManager::is_dark_color(rgb.0, rgb.1, rgb.2));
}

#[test]
fn linux_hidpi_input_shape_wayland_and_autostart_contracts() {
    assert_eq!(physical_coord_to_logical(400, 2.0), 200);
    assert_eq!(physical_size_to_logical(112, 2.0), 56);
    assert_eq!(logical_root_to_physical(200.0, 2.0), 400);
    let rects = compute_bubble_input_rects(116, 104, 56, 5, true);
    assert!(!rects.is_empty());
    assert!(rects.iter().all(|r| r.width > 0 && r.height > 0 && r.x >= 0 && r.y >= 0));
    assert!(LinuxCursorTracker::classify_gdk_display_name("wayland-0"));
    assert!(!LinuxCursorTracker::classify_gdk_display_name(":0"));
    assert!(!LinuxCursorTracker::classify_gdk_display_name(":1.0"));
    let resolved = LinuxAutostartManager::resolve_autostart_exe_path(Some("/home/user/Apps/DesktopBubbleChat.AppImage"), "/tmp/.mount_abc/usr/bin/desktop-bubble-chat");
    assert_eq!(resolved, "/home/user/Apps/DesktopBubbleChat.AppImage");
    let entry = LinuxAutostartManager::generate_desktop_entry("/opt/100%/$HOME/desktop-bubble-chat");
    assert!(entry.contains("100%%"));
    assert!(entry.contains("\\$HOME"));
    assert!(LinuxMemoryOptimizer::matches_profile_needle("/home/u/.local/share/profiles/messenger_03/cookies.sqlite", "messenger_03"));
    assert!(LinuxThemeManager::parse_hex_is_dark("#1E1E2E"));
    assert!(!LinuxThemeManager::parse_hex_is_dark("#F8FAFC"));
}

#[test]
fn shared_multi_os_data_store_ua_shortcuts_and_updater_contracts() {
    let mut ids = HashSet::new();
    for idx in 1..=5 {
        let id = data_store_identifier_for_account(idx);
        assert_eq!(id, data_store_identifier_for_account(idx));
        assert!(ids.insert(id));
    }
    assert!(platform_user_agent("windows").contains("Edg/131.0.0.0"));
    assert!(platform_user_agent("macos").contains("Macintosh"));
    assert!(platform_user_agent("linux").contains("Linux"));
    assert_eq!(global_shortcuts_for_os("windows"), &["CommandOrControl+Shift+M", "Super+Shift+M"]);
    assert_eq!(global_shortcuts_for_os("macos"), &["CommandOrControl+Shift+M", "Ctrl+Shift+M"]);
    assert_eq!(global_shortcuts_for_os("linux"), &["CommandOrControl+Shift+M"]);
    assert_eq!(extract_account_index_from_text("/home/u/.config/profiles/messenger_05/cache"), 5);
    assert!(is_stale_update_artifact_name("DesktopBubbleChat_1.1.7.exe"));
    assert!(is_stale_update_artifact_name("DesktopBubbleChat_1.1.7.dmg"));
    assert!(is_stale_update_artifact_name("DesktopBubbleChat.app.tar.gz"));
    assert!(is_stale_update_artifact_name("desktop-bubble-chat_1.1.7.AppImage"));
    assert!(is_stale_update_artifact_name("desktop-bubble-chat_1.1.7_amd64.deb"));
    assert!(!is_stale_update_artifact_name("other-app.dmg"));
}

#[test]
fn parity_spring_animation_lifecycle_matches_across_macos_and_linux() {
    let mut mac_anim = MacOSSpringAnim::default();
    let mut linux_anim = LinuxSpringAnim::default();
    mac_anim.start(100, 200, 0, 260, 50);
    linux_anim.start(100, 200, 0, 260, 50);
    assert!(mac_anim.is_animating && linux_anim.is_animating);
    std::thread::sleep(std::time::Duration::from_millis(80));
    let mac_step = mac_anim.step();
    let linux_step = linux_anim.step();
    assert_eq!(mac_step, Some((0, 260, true)));
    assert_eq!(linux_step, Some((0, 260, true)));
    assert!(!mac_anim.is_animating && !linux_anim.is_animating);
}

#[test]
fn parity_heartbeat_state_and_pause_lifecycle_matches_across_macos_and_linux() {
    use crate::operating_systems::linux::bubble::animator::HeartbeatAnimationState as LinuxHeartbeat;
    use crate::operating_systems::macos::bubble::animator::HeartbeatAnimationState as MacOSHeartbeat;
    let mut mac = MacOSHeartbeat::default();
    let mut linux = LinuxHeartbeat::default();
    assert!(!mac.should_animate() && !linux.should_animate());
    assert!(mac.set_alert_active(true) && linux.set_alert_active(true));
    assert!(mac.should_animate() && linux.should_animate());
    assert_eq!(mac.next_delay_ms(), Some(16));
    assert_eq!(linux.next_delay_ms(), Some(16));
    assert!(mac.set_paused(true) && linux.set_paused(true));
    assert!(!mac.should_animate() && !linux.should_animate());
    assert_eq!(mac.next_delay_ms(), None);
    assert_eq!(linux.next_delay_ms(), None);
    assert!(mac.set_paused(false) && linux.set_paused(false));
    assert!(mac.should_animate() && linux.should_animate());
    assert!(mac.set_alert_active(false) && linux.set_alert_active(false));
    assert!(!mac.should_animate() && !linux.should_animate());
}

#[test]
fn parity_action_dock_hit_test_and_hover_bridge_across_all_oses() {
    use crate::presentation::bubble::dock::{
        compute_action_dock_layout, is_point_in_dock_hover_corridor,
    };
    for (bubble_logical, scale) in [(48u32, 1.0f64), (56u32, 1.5f64), (64u32, 2.0f64)] {
        let screen = sample_screen(scale);
        let bubble_px = (bubble_logical as f64 * scale).round() as u32;
        let layout = compute_action_dock_layout(20, 120, bubble_px, &screen, 12.0, 60.0);
        let canvas_h_pt = (layout.canvas_height as f64) / scale;
        let linux_rects = compute_dock_input_rects(&layout, true);
        assert!(!linux_rects.is_empty());
        assert!(compute_dock_input_rects(&layout, false).is_empty());
        for slot in &layout.slots {
            let screen_x = layout.window_x + slot.local_cx.round() as i32;
            let screen_y = layout.window_y + slot.local_cy.round() as i32;
            let mac_x_pt = (slot.local_cx as f64) / scale;
            let mac_y_pt = canvas_h_pt - (slot.local_cy as f64) / scale;
            let linux_x = (slot.local_cx as f64) / scale;
            let linux_y = (slot.local_cy as f64) / scale;
            #[cfg(target_os = "windows")]
            {
                use crate::operating_systems::windows::bubble::dock::hit_test_action_dock_screen_point;
                use crate::operating_systems::windows::bubble::window::{HTCLIENT, HTTRANSPARENT};
                assert_eq!(hit_test_action_dock_screen_point(screen_x, screen_y, &layout, true, false), HTCLIENT);
                assert_eq!(hit_test_action_dock_screen_point(screen_x, screen_y, &layout, false, false), HTTRANSPARENT);
            }
            assert_eq!(hit_test_macos_dock_local_point(mac_x_pt, mac_y_pt, &layout, true), Some(slot.action));
            assert!(is_macos_dock_point_hittable(mac_x_pt, mac_y_pt, &layout, true, false));
            assert!(!is_macos_dock_point_hittable(mac_x_pt, mac_y_pt, &layout, false, false));
            assert_eq!(hit_test_linux_dock_local_logical(linux_x, linux_y, &layout, true), Some(slot.action));
            assert_eq!(hit_test_linux_dock_local_logical(linux_x, linux_y, &layout, false), None);
            let lx_i = linux_x.round() as i32;
            let ly_i = linux_y.round() as i32;
            assert!(linux_rects.iter().any(|r| lx_i >= r.x && lx_i < r.x + r.width && ly_i >= r.y && ly_i < r.y + r.height));
            assert!(is_point_in_dock_hover_corridor(screen_x, screen_y, &layout));
        }
        let gap_cx_px = layout.slots[0].local_cx;
        let gap_cy_px = (layout.slots[0].local_cy + layout.slots[1].local_cy) * 0.5;
        let gap_screen_x = layout.window_x + gap_cx_px.round() as i32;
        let gap_screen_y = layout.window_y + gap_cy_px.round() as i32;
        let gap_mac_x = (gap_cx_px as f64) / scale;
        let gap_mac_y = canvas_h_pt - (gap_cy_px as f64) / scale;
        let gap_linux_x = (gap_cx_px as f64) / scale;
        let gap_linux_y = (gap_cy_px as f64) / scale;
        #[cfg(target_os = "windows")]
        {
            use crate::operating_systems::windows::bubble::dock::hit_test_action_dock_screen_point;
            use crate::operating_systems::windows::bubble::window::HTTRANSPARENT;
            assert_eq!(hit_test_action_dock_screen_point(gap_screen_x, gap_screen_y, &layout, true, false), HTTRANSPARENT);
        }
        assert_eq!(hit_test_macos_dock_local_point(gap_mac_x, gap_mac_y, &layout, true), None);
        assert!(!is_macos_dock_point_hittable(gap_mac_x, gap_mac_y, &layout, true, false));
        assert_eq!(hit_test_linux_dock_local_logical(gap_linux_x, gap_linux_y, &layout, true), None);
        let glx_i = gap_linux_x.round() as i32;
        let gly_i = gap_linux_y.round() as i32;
        assert!(!linux_rects.iter().any(|r| glx_i >= r.x && glx_i < r.x + r.width && gly_i >= r.y && gly_i < r.y + r.height));
        assert!(is_point_in_dock_hover_corridor(gap_screen_x, gap_screen_y, &layout));
    }
}

#[test]
fn parity_action_dock_cocoa_frame_flip_and_linux_hidpi_scaling() {
    use crate::presentation::bubble::dock::{compute_action_dock_layout, DockDirection};
    let screen = sample_screen(2.0);
    let down = compute_action_dock_layout(40, 200, 112, &screen, 12.0, 60.0);
    assert_eq!(down.direction, DockDirection::Down);
    let up = compute_action_dock_layout(40, 1920, 112, &screen, 12.0, 60.0);
    assert_eq!(up.direction, DockDirection::Up);
    for layout in [&down, &up] {
        let cocoa = dock_physical_to_cocoa_rect_with_primary_height(layout, 1080.0);
        let expected_w = (layout.canvas_width as f64) / 2.0;
        let expected_h = (layout.canvas_height as f64) / 2.0;
        let expected_x = (layout.window_x as f64) / 2.0;
        let expected_y = 1080.0 - (layout.window_y as f64) / 2.0 - expected_h;
        assert!((cocoa.size.width - expected_w).abs() < 1e-6);
        assert!((cocoa.size.height - expected_h).abs() < 1e-6);
        assert!((cocoa.origin.x - expected_x).abs() < 1e-6);
        assert!((cocoa.origin.y - expected_y).abs() < 1e-6);
        let rects = compute_dock_input_rects(layout, true);
        let max_w = physical_size_to_logical(layout.canvas_width, 2.0);
        let max_h = physical_size_to_logical(layout.canvas_height, 2.0);
        assert!(!rects.is_empty());
        assert!(rects.iter().all(|r| r.x >= 0 && r.y >= 0 && r.width > 0 && r.height > 0 && r.x + r.width <= max_w && r.y + r.height <= max_h));
    }
}

#[test]
fn parity_action_dock_animation_click_and_sibling_push_across_macos_and_linux() {
    use crate::presentation::bubble::dock::{
        compute_dock_placement_plan, is_point_in_dock_hover_corridor, DockActionId,
        DockAnimationState, DockSiblingOffsetTracker, DockVisibilityPhase,
    };
    let screen = sample_screen(1.0);
    let cfg = BubblePhysicsConfig::default();
    let bubbles = vec![
        BubbleRect { account_index: 1, x: 0, y: 100, width: 56, height: 56 },
        BubbleRect { account_index: 2, x: -42, y: 168, width: 56, height: 56 },
    ];
    let plan = compute_dock_placement_plan(1, 0, 100, 56, &screen, &cfg, &bubbles);
    assert_eq!(plan.sibling_pushes.len(), 1);
    let mut tracker = DockSiblingOffsetTracker::default();
    let diff = tracker.apply_plan(1, &plan.sibling_pushes);
    assert!(diff.to_restore.is_empty());
    assert_eq!(diff.to_push[0].0, 2);
    let mut mac_anim = DockAnimationState::default();
    let mut linux_anim = DockAnimationState::default();
    mac_anim.start_appear();
    linux_anim.start_appear();
    assert!(mac_anim.needs_frame() && linux_anim.needs_frame());
    for _ in 0..20 {
        let _ = mac_anim.step(16.0);
        let _ = linux_anim.step(16.0);
    }
    assert_eq!(mac_anim.phase, DockVisibilityPhase::IdleVisible);
    assert_eq!(linux_anim.phase, DockVisibilityPhase::IdleVisible);
    assert!(!mac_anim.needs_frame() && !linux_anim.needs_frame());
    let mute_slot = plan.layout.slots[DockActionId::Mute.index()];
    let mac_x = mute_slot.local_cx as f64;
    let mac_y = plan.layout.canvas_height as f64 - mute_slot.local_cy as f64;
    let linux_x = mute_slot.local_cx as f64;
    let linux_y = mute_slot.local_cy as f64;
    let mac_hit = hit_test_macos_dock_local_point(mac_x, mac_y, &plan.layout, true);
    let linux_hit = hit_test_linux_dock_local_logical(linux_x, linux_y, &plan.layout, true);
    assert_eq!(mac_hit, Some(DockActionId::Mute));
    assert_eq!(linux_hit, Some(DockActionId::Mute));
    assert!(mac_anim.set_hovered(mac_hit) && linux_anim.set_hovered(linux_hit));
    assert!(mac_anim.set_pressed(mac_hit) && linux_anim.set_pressed(linux_hit));
    let corridor_x = plan.layout.window_x + mute_slot.local_cx.round() as i32;
    let corridor_y = plan.layout.window_y + mute_slot.local_cy.round() as i32;
    assert!(is_point_in_dock_hover_corridor(corridor_x, corridor_y, &plan.layout));
    mac_anim.hide_immediate();
    linux_anim.hide_immediate();
    assert!(!mac_anim.needs_frame() && !linux_anim.needs_frame());
    let restores = tracker.clear_for_close(Some(1)).expect("owner closes");
    assert_eq!(restores, vec![(2, 168)]);
}
