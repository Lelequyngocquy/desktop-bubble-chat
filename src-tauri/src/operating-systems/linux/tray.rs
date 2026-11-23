pub struct LinuxMenuManager;

impl LinuxMenuManager {
    /// Checks if D-Bus StatusNotifierItem host is present on desktop
    pub fn is_status_notifier_host_available() -> bool {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("busctl")
                .args(["--user", "status", "org.kde.StatusNotifierWatcher"])
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

    /// Opens or focuses dashboard window on native Linux bubble right click
    pub fn show_bubble_context_menu(app: &tauri::AppHandle, _account_index: u32, _x: f64, _y: f64) {
        let handle = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = crate::presentation::main_window::get_or_create_main_window(&handle);
        });
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
