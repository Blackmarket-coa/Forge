use std::fs;
use std::path::PathBuf;

use crate::app_state::model::ForgeState;
use crate::backend::errors::ForgeError;

pub fn state_path() -> Result<PathBuf, ForgeError> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| ForgeError::ConfigInvalid("unable to resolve home directory".to_string()))?;
    Ok(home.join(".forge").join("forge.json"))
}

pub fn load_state() -> Result<ForgeState, ForgeError> {
    let path = state_path()?;
    if !path.exists() {
        return Ok(ForgeState::default());
    }

    let content = fs::read_to_string(path)?;
    let state = serde_json::from_str::<ForgeState>(&content)?;
    Ok(state)
}

pub fn save_state(state: &ForgeState) -> Result<(), ForgeError> {
    let path = state_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(state)?;
    fs::write(path, content)?;
    Ok(())
}
