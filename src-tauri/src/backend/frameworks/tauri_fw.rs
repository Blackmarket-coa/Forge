//! Tauri framework adapter: desktop apps with a Rust core and the system
//! webview. This is Forge's original framework; the adapter wraps the same
//! behavior the app shipped with (scaffold templates in
//! [`crate::backend::web_app`], `cargo tauri dev/build`, artifacts under
//! `src-tauri/target/release/bundle`).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use url::Url;

use crate::backend::errors::ForgeError;
use crate::backend::frameworks::{
    BundleKind, CommandSpec, Framework, FrameworkAdapter, FrameworkInfo, TargetStatus, ToolCheck,
    ToolProbe,
};
use crate::backend::web_app::{scaffold_tauri_target, WebAppOptions};

pub struct TauriAdapter;

/// Locate a target's `tauri.conf.json` (nested `src-tauri/` layout or flat).
pub fn tauri_conf_path(target_dir: &Path) -> Option<PathBuf> {
    let nested = target_dir.join("src-tauri").join("tauri.conf.json");
    let flat = target_dir.join("tauri.conf.json");
    if nested.exists() {
        Some(nested)
    } else if flat.exists() {
        Some(flat)
    } else {
        None
    }
}

/// Read the `tauri` dependency version out of the target's Cargo.toml.
fn detect_tauri_dependency_version(target_dir: &Path) -> Option<String> {
    let tauri_dir = if target_dir.join("src-tauri").exists() {
        target_dir.join("src-tauri")
    } else {
        target_dir.to_path_buf()
    };
    let cargo_toml = tauri_dir.join("Cargo.toml");
    let content = fs::read_to_string(cargo_toml).ok()?;
    let parsed: toml::Value = toml::from_str(&content).ok()?;

    match parsed.get("dependencies").and_then(|v| v.get("tauri")) {
        Some(toml::Value::String(v)) => Some(v.clone()),
        Some(toml::Value::Table(t)) => t
            .get("version")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        _ => None,
    }
}

/// Validation rules for `tauri.conf.json`, shared by the config editor and
/// the deploy dashboard.
pub fn validate_tauri_config(target_dir: &Path, config: &Value) -> Vec<String> {
    let mut issues = Vec::new();

    if config
        .get("productName")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        != Some(true)
    {
        issues.push("Missing or empty productName".to_string());
    }

    let identifier = config
        .get("identifier")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    if identifier.is_empty() {
        issues.push("Missing identifier".to_string());
    } else if !super::is_reverse_domain(identifier) {
        issues.push(
            "identifier should match reverse-domain format (e.g. com.example.app)".to_string(),
        );
    }

    if let Some(windows) = config
        .get("app")
        .and_then(|v| v.get("windows"))
        .and_then(|v| v.as_array())
    {
        for (i, w) in windows.iter().enumerate() {
            let width = w.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let height = w.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if width <= 0.0 {
                issues.push(format!("window[{i}] width must be positive"));
            }
            if height <= 0.0 {
                issues.push(format!("window[{i}] height must be positive"));
            }
        }
    }

    if let Some(dev_url) = config
        .get("build")
        .and_then(|v| v.get("devUrl"))
        .and_then(|v| v.as_str())
    {
        if Url::parse(dev_url).is_err() {
            issues.push("build.devUrl must be a valid URL".to_string());
        }
    }

    if let Some(frontend_dist) = config
        .get("build")
        .and_then(|v| v.get("frontendDist"))
        .and_then(|v| v.as_str())
    {
        let path = target_dir.join(frontend_dist);
        // frontendDist is relative to the folder holding tauri.conf.json in
        // the nested layout; accept either resolution to avoid false alarms.
        let nested = target_dir.join("src-tauri").join(frontend_dist);
        if !path.exists() && !nested.exists() {
            issues.push(format!(
                "build.frontendDist path does not exist: {frontend_dist}"
            ));
        }
    }

    issues
}

impl FrameworkAdapter for TauriAdapter {
    fn framework(&self) -> Framework {
        Framework::Tauri
    }

