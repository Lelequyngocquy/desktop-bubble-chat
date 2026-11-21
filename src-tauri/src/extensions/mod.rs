use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// Validates that an extension ID is a single safe directory name without path traversal
pub fn validate_extension_id(extension_id: &str) -> Result<(), String> {
    if extension_id.is_empty()
        || extension_id == "."
        || extension_id == ".."
        || extension_id.contains('/')
        || extension_id.contains('\\')
        || extension_id.contains(':')
        || extension_id.contains('\0')
    {
        return Err("Invalid extension ID".into());
    }
    let mut comps = Path::new(extension_id).components();
    match (comps.next(), comps.next()) {
        (Some(Component::Normal(s)), None) if s == extension_id => Ok(()),
        _ => Err("Invalid extension ID".into()),
    }
}

/// Resolves a relative path inside base directory and ensures canonical containment
pub fn resolve_contained_path(base: &Path, rel: &Path) -> Result<PathBuf, String> {
    if rel.is_absolute() {
        return Err("Absolute path not allowed".into());
    }
    for comp in rel.components() {
        if matches!(comp, Component::ParentDir | Component::RootDir | Component::Prefix(_)) {
            return Err("Path traversal not allowed".into());
        }
    }
    let canon_base = std::fs::canonicalize(base).map_err(|e| e.to_string())?;
    let candidate = base.join(rel);
    let canon_target = std::fs::canonicalize(&candidate).map_err(|e| e.to_string())?;
    if !canon_target.starts_with(&canon_base) {
        return Err("Resolved path escapes base directory".into());
    }
    Ok(canon_target)
}

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
        if ty.is_symlink() {
            continue;
        }
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
    if icon_rel.is_none()
        && let Some(action) = json.get("action").or_else(|| json.get("browser_action"))
            && let Some(icon_obj) = action.get("default_icon") {
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
    if let Some(rel) = icon_rel {
        let trimmed = rel.trim_start_matches(['/', '\\']);
        if let Ok(icon_path) = resolve_contained_path(folder, Path::new(trimmed))
            && icon_path.is_file()
            && let Ok(bytes) = std::fs::read(&icon_path) {
                use base64::{engine::general_purpose::STANDARD, Engine as _};
                icon_base64 = Some(format!("data:image/png;base64,{}", STANDARD.encode(&bytes)));
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
    list.sort_by_key(|a| a.name.to_lowercase());
    Ok(list)
}

/// Toggles an extension enabled/disabled state by moving between directories
pub fn toggle_profile_extension(app_data_dir: &Path, account_index: u32, extension_id: &str, enabled: bool) -> Result<(), String> {
    validate_extension_id(extension_id)?;
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
    validate_extension_id(extension_id)?;
    let (enabled_dir, disabled_dir) = get_profile_extension_dirs(app_data_dir, account_index);
    let enabled_path = enabled_dir.join(extension_id);
    let disabled_path = disabled_dir.join(extension_id);
    if enabled_path.exists() {
        let safe_enabled = resolve_contained_path(&enabled_dir, Path::new(extension_id))?;
        std::fs::remove_dir_all(&safe_enabled).map_err(|e| format!("Failed to delete enabled extension: {}", e))?;
    }
    if disabled_path.exists() {
        let safe_disabled = resolve_contained_path(&disabled_dir, Path::new(extension_id))?;
        std::fs::remove_dir_all(&safe_disabled).map_err(|e| format!("Failed to delete disabled extension: {}", e))?;
    }
    Ok(())
}

/// Installs an unpacked extension folder into the profile's enabled extensions directory
pub fn install_unpacked_extension(app_data_dir: &Path, account_index: u32, source_path: &Path) -> Result<ProfileExtension, String> {
    let canon_source = std::fs::canonicalize(source_path).map_err(|_| format!("Source path is not a valid directory: {:?}", source_path))?;
    if !canon_source.is_dir() {
        return Err(format!("Source path is not a valid directory: {:?}", source_path));
    }
    let (name, version, description, icon_base64) = parse_manifest(&canon_source)?;
    let (enabled_dir, _) = get_profile_extension_dirs(app_data_dir, account_index);
    let id = canon_source.file_name().and_then(|s| s.to_str()).unwrap_or("ext").to_string();
    validate_extension_id(&id)?;
    let target_dir = enabled_dir.join(&id);
    if target_dir.exists() {
        let safe_target = resolve_contained_path(&enabled_dir, Path::new(&id))?;
        let _ = std::fs::remove_dir_all(&safe_target);
    }
    copy_dir_all(&canon_source, &target_dir).map_err(|e| format!("Failed to copy extension files: {}", e))?;
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

/// Prompts user to select an unpacked extension folder cleanly using native OS dialog
pub fn pick_folder_dialog() -> Option<PathBuf> {
    let picked = rfd::FileDialog::new()
        .set_title("Select unpacked extension folder")
        .pick_folder()?;
    if picked.exists() && picked.is_dir() {
        Some(picked)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_extension_id_rejects_traversal() {
        assert!(validate_extension_id("my-ext_1").is_ok());
        assert!(validate_extension_id("").is_err());
        assert!(validate_extension_id(".").is_err());
        assert!(validate_extension_id("..").is_err());
        assert!(validate_extension_id("../escape").is_err());
        assert!(validate_extension_id("a/b").is_err());
        assert!(validate_extension_id("a\\b").is_err());
        assert!(validate_extension_id("C:test").is_err());
    }

    #[test]
    fn test_resolve_contained_path_blocks_escape() {
        let temp_root = std::env::temp_dir().join(format!("dbc_ext_test_{}", std::process::id()));
        let base = temp_root.join("ext_ok");
        let outside = temp_root.join("secret.txt");
        let _ = std::fs::create_dir_all(&base);
        let _ = std::fs::write(&outside, b"secret");
        let _ = std::fs::write(base.join("icon.png"), b"png");
        assert!(resolve_contained_path(&base, Path::new("icon.png")).is_ok());
        assert!(resolve_contained_path(&base, Path::new("../secret.txt")).is_err());
        assert!(resolve_contained_path(&base, &outside).is_err());
        let _ = std::fs::remove_dir_all(&temp_root);
    }
}
