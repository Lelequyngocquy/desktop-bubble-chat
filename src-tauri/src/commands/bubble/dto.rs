use std::sync::Arc;
use crate::state::ChatState;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapCoordsDto {
    pub target_x: i32,
    pub target_y: i32,
    pub should_dismiss: bool,
    pub is_left_edge: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BubblePositionDto {
    pub account_index: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_visible: bool,
}

/// RAII Guard ensuring transition_lock is released upon completion
pub struct TransitionLockGuard {
    pub state: Option<Arc<ChatState>>,
    pub account_index: u32,
}

impl TransitionLockGuard {
    pub fn new(state: Option<Arc<ChatState>>, account_index: u32) -> Self {
        Self { state, account_index }
    }
}

impl Drop for TransitionLockGuard {
    fn drop(&mut self) {
        if let Some(state) = &self.state {
            state.release_transition_lock(self.account_index);
        }
    }
}
