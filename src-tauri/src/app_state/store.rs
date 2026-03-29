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
