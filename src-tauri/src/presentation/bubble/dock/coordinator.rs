use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{AppHandle, Manager};

use crate::movement::{
    get_bubble_physics_config, get_target_dashboard_screen, BubbleMovementEngine,
    BubblePhysicsConfig, BubbleRect, ScreenMetrics,
};
use crate::state::ChatState;
use super::layout::{compute_dock_placement_plan, SiblingPushOffset};
use super::model::snapshot_from_app;

/// Partitioned sibling moves produced when applying a dock placement plan
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SiblingPlanDiff {
    pub to_restore: Vec<(u32, i32)>,
    pub to_push: Vec<(u32, i32)>,
}

/// Tracks reversible Y displacements applied to sibling bubbles while ActionDock is open
#[derive(Debug, Default)]
pub struct DockSiblingOffsetTracker {
    active_account: Option<u32>,
    pushed_original_y: HashMap<u32, i32>,
}

impl DockSiblingOffsetTracker {
    /// Returns the account currently owning the visible ActionDock, if any
    pub fn active_account(&self) -> Option<u32> {
        self.active_account
    }

    /// Replaces pushed sibling Y coordinates with their saved baseline Y before layout computation
    pub fn normalize_baseline_bubbles(&self, live_bubbles: &[BubbleRect]) -> Vec<BubbleRect> {
        live_bubbles
            .iter()
            .map(|b| {
                let mut copy = *b;
                if let Some(&orig_y) = self.pushed_original_y.get(&b.account_index) {
                    copy.y = orig_y;
                }
                copy
            })
            .collect()
    }

    /// Commits a new sibling push plan and returns siblings to restore and push
    pub fn apply_plan(
        &mut self,
        active_account: u32,
        new_pushes: &[SiblingPushOffset],
    ) -> SiblingPlanDiff {
        let mut new_targets: HashMap<u32, i32> = HashMap::with_capacity(new_pushes.len());
        let mut next_originals: HashMap<u32, i32> = HashMap::with_capacity(new_pushes.len());
        let mut to_push: Vec<(u32, i32)> = Vec::with_capacity(new_pushes.len());
        for push in new_pushes {
            let baseline_y = self
                .pushed_original_y
                .get(&push.account_index)
                .copied()
                .unwrap_or(push.original_y);
            new_targets.insert(push.account_index, push.target_y);
            next_originals.insert(push.account_index, baseline_y);
            to_push.push((push.account_index, push.target_y));
        }
        let mut to_restore: Vec<(u32, i32)> = Vec::new();
        for (&sib_idx, &orig_y) in &self.pushed_original_y {
            if !new_targets.contains_key(&sib_idx) {
                to_restore.push((sib_idx, orig_y));
            }
        }
        self.active_account = Some(active_account);
        self.pushed_original_y = next_originals;
        SiblingPlanDiff { to_restore, to_push }
    }

    /// Clears dock ownership if expected account matches and returns siblings to restore
    pub fn clear_for_close(&mut self, expected_account: Option<u32>) -> Option<Vec<(u32, i32)>> {
        if let Some(expected) = expected_account
            && self.active_account != Some(expected) {
                return None;
            }
        if self.active_account.is_none() && self.pushed_original_y.is_empty() {
            return None;
        }
        self.active_account = None;
        let restores: Vec<(u32, i32)> = self.pushed_original_y.drain().collect();
        Some(restores)
    }

    /// Stops tracking a sibling bubble that was manually dragged or hidden by the user
    pub fn forget_sibling(&mut self, account_index: u32) -> bool {
        self.pushed_original_y.remove(&account_index).is_some()
    }

    /// Drains all tracked sibling baseline positions and clears active dock ownership
    pub fn take_all_restores(&mut self) -> Vec<(u32, i32)> {
        self.active_account = None;
        self.pushed_original_y.drain().collect()
    }
}

static DOCK_TRACKER: OnceLock<Mutex<DockSiblingOffsetTracker>> = OnceLock::new();

