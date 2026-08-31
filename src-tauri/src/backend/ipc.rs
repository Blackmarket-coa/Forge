use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use log::{info, warn};
use serde_json::json;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::app_state::model::{Artifact, BuildPreset, BuildRecord};
use crate::app_state::store::{load_state, save_state, state_path};
use crate::backend::config_manager;
use crate::backend::frameworks::{self, Framework, FrameworkInfo, ToolCheck};
use crate::backend::fs_util::write_atomic;

/// Cap how many build records we keep on disk so `build_history.json` cannot
/// grow without bound over the lifetime of a project.
const MAX_BUILD_HISTORY: usize = 200;
use crate::backend::license;
use crate::backend::process_manager::{ProcessExitPayload, ProcessManager, ProcessOutputPayload};
use crate::backend::project_manager::{
    detect_project_status as detect_status_impl, get_git_info as get_git_info_impl,
    refresh_project_meta, register_project as register_project_impl,
    scan_directory as scan_dir_impl, ProjectMeta, Workspace,
};
use crate::backend::web_app::{self, WebAppOptions};

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

    // Keep only the most recent records so the file stays bounded.
    let mut bounded = records.to_vec();
    if bounded.len() > MAX_BUILD_HISTORY {
        bounded.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        bounded.truncate(MAX_BUILD_HISTORY);
    }

    let content = serde_json::to_string_pretty(&bounded).map_err(|e| e.to_string())?;
    write_atomic(&path, content.as_bytes()).map_err(|e| e.to_string())
}

fn sync_tier_to_state(tier: &str) -> Result<(), String> {
    let mut state = load_state().map_err(|e| e.to_string())?;
    state.set_tier(tier);
    save_state(&state).map_err(|e| e.to_string())
}

/// Parse an optional framework id from the frontend, defaulting to Tauri for
/// calls that predate multi-framework support.
fn parse_framework(id: Option<String>) -> Result<Framework, String> {
    let id = id.unwrap_or_else(|| "tauri".to_string());
    Framework::from_id(&id).ok_or_else(|| format!("unknown framework: {id}"))
}

#[tauri::command]
pub async fn validate_license(key: String) -> Result<license::LicenseStatus, String> {
    let status = license::validate_and_store_license(key).await?;
    sync_tier_to_state(&status.tier)?;
    Ok(status)
}

#[tauri::command]
pub async fn get_license_status() -> Result<license::LicenseStatus, String> {
    let status = license::get_license_status()?;
    sync_tier_to_state(&status.tier)?;
    Ok(status)
}

#[tauri::command]
pub async fn clear_license() -> Result<license::LicenseStatus, String> {
    let status = license::clear_license()?;
    sync_tier_to_state(&status.tier)?;
    Ok(status)
}

