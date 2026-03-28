use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use walkdir::WalkDir;

use crate::backend::errors::ForgeError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub workspace_id: Option<String>,
    pub tauri_version: Option<String>,
    pub identifier: Option<String>,
    pub frontend_framework: Option<String>,
    pub platforms: Vec<String>,
    pub git_branch: Option<String>,
    pub git_dirty: bool,
    pub status: String,
    pub tags: Vec<String>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub project_ids: Vec<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TauriStatus {
    pub has_tauri_conf: bool,
    pub tauri_conf_path: Option<PathBuf>,
    pub product_name: Option<String>,
    pub identifier: Option<String>,
    pub version: Option<String>,
    pub tauri_version: Option<String>,
    pub frontend_framework: Option<String>,
    pub status: String,
}

pub fn detect_tauri_status(path: &Path) -> Result<TauriStatus, ForgeError> {
    let nested_conf = path.join("src-tauri").join("tauri.conf.json");
    let flat_conf = path.join("tauri.conf.json");

    let tauri_conf_path = if nested_conf.exists() {
        Some(nested_conf)
    } else if flat_conf.exists() {
        Some(flat_conf)
    } else {
        None
    };

    let mut product_name = None;
    let mut identifier = None;
    let mut version = None;

    if let Some(conf_path) = &tauri_conf_path {
        let content = fs::read_to_string(conf_path)?;
        let value: Value = serde_json::from_str(&content)?;

        product_name = value
            .get("productName")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        identifier = value
            .get("identifier")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        version = value
            .get("version")
            .and_then(|v| v.as_str())
            .map(str::to_string);
    }

    let tauri_dir = if path.join("src-tauri").exists() {
        path.join("src-tauri")
    } else {
        path.to_path_buf()
    };

    let tauri_version = detect_tauri_dependency_version(&tauri_dir)?;
    let frontend_framework = detect_frontend_framework(path)?;

    let status = match (tauri_conf_path.is_some(), tauri_version.is_some()) {
        (true, true) => "ready",
        (true, false) => "needs_config",
        (false, _) => "error",
    }
    .to_string();

    Ok(TauriStatus {
        has_tauri_conf: tauri_conf_path.is_some(),
        tauri_conf_path,
        product_name,
        identifier,
        version,
        tauri_version,
        frontend_framework,
        status,
    })
}

pub fn register_project(path: &Path, id: String) -> Result<ProjectMeta, ForgeError> {
    let status = detect_tauri_status(path)?;
    let (git_branch, git_dirty) = get_git_info(path)?;
    project_meta_from_status(path, id, status, git_branch, git_dirty)
}

pub fn get_git_info(path: &Path) -> Result<(Option<String>, bool), ForgeError> {
    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(path)
        .output();

    let branch = match branch_output {
        Ok(out) if out.status.success() => {
            let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if b.is_empty() {
                None
            } else {
                Some(b)
            }
        }
        _ => None,
    };

    let dirty_output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output();

    let git_dirty = match dirty_output {
        Ok(out) if out.status.success() => !String::from_utf8_lossy(&out.stdout).trim().is_empty(),
        _ => false,
    };

    Ok((branch, git_dirty))
}

pub fn scan_directory(path: &Path) -> Result<Vec<ProjectMeta>, ForgeError> {
    if !path.exists() {
        return Err(ForgeError::ProjectNotFound(path.display().to_string()));
    }

    let mut found = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        if entry.file_name() != "tauri.conf.json" {
            continue;
        }

        let conf_parent = match entry.path().parent() {
            Some(p) => p,
            None => continue,
        };

        let root = if conf_parent.ends_with("src-tauri") {
            conf_parent.parent().unwrap_or(conf_parent)
        } else {
            conf_parent
        };

        let status = detect_tauri_status(root)?;
        let (git_branch, git_dirty) = get_git_info(root)?;
        let id = root
            .canonicalize()
            .unwrap_or_else(|_| root.to_path_buf())
            .to_string_lossy()
            .to_string();

        let meta = project_meta_from_status(root, id, status, git_branch, git_dirty)?;
        found.push(meta);
    }

    Ok(found)
}

fn project_meta_from_status(
    path: &Path,
    id: String,
    status: TauriStatus,
    git_branch: Option<String>,
    git_dirty: bool,
) -> Result<ProjectMeta, ForgeError> {
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let name = status.product_name.clone().unwrap_or_else(|| {
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed-project")
            .to_string()
    });

    Ok(ProjectMeta {
        id,
        name,
        path: canonical_path,
        workspace_id: None,
        tauri_version: status.tauri_version,
        identifier: status.identifier,
        frontend_framework: status.frontend_framework,
        platforms: vec!["desktop".to_string()],
        git_branch,
        git_dirty,
        status: status.status,
        tags: vec![],
        role: None,
    })
}

fn detect_tauri_dependency_version(tauri_dir: &Path) -> Result<Option<String>, ForgeError> {
    let cargo_toml = tauri_dir.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(cargo_toml)?;
    let parsed: toml::Value = toml::from_str(&content)
        .map_err(|e| ForgeError::ConfigInvalid(format!("invalid Cargo.toml: {e}")))?;

    let dep = parsed
        .get("dependencies")
        .and_then(|v| v.get("tauri"))
        .cloned();

    let version = match dep {
        Some(toml::Value::String(v)) => Some(v),
        Some(toml::Value::Table(t)) => t
            .get("version")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        _ => None,
    };

    Ok(version)
}

fn detect_frontend_framework(project_root: &Path) -> Result<Option<String>, ForgeError> {
    let package_json = project_root.join("package.json");
    if !package_json.exists() {
        return Ok(Some("vanilla".to_string()));
    }

    let content = fs::read_to_string(package_json)?;
    let value: Value = serde_json::from_str(&content)?;

    let has_dep = |name: &str| {
        value
            .get("dependencies")
            .and_then(|d| d.get(name))
            .is_some()
            || value
                .get("devDependencies")
                .and_then(|d| d.get(name))
                .is_some()
    };

    let framework = if has_dep("react") {
        "react"
    } else if has_dep("svelte") {
        "svelte"
    } else if has_dep("vue") {
        "vue"
    } else {
        "vanilla"
    };

    Ok(Some(framework.to_string()))
}