fn dock_tracker() -> &'static Mutex<DockSiblingOffsetTracker> {
    DOCK_TRACKER.get_or_init(|| Mutex::new(DockSiblingOffsetTracker::default()))
}

/// Resolves target X coordinate for a sibling bubble respecting free vs edge-tuck mode
fn resolve_sibling_target_x(
    app: &AppHandle,
    sib_idx: u32,
    current_x: i32,
    screen: &ScreenMetrics,
    config: &BubblePhysicsConfig,
) -> i32 {
    if config.is_free_position {
        current_x
    } else {
        let stay_untucked = crate::presentation::bubble::is_bubble_stay_untucked(app, sib_idx);
        let ratio = BubbleMovementEngine::effective_edge_ratio(config, stay_untucked);
        BubbleMovementEngine::calculate_edge_x(screen, config, current_x, ratio)
    }
}

/// Animates a sibling bubble vertically while keeping bounds cache and ChatState synchronized
fn move_sibling_vertical(
    app: &AppHandle,
    sib_idx: u32,
    target_y: i32,
    screen: &ScreenMetrics,
    config: &BubblePhysicsConfig,
) {
    let is_suspended = app
        .try_state::<Arc<ChatState>>()
        .is_some_and(|cs| cs.is_account_suspended(sib_idx));
    if is_suspended {
        return;
    }
    if crate::operating_systems::current::bubble::has_native_bubble(sib_idx)
        && let Some((cur_x, cur_y, bw, bh)) = crate::operating_systems::current::bubble::NativeBubbleManager::global().get_bounds(sib_idx) {
            let target_x = resolve_sibling_target_x(app, sib_idx, cur_x, screen, config);
            if cur_x == target_x && cur_y == target_y {
                return;
            }
            crate::operating_systems::current::bubble::NativeBubbleManager::global().dispatch(
                crate::operating_systems::current::bubble::NativeBubbleAction::AnimateTo {
                    account_index: sib_idx,
                    target_x,
                    target_y,
                    duration_ms: 160,
                },
            );
            crate::operating_systems::current::bubble::update_cached_bubble_bounds(sib_idx, target_x, target_y, bw, bh);
            if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                cs.set_bubble_rect(sib_idx, BubbleRect { account_index: sib_idx, x: target_x, y: target_y, width: bw as i32, height: bh as i32 });
            }
        }
}

/// Opens or repositions ActionDock for an account bubble and applies reversible sibling pushes
pub fn open_action_dock_for_account(
    app: &AppHandle,
    account_index: u32,
    bubble_x: i32,
    bubble_y: i32,
    bubble_size_px: u32,
) {
    if !crate::operating_systems::current::bubble::has_native_bubble(account_index) {
        return;
    }
    let config = get_bubble_physics_config(app);
    let screen = get_target_dashboard_screen(Some(app));
    let live_bubbles = crate::presentation::bubble::get_active_bubble_rects(app, None);
    let baseline_bubbles = dock_tracker()
        .lock()
        .map(|tracker| tracker.normalize_baseline_bubbles(&live_bubbles))
        .unwrap_or(live_bubbles);
    let plan = compute_dock_placement_plan(
        account_index,
        bubble_x,
        bubble_y,
        bubble_size_px,
        &screen,
        &config,
        &baseline_bubbles,
    );
    let diff = dock_tracker()
        .lock()
        .map(|mut tracker| tracker.apply_plan(account_index, &plan.sibling_pushes))
        .unwrap_or_default();
    for (sib_idx, orig_y) in diff.to_restore {
        move_sibling_vertical(app, sib_idx, orig_y, &screen, &config);
    }
    for (sib_idx, target_y) in diff.to_push {
        move_sibling_vertical(app, sib_idx, target_y, &screen, &config);
    }
    let snapshot = snapshot_from_app(app, account_index, bubble_size_px);
    crate::operating_systems::current::bubble::show_native_action_dock(app, account_index, plan.layout, snapshot);
}

