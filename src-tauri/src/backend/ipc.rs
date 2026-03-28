use std::path::PathBuf;

use serde_json::json;

use crate::backend::project_manager::{
    detect_tauri_status as detect_status_impl, scan_directory as scan_dir_impl, ProjectManager,
    ProjectMeta,
};

#[tauri::command]
pub async fn register_project(path: String) -> Result<ProjectMeta, String> {
    let mut manager = ProjectManager::new();
    manager
        .register_project(&PathBuf::from(path))
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_projects(workspace_id: Option<String>) -> Result<Vec<ProjectMeta>, String> {
    let manager = ProjectManager::new();
    Ok(manager.get_projects(workspace_id))
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
    let path = PathBuf::from(project_path)
        .join("src-tauri")
        .join("tauri.conf.json");
    if !path.exists() {
        return Ok(json!({ "exists": false, "config": null }));
    }

    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let config = serde_json::from_str::<serde_json::Value>(&content).map_err(|e| e.to_string())?;
    Ok(json!({ "exists": true, "config": config }))
}

#[tauri::command]
pub async fn write_config(project_path: String, config: serde_json::Value) -> Result<(), String> {
    let path = PathBuf::from(project_path)
        .join("src-tauri")
        .join("tauri.conf.json");
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
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
    Ok(json!({
        "tauri_cli": "unknown",
        "rust": "unknown",
        "node": "unknown"
    }))
}

#[tauri::command]
pub async fn collect_artifacts(_project_path: String) -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}
