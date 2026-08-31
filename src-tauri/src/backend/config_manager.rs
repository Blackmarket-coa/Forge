//! Generic config editing for any framework target: locate the target's
//! canonical config file through its adapter, read/write it with a backup and
//! an atomic write, and route validation to the framework's own rules.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::backend::errors::ForgeError;
use crate::backend::frameworks::{adapter, resolve_target_dir, Framework};
use crate::backend::fs_util::write_atomic;

/// Absolute path of the framework target's editable config file.
pub fn config_path(project_root: &Path, framework: Framework) -> Result<PathBuf, ForgeError> {
    let target_dir = resolve_target_dir(project_root, framework)?;
    adapter(framework).config_path(&target_dir)
}

pub fn read_config(project_root: &Path, framework: Framework) -> Result<Value, ForgeError> {
    let path = config_path(project_root, framework)?;
    let content = fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<Value>(&content)?;
    Ok(parsed)
}

pub fn write_config(
    project_root: &Path,
    framework: Framework,
    config: &Value,
) -> Result<(), ForgeError> {
    let path = config_path(project_root, framework)?;

    if path.exists() {
        fs::copy(&path, backup_path(&path))?;
    }

    let content = serde_json::to_string_pretty(config)?;
    write_atomic(&path, content.as_bytes())?;
    Ok(())
}

pub fn validate_config(
    project_root: &Path,
    framework: Framework,
    config: &Value,
) -> Result<Vec<String>, ForgeError> {
    let target_dir = resolve_target_dir(project_root, framework)?;
    Ok(adapter(framework).validate_config(&target_dir, config))
}

/// `<file>.bak` next to the original, keeping the full original name.
fn backup_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(OsString::from)
        .unwrap_or_else(|| OsString::from("config"));
    name.push(".bak");
    match path.parent() {
        Some(parent) => parent.join(name),
        None => PathBuf::from(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn write_initial(dir: &Path) {
        fs::write(
            dir.join("tauri.conf.json"),
            serde_json::to_string_pretty(&json!({
                "productName": "Old",
                "identifier": "com.example.old"
            }))
            .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn write_config_backs_up_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        write_initial(dir.path());

        let next = json!({ "productName": "New", "identifier": "com.example.new" });
        write_config(dir.path(), Framework::Tauri, &next).unwrap();

        // The new value is persisted and re-readable.
        let read = read_config(dir.path(), Framework::Tauri).unwrap();
        assert_eq!(read["productName"], "New");

        // A backup of the previous content was created.
        let backup = dir.path().join("tauri.conf.json.bak");
        assert!(backup.exists());
        let backup_value: Value =
            serde_json::from_str(&fs::read_to_string(backup).unwrap()).unwrap();
        assert_eq!(backup_value["productName"], "Old");
    }

    #[test]
    fn write_config_errors_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let result = write_config(dir.path(), Framework::Tauri, &json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn validate_routes_to_framework_rules() {
        let dir = tempfile::tempdir().unwrap();
        write_initial(dir.path());
        let issues = validate_config(dir.path(), Framework::Tauri, &json!({})).unwrap();
        assert!(issues.iter().any(|i| i.contains("productName")));
        assert!(issues.iter().any(|i| i.contains("identifier")));
    }
}