    fn info(&self) -> FrameworkInfo {
        FrameworkInfo {
            id: "tauri",
            label: "Desktop app (Tauri)",
            tagline: "Small, fast desktop app for Windows, Mac, and Linux.",
            platforms: &["macOS", "Linux", "Windows"],
            bundle_kinds: vec![
                BundleKind {
                    id: "dmg",
                    label: "macOS app (.dmg)",
                    platform: "macOS",
                    note: Some("Build on a Mac"),
                },
                BundleKind {
                    id: "appimage",
                    label: "Linux app (AppImage)",
                    platform: "Linux",
                    note: None,
                },
                BundleKind {
                    id: "deb",
                    label: "Linux app (.deb)",
                    platform: "Linux",
                    note: None,
                },
                BundleKind {
                    id: "msi",
                    label: "Windows app (.msi)",
                    platform: "Windows",
                    note: Some("Build on Windows"),
                },
                BundleKind {
                    id: "nsis",
                    label: "Windows installer (.exe)",
                    platform: "Windows",
                    note: Some("Build on Windows"),
                },
            ],
            tools: vec![
                ToolCheck {
                    name: "rust",
                    label: "Rust",
                    probe: ToolProbe::Command {
                        program: "rustc",
                        args: &["--version"],
                    },
                    install_hint: "Install Rust from https://www.rust-lang.org/tools/install",
                    platform_only: None,
                },
                ToolCheck {
                    name: "cargo",
                    label: "Cargo",
                    probe: ToolProbe::Command {
                        program: "cargo",
                        args: &["--version"],
                    },
                    install_hint: "Cargo comes with Rust — install Rust first",
                    platform_only: None,
                },
                ToolCheck {
                    name: "tauri_cli",
                    label: "Tauri CLI",
                    probe: ToolProbe::Command {
                        program: "cargo",
                        args: &["tauri", "--version"],
                    },
                    install_hint: "Run `cargo install tauri-cli --version \"^2\"` in a terminal",
                    platform_only: None,
                },
                ToolCheck {
                    name: "webkit2gtk",
                    label: "webkit2gtk-4.1 development libraries",
                    probe: ToolProbe::PkgConfig {
                        name: "webkit2gtk-4.1",
                        dpkg_fallback: "libwebkit2gtk-4.1-dev",
                    },
                    install_hint:
                        "Install your distro's webkit2gtk 4.1 dev package (e.g. `sudo apt install libwebkit2gtk-4.1-dev`)",
                    platform_only: Some("linux"),
                },
            ],
            config_file: "tauri.conf.json",
            dev_label: "Preview app",
            dev_available: true,
        }
    }

