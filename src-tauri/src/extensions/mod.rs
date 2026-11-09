use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileExtension {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub icon_base64: Option<String>,
    pub enabled: bool,
    pub path: String,
}

/// Resolves active and disabled extensions directories for a specific account profile
pub fn get_profile_extension_dirs(app_data_dir: &Path, account_index: u32) -> (PathBuf, PathBuf) {
    let base = app_data_dir.join("profiles").join(format!("messenger_{:02}", account_index));
    let enabled = base.join("extensions");
    let disabled = base.join("extensions_disabled");
    let _ = std::fs::create_dir_all(&enabled);
    let _ = std::fs::create_dir_all(&disabled);
    (enabled, disabled)
}

/// Recursively copies a directory tree from src to dst
pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Parses manifest.json from an extension directory
pub fn parse_manifest(folder: &Path) -> Result<(String, String, String, Option<String>), String> {
    let manifest_path = folder.join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!("manifest.json not found in {:?}", folder));
    }
    let content = std::fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let fallback_name = folder.file_name().and_then(|s| s.to_str()).unwrap_or("Extension").to_string();
    let name = json.get("name").and_then(|v| v.as_str()).unwrap_or(&fallback_name).to_string();
    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string();
    let description = json.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let mut icon_base64 = None;
    let mut icon_rel: Option<String> = None;
    if let Some(icons) = json.get("icons").and_then(|v| v.as_object()) {
        for key in &["128", "64", "48", "32", "16"] {
            if let Some(path_str) = icons.get(*key).and_then(|v| v.as_str()) {
                icon_rel = Some(path_str.to_string());
                break;
            }
        }
    }
    if icon_rel.is_none() {
        if let Some(action) = json.get("action").or_else(|| json.get("browser_action")) {
            if let Some(icon_obj) = action.get("default_icon") {
                if let Some(s) = icon_obj.as_str() {
                    icon_rel = Some(s.to_string());
                } else if let Some(obj) = icon_obj.as_object() {
                    for key in &["128", "48", "32", "16"] {
                        if let Some(path_str) = obj.get(*key).and_then(|v| v.as_str()) {
                            icon_rel = Some(path_str.to_string());
                            break;
                        }
                    }
                }
            }
        }
    }
    if let Some(rel) = icon_rel {
        let icon_path = folder.join(rel.trim_start_matches('/'));
        if icon_path.exists() {
            if let Ok(bytes) = std::fs::read(&icon_path) {
                let mut base64_str = String::new();
                for chunk in bytes.chunks(3) {
                    const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                    let b0 = chunk[0];
                    let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
                    let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
                    base64_str.push(B64_CHARS[(b0 >> 2) as usize] as char);
                    base64_str.push(B64_CHARS[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
                    if chunk.len() > 1 {
                        base64_str.push(B64_CHARS[(((b1 & 0xf) << 2) | (b2 >> 6)) as usize] as char);
                    } else {
                        base64_str.push('=');
                    }
                    if chunk.len() > 2 {
                        base64_str.push(B64_CHARS[(b2 & 0x3f) as usize] as char);
                    } else {
                        base64_str.push('=');
                    }
                }
                icon_base64 = Some(format!("data:image/png;base64,{}", base64_str));
            }
        }
    }
    Ok((name, version, description, icon_base64))
}

/// Lists all installed extensions for a given profile (both enabled and disabled)
pub fn list_profile_extensions(app_data_dir: &Path, account_index: u32) -> Result<Vec<ProfileExtension>, String> {
    let (enabled_dir, disabled_dir) = get_profile_extension_dirs(app_data_dir, account_index);
    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&enabled_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("manifest.json").exists() {
                let id = entry.file_name().to_string_lossy().to_string();
                if let Ok((name, version, description, icon_base64)) = parse_manifest(&path) {
                    list.push(ProfileExtension {
                        id,
                        name,
                        version,
                        description,
                        icon_base64,
                        enabled: true,
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir(&disabled_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("manifest.json").exists() {
                let id = entry.file_name().to_string_lossy().to_string();
                if let Ok((name, version, description, icon_base64)) = parse_manifest(&path) {
                    list.push(ProfileExtension {
                        id,
                        name,
                        version,
                        description,
                        icon_base64,
                        enabled: false,
                        path: path.to_string_lossy().to_string(),
                    });
                }
            }
        }
    }
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(list)
}

/// Toggles an extension enabled/disabled state by moving between directories
pub fn toggle_profile_extension(app_data_dir: &Path, account_index: u32, extension_id: &str, enabled: bool) -> Result<(), String> {
    let (enabled_dir, disabled_dir) = get_profile_extension_dirs(app_data_dir, account_index);
    let src = if enabled { disabled_dir.join(extension_id) } else { enabled_dir.join(extension_id) };
    let dst = if enabled { enabled_dir.join(extension_id) } else { disabled_dir.join(extension_id) };
    if !src.exists() {
        return Err(format!("Extension '{}' not found at {:?}", extension_id, src));
    }
    std::fs::rename(&src, &dst).map_err(|e| format!("Failed to move extension: {}", e))
}

/// Removes an extension completely from the profile
pub fn remove_profile_extension(app_data_dir: &Path, account_index: u32, extension_id: &str) -> Result<(), String> {
    let (enabled_dir, disabled_dir) = get_profile_extension_dirs(app_data_dir, account_index);
    let enabled_path = enabled_dir.join(extension_id);
    let disabled_path = disabled_dir.join(extension_id);
    if enabled_path.exists() {
        std::fs::remove_dir_all(&enabled_path).map_err(|e| format!("Failed to delete enabled extension: {}", e))?;
    }
    if disabled_path.exists() {
        std::fs::remove_dir_all(&disabled_path).map_err(|e| format!("Failed to delete disabled extension: {}", e))?;
    }
    Ok(())
}

/// Installs an unpacked extension folder into the profile's enabled extensions directory
pub fn install_unpacked_extension(app_data_dir: &Path, account_index: u32, source_path: &Path) -> Result<ProfileExtension, String> {
    if !source_path.exists() || !source_path.is_dir() {
        return Err(format!("Source path is not a valid directory: {:?}", source_path));
    }
    let (name, version, description, icon_base64) = parse_manifest(source_path)?;
    let (enabled_dir, _) = get_profile_extension_dirs(app_data_dir, account_index);
    let id = source_path.file_name().and_then(|s| s.to_str()).unwrap_or("ext").to_string();
    let target_dir = enabled_dir.join(&id);
    if target_dir.exists() {
        let _ = std::fs::remove_dir_all(&target_dir);
    }
    copy_dir_all(source_path, &target_dir).map_err(|e| format!("Failed to copy extension files: {}", e))?;
    Ok(ProfileExtension {
        id,
        name,
        version,
        description,
        icon_base64,
        enabled: true,
        path: target_dir.to_string_lossy().to_string(),
    })
}

/// Prompts user to select a folder on Windows via PowerShell FolderBrowserDialog
pub fn pick_folder_dialog() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let script = "[System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms') | Out-Null; $f = New-Object System.Windows.Forms.FolderBrowserDialog; $f.Description = 'Select unpacked extension folder'; if ($f.ShowDialog() -eq 'OK') { Write-Output $f.SelectedPath }";
        let output = Command::new("powershell").args(["-NoProfile", "-Command", script]).output().ok()?;
        if output.status.success() {
            let res = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !res.is_empty() {
                let p = PathBuf::from(res);
                if p.exists() && p.is_dir() {
                    return Some(p);
                }
            }
        }
    }
    None
}
