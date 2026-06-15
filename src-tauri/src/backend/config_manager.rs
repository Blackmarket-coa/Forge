use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use url::Url;

use crate::backend::errors::ForgeError;
use crate::backend::fs_util::write_atomic;

fn config_path(project_path: &Path) -> Result<PathBuf, ForgeError> {
    let nested = project_path.join("src-tauri").join("tauri.conf.json");
    let flat = project_path.join("tauri.conf.json");

    if nested.exists() {
        Ok(nested)
    } else if flat.exists() {
        Ok(flat)
    } else {
        Err(ForgeError::ConfigNotFound(
            project_path.display().to_string(),
        ))
    }
}

pub fn read_config(project_path: &Path) -> Result<Value, ForgeError> {
    let path = config_path(project_path)?;
    let content = fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<Value>(&content)?;
    Ok(parsed)
}

pub fn write_config(project_path: &Path, config: &Value) -> Result<(), ForgeError> {
    let path = config_path(project_path)?;
    let backup = path.with_extension("json.bak");

    if path.exists() {
        fs::copy(&path, &backup)?;
    }

    let content = serde_json::to_string_pretty(config)?;
    write_atomic(&path, content.as_bytes())?;
    Ok(())
}

pub fn validate_config(project_path: &Path, config: &Value) -> Result<Vec<String>, ForgeError> {
    let mut issues = Vec::new();

    if config
        .get("productName")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        != Some(true)
    {
        issues.push("Missing or empty productName".to_string());
    }

    let identifier = config
        .get("identifier")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if identifier.is_empty() {
        issues.push("Missing identifier".to_string());
    } else if !is_reverse_domain(identifier) {
        issues.push(
            "identifier should match reverse-domain format (e.g. com.example.app)".to_string(),
        );
    }

    if let Some(windows) = config
        .get("app")
        .and_then(|v| v.get("windows"))
        .and_then(|v| v.as_array())
    {
        for (i, w) in windows.iter().enumerate() {
            let width = w.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let height = w.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if width <= 0.0 {
                issues.push(format!("window[{i}] width must be positive"));
            }
            if height <= 0.0 {
                issues.push(format!("window[{i}] height must be positive"));
            }
        }
    }

    if let Some(dev_url) = config
        .get("build")
        .and_then(|v| v.get("devUrl"))
        .and_then(|v| v.as_str())
    {
        if Url::parse(dev_url).is_err() {
            issues.push("build.devUrl must be a valid URL".to_string());
        }
    }

    if let Some(frontend_dist) = config
        .get("build")
        .and_then(|v| v.get("frontendDist"))
        .and_then(|v| v.as_str())
    {
        let path = project_path.join(frontend_dist);
        if !path.exists() {
            issues.push(format!(
                "build.frontendDist path does not exist: {frontend_dist}"
            ));
        }
    }

    Ok(issues)
}

fn is_reverse_domain(identifier: &str) -> bool {
    let parts: Vec<&str> = identifier.split('.').collect();
    if parts.len() < 3 {
        return false;
    }

    parts.iter().all(|p| {
        !p.is_empty()
            && p.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    })
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
        write_config(dir.path(), &next).unwrap();

        // The new value is persisted and re-readable.
        let read = read_config(dir.path()).unwrap();
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
        let result = write_config(dir.path(), &json!({}));
        assert!(matches!(result, Err(ForgeError::ConfigNotFound(_))));
    }

    #[test]
    fn validate_flags_missing_fields() {
        let dir = tempfile::tempdir().unwrap();
        let issues = validate_config(dir.path(), &json!({})).unwrap();
        assert!(issues.iter().any(|i| i.contains("productName")));
        assert!(issues.iter().any(|i| i.contains("identifier")));
    }

    #[test]
    fn validate_flags_bad_identifier_and_window() {
        let dir = tempfile::tempdir().unwrap();
        let config = json!({
            "productName": "App",
            "identifier": "notreverse",
            "app": { "windows": [{ "width": 0, "height": -1 }] }
        });
        let issues = validate_config(dir.path(), &config).unwrap();
        assert!(issues.iter().any(|i| i.contains("reverse-domain")));
        assert!(issues.iter().any(|i| i.contains("width")));
        assert!(issues.iter().any(|i| i.contains("height")));
    }

    #[test]
    fn validate_passes_clean_config() {
        let dir = tempfile::tempdir().unwrap();
        let config = json!({
            "productName": "App",
            "identifier": "com.example.app",
            "app": { "windows": [{ "width": 1200, "height": 800 }] }
        });
        let issues = validate_config(dir.path(), &config).unwrap();
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }
}
