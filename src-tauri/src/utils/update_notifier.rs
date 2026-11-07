use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

/// Initial startup delay before the first background update check (15 seconds)
pub const INITIAL_CHECK_DELAY: Duration = Duration::from_secs(15);
/// Background update check interval (20 minutes)
pub const CHECK_INTERVAL: Duration = Duration::from_secs(20 * 60);
/// Cooldown between repeat notifications for the same uninstalled update (4 hours)
pub const RE_NOTIFY_INTERVAL: Duration = Duration::from_secs(4 * 3600);

/// Starts background daemon checking for application updates
pub fn start_background_update_notifier(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(INITIAL_CHECK_DELAY).await;
        let mut last_notified_ver: Option<String> = None;
        let mut last_notified_at: Option<Instant> = None;
        loop {
            if let Ok(updater) = app.updater() {
                match updater.check().await {
                    Ok(Some(update)) => {
                        let ver = update.version.clone();
                        let should_notify = match &last_notified_ver {
                            Some(prev) if prev == &ver => {
                                last_notified_at.map(|t| t.elapsed() >= RE_NOTIFY_INTERVAL).unwrap_or(true)
                            }
                            _ => true,
                        };
                        if should_notify {
                            let is_notifications_enabled = app
                                .try_state::<Arc<crate::state::ChatState>>()
                                .map(|s| s.get_user_config().is_notifications_enabled)
                                .unwrap_or(true);
                            if is_notifications_enabled {
                                let (title, body) = get_notification_text(&app, &ver);
                                let _ = app.notification().builder().title(title).body(body).show();
                                last_notified_ver = Some(ver);
                                last_notified_at = Some(Instant::now());
                            }
                        }
                    }
                    Ok(None) => {}
                    Err(err) => {
                        eprintln!("[UpdateNotifier] Periodic check skipped: {}", err);
                    }
                }
            }
            tokio::time::sleep(CHECK_INTERVAL).await;
        }
    });
}

fn get_notification_text(app: &AppHandle, version: &str) -> (String, String) {
    let locale = app
        .try_state::<Arc<crate::state::ChatState>>()
        .map(|s| s.get_user_config().current_locale().to_string())
        .unwrap_or_else(|| "en".to_string());
    if locale == "vi" {
        (format!("Bản cập nhật mới v{}", version), format!("DesktopBubbleChat v{} đã sẵn sàng. Mở ứng dụng để cập nhật ngay!", version))
    } else {
        (format!("New update available v{}", version), format!("DesktopBubbleChat v{} is ready. Open the app to update now!", version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_constants() {
        assert_eq!(INITIAL_CHECK_DELAY.as_secs(), 15);
        assert_eq!(CHECK_INTERVAL.as_secs(), 1200);
        assert_eq!(RE_NOTIFY_INTERVAL.as_secs(), 14400);
    }

    #[test]
    fn test_notification_format() {
        let vi_title = format!("Bản cập nhật mới v{}", "0.5.3");
        let en_title = format!("New update available v{}", "0.5.3");
        assert!(vi_title.contains("0.5.3"));
        assert!(en_title.contains("0.5.3"));
    }
}
