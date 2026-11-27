use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::movement::{get_bubble_physics_config, get_target_dashboard_screen};
use crate::state::ChatState;

/// Canonical identifiers for vertical native action dock droplets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DockActionId {
    Pin,
    AllowResize,
    PositionMode,
    Mute,
    Reload,
    HideBubble,
}

/// Ordered action registry rendered from closest-to-bubble to furthest
pub const DOCK_ACTIONS: [DockActionId; 6] = [
    DockActionId::Pin,
    DockActionId::AllowResize,
    DockActionId::PositionMode,
    DockActionId::Mute,
    DockActionId::Reload,
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
            Self::Reload => 4,
            Self::HideBubble => 5,
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

/// Semantic Liquid Glass color tone for an action droplet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockButtonTone {
    Neutral,
    ActiveAccent,
    MutedWarning,
    DestructiveHover,
}

/// Immutable SSOT snapshot driving ActionDock rendering for an account
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockActionStateSnapshot {
    pub account_index: u32,
    pub is_pinned: bool,
    pub is_resizable: bool,
    pub is_free_position: bool,
    pub is_muted: bool,
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
            DockActionId::Reload | DockActionId::HideBubble => false,
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
}

/// Builds a live SSOT dock snapshot from Tauri managed states
pub fn snapshot_from_app(
    app: &AppHandle,
    account_index: u32,
    bubble_size_px: u32,
) -> DockActionStateSnapshot {
    let config = get_bubble_physics_config(app);
    let screen = get_target_dashboard_screen(Some(app));
    let (is_pinned, is_resizable, is_muted) = app
        .try_state::<Arc<ChatState>>()
        .map(|cs| (
            cs.is_chat_pinned(account_index),
            cs.get_chat_window_custom_size(account_index).is_some(),
            cs.is_account_muted(account_index),
        ))
        .unwrap_or((false, false, false));
    DockActionStateSnapshot {
        account_index,
        is_pinned,
        is_resizable,
        is_free_position: config.is_free_position,
        is_muted,
        bubble_size_px: bubble_size_px.max(24),
        scale_factor: screen.scale_factor.max(0.5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_registry_order_and_tones() {
        assert_eq!(DOCK_ACTIONS.len(), 6);
        assert_eq!(DOCK_ACTIONS[0], DockActionId::Pin);
        assert_eq!(DOCK_ACTIONS[5], DockActionId::HideBubble);
        assert!(DockActionId::HideBubble.has_extra_separator_before());
        assert!(!DockActionId::Pin.has_extra_separator_before());
        let snap = DockActionStateSnapshot {
            is_pinned: true,
            is_resizable: false,
            is_free_position: true,
            is_muted: true,
            ..Default::default()
        };
        assert_eq!(snap.button_tone(DockActionId::Pin, false), DockButtonTone::ActiveAccent);
        assert_eq!(snap.button_tone(DockActionId::AllowResize, false), DockButtonTone::Neutral);
        assert_eq!(snap.button_tone(DockActionId::PositionMode, false), DockButtonTone::ActiveAccent);
        assert_eq!(snap.button_tone(DockActionId::Mute, false), DockButtonTone::MutedWarning);
        assert_eq!(snap.button_tone(DockActionId::HideBubble, false), DockButtonTone::Neutral);
        assert_eq!(snap.button_tone(DockActionId::HideBubble, true), DockButtonTone::DestructiveHover);
    }
}
