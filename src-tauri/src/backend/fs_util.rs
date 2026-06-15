use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::backend::errors::ForgeError;

/// Write `bytes` to `path` atomically.
///
/// The data is written to a temporary file in the same directory, flushed and
/// synced to disk, then renamed over the destination. A rename within the same
/// filesystem is atomic, so a reader either sees the previous file or the new
/// one in full — never a half-written file, even if the process crashes mid
/// write. This protects `forge.json`, `tauri.conf.json`, and the license/
/// history caches from corruption.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), ForgeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = tmp_path(path);
    {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }

    // Rename over the destination. On failure, clean up the temp file so we
    // don't leave litter behind.
    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }

    Ok(())
}

fn tmp_path(path: &Path) -> std::path::PathBuf {
    let pid = std::process::id();
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(format!(".{pid}.tmp"));
    match path.parent() {
        Some(parent) => parent.join(name),
        None => std::path::PathBuf::from(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_atomic_creates_file_and_parent() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("nested").join("data.json");
        write_atomic(&target, b"{\"ok\":true}").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "{\"ok\":true}");
    }

    #[test]
    fn write_atomic_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("data.json");
        write_atomic(&target, b"old").unwrap();
        write_atomic(&target, b"new").unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "new");
    }

    #[test]
    fn write_atomic_leaves_no_temp_files() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("data.json");
        write_atomic(&target, b"x").unwrap();
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(entries, vec!["data.json".to_string()]);
    }
}
