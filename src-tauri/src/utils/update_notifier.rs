use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

/// Initial startup delay before the first background update check (5 seconds)
pub const INITIAL_CHECK_DELAY: Duration = Duration::from_secs(5);
/// Background update check interval (20 minutes)
pub const CHECK_INTERVAL: Duration = Duration::from_secs(20 * 60);
/// Cooldown between repeat notifications for the same uninstalled update (4 hours)
pub const RE_NOTIFY_INTERVAL: Duration = Duration::from_secs(4 * 3600);
/// Global guard preventing concurrent updater executions
static IS_UPDATING: AtomicBool = AtomicBool::new(false);

/// Cleans up stale installer and updater temporary files from OS temp directory
pub fn cleanup_stale_update_artifacts() {
    let temp_dir = std::env::temp_dir();
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let lower = name.to_lowercase();
                if (lower.starts_with("desktopbubblechat") || lower.starts_with("desktop-bubble-chat"))
                    && (lower.contains("updater") || lower.ends_with(".exe") || lower.ends_with(".msi"))
                {
                    let _ = if path.is_dir() {
                        std::fs::remove_dir_all(&path)
                    } else {
                        std::fs::remove_file(&path)
                    };
                }
            }
        }
    }
}

/// Performs update check, downloads and installs if auto-update enabled, or notifies user
pub async fn check_and_apply_update(
    app: &AppHandle,
    last_notified_ver: &mut Option<String>,
    last_notified_at: &mut Option<Instant>,
) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(err) => {
            eprintln!("[UpdateNotifier] Updater instance unavailable: {}", err);
            return;
        }
    };
    match updater.check().await {
        Ok(Some(update)) => {
            let ver = update.version.clone();
            let user_config = app
                .try_state::<Arc<crate::state::ChatState>>()
                .map(|s| s.get_user_config())
                .unwrap_or_default();
            if user_config.is_auto_update_enabled() {
                if IS_UPDATING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
                    println!("[UpdateNotifier] Update download already in progress, skipping.");
                    return;
                }
                println!("[UpdateNotifier] Auto-update active: Found v{}. Downloading...", ver);
                cleanup_stale_update_artifacts();
                let _ = app.emit("updater-status", serde_json::json!({ "status": "downloading", "version": &ver }));
                if user_config.is_notifications_enabled {
                    let (title, body) = get_auto_update_notification_text(app, &ver);
                    let _ = app.notification().builder().title(title).body(body).show();
                }
                let install_res = update.download_and_install(|_chunk, _total| {}, || {}).await;
                IS_UPDATING.store(false, Ordering::SeqCst);
                match install_res {
                    Ok(_) => {
                        let _ = app.emit("updater-status", serde_json::json!({ "status": "installing", "version": &ver }));
                        println!("[UpdateNotifier] Update v{} installed, application will restart.", ver);
                    }
                    Err(err) => {
                        let err_msg = err.to_string();
                        let _ = app.emit("updater-status", serde_json::json!({ "status": "error", "message": &err_msg }));
                        eprintln!("[UpdateNotifier] Failed to auto-install v{}: {}", ver, err);
                    }
                }
            } else {
                let should_notify = match last_notified_ver.as_ref() {
                    Some(prev) if prev == &ver => {
                        last_notified_at.map(|t| t.elapsed() >= RE_NOTIFY_INTERVAL).unwrap_or(true)
                    }
                    _ => true,
                };
                if should_notify && user_config.is_notifications_enabled {
                    let (title, body) = get_notification_text(app, &ver);
                    let _ = app.notification().builder().title(title).body(body).show();
                    *last_notified_ver = Some(ver.clone());
                    *last_notified_at = Some(Instant::now());
                }
                let _ = app.emit("updater-status", serde_json::json!({ "status": "available", "version": &ver }));
            }
        }
        Ok(None) => {
            let _ = app.emit("updater-status", serde_json::json!({ "status": "up-to-date" }));
        }
        Err(err) => {
            eprintln!("[UpdateNotifier] Periodic check skipped: {}", err);
        }
    }
}

/// Starts background daemon checking for application updates
pub fn start_background_update_notifier(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(INITIAL_CHECK_DELAY).await;
        let mut last_notified_ver: Option<String> = None;
        let mut last_notified_at: Option<Instant> = None;
        loop {
            check_and_apply_update(&app, &mut last_notified_ver, &mut last_notified_at).await;
            tokio::time::sleep(CHECK_INTERVAL).await;
        }
    });
}

fn get_auto_update_notification_text(app: &AppHandle, version: &str) -> (String, String) {
    let locale = app
        .try_state::<Arc<crate::state::ChatState>>()
        .map(|s| s.get_user_config().current_locale().to_string())
        .unwrap_or_else(|| "en".to_string());
    if locale == "vi" {
        (format!("Đang cập nhật v{}", version), format!("Phát hiện phiên bản mới v{}. Đang tự động tải và cài đặt...", version))
    } else {
        (format!("Updating to v{}", version), format!("New version v{} detected. Automatically downloading and installing...", version))
    }
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
        assert_eq!(INITIAL_CHECK_DELAY.as_secs(), 5);
        assert_eq!(CHECK_INTERVAL.as_secs(), 1200);
        assert_eq!(RE_NOTIFY_INTERVAL.as_secs(), 14400);
    }

    #[test]
    fn test_notification_format() {
        let vi_title = format!("Bản cập nhật mới v{}", "1.1.4");
        let en_title = format!("New update available v{}", "1.1.4");
        assert!(vi_title.contains("1.1.4"));
        assert!(en_title.contains("1.1.4"));
    }

    #[test]
    fn test_cleanup_stale_update_artifacts() {
        let temp_dir = std::env::temp_dir();
        let dummy_file = temp_dir.join("desktopbubblechat-test-updater-dummy.exe");
        let _ = std::fs::write(&dummy_file, b"dummy content");
        assert!(dummy_file.exists());
        cleanup_stale_update_artifacts();
        assert!(!dummy_file.exists());
    }
}
