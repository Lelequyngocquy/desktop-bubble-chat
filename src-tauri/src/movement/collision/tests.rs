use super::*;

#[test]
fn test_collision_resolution_same_y() {
    let screen = ScreenMetrics {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale_factor: 1.0,
    };
    let config = BubblePhysicsConfig::default();
    let bubble_size = config.size as i32; // 56
    let min_spacing = bubble_size + config.gap as i32;
    let obstacles = vec![
        BubbleRect {
            account_index: 1,
            x: 1906,
            y: 300,
            width: bubble_size,
            height: bubble_size,
        },
        BubbleRect {
            account_index: 2,
            x: 1800,
            y: 300,
            width: bubble_size,
            height: bubble_size,
        },
    ];
    let layout = CollisionResolver::resolve_mutual_column_layout(
        2,
        300,
        false,
        &screen,
        &config,
        &obstacles,
    );
    let b1 = layout.iter().find(|s| s.account_index == 1).unwrap();
    let b2 = layout.iter().find(|s| s.account_index == 2).unwrap();
    assert!((b1.target_y - b2.target_y).abs() >= min_spacing);
}

#[test]
fn test_mutual_column_layout_same_y() {
    let screen = ScreenMetrics {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale_factor: 1.0,
    };
    let config = BubblePhysicsConfig::default();
    let bubble_size = config.size as i32; // 56
    let min_spacing = bubble_size + config.gap as i32; // 68
    let obstacles = vec![
        BubbleRect {
            account_index: 1,
            x: 1906,
            y: 300,
            width: bubble_size,
            height: bubble_size,
        },
    ];
    let layout = CollisionResolver::resolve_mutual_column_layout(
        2,
        300,
        false,
        &screen,
        &config,
        &obstacles,
    );
    assert_eq!(layout.len(), 2);
    let b2 = layout.iter().find(|s| s.account_index == 2).unwrap();
    let b1 = layout.iter().find(|s| s.account_index == 1).unwrap();
    assert_eq!(b2.target_y, 300);
    assert_eq!(b1.target_y, 300 + min_spacing);
    assert!((b1.target_y - b2.target_y).abs() >= min_spacing);
}

#[test]
fn test_mutual_column_layout_top_boundary() {
    let screen = ScreenMetrics {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale_factor: 1.0,
    };
    let config = BubblePhysicsConfig::default();
    let bubble_size = config.size as i32;
    let min_spacing = bubble_size + config.gap as i32;
    let min_y = screen.y + config.margin as i32;
    let obstacles = vec![
        BubbleRect {
            account_index: 1,
            x: 1906,
            y: 15,
            width: bubble_size,
            height: bubble_size,
        },
    ];
    let layout = CollisionResolver::resolve_mutual_column_layout(
        2,
        15,
        false,
        &screen,
        &config,
        &obstacles,
    );
    assert_eq!(layout.len(), 2);
    for s in &layout {
        assert!(s.target_y >= min_y);
    }
    let b1 = layout.iter().find(|s| s.account_index == 1).unwrap();
    let b2 = layout.iter().find(|s| s.account_index == 2).unwrap();
    assert!((b1.target_y - b2.target_y).abs() >= min_spacing);
}

#[test]
fn test_mutual_column_layout_bottom_boundary() {
    let screen = ScreenMetrics {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale_factor: 1.0,
    };
    let config = BubblePhysicsConfig::default();
    let bubble_size = config.size as i32;
    let min_spacing = bubble_size + config.gap as i32;
    let max_y = screen.y + screen.height - bubble_size - config.taskbar_margin as i32;
    let obstacles = vec![
        BubbleRect {
            account_index: 1,
            x: 1906,
            y: max_y,
            width: bubble_size,
            height: bubble_size,
        },
    ];
    let layout = CollisionResolver::resolve_mutual_column_layout(
        2,
        max_y,
        false,
        &screen,
        &config,
        &obstacles,
    );
    assert_eq!(layout.len(), 2);
    for s in &layout {
        assert!(s.target_y <= max_y);
    }
    let b1 = layout.iter().find(|s| s.account_index == 1).unwrap();
    let b2 = layout.iter().find(|s| s.account_index == 2).unwrap();
    assert!((b1.target_y - b2.target_y).abs() >= min_spacing);
}

