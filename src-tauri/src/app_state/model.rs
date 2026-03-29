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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_current_schema_version() {
        let state = ForgeState::default();
        assert_eq!(state.schema_version, STATE_SCHEMA_VERSION);
    }

    #[test]
    fn default_state_tier_is_free() {
        let state = ForgeState::default();
        assert_eq!(state.tier, "free");
    }

    #[test]
    fn set_tier_updates_tier_field() {
        let mut state = ForgeState::default();
        state.set_tier("pro");
        assert_eq!(state.tier, "pro");
    }

    #[test]
    fn default_preferences_match_expected_values() {
        let prefs = ForgePreferences::default();
        assert_eq!(prefs.theme, "system");
        assert_eq!(prefs.terminal_font_size, 13);
        assert_eq!(prefs.default_package_manager, "npm");
        assert!(prefs.auto_check_updates);
    }

    #[test]
    fn state_round_trips_through_json() {
        let mut state = ForgeState::default();
        state.set_tier("team");

        let json = serde_json::to_string(&state).unwrap();
        let restored: ForgeState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.schema_version, state.schema_version);
        assert_eq!(restored.tier, "team");
        assert_eq!(restored.preferences.theme, state.preferences.theme);
    }

    /// A state file written before schema_version was added should deserialise
    /// with the default value of 1 rather than failing.
    #[test]
    fn missing_schema_version_in_json_defaults_to_one() {
        let json = r#"{
            "projects": [],
            "workspaces": [],
            "build_presets": [],
            "build_history": [],
            "preferences": {
                "theme": "system",
                "terminal_font_size": 13,
                "default_package_manager": "npm",
                "auto_check_updates": true
            },
            "tier": "free"
        }"#;
        let state: ForgeState = serde_json::from_str(json).unwrap();
        assert_eq!(state.schema_version, 1);
    }
}
