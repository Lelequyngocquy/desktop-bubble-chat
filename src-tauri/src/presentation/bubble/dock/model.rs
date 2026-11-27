use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::movement::{get_bubble_physics_config, get_target_dashboard_screen};
use crate::state::ChatState;

/// Total number of droplets rendered in the vertical native ActionDock
pub const DOCK_ACTION_COUNT: usize = 5;

/// Canonical identifiers for vertical native action dock droplets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockActionId {
    Pin,
    AllowResize,
    PositionMode,
    Mute,
    HideBubble,
}

/// Ordered action registry rendered from closest-to-bubble to furthest
pub const DOCK_ACTIONS: [DockActionId; DOCK_ACTION_COUNT] = [
    DockActionId::Pin,
    DockActionId::AllowResize,
    DockActionId::PositionMode,
    DockActionId::Mute,
    DockActionId::HideBubble,
];

impl DockActionId {
    /// Returns zero-based slot index within DOCK_ACTIONS
    pub const fn index(self) -> usize {
        match self {
            Self::Pin => 0,
            Self::AllowResize => 1,
            Self::PositionMode => 2,
            Self::Mute => 3,
            Self::HideBubble => 4,
        }
    }

    /// Returns whether a subtle visual separator gap precedes this action
    pub const fn has_extra_separator_before(self) -> bool {
        matches!(self, Self::HideBubble)
    }
}

/// Vertical expansion direction relative to the anchored bubble
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDirection {
    Down,
    Up,
}

/// Horizontal side on which the hover tooltip dialog expands inward toward screen center
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockTooltipSide {
    Left,
    Right,
}

/// Semantic Liquid Glass color tone for an action droplet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockButtonTone {
    Neutral,
    ActiveAccent,
    MutedWarning,
    DestructiveHover,
}

/// Immutable SSOT snapshot driving ActionDock rendering and localized tooltips for an account
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockActionStateSnapshot {
    pub account_index: u32,
    pub is_pinned: bool,
    pub is_resizable: bool,
    pub is_free_position: bool,
    pub is_muted: bool,
    pub is_vietnamese: bool,
    pub bubble_size_px: u32,
    pub scale_factor: f64,
}

impl Default for DockActionStateSnapshot {
    fn default() -> Self {
        Self {
            account_index: 1,
            is_pinned: false,
            is_resizable: false,
            is_free_position: false,
            is_muted: false,
            is_vietnamese: false,
            bubble_size_px: 56,
            scale_factor: 1.0,
        }
    }
}

impl DockActionStateSnapshot {
    /// Returns whether the given toggle action is currently active
    pub const fn is_action_active(&self, action: DockActionId) -> bool {
        match action {
            DockActionId::Pin => self.is_pinned,
            DockActionId::AllowResize => self.is_resizable,
            DockActionId::PositionMode => self.is_free_position,
            DockActionId::Mute => self.is_muted,
            DockActionId::HideBubble => false,
        }
    }

    /// Resolves semantic glass tone for an action and hover flag
    pub const fn button_tone(&self, action: DockActionId, is_hovered: bool) -> DockButtonTone {
        match action {
            DockActionId::Pin if self.is_pinned => DockButtonTone::ActiveAccent,
            DockActionId::AllowResize if self.is_resizable => DockButtonTone::ActiveAccent,
            DockActionId::PositionMode if self.is_free_position => DockButtonTone::ActiveAccent,
            DockActionId::Mute if self.is_muted => DockButtonTone::MutedWarning,
            DockActionId::HideBubble if is_hovered => DockButtonTone::DestructiveHover,
            _ => DockButtonTone::Neutral,
        }
    }

    /// Returns concise state-aware tooltip description localized to Vietnamese or English
    pub const fn tooltip_label(&self, action: DockActionId) -> &'static str {
        if self.is_vietnamese {
            match action {
                DockActionId::Pin => if self.is_pinned { "Bỏ ghim khung chat" } else { "Ghim khung chat" },
                DockActionId::AllowResize => if self.is_resizable { "Khóa cỡ mặc định" } else { "Cho phép đổi cỡ" },
                DockActionId::PositionMode => if self.is_free_position { "Tự thụt mép màn hình" } else { "Kéo thả tự do" },
                DockActionId::Mute => if self.is_muted { "Bật âm thông báo" } else { "Tắt âm thông báo" },
                DockActionId::HideBubble => "Ẩn bong bóng",
            }
        } else {
            match action {
                DockActionId::Pin => if self.is_pinned { "Unpin chat" } else { "Pin chat" },
                DockActionId::AllowResize => if self.is_resizable { "Reset default size" } else { "Allow resize" },
                DockActionId::PositionMode => if self.is_free_position { "Switch to auto-tuck" } else { "Switch to free move" },
                DockActionId::Mute => if self.is_muted { "Unmute alerts" } else { "Mute alerts" },
                DockActionId::HideBubble => "Hide bubble",
            }
        }
    }
}

/// Builds a live SSOT dock snapshot from Tauri managed states
pub fn snapshot_from_app(
    app: &AppHandle,
    account_index: u32,
    bubble_size_px: u32,
) -> DockActionStateSnapshot {
    let config = get_bubble_physics_config(app);
    let screen = get_target_dashboard_screen(Some(app));
    let (is_pinned, is_resizable, is_muted, is_vietnamese) = app
        .try_state::<Arc<ChatState>>()
        .map(|cs| (
            cs.is_chat_pinned(account_index),
            cs.get_chat_window_custom_size(account_index).is_some(),
            cs.is_account_muted(account_index),
            cs.get_user_config().current_locale().starts_with("vi"),
        ))
        .unwrap_or((false, false, false, false));
    DockActionStateSnapshot {
        account_index,
        is_pinned,
        is_resizable,
        is_free_position: config.is_free_position,
        is_muted,
        is_vietnamese,
        bubble_size_px: bubble_size_px.max(24),
        scale_factor: screen.scale_factor.max(0.5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_registry_order_tones_and_tooltips() {
        assert_eq!(DOCK_ACTIONS.len(), 5);
        assert_eq!(DOCK_ACTIONS[0], DockActionId::Pin);
        assert_eq!(DOCK_ACTIONS[4], DockActionId::HideBubble);
        assert!(DockActionId::HideBubble.has_extra_separator_before());
        assert!(!DockActionId::Pin.has_extra_separator_before());
        let snap = DockActionStateSnapshot {
            is_pinned: true,
            is_resizable: false,
            is_free_position: true,
            is_muted: true,
            is_vietnamese: true,
            ..Default::default()
        };
        assert_eq!(snap.button_tone(DockActionId::Pin, false), DockButtonTone::ActiveAccent);
        assert_eq!(snap.button_tone(DockActionId::AllowResize, false), DockButtonTone::Neutral);
        assert_eq!(snap.button_tone(DockActionId::Mute, false), DockButtonTone::MutedWarning);
        assert_eq!(snap.button_tone(DockActionId::HideBubble, true), DockButtonTone::DestructiveHover);
        assert_eq!(snap.tooltip_label(DockActionId::Pin), "Bỏ ghim khung chat");
        assert_eq!(snap.tooltip_label(DockActionId::HideBubble), "Ẩn bong bóng");
        let en = DockActionStateSnapshot { is_vietnamese: false, ..Default::default() };
        assert_eq!(en.tooltip_label(DockActionId::Pin), "Pin chat");
        assert_eq!(en.tooltip_label(DockActionId::AllowResize), "Allow resize");
    }
}