/// Closes ActionDock for a specific account (or any owner if None) and restores pushed siblings
pub fn close_action_dock_for_account(app: &AppHandle, account_index: u32, immediate: bool) {
    let restores_opt = dock_tracker()
        .lock()
        .ok()
        .and_then(|mut tracker| tracker.clear_for_close(Some(account_index)));
    if let Some(restores) = restores_opt {
        crate::operating_systems::current::bubble::hide_native_action_dock(immediate);
        if !restores.is_empty() {
            let config = get_bubble_physics_config(app);
            let screen = get_target_dashboard_screen(Some(app));
            for (sib_idx, orig_y) in restores {
                move_sibling_vertical(app, sib_idx, orig_y, &screen, &config);
            }
        }
    }
}

/// Syncs SSOT toggle state to the ActionDock if the given account currently owns it
pub fn sync_action_dock_for_account(app: &AppHandle, account_index: u32) {
    let is_owner = dock_tracker()
        .lock()
        .ok()
        .is_some_and(|tracker| tracker.active_account() == Some(account_index));
    if is_owner {
        crate::operating_systems::current::bubble::sync_native_action_dock_state(app, account_index, None);
    }
}

/// Syncs SSOT toggle state to whichever account currently owns the visible ActionDock
pub fn sync_active_action_dock(app: &AppHandle) {
    let owner = dock_tracker().lock().ok().and_then(|tracker| tracker.active_account());
    if let Some(account_index) = owner {
        crate::operating_systems::current::bubble::sync_native_action_dock_state(app, account_index, None);
    }
}

/// Recomputes dock layout and sibling push plan for the active open dock after bubble resize
pub fn reanchor_active_action_dock_if_open(app: &AppHandle) {
    let owner = dock_tracker().lock().ok().and_then(|tracker| tracker.active_account());
    let Some(account_index) = owner else {
        return;
    };
    let chat_label = crate::platforms::messenger::windows::get_messenger_window_label(account_index);
    let in_auth = app
        .try_state::<Arc<ChatState>>()
        .is_some_and(|cs| cs.is_window_in_auth_mode(&chat_label));
    if !in_auth
        && let Some(chat_win) = app.get_webview_window(&chat_label)
        && chat_win.is_visible().unwrap_or(false) {
            let (ax, ay, bsz) = crate::platforms::messenger::services::ChatService::dock_chat_window(app, &chat_win, account_index);
            open_action_dock_for_account(app, account_index, ax, ay, bsz);
        }
}

/// Restores saved baseline sibling Y coordinates into caches right before a global edge snap
pub fn restore_siblings_before_global_snap(app: &AppHandle) {
    let restores = dock_tracker()
        .lock()
        .map(|mut tracker| tracker.take_all_restores())
        .unwrap_or_default();
    for (sib_idx, orig_y) in restores {
        let is_suspended = app
            .try_state::<Arc<ChatState>>()
            .is_some_and(|cs| cs.is_account_suspended(sib_idx));
        if is_suspended {
            continue;
        }
        if crate::operating_systems::current::bubble::has_native_bubble(sib_idx)
            && let Some((cur_x, _, bw, bh)) = crate::operating_systems::current::bubble::NativeBubbleManager::global().get_bounds(sib_idx) {
                crate::operating_systems::current::bubble::update_cached_bubble_bounds(sib_idx, cur_x, orig_y, bw, bh);
                if let Some(cs) = app.try_state::<Arc<ChatState>>() {
                    cs.set_bubble_rect(sib_idx, BubbleRect { account_index: sib_idx, x: cur_x, y: orig_y, width: bw as i32, height: bh as i32 });
                }
            }
    }
}

