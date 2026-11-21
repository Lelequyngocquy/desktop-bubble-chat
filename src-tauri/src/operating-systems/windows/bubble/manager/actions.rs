use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use super::{
    reassert_topmost_for_bubbles, BubbleBoundsCache, GetForegroundWindow, KillTimer,
    NativeBubbleAction, NativeBubbleWindow, TIMER_ID_SETTLE_TOPMOST,
};

/// Animates target bubble and any displaced siblings during edge tuck or untuck transition
fn apply_hover_tuck_transition(
    bubbles: &HashMap<u32, NativeBubbleWindow>,
    bounds_cache: &BubbleBoundsCache,
    account_index: u32,
    app: &AppHandle,
    is_hovered: bool,
) {
    let config = crate::movement::get_bubble_physics_config(app);
    if config.is_free_position {
        return;
    }
    let Some(win) = bubbles.get(&account_index) else {
        return;
    };
    let screen = crate::movement::get_target_dashboard_screen(Some(app));
    let (cur_x, cur_y, _, _) = win.get_bounds();
    let target_pos = crate::movement::BubbleMovementEngine::calculate_hover_pos(
        &screen,
        &config,
        cur_x,
        cur_y,
        is_hovered,
    );
    let mid_x = screen.x + screen.width / 2;
    let is_left_edge = cur_x + (config.size * screen.scale_factor) as i32 / 2 < mid_x;
    let all_bubbles: Vec<crate::movement::BubbleRect> = if let Ok(cache) = bounds_cache.read() {
        cache
            .iter()
            .map(|(&idx, &(x, y, w, h))| crate::movement::BubbleRect {
                account_index: idx,
                x,
                y,
                width: w as i32,
                height: h as i32,
            })
            .collect()
    } else {
        Vec::new()
    };
    let slots = crate::movement::CollisionResolver::resolve_mutual_column_layout(
        account_index,
        cur_y,
        is_left_edge,
        &screen,
        &config,
        &all_bubbles,
    );
    let resolved_y = slots
        .iter()
        .find(|s| s.account_index == account_index)
        .map(|s| s.target_y)
        .unwrap_or(cur_y);
    win.animate_to(target_pos.x, resolved_y, 180);
    if let Ok(mut cache) = bounds_cache.write()
        && let Some(b) = cache.get_mut(&account_index) {
            b.0 = target_pos.x;
            b.1 = resolved_y;
        }
    for slot in &slots {
        if slot.account_index == account_index {
            continue;
        }
        let sibling_x = all_bubbles
            .iter()
            .find(|b| b.account_index == slot.account_index)
            .map(|b| b.x)
            .unwrap_or(target_pos.x);
        if let Some(sibling_win) = bubbles.get(&slot.account_index) {
            sibling_win.animate_to(sibling_x, slot.target_y, 180);
        }
        if let Ok(mut cache) = bounds_cache.write()
            && let Some(b) = cache.get_mut(&slot.account_index) {
                b.0 = sibling_x;
                b.1 = slot.target_y;
            }
    }
}

/// Processes a single NativeBubbleAction on the dedicated Win32 UI worker thread
pub(super) fn handle_bubble_action(
    action: NativeBubbleAction,
    bubbles: &mut HashMap<u32, NativeBubbleWindow>,
    bounds_cache: &BubbleBoundsCache,
    is_always_on_top: &mut bool,
    settle_timer_active: &mut bool,
    should_shutdown: &mut bool,
    msg_hwnd: *mut std::ffi::c_void,
) {
    match action {
        NativeBubbleAction::Show { account_index, x, y, size, app } => {
            if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
                *is_always_on_top = chat_state.is_always_on_top.load(std::sync::atomic::Ordering::SeqCst);
            }
            if let Some(win) = bubbles.get_mut(&account_index) {
                win.set_position(x, y);
                win.resize(size);
                win.show();
                if let Ok(mut cache) = bounds_cache.write() {
                    cache.insert(account_index, (x, y, size, size));
                }
            } else {
                match NativeBubbleWindow::new(Some(app), account_index, x, y, size) {
                    Ok(win) => {
                        win.show();
                        bubbles.insert(account_index, win);
                        if let Ok(mut cache) = bounds_cache.write() {
                            cache.insert(account_index, (x, y, size, size));
                        }
                    }
                    Err(e) => {
                        eprintln!("[NativeBubble ERROR] Failed to create window for account {}: {}", account_index, e);
                    }
                }
            }
        }
        NativeBubbleAction::Hide { account_index } => {
            if let Some(win) = bubbles.remove(&account_index) {
                win.hide();
            }
            if let Ok(mut cache) = bounds_cache.write() {
                cache.remove(&account_index);
            }
        }
        NativeBubbleAction::SetPosition { account_index, x, y } => {
            if let Some(win) = bubbles.get(&account_index) {
                win.set_position(x, y);
                if let Ok(mut cache) = bounds_cache.write()
                    && let Some(b) = cache.get_mut(&account_index) {
                        b.0 = x;
                        b.1 = y;
                    }
            }
        }
        NativeBubbleAction::AnimateTo { account_index, target_x, target_y, duration_ms } => {
            if let Some(win) = bubbles.get(&account_index) {
                win.animate_to(target_x, target_y, duration_ms);
                if let Ok(mut cache) = bounds_cache.write()
                    && let Some(b) = cache.get_mut(&account_index) {
                        b.0 = target_x;
                        b.1 = target_y;
                    }
            }
        }
        NativeBubbleAction::SetUnread { account_index, count } => {
            if let Some(win) = bubbles.get(&account_index) {
                win.set_unread_count(count);
            }
        }
        NativeBubbleAction::TriggerPulse { account_index } => {
            if let Some(win) = bubbles.get(&account_index) {
                win.trigger_pulse();
            }
        }
        NativeBubbleAction::Tuck { account_index, app } => {
            crate::utils::trim_controller::schedule_auto_trim_on_tuck(account_index, app.clone());
            apply_hover_tuck_transition(bubbles, bounds_cache, account_index, &app, false);
        }
        NativeBubbleAction::Untuck { account_index, app } => {
            crate::utils::trim_controller::cancel_auto_trim(account_index);
            crate::utils::trim_controller::record_user_interaction(account_index);
            crate::platforms::messenger::windows::set_chat_window_memory_target(&app, account_index, false);
            apply_hover_tuck_transition(bubbles, bounds_cache, account_index, &app, true);
        }
        NativeBubbleAction::SetAlwaysOnTop { enabled } => {
            *is_always_on_top = enabled;
            if !enabled && *settle_timer_active {
                unsafe { KillTimer(msg_hwnd, TIMER_ID_SETTLE_TOPMOST); }
                *settle_timer_active = false;
            }
            for win in bubbles.values() {
                win.set_always_on_top(enabled);
            }
        }
        NativeBubbleAction::AssertTopmost => {
            if *is_always_on_top && !bubbles.is_empty() {
                let fg_hwnd = unsafe { GetForegroundWindow() };
                reassert_topmost_for_bubbles(bubbles, fg_hwnd);
            }
        }
        NativeBubbleAction::SetSize { account_index, size } => {
            if let Some(win) = bubbles.get_mut(&account_index) {
                win.resize(size);
            }
            if let Ok(mut cache) = bounds_cache.write()
                && let Some(b) = cache.get_mut(&account_index) {
                    b.2 = size;
                    b.3 = size;
                }
        }
        NativeBubbleAction::Shutdown => {
            bubbles.clear();
            if let Ok(mut cache) = bounds_cache.write() {
                cache.clear();
            }
            *should_shutdown = true;
        }
    }
}
