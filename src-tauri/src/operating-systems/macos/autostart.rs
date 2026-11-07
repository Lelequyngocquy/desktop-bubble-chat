use std::path::PathBuf;

pub struct MacOSAutostartManager;

impl MacOSAutostartManager {
    /// Resolves path to LaunchAgent plist for desktop-bubble-chat
    pub fn get_plist_path() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/LaunchAgents/com.desktopbubblechat.app.plist"))
    }

    /// Generates standard Apple Launchd property list XML string
    pub fn generate_launch_agent_plist(exe_path: &str) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
            <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
            <plist version=\"1.0\">\n\
            <dict>\n\
                <key>Label</key>\n\
                <string>com.desktopbubblechat.app</string>\n\
                <key>ProgramArguments</key>\n\
                <array>\n\
                    <string>{}</string>\n\
                </array>\n\
                <key>RunAtLoad</key>\n\
                <true/>\n\
                <key>KeepAlive</key>\n\
                <false/>\n\
            </dict>\n\
            </plist>\n",
            exe_path
        )
    }

    /// Retrieves user display name via id -F or dscl RealName
    pub fn get_windows_display_name(username: &str) -> Option<String> {
        if let Ok(output) = std::process::Command::new("id").arg("-F").output() {
            if output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
        if let Ok(dscl_output) = std::process::Command::new("dscl").args(&[".", "-read", &format!("/Users/{}", username), "RealName"]).output() {
            if dscl_output.status.success() {
                let raw = String::from_utf8_lossy(&dscl_output.stdout);
                let cleaned = raw.replace("RealName:", "").trim().to_string();
                if !cleaned.is_empty() {
                    return Some(cleaned);
                }
            }
        }
        None
    }

    /// Toggles LaunchAgent plist file and registers with launchctl
    pub fn toggle_autostart_registry(enabled: bool) -> Result<bool, String> {
        let plist_path = Self::get_plist_path().ok_or_else(|| "Failed to resolve LaunchAgents directory".to_string())?;
        if enabled {
            let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = current_exe.to_string_lossy();
            if let Some(parent) = plist_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let plist_content = Self::generate_launch_agent_plist(&exe_str);
            std::fs::write(&plist_path, plist_content).map_err(|e| e.to_string())?;
            let _ = std::process::Command::new("launchctl").args(&["load", &plist_path.to_string_lossy()]).output();
        } else {
            if plist_path.exists() {
                let _ = std::process::Command::new("launchctl").args(&["unload", &plist_path.to_string_lossy()]).output();
                let _ = std::fs::remove_file(&plist_path);
            }
        }
        Ok(enabled)
    }

    /// Checks if LaunchAgent plist file currently exists
    pub fn is_autostart_registry_enabled() -> bool {
        Self::get_plist_path().map(|p| p.exists() && p.metadata().map(|m| m.len() > 0).unwrap_or(false)).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_launch_agent_plist() {
        let plist = MacOSAutostartManager::generate_launch_agent_plist("/Applications/DesktopBubbleChat.app/Contents/MacOS/app");
        assert!(plist.contains("<key>Label</key>"));
        assert!(plist.contains("<string>com.desktopbubblechat.app</string>"));
        assert!(plist.contains("<string>/Applications/DesktopBubbleChat.app/Contents/MacOS/app</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
    }

    #[test]
    fn test_plist_path_format() {
        if let Some(path) = MacOSAutostartManager::get_plist_path() {
            let path_str = path.to_string_lossy();
            assert!(path_str.ends_with("com.desktopbubblechat.app.plist"));
        }
    }
}