/// Handles drag start: hides dock immediately for active bubble or forgets dragged sibling offset
pub fn on_bubble_drag_started(app: &AppHandle, account_index: u32) {
    let (is_owner, restores) = dock_tracker()
        .lock()
        .map(|mut tracker| {
            if tracker.active_account() == Some(account_index) {
                (true, tracker.clear_for_close(Some(account_index)).unwrap_or_default())
            } else {
                tracker.forget_sibling(account_index);
                (false, Vec::new())
            }
        })
        .unwrap_or((false, Vec::new()));
    if is_owner {
        crate::operating_systems::current::bubble::hide_native_action_dock(true);
        if !restores.is_empty() {
            let config = get_bubble_physics_config(app);
            let screen = get_target_dashboard_screen(Some(app));
            for (sib_idx, orig_y) in restores {
                move_sibling_vertical(app, sib_idx, orig_y, &screen, &config);
            }
        }
    }
}

/// Handles drag drop: re-docks open chat and ActionDock if dragged bubble still has visible chat
pub fn on_bubble_drag_finished(app: &AppHandle, account_index: u32) {
    let chat_label = crate::platforms::messenger::windows::get_messenger_window_label(account_index);
    let in_auth = app
        .try_state::<Arc<ChatState>>()
        .is_some_and(|cs| cs.is_window_in_auth_mode(&chat_label));
    if !in_auth
        && let Some(chat_win) = app.get_webview_window(&chat_label)
        && chat_win.is_visible().unwrap_or(false) {
            let (ax, ay, bsz) = crate::platforms::messenger::services::ChatService::dock_chat_window(app, &chat_win, account_index);
            open_action_dock_for_account(app, account_index, ax, ay, bsz);
        }
}

/// Handles bubble suspension: closes dock immediately if owner or drops sibling tracking
pub fn on_account_suspended(app: &AppHandle, account_index: u32) {
    on_bubble_drag_started(app, account_index);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sibling_tracker_push_reanchor_without_compounding_and_restore() {
        let mut tracker = DockSiblingOffsetTracker::default();
        let initial_pushes = vec![
            SiblingPushOffset { account_index: 2, original_y: 168, target_y: 390 },
            SiblingPushOffset { account_index: 3, original_y: 236, target_y: 458 },
        ];
        let diff1 = tracker.apply_plan(1, &initial_pushes);
        assert!(diff1.to_restore.is_empty());
        assert_eq!(diff1.to_push, vec![(2, 390), (3, 458)]);
        assert_eq!(tracker.active_account(), Some(1));
        let live = vec![
            BubbleRect { account_index: 1, x: 0, y: 100, width: 56, height: 56 },
            BubbleRect { account_index: 2, x: -42, y: 390, width: 56, height: 56 },
            BubbleRect { account_index: 3, x: -42, y: 458, width: 56, height: 56 },
        ];
        let normalized = tracker.normalize_baseline_bubbles(&live);
        assert_eq!(normalized[1].y, 168);
        assert_eq!(normalized[2].y, 236);
        let resize_pushes = vec![SiblingPushOffset { account_index: 2, original_y: 168, target_y: 410 }];
        let diff2 = tracker.apply_plan(1, &resize_pushes);
        assert_eq!(diff2.to_restore, vec![(3, 236)]);
        assert_eq!(diff2.to_push, vec![(2, 410)]);
        assert!(tracker.clear_for_close(Some(2)).is_none());
        let final_restores = tracker.clear_for_close(Some(1)).expect("account 1 owns dock");
        assert_eq!(final_restores, vec![(2, 168)]);
        assert_eq!(tracker.active_account(), None);
    }

    #[test]
    fn test_sibling_tracker_forget_dragged_sibling_prevents_stale_restore() {
        let mut tracker = DockSiblingOffsetTracker::default();
        let pushes = vec![
            SiblingPushOffset { account_index: 2, original_y: 168, target_y: 390 },
            SiblingPushOffset { account_index: 3, original_y: 236, target_y: 458 },
        ];
        let _ = tracker.apply_plan(1, &pushes);
        assert!(tracker.forget_sibling(2));
        assert!(!tracker.forget_sibling(2));
        let restores = tracker.clear_for_close(Some(1)).expect("active owner closes");
        assert_eq!(restores, vec![(3, 236)]);
    }
}
