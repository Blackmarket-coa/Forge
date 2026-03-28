use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use url::Url;

use crate::backend::errors::ForgeError;

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
    fs::write(path, content)?;
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