/// Static descriptions of every supported framework — the single source of
/// truth the UI uses for labels, bundle kinds, dev actions, and tools.
#[tauri::command]
pub async fn get_frameworks() -> Result<Vec<FrameworkInfo>, String> {
    Ok(frameworks::adapters().iter().map(|a| a.info()).collect())
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
        project.id = existing.id.clone();
        project.workspace_id = existing.workspace_id.clone();
        project.tags = existing.tags.clone();
        project.role = existing.role.clone();
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
        // Targets can change outside Forge (or a target added inside it);
        // keep the cheap detection-derived fields fresh.
        refresh_project_meta(project);
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

/// Detect a project's framework targets (manifest first, marker probing as a
/// fallback). Replaces the Tauri-only `detect_tauri_status`.
#[tauri::command]
pub async fn detect_project_status(path: String) -> Result<serde_json::Value, String> {
    let status = detect_status_impl(&PathBuf::from(path)).map_err(|e| e.to_string())?;
    serde_json::to_value(status).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_directory(path: String) -> Result<Vec<ProjectMeta>, String> {
    scan_dir_impl(&PathBuf::from(path)).map_err(Into::into)
}

#[tauri::command]
pub async fn read_config(
    project_path: String,
    framework: Option<String>,
) -> Result<serde_json::Value, String> {
    let fw = parse_framework(framework)?;
    config_manager::read_config(&PathBuf::from(project_path), fw).map_err(Into::into)
}

#[tauri::command]
pub async fn write_config(
    project_path: String,
    framework: Option<String>,
    config: serde_json::Value,
) -> Result<(), String> {
    let fw = parse_framework(framework)?;
    config_manager::write_config(&PathBuf::from(project_path), fw, &config).map_err(Into::into)
}

#[tauri::command]
pub async fn validate_config(
    project_path: String,
    framework: Option<String>,
    config: serde_json::Value,
) -> Result<Vec<String>, String> {
    let fw = parse_framework(framework)?;
    config_manager::validate_config(&PathBuf::from(project_path), fw, &config).map_err(Into::into)
}

/// Start the framework's preview/dev loop. Returns the process id the
/// terminal and Stop button use.
#[tauri::command]
pub async fn run_dev(
    project_path: String,
    framework: Option<String>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let fw = parse_framework(framework)?;
    let root = PathBuf::from(&project_path);
    let target_dir = frameworks::resolve_target_dir(&root, fw).map_err(|e| e.to_string())?;
    let adapter = frameworks::adapter(fw);

    let steps = adapter.dev_steps(&target_dir);
    if steps.is_empty() {
        return Err(format!(
            "{} doesn't have a live preview — build it instead.",
            adapter.info().label
        ));
    }

    let process_id = format!("dev:{}:{}", project_path, fw.id());
    info!("run_dev: starting {} preview for {project_path}", fw.id());

    let mut manager = process_manager()
        .lock()
        .map_err(|_| "failed to lock process manager".to_string())?;
    manager
        .spawn_sequence(&process_id, steps, Arc::new(app_handle))
        .map_err(|e| e.to_string())?;

    Ok(process_id)
}

#[tauri::command]
pub async fn run_build(
    project_path: String,
    framework: Option<String>,
    targets: Vec<String>,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let fw = parse_framework(framework)?;
    run_build_internal(&project_path, fw, &targets, &app_handle, None)
}

#[tauri::command]
pub async fn kill_process(process_id: String) -> Result<(), String> {
    info!("kill_process: terminating {process_id}");
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
            .spawn_command(
                &process_id,
                &parent_dir,
                cmd,
                &args_ref,
                Arc::new(app_handle.clone()),
            )
            .map_err(|e| e.to_string())?;
    }

    let exit = wait_for_process(&process_id)?;
    if exit != 0 {
        return Err(format!("create project failed with exit code {exit}"));
    }

    let new_project_dir = parent_dir.join(&name);
    register_project(new_project_dir.to_string_lossy().to_string()).await
}

/// Generate app(s) that wrap a website URL — one project holding a target for
/// each requested framework.
///
/// Unlike [`create_project`], this writes the project files directly, so it
/// needs no Node.js, package manager, or framework — only a website address
/// and an app name. This is the engine behind Forge's "turn your website into
/// an app" flow for non-technical users.
#[tauri::command]
pub async fn create_web_app(
    parent_dir: String,
    name: String,
    url: String,
    width: Option<u32>,
    height: Option<u32>,
    identifier: Option<String>,
    frameworks: Option<Vec<String>>,
) -> Result<ProjectMeta, String> {
    let framework_ids = frameworks.unwrap_or_else(|| vec!["tauri".to_string()]);
    let chosen: Vec<Framework> = framework_ids
        .iter()
        .map(|id| Framework::from_id(id).ok_or_else(|| format!("unknown framework: {id}")))
        .collect::<Result<_, _>>()?;

    let opts = WebAppOptions {
        name,
        url,
        identifier,
        width,
        height,
    };

    let project_dir = frameworks::scaffold_project(&PathBuf::from(parent_dir), &opts, &chosen)
        .map_err(|e| e.to_string())?;
    info!("create_web_app: generated {}", project_dir.display());

    register_project(project_dir.to_string_lossy().to_string()).await
}

/// Add another framework target to an existing project (e.g. a mobile app
/// alongside the desktop one), reusing the project's recorded website.
#[tauri::command]
pub async fn add_target(
    project_path: String,
    framework: String,
    url: Option<String>,
) -> Result<ProjectMeta, String> {
    let fw =
        Framework::from_id(&framework).ok_or_else(|| format!("unknown framework: {framework}"))?;
    let root = PathBuf::from(&project_path);

    let status = detect_status_impl(&root).map_err(|e| e.to_string())?;
    let source_url = url
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
        .or(status.source_url.clone())
        .ok_or_else(|| {
            "This project doesn't record which website it wraps — enter the website address."
                .to_string()
        })?;
    let name = status.name.clone().unwrap_or_else(|| {
        root.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("My App")
            .to_string()
    });

    let opts = WebAppOptions {
        name,
        url: source_url,
        identifier: status.identifier.clone(),
        width: None,
        height: None,
    };

    frameworks::add_target_to_project(&root, fw, &opts).map_err(|e| e.to_string())?;
    info!("add_target: added {} to {}", fw.id(), project_path);

    register_project(project_path).await
}

/// Suggest a friendly default folder (`~/Forge Apps`) for saving generated apps.
#[tauri::command]
pub async fn get_default_app_dir() -> Result<String, String> {
    Ok(web_app::default_app_dir().to_string_lossy().to_string())
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
                Arc::new(app_handle.clone()),
            )
            .map_err(|e| e.to_string())?;
    }

    let exit = wait_for_process(&process_id)?;
    if exit != 0 {
        return Err(format!("cargo tauri init failed with exit code {exit}"));
    }

    detect_project_status(project_path).await
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
        let step_fw = parse_framework(Some(step.framework.clone()))?;

        let result = run_build_internal(
            &project.path.to_string_lossy(),
            step_fw,
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
            let next_fw = parse_framework(Some(next.framework.clone()))?;

            let result_next = run_build_internal(
                &project_next.path.to_string_lossy(),
                next_fw,
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
pub async fn get_deploy_status(workspace_id: String) -> Result<serde_json::Value, String> {
    crate::backend::env_path::ensure_cargo_bin_in_path();
    let state = load_state().map_err(|e| e.to_string())?;
    let projects: Vec<ProjectMeta> = state
        .projects
        .into_iter()
        .filter(|p| p.workspace_id.as_deref() == Some(workspace_id.as_str()))
        .collect();

    let mut blockers: Vec<serde_json::Value> = Vec::new();
    let mut matrix = Vec::new();
    let mut checklist = Vec::new();
    let mut built_count = 0usize;
    let mut total_count = 0usize;
    let mut frameworks_in_use: Vec<Framework> = Vec::new();

    for project in &projects {
        let project_path = PathBuf::from(&project.path);
        let status = detect_status_impl(&project_path).map_err(|e| e.to_string())?;
        let (git_branch, git_dirty) =
            get_git_info_impl(&project_path).map_err(|e| e.to_string())?;
        let artifacts = collect_artifacts_internal(&project.path.to_string_lossy())?;

        if status.targets.is_empty() {
            blockers.push(json!({
                "message": "No app targets found in this project",
                "affected_project": project.name,
                "severity": "high",
                "fix_hint": "Open the project and add an app type, or repair its config"
            }));
        }

        for target in &status.targets {
            let Some(fw) = Framework::from_id(&target.framework) else {
                continue;
            };
            if !frameworks_in_use.contains(&fw) {
                frameworks_in_use.push(fw);
            }
            let adapter = frameworks::adapter(fw);
            let framework_label = adapter.info().label;

            let mut platform_status = serde_json::Map::new();
            for platform in adapter.info().platforms {
                total_count += 1;

                let built = artifacts.iter().any(|a| {
                    let fmt = a.get("format").and_then(|v| v.as_str()).unwrap_or("");
                    let art_fw = a.get("framework").and_then(|v| v.as_str()).unwrap_or("");
                    art_fw == target.framework
                        && adapter.platform_for_artifact(fmt) == Some(*platform)
                });

                let s = if built {
                    built_count += 1;
                    "built"
                } else if target.status == "ready" {
                    "configured"
                } else {
                    "not_started"
                };
                platform_status.insert((*platform).to_string(), json!(s));
            }

            checklist.push(json!({
                "project": project.name,
                "framework": target.framework,
                "framework_label": framework_label,
                "initialized": target.status != "error",
                "git_branch": git_branch,
                "git_dirty": git_dirty,
                "config_ok": target.config_ok,
                "config_issues": target.config_issues,
            }));

            matrix.push(json!({
                "project_id": project.id,
                "project_name": project.name,
                "framework": target.framework,
                "framework_label": framework_label,
                "statuses": platform_status,
            }));

            if !target.config_ok {
                blockers.push(json!({
                    "message": format!("{framework_label}: configuration needs attention"),
                    "affected_project": project.name,
                    "severity": "medium",
                    "fix_hint": "Open App settings to review and fix the reported issues"
                }));
            }
        }
    }

    // Missing tools, only for the frameworks these projects actually use.
    let mut reported_tools: Vec<&'static str> = Vec::new();
    for fw in &frameworks_in_use {
        for tool in frameworks::adapter(*fw).info().tools {
            if !tool.applies_here() || reported_tools.contains(&tool.name) {
                continue;
            }
            reported_tools.push(tool.name);
            let (installed, _version) = tool.run_probe();
            if !installed {
                blockers.push(json!({
                    "message": format!("{} not installed", tool.label),
                    "affected_project": null,
                    "severity": "high",
                    "fix_hint": tool.install_hint,
                }));
            }
        }
    }

    let overall_progress = if total_count == 0 {
        0.0
    } else {
        (built_count as f64 / total_count as f64) * 100.0
    };

    Ok(json!({
        "workspace_id": workspace_id,
        "overall_progress": overall_progress,
        "matrix": matrix,
        "checklist": checklist,
        "blockers": blockers,
    }))
}

/// Check which tools are installed, grouped so the UI can say which app types
/// need each one. Deduped across frameworks (Node.js is listed once even
/// though several frameworks need it).
#[tauri::command]
pub async fn check_environment() -> Result<serde_json::Value, String> {
    // Re-run per check so tools installed after launch (e.g. rustup while
    // Forge is open) are picked up by the "Check again" button.
    crate::backend::env_path::ensure_cargo_bin_in_path();

    let mut order: Vec<&'static str> = Vec::new();
    let mut by_name: BTreeMap<&'static str, (ToolCheck, Vec<&'static str>)> = BTreeMap::new();

    for adapter in frameworks::adapters() {
        let info = adapter.info();
        for tool in info.tools {
            if !tool.applies_here() {
                continue;
            }
            if let Some((_, needed_by)) = by_name.get_mut(tool.name) {
                if !needed_by.contains(&info.id) {
                    needed_by.push(info.id);
                }
            } else {
                order.push(tool.name);
                by_name.insert(tool.name, (tool, vec![info.id]));
            }
        }
    }

    let mut tools = Vec::new();
    for name in order {
        let (tool, needed_by) = &by_name[name];
        let (installed, version) = tool.run_probe();
        tools.push(json!({
            "name": tool.name,
            "label": tool.label,
            "installed": installed,
            "version": version,
            "install_hint": tool.install_hint,
            "needed_by": needed_by,
        }));
    }

    Ok(json!({ "tools": tools }))
}

#[tauri::command]
pub async fn collect_artifacts(project_path: String) -> Result<Vec<serde_json::Value>, String> {
    collect_artifacts_internal(&project_path)
}

/// Collect installer artifacts from every framework target in the project,
/// tagging each with the framework that produced it. Only files whose
/// extension is an installable output are listed.
fn collect_artifacts_internal(project_path: &str) -> Result<Vec<serde_json::Value>, String> {
    let root = PathBuf::from(project_path);
    let status = detect_status_impl(&root).map_err(|e| e.to_string())?;

    let mut artifacts = Vec::new();
    for target in &status.targets {
        let Some(fw) = Framework::from_id(&target.framework) else {
            continue;
        };
        let adapter = frameworks::adapter(fw);
        let target_dir = frameworks::target_path(&root, &target.dir);

        for base in adapter.artifact_dirs(&target_dir) {
            if !base.exists() {
                continue;
            }
            let mut stack = vec![base];
            while let Some(dir) = stack.pop() {
                for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
                    let entry = entry.map_err(|e| e.to_string())?;
                    let path = entry.path();
                    let metadata = entry.metadata().map_err(|e| e.to_string())?;

                    if metadata.is_dir() {
                        stack.push(path);
                        continue;
                    }

                    let format = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_string();
                    if adapter.platform_for_artifact(&format).is_none() {
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
                        "format": format,
                        "created_at": created_at,
                        "framework": target.framework,
                    }));
                }
            }
        }
    }

    Ok(artifacts)
}

