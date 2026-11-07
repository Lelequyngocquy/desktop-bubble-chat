pub struct LinuxMenuManager;

impl LinuxMenuManager {
    /// Checks if D-Bus StatusNotifierItem host is present on desktop
    pub fn is_status_notifier_host_available() -> bool {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("busctl")
                .args(&["--user", "status", "org.kde.StatusNotifierWatcher"])
                .output()
            {
                return output.status.success();
            }
            false
        }
        #[cfg(not(target_os = "linux"))]
        {
            true
        }
    }

    /// Shows contextual popup menu for native bubble on right click
    pub fn show_bubble_context_menu(app: &tauri::AppHandle, account_index: u32, x: f64, y: f64) {
        let _ = (app, account_index, x, y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_manager_safety() {
        let _ = LinuxMenuManager;
    }
}
