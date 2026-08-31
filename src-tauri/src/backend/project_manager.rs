use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use walkdir::{DirEntry, WalkDir};

use crate::backend::errors::ForgeError;
use crate::backend::frameworks::{self, Framework, TargetStatus};
use crate::backend::fs_util::write_atomic;

/// File name of the per-project manifest declaring which framework targets a
/// project contains. (Distinct from `~/.forge/forge.json`, Forge's own state.)
pub const PROJECT_MANIFEST: &str = "forge.project.json";

/// One framework target registered in a project.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TargetMeta {
    pub framework: String,
    /// Directory relative to the project root ("." for Tauri's root layout).
    pub dir: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub status: String,
}

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
    /// Framework targets in this project. Empty in state files written before
    /// multi-framework support; refreshed from disk on load/registration.
    #[serde(default)]
    pub targets: Vec<TargetMeta>,
    /// Website the project wraps, when known.
    #[serde(default)]
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub project_ids: Vec<String>,
    pub color: Option<String>,
}

/// The `forge.project.json` manifest written into generated projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub schema_version: u32,
    pub name: String,
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    pub targets: Vec<ManifestTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestTarget {
    pub framework: String,
    pub dir: String,
}

pub fn read_manifest(project_root: &Path) -> Result<Option<ProjectManifest>, ForgeError> {
    let path = project_root.join(PROJECT_MANIFEST);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let manifest = serde_json::from_str::<ProjectManifest>(&content)?;
    Ok(Some(manifest))
}

pub fn write_manifest(project_root: &Path, manifest: &ProjectManifest) -> Result<(), ForgeError> {
    let content = serde_json::to_string_pretty(manifest)?;
    write_atomic(&project_root.join(PROJECT_MANIFEST), content.as_bytes())
}

/// Everything detection learned about a project: its declared or discovered
/// framework targets plus the identity fields the UI shows.
#[derive(Debug, Clone, Serialize, Default)]
pub struct ProjectStatus {
    pub name: Option<String>,
    pub identifier: Option<String>,
    pub source_url: Option<String>,
    pub targets: Vec<TargetStatus>,
    /// "ready" when every target is ready, "needs_config" when any target
    /// needs attention, "error" when no target was found at all.
    pub status: String,
}

/// Detect a project's framework targets.
///
/// The `forge.project.json` manifest wins when present (each declared target
/// is verified by its adapter); otherwise every registered adapter probes the
/// folder, so projects created outside Forge are still recognized.
pub fn detect_project_status(path: &Path) -> Result<ProjectStatus, ForgeError> {
    let manifest = read_manifest(path)?;

    let targets: Vec<TargetStatus> = match &manifest {
        Some(manifest) => {
            let mut found = Vec::new();
            for declared in &manifest.targets {
                match Framework::from_id(&declared.framework) {
                    Some(fw) => match frameworks::adapter(fw).detect(path) {
                        Some(mut status) => {
                            status.dir = declared.dir.clone();
                            found.push(status);
                        }
                        None => found.push(TargetStatus {
                            framework: declared.framework.clone(),
                            dir: declared.dir.clone(),
                            product_name: None,
                            identifier: None,
                            source_url: None,
                            version: None,
                            config_ok: false,
                            config_issues: vec![format!(
                                "declared {} app not found on disk",
                                declared.framework
                            )],
                            status: "error".to_string(),
                        }),
                    },
                    None => found.push(TargetStatus {
                        framework: declared.framework.clone(),
                        dir: declared.dir.clone(),
                        product_name: None,
                        identifier: None,
                        source_url: None,
                        version: None,
                        config_ok: false,
                        config_issues: vec![format!(
                            "unknown framework \"{}\" in {PROJECT_MANIFEST}",
                            declared.framework
                        )],
                        status: "error".to_string(),
                    }),
                }
            }
            found
        }
        None => frameworks::detect_targets(path),
    };

    let status = if targets.is_empty() {
        "error"
    } else if targets.iter().all(|t| t.status == "ready") {
        "ready"
    } else {
        "needs_config"
    }
    .to_string();

    let first_named = targets.iter().find_map(|t| t.product_name.clone());
    let first_identifier = targets.iter().find_map(|t| t.identifier.clone());
    let first_url = targets.iter().find_map(|t| t.source_url.clone());

    Ok(ProjectStatus {
        name: manifest
            .as_ref()
            .map(|m| m.name.clone())
            .filter(|n| !n.trim().is_empty())
            .or(first_named),
        identifier: manifest
            .as_ref()
            .and_then(|m| m.identifier.clone())
            .or(first_identifier),
        source_url: manifest
            .as_ref()
            .and_then(|m| m.source_url.clone())
            .or(first_url),
        targets,
        status,
    })
}

pub fn register_project(path: &Path, id: String) -> Result<ProjectMeta, ForgeError> {
    let status = detect_project_status(path)?;
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

/// Directories that never contain project markers worth walking into.
fn keep_entry(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return true;
    }
    let name = entry.file_name().to_string_lossy();
    !matches!(
        name.as_ref(),
        "node_modules" | "target" | "dist" | "www" | ".git" | ".expo" | "build"
    )
}

