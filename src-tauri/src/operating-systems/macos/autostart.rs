use std::path::PathBuf;

pub struct MacOSAutostartManager;

impl MacOSAutostartManager {
    /// Resolves path to LaunchAgent plist for desktop-bubble-chat
    pub fn get_plist_path() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/LaunchAgents/com.desktopbubblechat.app.plist"))
    }

    /// Escapes special XML characters for safe embedding inside plist elements
    pub fn escape_xml(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        for ch in input.chars() {
            match ch {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                '\'' => out.push_str("&apos;"),
                _ => out.push(ch),
            }
        }
        out
    }

    /// Generates standard Apple Launchd property list XML string with XML-escaped program path
    pub fn generate_launch_agent_plist(exe_path: &str) -> String {
        let escaped_path = Self::escape_xml(exe_path);
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
            escaped_path
        )
    }

    /// Retrieves user display name via id -F <username> or dscl RealName
    pub fn get_windows_display_name(username: &str) -> Option<String> {
        let trimmed = username.trim();
        let id_cmd = if trimmed.is_empty() {
            std::process::Command::new("id").arg("-F").output()
        } else {
            std::process::Command::new("id").args(["-F", trimmed]).output()
        };
        if let Ok(output) = id_cmd
            && output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        if !trimmed.is_empty()
            && let Ok(dscl_output) = std::process::Command::new("dscl").args([".", "-read", &format!("/Users/{}", trimmed), "RealName"]).output()
            && dscl_output.status.success() {
                let raw = String::from_utf8_lossy(&dscl_output.stdout);
                let cleaned = raw.replace("RealName:", "").trim().to_string();
                if !cleaned.is_empty() {
                    return Some(cleaned);
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
            let _ = std::process::Command::new("launchctl").args(["load", &plist_path.to_string_lossy()]).output();
        } else if plist_path.exists() {
            let _ = std::process::Command::new("launchctl").args(["unload", &plist_path.to_string_lossy()]).output();
            let _ = std::fs::remove_file(&plist_path);
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

    #[test]
    fn test_generate_launch_agent_plist_escapes_xml() {
        let plist = MacOSAutostartManager::generate_launch_agent_plist("/Apps/A&B <Test>/\"App's\"");
        assert!(plist.contains("<string>/Apps/A&amp;B &lt;Test&gt;/&quot;App&apos;s&quot;</string>"));
    }
}
