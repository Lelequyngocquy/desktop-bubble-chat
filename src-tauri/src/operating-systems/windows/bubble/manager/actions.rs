use std::sync::Arc;
use tauri::Manager;

use super::super::dock::NativeActionDockWindow;
use super::{
    reassert_topmost_for_bubbles, BubbleWorkerContext, GetForegroundWindow, KillTimer,
    NativeBubbleAction, NativeBubbleWindow, TIMER_ID_SETTLE_TOPMOST,
};

/// Processes a single NativeBubbleAction on the dedicated Win32 UI worker thread
pub(super) fn handle_bubble_action(
    action: NativeBubbleAction,
    ctx: &mut BubbleWorkerContext,
) {
    match action {
        NativeBubbleAction::Show { account_index, x, y, size, app } => {
            if let Some(chat_state) = app.try_state::<Arc<crate::state::ChatState>>() {
                ctx.is_always_on_top = chat_state.is_always_on_top.load(std::sync::atomic::Ordering::SeqCst);
            }
            if let Some(win) = ctx.bubbles.get_mut(&account_index) {
                win.set_position(x, y);
                win.resize(size);
                win.show();
                if let Ok(mut cache) = ctx.bounds_cache.write() {
                    cache.insert(account_index, (x, y, size, size));
                }
            } else {
                match NativeBubbleWindow::new(Some(app), account_index, x, y, size) {
                    Ok(win) => {
                        win.show();
                        ctx.bubbles.insert(account_index, win);
                        if let Ok(mut cache) = ctx.bounds_cache.write() {
                            cache.insert(account_index, (x, y, size, size));
                        }
                    }
                    Err(e) => {
                        eprintln!("[NativeBubble ERROR] Failed to create window for account {}: {}", account_index, e);
                        if let Ok(mut cache) = ctx.bounds_cache.write() {
                            cache.remove(&account_index);
                        }
                    }
                }
            }
        }
        NativeBubbleAction::Hide { account_index } => {
            if let Some(win) = ctx.bubbles.remove(&account_index) {
                win.hide();
            }
            if let Ok(mut cache) = ctx.bounds_cache.write() {
                cache.remove(&account_index);
            }
        }
        NativeBubbleAction::SetPosition { account_index, x, y } => {
            if let Some(win) = ctx.bubbles.get(&account_index) {
                win.set_position(x, y);
                if let Ok(mut cache) = ctx.bounds_cache.write()
                    && let Some(b) = cache.get_mut(&account_index) {
                        b.0 = x;
                        b.1 = y;
                    }
            }
        }
        NativeBubbleAction::AnimateTo { account_index, target_x, target_y, duration_ms } => {
            if let Some(win) = ctx.bubbles.get(&account_index) {
                win.animate_to(target_x, target_y, duration_ms);
                if let Ok(mut cache) = ctx.bounds_cache.write()
                    && let Some(b) = cache.get_mut(&account_index) {
                        b.0 = target_x;
                        b.1 = target_y;
                    }
            }
        }
        NativeBubbleAction::SetUnread { account_index, count } => {
            if let Some(win) = ctx.bubbles.get(&account_index) {
                win.set_unread_count(count);
            }
        }
        NativeBubbleAction::SetAlertState { account_index, unread_count, alert_active } => {
            if let Some(win) = ctx.bubbles.get(&account_index) {
                win.set_alert_state(unread_count, alert_active);
            }
        }
        NativeBubbleAction::SetChatOpen { account_index, is_open } => {
            if let Some(win) = ctx.bubbles.get(&account_index) {
                win.set_chat_open(is_open);
            }
        }
        NativeBubbleAction::TriggerPulse { account_index } => {
            if let Some(win) = ctx.bubbles.get(&account_index) {
                win.trigger_pulse();
            }
        }
        NativeBubbleAction::Tuck { account_index, .. } => {
            if let Some(win) = ctx.bubbles.get(&account_index) {
                win.tuck();
            }
        }
        NativeBubbleAction::Untuck { account_index, mode, .. } => {
            if let Some(win) = ctx.bubbles.get(&account_index) {
                win.untuck(mode);
            }
        }
        NativeBubbleAction::SetAlwaysOnTop { enabled } => {
            ctx.is_always_on_top = enabled;
            if !enabled && ctx.settle_timer_active {
                unsafe { KillTimer(ctx.msg_hwnd, TIMER_ID_SETTLE_TOPMOST); }
                ctx.settle_timer_active = false;
            }
            for win in ctx.bubbles.values() {
                win.set_always_on_top(enabled);
            }
            if let Some(ref dock) = ctx.action_dock {
                dock.set_always_on_top(enabled);
            }
        }
        NativeBubbleAction::AssertTopmost => {
            if ctx.is_always_on_top && !ctx.bubbles.is_empty() {
                let fg_hwnd = unsafe { GetForegroundWindow() };
                reassert_topmost_for_bubbles(&ctx.bubbles, ctx.action_dock.as_ref(), fg_hwnd);
            }
        }
        NativeBubbleAction::SetSize { account_index, size } => {
            if let Some(win) = ctx.bubbles.get_mut(&account_index) {
                win.resize(size);
            }
            if let Ok(mut cache) = ctx.bounds_cache.write()
                && let Some(b) = cache.get_mut(&account_index) {
                    b.2 = size;
                    b.3 = size;
                }
        }
        NativeBubbleAction::ShowActionDock { account_index, layout, snapshot, app } => {
            if ctx.action_dock.is_none() {
                match NativeActionDockWindow::new(ctx.is_always_on_top, Arc::clone(&ctx.dock_hover_corridor)) {
                    Ok(dock) => {
                        ctx.action_dock = Some(dock);
                    }
                    Err(err) => {
                        eprintln!("[ActionDock ERROR] Failed to create singleton overlay: {}", err);
                    }
                }
            }
            if let Some(ref dock) = ctx.action_dock {
                dock.show_for_bubble(account_index, layout, snapshot, app);
            }
        }
        NativeBubbleAction::SyncActionDock { account_index, snapshot, layout } => {
            if let Some(ref dock) = ctx.action_dock {
                dock.sync_state(account_index, snapshot, layout);
            }
        }
        NativeBubbleAction::HideActionDock { immediate } => {
            if let Some(ref dock) = ctx.action_dock {
                dock.hide(immediate);
            }
        }
        NativeBubbleAction::Shutdown => {
            ctx.action_dock = None;
            if let Ok(mut corridor) = ctx.dock_hover_corridor.write() {
                *corridor = None;
            }
            ctx.bubbles.clear();
            if let Ok(mut cache) = ctx.bounds_cache.write() {
                cache.clear();
            }
            ctx.should_shutdown = true;
        }
    }
}