/// Marker file names that identify a framework target during a scan. The
/// walk visits files once and resolves every marker back to its project root.
fn marker_root(entry_path: &Path, file_name: &str) -> Option<PathBuf> {
    let parent = entry_path.parent()?;

    let candidate = match file_name {
        PROJECT_MANIFEST => Some(parent.to_path_buf()),
        "tauri.conf.json" => {
            // Nested layout: <root>/src-tauri/tauri.conf.json.
            if parent.ends_with("src-tauri") {
                parent.parent().map(Path::to_path_buf)
            } else {
                Some(parent.to_path_buf())
            }
        }
        "capacitor.config.json" | "capacitor.config.ts" | "manifest.webmanifest" => {
            Some(parent.to_path_buf())
        }
        "app.json" => {
            // Only Expo app.json files count as React Native markers.
            let content = fs::read_to_string(entry_path).ok()?;
            let value: Value = serde_json::from_str(&content).ok()?;
            value.get("expo")?;
            Some(parent.to_path_buf())
        }
        "package.json" => {
            // Only package.json files that depend on Electron are markers.
            let content = fs::read_to_string(entry_path).ok()?;
            let value: Value = serde_json::from_str(&content).ok()?;
            let has_electron = value
                .get("dependencies")
                .and_then(|d| d.get("electron"))
                .is_some()
                || value
                    .get("devDependencies")
                    .and_then(|d| d.get("electron"))
                    .is_some();
            if has_electron {
                Some(parent.to_path_buf())
            } else {
                None
            }
        }
        _ => None,
    }?;

    // A target folder inside a Forge project (e.g. <root>/capacitor) belongs
    // to the surrounding project, not a project of its own.
    if let Some(project_parent) = candidate.parent() {
        if project_parent.join(PROJECT_MANIFEST).exists() {
            return Some(project_parent.to_path_buf());
        }
    }

    Some(candidate)
}

pub fn scan_directory(path: &Path) -> Result<Vec<ProjectMeta>, ForgeError> {
    if !path.exists() {
        return Err(ForgeError::ProjectNotFound(path.display().to_string()));
    }

    let mut roots: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(keep_entry)
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if let Some(root) = marker_root(entry.path(), &file_name) {
            let canonical = root.canonicalize().unwrap_or(root);
            if !roots.contains(&canonical) {
                roots.push(canonical);
            }
        }
    }

    let mut found = Vec::new();
    for root in roots {
        let status = detect_project_status(&root)?;
        if status.targets.is_empty() {
            continue;
        }
        let (git_branch, git_dirty) = get_git_info(&root)?;
        let id = root.to_string_lossy().to_string();
        let meta = project_meta_from_status(&root, id, status, git_branch, git_dirty)?;
        found.push(meta);
    }

    Ok(found)
}

fn project_meta_from_status(
    path: &Path,
    id: String,
    status: ProjectStatus,
    git_branch: Option<String>,
    git_dirty: bool,
) -> Result<ProjectMeta, ForgeError> {
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let name = status.name.clone().unwrap_or_else(|| {
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed-project")
            .to_string()
    });

    let targets: Vec<TargetMeta> = status
        .targets
        .iter()
        .map(|t| TargetMeta {
            framework: t.framework.clone(),
            dir: t.dir.clone(),
            version: t.version.clone(),
            status: t.status.clone(),
        })
        .collect();

    // Platforms = union of the platforms of every detected framework, in the
    // deploy dashboard's display order.
    let mut platforms: Vec<String> = Vec::new();
    for t in &status.targets {
        if let Some(fw) = Framework::from_id(&t.framework) {
            for p in frameworks::adapter(fw).info().platforms {
                if !platforms.iter().any(|existing| existing == p) {
                    platforms.push((*p).to_string());
                }
            }
        }
    }

    let tauri_version = status
        .targets
        .iter()
        .find(|t| t.framework == "tauri")
        .and_then(|t| t.version.clone());

    let frontend_framework = detect_frontend_framework(path)?;

    Ok(ProjectMeta {
        id,
        name,
        path: canonical_path,
        workspace_id: None,
        tauri_version,
        identifier: status.identifier.clone(),
        frontend_framework,
        platforms,
        git_branch,
        git_dirty,
        status: status.status.clone(),
        tags: vec![],
        role: None,
        targets,
        source_url: status.source_url,
    })
}

