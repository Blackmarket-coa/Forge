use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TauriStatus {
    pub has_tauri_conf: bool,
    pub has_tauri_dep: bool,
    pub status: String,
}

#[derive(Default)]
pub struct ProjectManager {
    projects: HashMap<String, ProjectMeta>,
}

impl ProjectManager {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
        }
    }

    pub fn detect_tauri_status(&self, path: &Path) -> Result<TauriStatus, ForgeError> {
        detect_tauri_status(path)
    }

    pub fn scan_directory(&self, path: &Path) -> Result<Vec<ProjectMeta>, ForgeError> {
        scan_directory(path)
    }

    pub fn register_project(&mut self, path: &Path) -> Result<ProjectMeta, ForgeError> {
        let status = detect_tauri_status(path)?;
        let metadata = extract_project_meta(path, status)?;
        self.projects.insert(metadata.id.clone(), metadata.clone());
        Ok(metadata)
    }

    pub fn get_projects(&self, workspace_id: Option<String>) -> Vec<ProjectMeta> {
        let mut projects: Vec<ProjectMeta> = self.projects.values().cloned().collect();
        if let Some(id) = workspace_id {
            projects.retain(|p| p.workspace_id.as_deref() == Some(id.as_str()));
        }
        projects
    }

    pub fn get_git_info(&self, path: &Path) -> Result<(Option<String>, bool), ForgeError> {
        get_git_info(path)
    }
}

pub fn detect_tauri_status(path: &Path) -> Result<TauriStatus, ForgeError> {
    let tauri_conf = path.join("src-tauri").join("tauri.conf.json");
    let cargo_toml = path.join("src-tauri").join("Cargo.toml");

    let has_tauri_conf = tauri_conf.exists();
    let has_tauri_dep = if cargo_toml.exists() {
        let content = fs::read_to_string(cargo_toml)?;
        content.contains("tauri")
    } else {
        false
    };

    let status = match (has_tauri_conf, has_tauri_dep) {
        (true, true) => "ready",
        (false, false) => "needs_init",
        _ => "needs_config",
    }
    .to_string();

    Ok(TauriStatus {
        has_tauri_conf,
        has_tauri_dep,
        status,
    })
}

pub fn scan_directory(path: &Path) -> Result<Vec<ProjectMeta>, ForgeError> {
    if !path.exists() {
        return Err(ForgeError::ProjectNotFound(path.display().to_string()));
    }

    let mut projects = Vec::new();
    walk_for_tauri(path, &mut projects)?;
    Ok(projects)
}

fn walk_for_tauri(path: &Path, projects: &mut Vec<ProjectMeta>) -> Result<(), ForgeError> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            let possible = entry_path.join("src-tauri").join("tauri.conf.json");
            if possible.exists() {
                let status = detect_tauri_status(&entry_path)?;
                projects.push(extract_project_meta(&entry_path, status)?);
            }
            walk_for_tauri(&entry_path, projects)?;
        }
    }

    Ok(())
}

pub fn get_git_info(path: &Path) -> Result<(Option<String>, bool), ForgeError> {
    let git_head = path.join(".git").join("HEAD");
    if !git_head.exists() {
        return Ok((None, false));
    }

    let content = fs::read_to_string(git_head)?;
    let branch = content
        .trim()
        .strip_prefix("ref: refs/heads/")
        .map(str::to_string);

    let dirty = false;
    Ok((branch, dirty))
}

fn extract_project_meta(path: &Path, status: TauriStatus) -> Result<ProjectMeta, ForgeError> {
    let project_id = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();

    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unnamed-project")
        .to_string();

    let conf_path = path.join("src-tauri").join("tauri.conf.json");
    let identifier = if conf_path.exists() {
        let value: serde_json::Value = serde_json::from_str(&fs::read_to_string(conf_path)?)?;
        value
            .get("identifier")
            .and_then(|v| v.as_str())
            .map(str::to_string)
    } else {
        None
    };

    let (git_branch, git_dirty) = get_git_info(path)?;

    Ok(ProjectMeta {
        id: project_id,
        name,
        path: path.to_path_buf(),
        workspace_id: None,
        tauri_version: None,
        identifier,
        frontend_framework: None,
        platforms: vec![],
        git_branch,
        git_dirty,
        status: status.status,
        tags: vec![],
        role: None,
    })
}
