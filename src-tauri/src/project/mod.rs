use std::fs;
use std::path::{Path, PathBuf};

use crate::backend::errors::ForgeError;

pub fn create_project_dir(path: &Path) -> Result<PathBuf, ForgeError> {
    fs::create_dir_all(path)?;
    Ok(path.to_path_buf())
}

pub fn ensure_directory(path: &Path) -> Result<(), ForgeError> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn list_directories(path: &Path) -> Result<Vec<PathBuf>, ForgeError> {
    let mut dirs = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            dirs.push(p);
        }
    }

    Ok(dirs)
}