    fn default_dir(&self) -> &'static str {
        "."
    }

    fn detect(&self, project_root: &Path) -> Option<TargetStatus> {
        let conf_path = tauri_conf_path(project_root)?;
        let mut product_name = None;
        let mut identifier = None;
        let mut source_url = None;
        let mut config_ok = false;
        let mut config_issues = Vec::new();

        match fs::read_to_string(&conf_path)
            .map_err(ForgeError::from)
            .and_then(|c| serde_json::from_str::<Value>(&c).map_err(ForgeError::from))
        {
            Ok(value) => {
                product_name = value
                    .get("productName")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                identifier = value
                    .get("identifier")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                source_url = value
                    .get("app")
                    .and_then(|v| v.get("windows"))
                    .and_then(|v| v.get(0))
                    .and_then(|v| v.get("url"))
                    .and_then(|v| v.as_str())
                    .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
                    .map(str::to_string);
                config_issues = validate_tauri_config(project_root, &value);
                config_ok = config_issues.is_empty();
            }
            Err(e) => config_issues.push(format!("tauri.conf.json could not be read: {e}")),
        }

        let version = detect_tauri_dependency_version(project_root);
        let status = if version.is_some() && config_ok {
            "ready"
        } else {
            "needs_config"
        };

        Some(TargetStatus {
            framework: "tauri".to_string(),
            dir: ".".to_string(),
            product_name,
            identifier,
            source_url,
            version,
            config_ok,
            config_issues,
            status: status.to_string(),
        })
    }

    fn scaffold(&self, project_root: &Path, opts: &WebAppOptions) -> Result<PathBuf, ForgeError> {
        scaffold_tauri_target(project_root, opts)
    }

    fn readme_section(&self) -> &'static str {
        "## Desktop app (Tauri)\n\n\
         Lives in `src-tauri/`. You need the free Tauri build tools:\n\n\
         1. Install Rust: <https://www.rust-lang.org/tools/install>\n\
         2. Install the Tauri CLI: `cargo install tauri-cli --version \"^2\"`\n\n\
         Preview with `cargo tauri dev`, or build an installer with\n\
         `cargo tauri build` from this folder. Finished installers appear in\n\
         `src-tauri/target/release/bundle/`.\n"
    }

    fn dev_steps(&self, target_dir: &Path) -> Vec<CommandSpec> {
        vec![CommandSpec::new("cargo", vec!["tauri", "dev"], target_dir)]
    }

    fn build_steps(
        &self,
        target_dir: &Path,
        bundle_kind: &str,
    ) -> Result<Vec<CommandSpec>, ForgeError> {
        Ok(vec![CommandSpec::new(
            "cargo",
            vec!["tauri", "build", "--bundles", bundle_kind],
            target_dir,
        )])
    }

    fn artifact_dirs(&self, target_dir: &Path) -> Vec<PathBuf> {
        vec![target_dir
            .join("src-tauri")
            .join("target")
            .join("release")
            .join("bundle")]
    }

    fn config_path(&self, target_dir: &Path) -> Result<PathBuf, ForgeError> {
        tauri_conf_path(target_dir)
            .ok_or_else(|| ForgeError::ConfigNotFound(target_dir.display().to_string()))
    }

    fn validate_config(&self, target_dir: &Path, config: &Value) -> Vec<String> {
        validate_tauri_config(target_dir, config)
    }

    fn platform_for_artifact(&self, extension: &str) -> Option<&'static str> {
        match extension {
            "dmg" | "app" => Some("macOS"),
            "AppImage" | "appimage" | "deb" | "rpm" => Some("Linux"),
            "msi" | "exe" | "nsis" => Some("Windows"),
            "ipa" => Some("iOS"),
            "apk" | "aab" => Some("Android"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::frameworks::FrameworkAdapter;

    fn scaffold_min(root: &Path, product: &str) {
        let tauri = root.join("src-tauri");
        fs::create_dir_all(&tauri).unwrap();
        fs::write(
            tauri.join("tauri.conf.json"),
            format!(
                r#"{{ "productName": "{product}", "identifier": "com.example.{product}", "version": "1.2.3",
                     "app": {{ "windows": [{{ "url": "https://example.com/", "width": 800, "height": 600 }}] }} }}"#
            ),
        )
        .unwrap();
        fs::write(
            tauri.join("Cargo.toml"),
            "[dependencies]\ntauri = { version = \"2.1.0\" }\n",
        )
        .unwrap();
    }

    #[test]
    fn detect_reads_conf_and_dependency() {
        let dir = tempfile::tempdir().unwrap();
        scaffold_min(dir.path(), "demo");

        let status = TauriAdapter.detect(dir.path()).unwrap();
        assert_eq!(status.framework, "tauri");
        assert_eq!(status.product_name.as_deref(), Some("demo"));
        assert_eq!(status.identifier.as_deref(), Some("com.example.demo"));
        assert_eq!(status.version.as_deref(), Some("2.1.0"));
        assert_eq!(status.source_url.as_deref(), Some("https://example.com/"));
        assert_eq!(status.status, "ready");
    }

    #[test]
    fn detect_returns_none_without_conf() {
        let dir = tempfile::tempdir().unwrap();
        assert!(TauriAdapter.detect(dir.path()).is_none());
    }

    #[test]
    fn validate_flags_missing_fields() {
        let dir = tempfile::tempdir().unwrap();
        let issues = validate_tauri_config(dir.path(), &serde_json::json!({}));
        assert!(issues.iter().any(|i| i.contains("productName")));
        assert!(issues.iter().any(|i| i.contains("identifier")));
    }

    #[test]
    fn validate_flags_bad_identifier_and_window() {
        let dir = tempfile::tempdir().unwrap();
        let config = serde_json::json!({
            "productName": "App",
            "identifier": "notreverse",
            "app": { "windows": [{ "width": 0, "height": -1 }] }
        });
        let issues = validate_tauri_config(dir.path(), &config);
        assert!(issues.iter().any(|i| i.contains("reverse-domain")));
        assert!(issues.iter().any(|i| i.contains("width")));
        assert!(issues.iter().any(|i| i.contains("height")));
    }

    #[test]
    fn build_steps_use_cargo_tauri() {
        let dir = tempfile::tempdir().unwrap();
        let steps = TauriAdapter.build_steps(dir.path(), "deb").unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].program, "cargo");
        assert_eq!(steps[0].args, vec!["tauri", "build", "--bundles", "deb"]);
    }
}
