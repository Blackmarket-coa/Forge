use std::path::PathBuf;

/// Append `~/.cargo/bin` (Unix) / `%USERPROFILE%\.cargo\bin` (Windows) to
/// `PATH` when the directory exists and isn't already listed.
///
/// GUI launches don't run the user's shell rc files, and `fix_path_env::fix`
/// is a no-op on Windows, so rustup-installed tools can be invisible to the
/// process even though they are on the user's interactive PATH. Idempotent so
/// it can run before every environment probe, which also picks up toolchains
/// installed after the app started.
pub fn ensure_cargo_bin_in_path() {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    let Some(home) = home else { return };
    let cargo_bin = PathBuf::from(home).join(".cargo").join("bin");
    if !cargo_bin.is_dir() {
        return;
    }

    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = std::env::split_paths(&path).collect();
    if paths.iter().any(|p| p == &cargo_bin) {
        return;
    }

    paths.push(cargo_bin);
    if let Ok(joined) = std::env::join_paths(paths) {
        std::env::set_var("PATH", joined);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appends_cargo_bin_once() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".cargo").join("bin")).expect("mkdir");

        let old_home = std::env::var_os("HOME");
        let old_path = std::env::var_os("PATH");
        std::env::set_var("HOME", dir.path());

        ensure_cargo_bin_in_path();
        ensure_cargo_bin_in_path();

        let cargo_bin = dir.path().join(".cargo").join("bin");
        let path = std::env::var_os("PATH").unwrap_or_default();
        let occurrences = std::env::split_paths(&path)
            .filter(|p| p == &cargo_bin)
            .count();
        assert_eq!(occurrences, 1);

        match old_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        match old_path {
            Some(v) => std::env::set_var("PATH", v),
            None => std::env::remove_var("PATH"),
        }
    }
}
