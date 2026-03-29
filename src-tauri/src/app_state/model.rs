use serde::{Deserialize, Serialize};

use crate::backend::project_manager::{ProjectMeta, Workspace};

/// Increment this constant whenever the ForgeState schema changes in a
/// backwards-incompatible way.  The loader uses it to detect stale state
/// files and apply any necessary migrations before deserialising.
pub const STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeState {
    /// Schema version written when the file was last saved.  Used by the
    /// loader to decide whether migrations are needed.  Defaults to 1.
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub projects: Vec<ProjectMeta>,
    pub workspaces: Vec<Workspace>,
    pub build_presets: Vec<BuildPreset>,
    pub build_history: Vec<BuildRecord>,
    pub preferences: ForgePreferences,
    pub tier: String,
}

fn default_schema_version() -> u32 {
    1
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

impl ForgeState {
    pub fn set_tier(&mut self, tier: &str) {
        self.tier = tier.to_string();
    }
}

impl Default for ForgeState {
    fn default() -> Self {
        Self {
            schema_version: STATE_SCHEMA_VERSION,
            projects: vec![],
            workspaces: vec![],
            build_presets: vec![],
            build_history: vec![],
            preferences: ForgePreferences::default(),
            tier: "free".to_string(),
        }
    }
}
