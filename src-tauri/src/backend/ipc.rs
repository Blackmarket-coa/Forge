use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use serde_json::json;
use tauri::AppHandle;
use uuid::Uuid;

use crate::app_state::store::{load_state, save_state};
use crate::backend::config_manager;
use crate::backend::process_manager::ProcessManager;
use crate::backend::project_manager::{
    detect_tauri_status as detect_status_impl, get_git_info as get_git_info_impl,
    register_project as register_project_impl, scan_directory as scan_dir_impl, ProjectMeta,
};

static PROCESS_MANAGER: OnceLock<Mutex<ProcessManager>> = OnceLock::new();

fn process_manager() -> &'static Mutex<ProcessManager> {
    PROCESS_MANAGER.get_or_init(|| Mutex::new(ProcessManager::new()))
}

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
    config_manager::read_config(&PathBuf::from(project_path)).map_err(Into::into)
}

#[tauri::command]
pub async fn write_config(project_path: String, config: serde_json::Value) -> Result<(), String> {
    config_manager::write_config(&PathBuf::from(project_path), &config).map_err(Into::into)
}

#[tauri::command]
pub async fn validate_config(
    project_path: String,
    config: serde_json::Value,
) -> Result<Vec<String>, String> {
    config_manager::validate_config(&PathBuf::from(project_path), &config).map_err(Into::into)
}

#[tauri::command]
pub async fn run_dev(project_path: String, app_handle: AppHandle) -> Result<u32, String> {
    let project_dir = PathBuf::from(&project_path);
    let process_id = format!("dev:{}", project_path);

    let _pm = detect_package_manager(&project_dir);

    let mut manager = process_manager()
        .lock()
        .map_err(|_| "failed to lock process manager".to_string())?;

    manager
        .spawn_command(
            &process_id,
            &project_dir,
            "cargo",
            &["tauri", "dev"],
            &app_handle,
        )
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_build(
    project_path: String,
    targets: Vec<String>,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let project_dir = PathBuf::from(&project_path);
    let mut status = "success".to_string();

    for target in &targets {
        let process_id = format!("build:{}:{}", project_path, target);
        {
            let mut manager = process_manager()
                .lock()
                .map_err(|_| "failed to lock process manager".to_string())?;
            manager
                .spawn_command(
                    &process_id,
                    &project_dir,
                    "cargo",
                    &["tauri", "build", "--bundles", target],
                    &app_handle,
                )
                .map_err(|e| e.to_string())?;
        }

        let exit_code = {
            let manager = process_manager()
                .lock()
                .map_err(|_| "failed to lock process manager".to_string())?;
            manager
                .wait_for_exit(&process_id)
                .map_err(|e| e.to_string())?
        };

        if exit_code != 0 {
            status = "failed".to_string();
            break;
        }
    }

    let artifacts = collect_artifacts(project_path.clone()).await?;
    let duration_secs = started.elapsed().as_secs_f64();

    Ok(json!({
        "status": status,
        "duration_secs": duration_secs,
        "artifacts": artifacts
    }))
}

#[tauri::command]
pub async fn kill_process(process_id: String) -> Result<(), String> {
    let mut manager = process_manager()
        .lock()
        .map_err(|_| "failed to lock process manager".to_string())?;
    manager.kill(&process_id).map_err(Into::into)
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
pub async fn collect_artifacts(project_path: String) -> Result<Vec<serde_json::Value>, String> {
    let bundle_dir = PathBuf::from(project_path)
        .join("src-tauri")
        .join("target")
        .join("release")
        .join("bundle");

    let mut artifacts = Vec::new();
    if !bundle_dir.exists() {
        return Ok(artifacts);
    }

    let mut stack = vec![bundle_dir];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let metadata = entry.metadata().map_err(|e| e.to_string())?;

            if metadata.is_dir() {
                stack.push(path);
                continue;
            }

            let created_at = metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or_default();

            artifacts.push(json!({
                "path": path.to_string_lossy().to_string(),
                "size_bytes": metadata.len(),
                "format": path.extension().and_then(|e| e.to_str()).unwrap_or(""),
                "created_at": created_at,
            }));
        }
    }

    Ok(artifacts)
}

fn detect_package_manager(project_dir: &Path) -> &'static str {
    if project_dir.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if project_dir.join("yarn.lock").exists() {
        "yarn"
    } else if project_dir.join("bun.lockb").exists() || project_dir.join("bun.lock").exists() {
        "bun"
    } else {
        "npm"
    }
}