/// Refresh the detection-derived fields of an already-registered project
/// (targets, platforms, status, versions) while preserving its identity,
/// workspace assignment, and tags.
pub fn refresh_project_meta(project: &mut ProjectMeta) {
    if let Ok(status) = detect_project_status(&project.path) {
        if let Ok(meta) = project_meta_from_status(
            &project.path,
            project.id.clone(),
            status,
            project.git_branch.clone(),
            project.git_dirty,
        ) {
            project.name = meta.name;
            project.tauri_version = meta.tauri_version;
            project.identifier = meta.identifier;
            project.frontend_framework = meta.frontend_framework;
            project.platforms = meta.platforms;
            project.status = meta.status;
            project.targets = meta.targets;
            project.source_url = meta.source_url;
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Scaffold a minimal Tauri project under `root` and return its path.
    fn scaffold(root: &Path, product: &str, framework_dep: &str) {
        let tauri = root.join("src-tauri");
        fs::create_dir_all(&tauri).unwrap();
        fs::write(
            tauri.join("tauri.conf.json"),
            format!(
                r#"{{ "productName": "{product}", "identifier": "com.example.{product}", "version": "1.2.3" }}"#
            ),
        )
        .unwrap();
        fs::write(
            tauri.join("Cargo.toml"),
            "[dependencies]\ntauri = { version = \"2.1.0\" }\n",
        )
        .unwrap();
        let deps = if framework_dep.is_empty() {
            "{}".to_string()
        } else {
            format!(r#"{{ "{framework_dep}": "^1.0.0" }}"#)
        };
        fs::write(
            root.join("package.json"),
            format!(r#"{{ "dependencies": {deps} }}"#),
        )
        .unwrap();
    }

    #[test]
    fn detect_status_reads_conf_and_dependency() {
        let dir = tempfile::tempdir().unwrap();
        scaffold(dir.path(), "Demo", "react");

        let status = detect_project_status(dir.path()).unwrap();
        assert_eq!(status.targets.len(), 1);
        assert_eq!(status.targets[0].framework, "tauri");
        assert_eq!(status.name.as_deref(), Some("Demo"));
        assert_eq!(status.identifier.as_deref(), Some("com.example.Demo"));
        assert_eq!(status.targets[0].version.as_deref(), Some("2.1.0"));
    }

    #[test]
    fn detect_status_errors_without_targets() {
        let dir = tempfile::tempdir().unwrap();
        let status = detect_project_status(dir.path()).unwrap();
        assert!(status.targets.is_empty());
        assert_eq!(status.status, "error");
    }

    #[test]
    fn register_project_uses_product_name() {
        let dir = tempfile::tempdir().unwrap();
        scaffold(dir.path(), "MyApp", "svelte");

        let meta = register_project(dir.path(), "id-1".to_string()).unwrap();
        assert_eq!(meta.id, "id-1");
        assert_eq!(meta.name, "MyApp");
        assert_eq!(meta.frontend_framework.as_deref(), Some("svelte"));
        assert_eq!(meta.targets.len(), 1);
        assert_eq!(meta.targets[0].framework, "tauri");
        assert_eq!(meta.tauri_version.as_deref(), Some("2.1.0"));
        assert!(meta.platforms.iter().any(|p| p == "Linux"));
    }

    #[test]
    fn scan_directory_finds_nested_projects() {
        let dir = tempfile::tempdir().unwrap();
        scaffold(&dir.path().join("alpha"), "Alpha", "vue");
        scaffold(&dir.path().join("beta"), "Beta", "");
        // A non-project directory should be ignored.
        fs::create_dir_all(dir.path().join("docs")).unwrap();

        let mut found = scan_directory(dir.path()).unwrap();
        found.sort_by(|a, b| a.name.cmp(&b.name));
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, vec!["Alpha", "Beta"]);
        assert_eq!(found[1].frontend_framework.as_deref(), Some("vanilla"));
    }

    #[test]
    fn scan_directory_errors_on_missing_path() {
        let result = scan_directory(Path::new("/no/such/forge/path"));
        assert!(matches!(result, Err(ForgeError::ProjectNotFound(_))));
    }

    #[test]
    fn framework_defaults_to_vanilla_without_package_json() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            detect_frontend_framework(dir.path()).unwrap().as_deref(),
            Some("vanilla")
        );
    }

    #[test]
    fn manifest_round_trips_and_wins_detection() {
        let dir = tempfile::tempdir().unwrap();
        scaffold(dir.path(), "Named", "");

        let manifest = ProjectManifest {
            schema_version: 1,
            name: "Manifest Name".to_string(),
            identifier: Some("com.example.manifest".to_string()),
            source_url: Some("https://example.com/".to_string()),
            targets: vec![ManifestTarget {
                framework: "tauri".to_string(),
                dir: ".".to_string(),
            }],
        };
        write_manifest(dir.path(), &manifest).unwrap();

        let read = read_manifest(dir.path()).unwrap().unwrap();
        assert_eq!(read.name, "Manifest Name");

        let status = detect_project_status(dir.path()).unwrap();
        assert_eq!(status.name.as_deref(), Some("Manifest Name"));
        assert_eq!(status.source_url.as_deref(), Some("https://example.com/"));
        assert_eq!(status.targets.len(), 1);
    }

    #[test]
    fn manifest_declared_but_missing_target_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = ProjectManifest {
            schema_version: 1,
            name: "Ghost".to_string(),
            identifier: None,
            source_url: None,
            targets: vec![ManifestTarget {
                framework: "tauri".to_string(),
                dir: ".".to_string(),
            }],
        };
        write_manifest(dir.path(), &manifest).unwrap();

        let status = detect_project_status(dir.path()).unwrap();
        assert_eq!(status.targets.len(), 1);
        assert_eq!(status.targets[0].status, "error");
        assert_eq!(status.status, "needs_config");
    }
}
