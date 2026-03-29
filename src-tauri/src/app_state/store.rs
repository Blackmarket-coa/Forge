use std::fs;
use std::path::PathBuf;

use log::{debug, info, warn};

use crate::app_state::model::{ForgeState, STATE_SCHEMA_VERSION};
use crate::backend::errors::ForgeError;

pub fn state_path() -> Result<PathBuf, ForgeError> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| ForgeError::ConfigInvalid("unable to resolve home directory".to_string()))?;
    Ok(home.join(".forge").join("forge.json"))
}

/// Load persisted state, applying any schema migrations required to bring an
/// older file up to the current [`STATE_SCHEMA_VERSION`].
///
/// Migration strategy: each version bump adds an arm to the `while` loop
/// below.  Migrations run in order until the state reaches the current
/// version, so callers always receive a fully up-to-date value regardless of
/// how old the on-disk file is.
pub fn load_state() -> Result<ForgeState, ForgeError> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(ForgeState::default());
    }

    let content = fs::read_to_string(path)?;
    let mut state = serde_json::from_str::<ForgeState>(&content)?;

    // Run incremental migrations.
    if state.schema_version < STATE_SCHEMA_VERSION {
        warn!(
            "state file is schema version {}; current version is {} — migrating",
            state.schema_version, STATE_SCHEMA_VERSION
        );
    }
    while state.schema_version < STATE_SCHEMA_VERSION {
        match state.schema_version {
            // Example migration template — add real migrations here as the
            // schema evolves:
            //
            //   1 => { state.new_field = default_value(); state.schema_version = 2; }
            //
            v => {
                return Err(ForgeError::ConfigInvalid(format!(
                    "unknown state schema version {v}; please upgrade Forge"
                )))
            }
        }
    }

    debug!("loaded state (schema_version={})", state.schema_version);
    Ok(state)
}

pub fn save_state(state: &ForgeState) -> Result<(), ForgeError> {
    let path = state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut state = state.clone();
    state.schema_version = STATE_SCHEMA_VERSION;

    let content = serde_json::to_string_pretty(&state)?;
    fs::write(&path, content)?;
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
