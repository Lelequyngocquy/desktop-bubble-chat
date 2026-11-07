/// Retrieves system account display name
#[tauri::command]
pub fn get_system_username() -> String {
    let raw_username = std::env::var("USERNAME").or_else(|_| std::env::var("USER"));
    if let Ok(username) = raw_username {
        let trimmed = username.trim();
        if !trimmed.is_empty() {
            if let Some(display_name) = crate::operating_systems::current::AutostartManager::get_windows_display_name(trimmed) {
                return display_name;
            }
            let mut chars = trimmed.chars();
            if let Some(first) = chars.next() {
                return first.to_uppercase().collect::<String>() + chars.as_str();
            }
            return trimmed.to_string();
        }
    }
    "User".to_string()
}

/// Toggles autostart with system
#[tauri::command]
pub fn toggle_autostart(enabled: bool) -> Result<bool, String> {
    crate::operating_systems::current::AutostartManager::toggle_autostart_registry(enabled)
}

/// Checks if autostart with system is enabled
#[tauri::command]
pub fn is_autostart_enabled() -> bool {
    crate::operating_systems::current::AutostartManager::is_autostart_registry_enabled()
}
