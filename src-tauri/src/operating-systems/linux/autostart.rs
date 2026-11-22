use std::path::PathBuf;

pub struct LinuxAutostartManager;

impl LinuxAutostartManager {
    /// Resolves path to XDG autostart desktop entry for desktop-bubble-chat
    pub fn get_desktop_entry_path() -> Option<PathBuf> {
        let base_dir = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base_dir.join("autostart/desktop-bubble-chat.desktop"))
    }

    /// Resolves persistent executable path prioritizing APPIMAGE over temporary mount current_exe
    pub fn resolve_autostart_exe_path(appimage_env: Option<&str>, current_exe: &str) -> String {
        if let Some(appimage) = appimage_env
            && !appimage.trim().is_empty() {
                return appimage.trim().to_string();
            }
        current_exe.to_string()
    }

    /// Generates standard XDG autostart desktop entry content with escaped Exec value
    pub fn generate_desktop_entry(exe_path: &str) -> String {
        let escaped = exe_path
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('%', "%%");
        format!(
            "[Desktop Entry]\n\
            Type=Application\n\
            Name=DesktopBubbleChat\n\
            Exec=\"{}\"\n\
            Icon=desktop-bubble-chat\n\
            Comment=Floating Bubble Chat for Messenger\n\
            X-GNOME-Autostart-enabled=true\n\
            Terminal=false\n",
            escaped
        )
    }

    /// Retrieves user display name via getent passwd GECOS field
    pub fn get_windows_display_name(username: &str) -> Option<String> {
        if let Ok(output) = std::process::Command::new("getent").args(["passwd", username]).output()
            && output.status.success() {
                let line = String::from_utf8_lossy(&output.stdout);
                let fields: Vec<&str> = line.trim().split(':').collect();
                if fields.len() >= 5 {
                    let gecos = fields[4].split(',').next().unwrap_or("").trim();
                    if !gecos.is_empty() {
                        return Some(gecos.to_string());
                    }
                }
            }
        None
    }

    /// Toggles XDG autostart desktop entry file
    pub fn toggle_autostart_registry(enabled: bool) -> Result<bool, String> {
        let path = Self::get_desktop_entry_path().ok_or_else(|| "Failed to resolve autostart directory".to_string())?;
        if enabled {
            let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let appimage_env = std::env::var("APPIMAGE").ok();
            let exe_str = Self::resolve_autostart_exe_path(appimage_env.as_deref(), &current_exe.to_string_lossy());
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let content = Self::generate_desktop_entry(&exe_str);
            std::fs::write(&path, content).map_err(|e| e.to_string())?;
        } else if path.exists() {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        Ok(enabled)
    }

    /// Checks if XDG autostart desktop entry file currently exists
    pub fn is_autostart_registry_enabled() -> bool {
        Self::get_desktop_entry_path().map(|p| p.exists() && p.metadata().map(|m| m.len() > 0).unwrap_or(false)).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_desktop_entry() {
        let entry = LinuxAutostartManager::generate_desktop_entry("/usr/bin/desktop-bubble-chat");
        assert!(entry.contains("[Desktop Entry]"));
        assert!(entry.contains("Name=DesktopBubbleChat"));
        assert!(entry.contains("Exec=\"/usr/bin/desktop-bubble-chat\""));
        assert!(entry.contains("X-GNOME-Autostart-enabled=true"));
    }

    #[test]
    fn test_desktop_entry_path_format() {
        if let Some(path) = LinuxAutostartManager::get_desktop_entry_path() {
            let path_str = path.to_string_lossy();
            assert!(path_str.ends_with("desktop-bubble-chat.desktop"));
        }
    }
}
