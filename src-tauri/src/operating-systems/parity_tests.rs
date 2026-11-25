//! Cross-OS parity contract tests compiled and executed on every host OS.
use crate::commands::system::app::global_shortcuts_for_os;
use crate::movement::{BubbleMovementEngine, BubblePhysicsConfig, BubbleRect, CollisionResolver, ScreenMetrics};
use crate::operating_systems::linux::autostart::LinuxAutostartManager;
use crate::operating_systems::linux::bubble::animator::SpringAnimationState as LinuxSpringAnim;
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
