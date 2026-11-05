pub mod collision;
pub mod config;
pub mod engine;

pub use collision::{BubbleRect, CollisionResolver, ColumnBubbleSlot};
pub use config::{
    get_bubble_physics_config, get_target_dashboard_screen, update_target_dashboard_screen,
    BubbleConfigState, BubblePhysicsConfig, ScreenMetrics, SnapResult,
};
pub use engine::{BubbleMovementEngine, MutualSnapResult};