#[test]
fn test_mutual_column_layout_three_bubbles_cascade() {
    let screen = ScreenMetrics {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale_factor: 1.0,
    };
    let config = BubblePhysicsConfig::default();
    let bubble_size = config.size as i32;
    let min_spacing = bubble_size + config.gap as i32;
    let obstacles = vec![
        BubbleRect {
            account_index: 1,
            x: 1906,
            y: 300,
            width: bubble_size,
            height: bubble_size,
        },
        BubbleRect {
            account_index: 3,
            x: 1906,
            y: 368,
            width: bubble_size,
            height: bubble_size,
        },
    ];
    let layout = CollisionResolver::resolve_mutual_column_layout(
        2,
        300,
        false,
        &screen,
        &config,
        &obstacles,
    );
    assert_eq!(layout.len(), 3);
    let mut sorted_y: Vec<i32> = layout.iter().map(|s| s.target_y).collect();
    sorted_y.sort();
    for window in sorted_y.windows(2) {
        assert!(window[1] - window[0] >= min_spacing);
    }
}

#[test]
fn test_resolve_2d_free_collision_avoidance() {
    let screen = ScreenMetrics {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale_factor: 1.0,
    };
    let config = BubblePhysicsConfig::default();
    let bubble_size = config.size as i32;
    let min_spacing = (bubble_size + config.gap as i32) as f64;
    // Bubble 1 placed in middle of screen (X: 500, Y: 500)
    let obstacles = vec![BubbleRect {
        account_index: 1,
        x: 500,
        y: 500,
        width: bubble_size,
        height: bubble_size,
    }];
    // Moving bubble dropped exactly at the same position (X: 500, Y: 500)
    let (resolved_x, resolved_y) = CollisionResolver::resolve_2d_free_collision(
        2,
        500,
        500,
        &screen,
        &config,
        &obstacles,
    );
    // Distance after resolution must be >= min_spacing
    let dx = (resolved_x - 500) as f64;
    let dy = (resolved_y - 500) as f64;
    let dist = (dx * dx + dy * dy).sqrt();
    assert!(dist >= min_spacing - 1.0);
}

#[test]
fn test_horizontal_and_2d_overlap_detection() {
    assert!(CollisionResolver::check_horizontal_overlap(100, 56, 120, 56, 12));
    assert!(!CollisionResolver::check_horizontal_overlap(100, 56, 200, 56, 12));
    let b1 = BubbleRect { account_index: 1, x: 100, y: 100, width: 56, height: 56 };
    let b2 = BubbleRect { account_index: 2, x: 120, y: 120, width: 56, height: 56 };
    let b3 = BubbleRect { account_index: 3, x: 300, y: 100, width: 56, height: 56 };
    assert!(CollisionResolver::check_2d_overlap(&b1, &b2, 12));
    assert!(!CollisionResolver::check_2d_overlap(&b1, &b3, 12));
}

#[test]
fn test_resolve_2d_free_collision_screen_boundary() {
    let screen = ScreenMetrics { x: 0, y: 0, width: 1920, height: 1080, scale_factor: 1.0 };
    let config = BubblePhysicsConfig::default();
    let bubble_size = config.size as i32;
    let margin = config.margin as i32;
    let obstacles = vec![BubbleRect { account_index: 1, x: margin, y: margin, width: bubble_size, height: bubble_size }];
    // Drop bubble directly on the obstacle at screen top-left corner
    let (rx, ry) = CollisionResolver::resolve_2d_free_collision(2, margin, margin, &screen, &config, &obstacles);
    assert!(rx >= margin);
    assert!(ry >= margin);
    let dist = (((rx - margin).pow(2) + (ry - margin).pow(2)) as f64).sqrt();
    assert!(dist >= (bubble_size + config.gap as i32) as f64 - 1.0);
}

#[test]
fn test_resolve_2d_free_collision_narrow_viewport_no_panic() {
    let screen = ScreenMetrics { x: 0, y: 0, width: 40, height: 40, scale_factor: 1.0 };
    let config = BubblePhysicsConfig::default();
    let (rx, ry) = CollisionResolver::resolve_2d_free_collision(1, 10, 10, &screen, &config, &[]);
    assert_eq!(rx, config.margin as i32);
    assert_eq!(ry, config.margin as i32);
}
