use serde::{Deserialize, Serialize};

use crate::backend::project_manager::{ProjectMeta, Workspace};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForgeState {
    pub projects: Vec<ProjectMeta>,
    pub workspaces: Vec<Workspace>,
    pub build_history: Vec<serde_json::Value>,
    pub preferences: ForgePreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgePreferences {
    pub theme: String,
    pub terminal_font_size: u16,
    pub default_package_manager: String,
    pub auto_check_updates: bool,
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
