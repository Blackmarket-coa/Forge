use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::json;
use tauri::AppHandle;
use uuid::Uuid;

use crate::app_state::model::{Artifact, BuildPreset, BuildRecord, BuildStep};
use crate::app_state::store::{load_state, save_state, state_path};
use crate::backend::config_manager;
use crate::backend::process_manager::ProcessManager;
use crate::backend::project_manager::{
    detect_tauri_status as detect_status_impl, get_git_info as get_git_info_impl,
    register_project as register_project_impl, scan_directory as scan_dir_impl, ProjectMeta,
    Workspace,
};

static PROCESS_MANAGER: OnceLock<Mutex<ProcessManager>> = OnceLock::new();

fn process_manager() -> &'static Mutex<ProcessManager> {
    PROCESS_MANAGER.get_or_init(|| Mutex::new(ProcessManager::new()))
}

fn build_history_path() -> Result<PathBuf, String> {
    let base = state_path().map_err(|e| e.to_string())?;
    let dir = base
        .parent()
        .ok_or_else(|| "invalid state file path".to_string())?;
    Ok(dir.join("build_history.json"))
}

fn load_history() -> Result<Vec<BuildRecord>, String> {
    let path = build_history_path()?;
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn save_history(records: &[BuildRecord]) -> Result<(), String> {
    let path = build_history_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(records).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
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
    run_build_internal(&project_path, &targets, &app_handle, None)
}

#[tauri::command]
pub async fn kill_process(process_id: String) -> Result<(), String> {
    let mut manager = process_manager()
        .lock()
        .map_err(|_| "failed to lock process manager".to_string())?;
    manager.kill(&process_id).map_err(Into::into)
}

#[tauri::command]
pub async fn create_project(
    path: String,
    name: String,
    template: String,
    package_manager: String,
    app_handle: AppHandle,
) -> Result<ProjectMeta, String> {
    let parent_dir = PathBuf::from(path);
    let process_id = format!("create:{}", name);

    let (cmd, args): (&str, Vec<String>) = match package_manager.as_str() {
        "pnpm" => (
            "pnpm",
            vec![
                "create".into(),
                "tauri-app".into(),
                name.clone(),
                "--template".into(),
                template.clone(),
                "--manager".into(),
                "pnpm".into(),
            ],
        ),
        "yarn" => (
            "yarn",
            vec![
                "create".into(),
                "tauri-app".into(),
                name.clone(),
                "--template".into(),
                template.clone(),
                "--manager".into(),
                "yarn".into(),
            ],
        ),
        "bun" => (
            "bun",
            vec![
                "create".into(),
                "tauri-app".into(),
                name.clone(),
                "--template".into(),
                template.clone(),
                "--manager".into(),
                "bun".into(),
            ],
        ),
        _ => (
            "npm",
            vec![
                "create".into(),
                "tauri-app@latest".into(),
                name.clone(),
                "--".into(),
                "--template".into(),
                template.clone(),
                "--manager".into(),
                "npm".into(),
            ],
        ),
    };

    {
        let mut manager = process_manager()
            .lock()
            .map_err(|_| "failed to lock process manager".to_string())?;
        let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
        manager
            .spawn_command(&process_id, &parent_dir, cmd, &args_ref, &app_handle)
            .map_err(|e| e.to_string())?;
    }

    {
        let manager = process_manager()
            .lock()
            .map_err(|_| "failed to lock process manager".to_string())?;
        let exit = manager
            .wait_for_exit(&process_id)
            .map_err(|e| e.to_string())?;
        if exit != 0 {
            return Err(format!("create project failed with exit code {exit}"));
        }
    }

    let new_project_dir = parent_dir.join(&name);
    register_project(new_project_dir.to_string_lossy().to_string()).await
}

#[tauri::command]
pub async fn init_tauri(
    project_path: String,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let project_dir = PathBuf::from(&project_path);
    let process_id = format!("init:{}", project_path);

    {
        let mut manager = process_manager()
            .lock()
            .map_err(|_| "failed to lock process manager".to_string())?;
        manager
            .spawn_command(
                &process_id,
                &project_dir,
                "cargo",
                &["tauri", "init"],
                &app_handle,
            )
            .map_err(|e| e.to_string())?;
    }

    {
        let manager = process_manager()
            .lock()
            .map_err(|_| "failed to lock process manager".to_string())?;
        let exit = manager
            .wait_for_exit(&process_id)
            .map_err(|e| e.to_string())?;
        if exit != 0 {
            return Err(format!("cargo tauri init failed with exit code {exit}"));
        }
    }

    detect_tauri_status(project_path).await
}

#[tauri::command]
pub async fn create_workspace(name: String) -> Result<Workspace, String> {
    let mut state = load_state().map_err(|e| e.to_string())?;
    let workspace = Workspace {
        id: Uuid::new_v4().to_string(),
        name,
        project_ids: vec![],
        color: None,
    };
    state.workspaces.push(workspace.clone());
    save_state(&state).map_err(|e| e.to_string())?;
    Ok(workspace)
}

#[tauri::command]
pub async fn get_workspaces() -> Result<Vec<Workspace>, String> {
    let state = load_state().map_err(|e| e.to_string())?;
    Ok(state.workspaces)
}

#[tauri::command]
pub async fn update_workspace(
    id: String,
    name: Option<String>,
    color: Option<String>,
) -> Result<Workspace, String> {
    let mut state = load_state().map_err(|e| e.to_string())?;
    let ws = state
        .workspaces
        .iter_mut()
        .find(|w| w.id == id)
        .ok_or_else(|| "workspace not found".to_string())?;

    if let Some(n) = name {
        ws.name = n;
    }
    if color.is_some() {
        ws.color = color;
    }

    let updated = ws.clone();
    save_state(&state).map_err(|e| e.to_string())?;
    Ok(updated)
}

#[tauri::command]
pub async fn delete_workspace(id: String) -> Result<(), String> {
    let mut state = load_state().map_err(|e| e.to_string())?;
    state.workspaces.retain(|w| w.id != id);
    for p in &mut state.projects {
        if p.workspace_id.as_deref() == Some(id.as_str()) {
            p.workspace_id = None;
        }
    }
    save_state(&state).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_project_to_workspace(
    workspace_id: String,
    project_id: String,
) -> Result<(), String> {
    let mut state = load_state().map_err(|e| e.to_string())?;

    let ws = state
        .workspaces
        .iter_mut()
        .find(|w| w.id == workspace_id)
        .ok_or_else(|| "workspace not found".to_string())?;
    if !ws.project_ids.contains(&project_id) {
        ws.project_ids.push(project_id.clone());
    }

    if let Some(project) = state.projects.iter_mut().find(|p| p.id == project_id) {
        project.workspace_id = Some(workspace_id.clone());
    }

    save_state(&state).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_project_from_workspace(
    workspace_id: String,
    project_id: String,
) -> Result<(), String> {
    let mut state = load_state().map_err(|e| e.to_string())?;

    if let Some(ws) = state.workspaces.iter_mut().find(|w| w.id == workspace_id) {
        ws.project_ids.retain(|id| id != &project_id);
    }

    if let Some(project) = state.projects.iter_mut().find(|p| p.id == project_id) {
        if project.workspace_id.as_deref() == Some(workspace_id.as_str()) {
            project.workspace_id = None;
        }
    }

    save_state(&state).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_build_preset(mut preset: BuildPreset) -> Result<(), String> {
    let mut state = load_state().map_err(|e| e.to_string())?;
    if preset.id.is_empty() {
        preset.id = Uuid::new_v4().to_string();
    }

    if let Some(existing) = state.build_presets.iter_mut().find(|p| p.id == preset.id) {
        *existing = preset;
    } else {
        state.build_presets.push(preset);
    }

    save_state(&state).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_build_presets(workspace_id: String) -> Result<Vec<BuildPreset>, String> {
    let state = load_state().map_err(|e| e.to_string())?;
    Ok(state
        .build_presets
        .into_iter()
        .filter(|p| p.workspace_id == workspace_id)
        .collect())
}

#[tauri::command]
pub async fn run_build_preset(
    preset_id: String,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let state = load_state().map_err(|e| e.to_string())?;
    let preset = state
        .build_presets
        .iter()
        .find(|p| p.id == preset_id)
        .cloned()
        .ok_or_else(|| "preset not found".to_string())?;

    let mut timeline = Vec::new();
    let mut i = 0;
    while i < preset.steps.len() {
        let step = preset.steps[i].clone();
        let project = state
            .projects
            .iter()
            .find(|p| p.id == step.project_id)
            .ok_or_else(|| format!("project not found for step: {}", step.project_id))?;

        let result = run_build_internal(
            &project.path.to_string_lossy(),
            &step.targets,
            &app_handle,
            Some(step.project_id.clone()),
        )?;
        timeline.push(result);

        if step.parallel_with_next && i + 1 < preset.steps.len() {
            let next = preset.steps[i + 1].clone();
            let project_next = state
                .projects
                .iter()
                .find(|p| p.id == next.project_id)
                .ok_or_else(|| format!("project not found for step: {}", next.project_id))?;

            let result_next = run_build_internal(
                &project_next.path.to_string_lossy(),
                &next.targets,
                &app_handle,
                Some(next.project_id.clone()),
            )?;
            timeline.push(result_next);
            i += 1;
        }

        i += 1;
    }

    Ok(json!({"preset_id": preset.id, "timeline": timeline}))
}

#[tauri::command]
pub async fn get_build_history(
    project_id: Option<String>,
    limit: u32,
) -> Result<Vec<BuildRecord>, String> {
    let mut records = load_history()?;
    if let Some(pid) = project_id {
        records.retain(|r| r.project_id == pid);
    }
    records.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    records.truncate(limit as usize);
    Ok(records)
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
    collect_artifacts_internal(&project_path)
}

fn collect_artifacts_internal(project_path: &str) -> Result<Vec<serde_json::Value>, String> {
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
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
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

fn run_build_internal(
    project_path: &str,
    targets: &[String],
    app_handle: &AppHandle,
    project_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let started_at = chrono_like_now();
    let project_dir = PathBuf::from(project_path);
    let mut status = "success".to_string();

    for target in targets {
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
                    app_handle,
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

    let artifacts_json = collect_artifacts_internal(project_path)?;
    let duration_secs = started.elapsed().as_secs();

    let artifacts: Vec<Artifact> = artifacts_json
        .iter()
        .map(|v| Artifact {
            path: v
                .get("path")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            size_bytes: v
                .get("size_bytes")
                .and_then(|x| x.as_u64())
                .unwrap_or_default(),
            format: v
                .get("format")
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            created_at: v
                .get("created_at")
                .and_then(|x| x.as_u64())
                .unwrap_or_default(),
        })
        .collect();

    let record = BuildRecord {
        id: Uuid::new_v4().to_string(),
        project_id: project_id.unwrap_or_else(|| project_path.to_string()),
        targets: targets.to_vec(),
        status: status.clone(),
        started_at,
        duration_secs,
        artifacts,
        log_path: "".to_string(),
    };

    let mut history = load_history()?;
    history.push(record);
    save_history(&history)?;

    Ok(json!({
        "status": status,
        "duration_secs": duration_secs,
        "artifacts": artifacts_json
    }))
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

fn chrono_like_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    secs.to_string()
}
