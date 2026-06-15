use std::fs;
use std::path::PathBuf;

use log::{debug, info};

use crate::app_state::model::{ForgeState, STATE_SCHEMA_VERSION};
use crate::backend::errors::ForgeError;
use crate::backend::fs_util::write_atomic;

pub fn state_path() -> Result<PathBuf, ForgeError> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| ForgeError::ConfigInvalid("unable to resolve home directory".to_string()))?;
    Ok(home.join(".forge").join("forge.json"))
}

/// Load persisted state, bringing an older file up to the current
/// [`STATE_SCHEMA_VERSION`] when needed.
///
/// Migration strategy: as the schema evolves, add transforms in `migrate`
/// below that step an older value up one version at a time, so callers always
/// receive a fully up-to-date value regardless of how old the on-disk file is.
pub fn load_state() -> Result<ForgeState, ForgeError> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(ForgeState::default());
    }

    let content = fs::read_to_string(path)?;
    let state = serde_json::from_str::<ForgeState>(&content)?;
    let state = migrate(state)?;

    debug!("loaded state (schema_version={})", state.schema_version);
    Ok(state)
}

/// Upgrade an older on-disk state to the current schema. Today only version 1
/// exists; future versions add arms that mutate and bump `schema_version`.
fn migrate(state: ForgeState) -> Result<ForgeState, ForgeError> {
    if state.schema_version > STATE_SCHEMA_VERSION {
        return Err(ForgeError::ConfigInvalid(format!(
            "state schema version {} is newer than supported version {}; please upgrade Forge",
            state.schema_version, STATE_SCHEMA_VERSION
        )));
    }
    Ok(state)
}

pub fn save_state(state: &ForgeState) -> Result<(), ForgeError> {
    let path = state_path()?;

    let mut state = state.clone();
    state.schema_version = STATE_SCHEMA_VERSION;

    let content = serde_json::to_string_pretty(&state)?;
    write_atomic(&path, content.as_bytes())?;
    info!("state saved to {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialise env-mutating tests so they don't race each other.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_home<F: FnOnce()>(dir: &std::path::Path, f: F) {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var("HOME").ok();
        std::env::set_var("HOME", dir);
        f();
        match prev {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn state_path_ends_with_forge_json() {
        let dir = tempfile::tempdir().unwrap();
        with_home(dir.path(), || {
            let path = state_path().unwrap();
            assert!(path.ends_with(".forge/forge.json"));
        });
    }

    #[test]
    fn load_returns_default_when_no_file_exists() {
        let dir = tempfile::tempdir().unwrap();
        with_home(dir.path(), || {
            let state = load_state().unwrap();
            assert_eq!(state.schema_version, STATE_SCHEMA_VERSION);
            assert_eq!(state.tier, "free");
            assert!(state.projects.is_empty());
        });
    }

    #[test]
    fn save_and_load_roundtrip_preserves_data() {
        let dir = tempfile::tempdir().unwrap();
        with_home(dir.path(), || {
            let mut state = ForgeState::default();
            state.set_tier("pro");

            save_state(&state).unwrap();
            let loaded = load_state().unwrap();

            assert_eq!(loaded.tier, "pro");
            assert_eq!(loaded.schema_version, STATE_SCHEMA_VERSION);
        });
    }

    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn save_stamps_current_schema_version() {
        let dir = tempfile::tempdir().unwrap();
        with_home(dir.path(), || {
            // Write a state that has an older schema_version directly.
            let mut state = ForgeState::default();
            state.schema_version = 0; // pretend it's old
                                      // save_state should overwrite schema_version with STATE_SCHEMA_VERSION.
            save_state(&state).unwrap();
            let loaded = load_state().unwrap();
            assert_eq!(loaded.schema_version, STATE_SCHEMA_VERSION);
        });
    }
}
