use serde::{Deserialize, Serialize};

use crate::backend::project_manager::{ProjectMeta, Workspace};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForgeState {
    pub projects: Vec<ProjectMeta>,
    pub workspaces: Vec<Workspace>,
    pub build_presets: Vec<BuildPreset>,
    pub build_history: Vec<BuildRecord>,
    pub preferences: ForgePreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgePreferences {
    pub theme: String,
    pub terminal_font_size: u16,
    pub default_package_manager: String,
    pub auto_check_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildPreset {
    pub id: String,
    pub name: String,
    pub workspace_id: String,
    pub steps: Vec<BuildStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildStep {
    pub project_id: String,
    pub targets: Vec<String>,
    pub parallel_with_next: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Artifact {
    pub path: String,
    pub size_bytes: u64,
    pub format: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildRecord {
    pub id: String,
    pub project_id: String,
    pub targets: Vec<String>,
    pub status: String,
    pub started_at: String,
    pub duration_secs: u64,
    pub artifacts: Vec<Artifact>,
    pub log_path: String,
}

impl Default for ForgePreferences {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            terminal_font_size: 13,
            default_package_manager: "npm".to_string(),
            auto_check_updates: true,
        }
    }
}