/// Wait for a managed process without holding the process-manager lock, so
/// Stop stays responsive while something runs.
fn wait_for_process(process_id: &str) -> Result<i32, String> {
    let handle = {
        let manager = process_manager()
            .lock()
            .map_err(|_| "failed to lock process manager".to_string())?;
        manager.wait_handle(process_id).map_err(|e| e.to_string())?
    };
    Ok(handle.wait())
}

fn run_build_internal(
    project_path: &str,
    framework: Framework,
    targets: &[String],
    app_handle: &AppHandle,
    project_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let started_at = chrono_like_now();
    let root = PathBuf::from(project_path);
    let target_dir = frameworks::resolve_target_dir(&root, framework).map_err(|e| e.to_string())?;
    let adapter = frameworks::adapter(framework);
    let mut status = "success".to_string();

    info!(
        "run_build: building {project_path} ({}) for targets [{}]",
        framework.id(),
        targets.join(", ")
    );

    for bundle_kind in targets {
        let process_id = format!("build:{}:{}:{}", project_path, framework.id(), bundle_kind);

        // Some outputs (the PWA kit) are produced in-process with no external
        // toolchain; emit matching terminal events so the activity view shows
        // what happened.
        match adapter.build_in_process(&target_dir, bundle_kind) {
            Ok(true) => {
                let _ = app_handle.emit(
                    "process-output",
                    ProcessOutputPayload {
                        process_id: process_id.clone(),
                        data: format!("Packaged {} ({bundle_kind})", adapter.info().label),
                        is_stderr: false,
                    },
                );
                let _ = app_handle.emit(
                    "process-exit",
                    ProcessExitPayload {
                        id: process_id.clone(),
                        code: 0,
                    },
                );
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                warn!("run_build: in-process build failed: {e}");
                let _ = app_handle.emit(
                    "process-output",
                    ProcessOutputPayload {
                        process_id: process_id.clone(),
                        data: e.to_string(),
                        is_stderr: true,
                    },
                );
                status = "failed".to_string();
                break;
            }
        }

        let steps = adapter
            .build_steps(&target_dir, bundle_kind)
            .map_err(|e| e.to_string())?;

        {
            let mut manager = process_manager()
                .lock()
                .map_err(|_| "failed to lock process manager".to_string())?;
            manager
                .spawn_sequence(&process_id, steps, Arc::new(app_handle.clone()))
                .map_err(|e| e.to_string())?;
        }

        let exit_code = wait_for_process(&process_id)?;
        if exit_code != 0 {
            warn!("run_build: target {bundle_kind} exited with code {exit_code}");
            status = "failed".to_string();
            break;
        }
    }

    let all_artifacts = collect_artifacts_internal(project_path)?;
    let artifacts_json: Vec<serde_json::Value> = all_artifacts
        .into_iter()
        .filter(|a| a.get("framework").and_then(|v| v.as_str()) == Some(framework.id()))
        .collect();
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
        framework: framework.id().to_string(),
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
        "framework": framework.id(),
        "duration_secs": duration_secs,
        "artifacts": artifacts_json
    }))
}

fn chrono_like_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default();
    secs.to_string()
}
