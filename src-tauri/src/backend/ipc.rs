use std::path::PathBuf;
use std::process::Command;

use serde_json::json;
use uuid::Uuid;

use crate::app_state::store::{load_state, save_state};
use crate::backend::project_manager::{
    detect_tauri_status as detect_status_impl, get_git_info as get_git_info_impl,
    register_project as register_project_impl, scan_directory as scan_dir_impl, ProjectMeta,
};

#[tauri::command]
pub async fn register_project(path: String) -> Result<ProjectMeta, String> {
    let project_path = PathBuf::from(path);
    let project_id = Uuid::new_v4().to_string();

    let mut project =
        register_project_impl(&project_path, project_id).map_err(|e| e.to_string())?;
    let (git_branch, git_dirty) = get_git_info_impl(&project_path).map_err(|e| e.to_string())?;
    project.git_branch = git_branch;
    project.git_dirty = git_dirty;

    let mut state = load_state().map_err(|e| e.to_string())?;
    if let Some(existing) = state
        .projects
        .iter_mut()
        .find(|p| p.path == project.path || p.id == project.id)
    {
        *existing = project.clone();
    } else {
        state.projects.push(project.clone());
    }

    save_state(&state).map_err(|e| e.to_string())?;
    Ok(project)
}

#[tauri::command]
pub async fn get_projects(workspace_id: Option<String>) -> Result<Vec<ProjectMeta>, String> {
    let mut state = load_state().map_err(|e| e.to_string())?;

    for project in &mut state.projects {
        let (branch, dirty) = get_git_info_impl(&project.path).map_err(|e| e.to_string())?;
        project.git_branch = branch;
        project.git_dirty = dirty;
    }

    save_state(&state).map_err(|e| e.to_string())?;

    let projects = if let Some(id) = workspace_id {
        state
            .projects
            .into_iter()
            .filter(|p| p.workspace_id.as_deref() == Some(id.as_str()))
            .collect()
    } else {
        state.projects
    };

    Ok(projects)
}

#[tauri::command]
pub async fn detect_tauri_status(path: String) -> Result<serde_json::Value, String> {
    let status = detect_status_impl(&PathBuf::from(path)).map_err(|e| e.to_string())?;
    serde_json::to_value(status).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_directory(path: String) -> Result<Vec<ProjectMeta>, String> {
    scan_dir_impl(&PathBuf::from(path)).map_err(Into::into)
}

#[tauri::command]
pub async fn read_config(project_path: String) -> Result<serde_json::Value, String> {
    let root = PathBuf::from(project_path);
    let nested = root.join("src-tauri").join("tauri.conf.json");
    let flat = root.join("tauri.conf.json");
    let config_path = if nested.exists() { nested } else { flat };

    if !config_path.exists() {
        return Ok(json!({ "exists": false, "config": null }));
    }

    let content = std::fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    let config = serde_json::from_str::<serde_json::Value>(&content).map_err(|e| e.to_string())?;
    Ok(json!({ "exists": true, "config": config }))
}

#[tauri::command]
pub async fn write_config(project_path: String, config: serde_json::Value) -> Result<(), String> {
    let root = PathBuf::from(project_path);
    let nested = root.join("src-tauri").join("tauri.conf.json");
    let flat = root.join("tauri.conf.json");
    let config_path = if nested.exists() { nested } else { flat };

    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(config_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_config(config: serde_json::Value) -> Result<Vec<String>, String> {
    let mut issues = Vec::new();
    if config.get("productName").is_none() {
        issues.push("Missing productName".to_string());
    }
    if config.get("identifier").is_none() {
        issues.push("Missing identifier".to_string());
    }
    Ok(issues)
}

#[tauri::command]
pub async fn run_dev(_project_path: String) -> Result<u32, String> {
    Ok(0)
}

#[tauri::command]
pub async fn run_build(_project_path: String, targets: Vec<String>) -> Result<String, String> {
    Ok(format!("Build queued for targets: {}", targets.join(",")))
}

#[tauri::command]
pub async fn kill_process(_process_id: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn check_environment() -> Result<serde_json::Value, String> {
    fn check_command(cmd: &str, args: &[&str]) -> serde_json::Value {
        match Command::new(cmd).args(args).output() {
            Ok(out) if out.status.success() => {
                let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
                json!({"installed": true, "version": version})
            }
            _ => json!({"installed": false, "version": "not found"}),
        }
    }

    let rust = check_command("rustc", &["--version"]);
    let cargo = check_command("cargo", &["--version"]);
    let node = check_command("node", &["--version"]);
    let tauri_cli = check_command("cargo", &["tauri", "--version"]);

    let mut platform_deps = Vec::new();
    if cfg!(target_os = "linux") {
        let webkit = Command::new("dpkg")
            .args(["-l", "libwebkit2gtk-4.1-dev"])
            .output();

        let installed = matches!(webkit, Ok(out) if out.status.success());
        platform_deps.push(json!({
            "name": "libwebkit2gtk-4.1-dev",
            "installed": installed
        }));
    }

    Ok(json!({
        "rust": rust,
        "cargo": cargo,
        "node": node,
        "tauri_cli": tauri_cli,
        "platform_deps": platform_deps
    }))
}

#[tauri::command]
pub async fn collect_artifacts(_project_path: String) -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}
