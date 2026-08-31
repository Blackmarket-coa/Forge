//! Framework adapter layer.
//!
//! Forge supports several app frameworks (Tauri, Capacitor, Electron, PWA,
//! React Native/Expo). Everything the rest of the backend needs to do with a
//! framework — generate a target, detect one on disk, run dev/build commands,
//! locate artifacts, edit config, check required tools — goes through the
//! [`FrameworkAdapter`] trait so IPC handlers never special-case a framework.
//!
//! A *project* is one folder wrapping one website. Each framework the user
//! picked lives in its own target directory inside that folder: Tauri keeps
//! its historical root-level layout (`src-tauri/` in the project root), while
//! the other frameworks scaffold into `capacitor/`, `electron/`, `pwa/`, and
//! `react-native/`. The `forge.project.json` manifest (see
//! [`crate::backend::project_manager`]) records the declared targets;
//! detection falls back to probing marker files for projects created outside
//! Forge.

pub mod tauri_fw;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backend::errors::ForgeError;
use crate::backend::fs_util::write_atomic;
use crate::backend::web_app::{normalize_url, slug, WebAppOptions};

/// Every app framework Forge can generate and manage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Framework {
    Tauri,
    Capacitor,
    Electron,
    Pwa,
    ReactNative,
}

impl Framework {
    pub const ALL: [Framework; 5] = [
        Framework::Tauri,
        Framework::Capacitor,
        Framework::Electron,
        Framework::Pwa,
        Framework::ReactNative,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Framework::Tauri => "tauri",
            Framework::Capacitor => "capacitor",
            Framework::Electron => "electron",
            Framework::Pwa => "pwa",
            Framework::ReactNative => "react-native",
        }
    }

    pub fn from_id(id: &str) -> Option<Framework> {
        Self::ALL.into_iter().find(|f| f.id() == id)
    }
}

/// One kind of installable output a framework can produce (e.g. a `.dmg`).
#[derive(Debug, Clone, Serialize)]
pub struct BundleKind {
    /// Stable id passed to `run_build` (e.g. "dmg", "apk", "zip").
    pub id: &'static str,
    /// Plain-language label shown to the user.
    pub label: &'static str,
    /// The platform this output is for ("macOS", "Linux", "Windows", "iOS",
    /// "Android", "Web") — matches the deploy dashboard's columns.
    pub platform: &'static str,
    /// Honest caveat shown next to the option (e.g. "Needs a Mac with Xcode").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<&'static str>,
}

/// How to probe whether a required tool is present.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ToolProbe {
    /// Run a command; success = installed. Version comes from the first line
    /// of stdout (or stderr — `java -version` prints there).
    Command {
        program: &'static str,
        args: &'static [&'static str],
    },
    /// Any of these environment variables pointing at an existing directory
    /// counts as installed (e.g. ANDROID_HOME for the Android SDK).
    EnvDir { vars: &'static [&'static str] },
    /// `pkg-config --exists <name>`, with a dpkg package fallback for
    /// Debian/Ubuntu systems without pkg-config.
    PkgConfig {
        name: &'static str,
        dpkg_fallback: &'static str,
    },
}

/// A tool a framework needs on this computer to preview or build apps.
#[derive(Debug, Clone, Serialize)]
pub struct ToolCheck {
    /// Stable id, shared across frameworks so checks dedupe (e.g. "node").
    pub name: &'static str,
    /// Plain-language label (e.g. "Node.js").
    pub label: &'static str,
    pub probe: ToolProbe,
    /// Plain-language instruction for installing the tool.
    pub install_hint: &'static str,
    /// Only relevant on this OS ("macos" | "linux" | "windows"); None = all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_only: Option<&'static str>,
}

impl ToolCheck {
    /// Whether this check applies to the OS Forge is running on.
    pub fn applies_here(&self) -> bool {
        match self.platform_only {
            None => true,
            Some("macos") => cfg!(target_os = "macos"),
            Some("linux") => cfg!(target_os = "linux"),
            Some("windows") => cfg!(target_os = "windows"),
            Some(_) => true,
        }
    }

    /// Run the probe now. Returns (installed, version-or-location text).
    pub fn run_probe(&self) -> (bool, String) {
        use std::process::Command;
        match &self.probe {
            ToolProbe::Command { program, args } => {
                match Command::new(program).args(*args).output() {
                    Ok(out) if out.status.success() => {
                        // `java -version` prints to stderr; take the first
                        // non-empty line from either stream.
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        let line = stdout
                            .lines()
                            .chain(stderr.lines())
                            .find(|l| !l.trim().is_empty())
                            .unwrap_or("")
                            .trim()
                            .to_string();
                        (true, line)
                    }
                    _ => (false, "not found".to_string()),
                }
            }
            ToolProbe::EnvDir { vars } => {
                for var in *vars {
                    if let Some(val) = std::env::var_os(var) {
                        let path = PathBuf::from(&val);
                        if path.is_dir() {
                            return (true, val.to_string_lossy().to_string());
                        }
                    }
                }
                (false, "not found".to_string())
            }
            ToolProbe::PkgConfig {
                name,
                dpkg_fallback,
            } => {
                // pkg-config probes the actual library and works across
                // distros; the dpkg fallback keeps Debian/Ubuntu covered when
                // pkg-config itself isn't installed.
                let via_pkg_config = Command::new("pkg-config")
                    .args(["--exists", name])
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false);
                let installed = via_pkg_config
                    || Command::new("dpkg")
                        .args(["-l", dpkg_fallback])
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false);
                let text = if installed { "installed" } else { "not found" };
                (installed, text.to_string())
            }
        }
    }
}

/// Static description of a framework, served to the frontend by
/// `get_frameworks` as the single source of truth for labels, bundle kinds,
/// dev affordances, and required tools.
#[derive(Debug, Clone, Serialize)]
pub struct FrameworkInfo {
    pub id: &'static str,
    /// Product label (e.g. "Desktop app (Tauri)").
    pub label: &'static str,
    /// One-line plain-language pitch shown in the creation wizard.
    pub tagline: &'static str,
    /// Platforms apps of this framework run on.
    pub platforms: &'static [&'static str],
    pub bundle_kinds: Vec<BundleKind>,
    pub tools: Vec<ToolCheck>,
    /// File name of the target's editable config (relative to its directory).
    pub config_file: &'static str,
    /// Label for the preview/dev action (e.g. "Preview on Android device").
    pub dev_label: &'static str,
    /// False when the framework has no dev loop (PWA).
    pub dev_available: bool,
}

/// One step of an external command sequence (dev servers, builds).
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

impl CommandSpec {
    pub fn new<P: Into<String>, A: Into<String>>(
        program: P,
        args: Vec<A>,
        cwd: &Path,
    ) -> CommandSpec {
        CommandSpec {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            cwd: cwd.to_path_buf(),
        }
    }
}

/// What detection learned about one framework target inside a project.
#[derive(Debug, Clone, Serialize)]
pub struct TargetStatus {
    pub framework: String,
    /// Target directory relative to the project root ("." for Tauri's
    /// root-level layout).
    pub dir: String,
    pub product_name: Option<String>,
    pub identifier: Option<String>,
    /// Website address the target wraps, when the config declares one.
    pub source_url: Option<String>,
    /// Detected framework/dependency version, when cheap to read.
    pub version: Option<String>,
    pub config_ok: bool,
    pub config_issues: Vec<String>,
    /// "ready" | "needs_config" | "error" — same scale the UI already uses.
    pub status: String,
}

pub trait FrameworkAdapter: Sync {
    fn framework(&self) -> Framework;
    fn info(&self) -> FrameworkInfo;

    /// Directory (relative to the project root) this framework scaffolds into.
    /// "." means the target shares the project root (Tauri's layout).
    fn default_dir(&self) -> &'static str;

    /// Probe `project_root` for a target of this framework. Checks both the
    /// root itself (standalone projects the user registered) and
    /// [`Self::default_dir`].
    fn detect(&self, project_root: &Path) -> Option<TargetStatus>;

    /// Write a complete, build-ready target into the project. Pure inline
    /// templates: no network access and no package manager involvement.
    /// Returns the target directory. Must refuse to clobber an existing
    /// non-empty target directory.
    fn scaffold(&self, project_root: &Path, opts: &WebAppOptions) -> Result<PathBuf, ForgeError>;

    /// Markdown section for the generated project README explaining, in plain
    /// language, how to preview and build this target.
    fn readme_section(&self) -> &'static str;

    /// Commands that start this target's preview/dev loop. Empty when the
    /// framework has no dev loop (PWA).
    fn dev_steps(&self, target_dir: &Path) -> Vec<CommandSpec>;

    /// External commands that build the given bundle kind, in order.
    fn build_steps(
        &self,
        target_dir: &Path,
        bundle_kind: &str,
    ) -> Result<Vec<CommandSpec>, ForgeError>;

    /// Perform any in-process build work for a bundle kind (e.g. the PWA zip,
    /// which needs no external toolchain). Returns true when the kind was
    /// fully handled here and `build_steps` should be skipped.
    fn build_in_process(&self, _target_dir: &Path, _bundle_kind: &str) -> Result<bool, ForgeError> {
        Ok(false)
    }

    /// Directories to scan for installers/artifacts (may not exist yet).
    fn artifact_dirs(&self, target_dir: &Path) -> Vec<PathBuf>;

    /// Path of the target's editable config file.
    fn config_path(&self, target_dir: &Path) -> Result<PathBuf, ForgeError>;

    /// Plain-language issues with a proposed config value (empty = valid).
    fn validate_config(&self, target_dir: &Path, config: &Value) -> Vec<String>;

    /// Map an artifact file extension to the platform it installs on, for the
    /// deploy dashboard ("dmg" → "macOS"). None when the extension isn't one
    /// of this framework's installable outputs.
    fn platform_for_artifact(&self, extension: &str) -> Option<&'static str>;
}

/// Look up the adapter for a framework.
pub fn adapter(framework: Framework) -> &'static dyn FrameworkAdapter {
    match framework {
        Framework::Tauri => &tauri_fw::TauriAdapter,
        // The remaining adapters are registered as they are implemented.
        _ => &tauri_fw::TauriAdapter,
    }
}

/// All adapters, in the order frameworks are shown to the user.
pub fn adapters() -> Vec<&'static dyn FrameworkAdapter> {
    registered_frameworks().into_iter().map(adapter).collect()
}

/// Frameworks that currently have a real adapter behind [`adapter`].
pub fn registered_frameworks() -> Vec<Framework> {
    vec![Framework::Tauri]
}

/// Whether an app identifier looks like reverse-domain form
/// (e.g. `com.example.app`) — shared validation across frameworks.
pub(crate) fn is_reverse_domain(identifier: &str) -> bool {
    let parts: Vec<&str> = identifier.split('.').collect();
    if parts.len() < 3 {
        return false;
    }
    parts.iter().all(|p| {
        !p.is_empty()
            && p.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    })
}

/// Resolve a target's directory entry ("." or a subdir name) to an absolute
/// path under the project root.
pub fn target_path(project_root: &Path, dir: &str) -> PathBuf {
    if dir == "." || dir.is_empty() {
        project_root.to_path_buf()
    } else {
        project_root.join(dir)
    }
}

/// Probe every registered adapter and collect the targets found in a project.
pub fn detect_targets(project_root: &Path) -> Vec<TargetStatus> {
    adapters()
        .iter()
        .filter_map(|a| a.detect(project_root))
        .collect()
}

/// Locate a framework's target directory inside a project: the manifest's
/// declared entry when present, else whatever detection finds on disk.
pub fn resolve_target_dir(
    project_root: &Path,
    framework: Framework,
) -> Result<PathBuf, ForgeError> {
    if let Some(manifest) = crate::backend::project_manager::read_manifest(project_root)? {
        if let Some(declared) = manifest
            .targets
            .iter()
            .find(|t| t.framework == framework.id())
        {
            return Ok(target_path(project_root, &declared.dir));
        }
    }

    if let Some(status) = adapter(framework).detect(project_root) {
        return Ok(target_path(project_root, &status.dir));
    }

    Err(ForgeError::ConfigNotFound(format!(
        "this project has no {} app",
        adapter(framework).info().label
    )))
}

/// `npm` / `npx` executables (Windows needs the `.cmd` shims).
pub fn npm_program() -> &'static str {
    if cfg!(windows) {
        "npm.cmd"
    } else {
        "npm"
    }
}

pub fn npx_program() -> &'static str {
    if cfg!(windows) {
        "npx.cmd"
    } else {
        "npx"
    }
}

/// `npm install` step, emitted only while `node_modules` is missing so repeat
/// previews/builds skip straight to the real work.
pub fn npm_install_step(target_dir: &Path) -> Option<CommandSpec> {
    if target_dir.join("node_modules").exists() {
        None
    } else {
        Some(CommandSpec::new(npm_program(), vec!["install"], target_dir))
    }
}

/// The Gradle wrapper inside a generated Android project.
pub fn gradlew_step(android_dir: &Path, task: &str) -> CommandSpec {
    let program = if cfg!(windows) {
        android_dir.join("gradlew.bat")
    } else {
        android_dir.join("gradlew")
    };
    CommandSpec::new(
        program.to_string_lossy().to_string(),
        vec![task.to_string()],
        android_dir,
    )
}

/// Shared tool checks, referenced by several adapters so the environment
/// screen can dedupe them by name.
pub mod tools {
    use super::{ToolCheck, ToolProbe};

    pub const NODE: ToolCheck = ToolCheck {
        name: "node",
        label: "Node.js",
        probe: ToolProbe::Command {
            program: "node",
            args: &["--version"],
        },
        install_hint: "Install Node.js (LTS) from https://nodejs.org",
        platform_only: None,
    };

    pub const NPM: ToolCheck = ToolCheck {
        name: "npm",
        label: "npm",
        probe: ToolProbe::Command {
            program: if cfg!(windows) { "npm.cmd" } else { "npm" },
            args: &["--version"],
        },
        install_hint: "npm comes with Node.js — install Node.js from https://nodejs.org",
        platform_only: None,
    };

    pub const JAVA: ToolCheck = ToolCheck {
        name: "java",
        label: "Java (JDK 17+)",
        probe: ToolProbe::Command {
            program: "java",
            args: &["-version"],
        },
        install_hint: "Install a JDK, e.g. from https://adoptium.net (needed for Android builds)",
        platform_only: None,
    };

    pub const ANDROID_SDK: ToolCheck = ToolCheck {
        name: "android_sdk",
        label: "Android SDK",
        probe: ToolProbe::EnvDir {
            vars: &["ANDROID_HOME", "ANDROID_SDK_ROOT"],
        },
        install_hint:
            "Install Android Studio from https://developer.android.com/studio (sets up the SDK)",
        platform_only: None,
    };

    pub const XCODE: ToolCheck = ToolCheck {
        name: "xcode",
        label: "Xcode",
        probe: ToolProbe::Command {
            program: "xcodebuild",
            args: &["-version"],
        },
        install_hint: "Install Xcode from the Mac App Store (needed for iPhone/iPad builds)",
        platform_only: Some("macos"),
    };
}

/// Scaffold a complete multi-target project wrapping `opts.url`.
///
/// Creates `<parent_dir>/<slug>` (refusing a non-empty existing folder),
/// scaffolds every requested framework target, writes the
/// `forge.project.json` manifest, and finishes with a project README and
/// `.gitignore` covering all targets. Returns the project directory.
pub fn scaffold_project(
    parent_dir: &Path,
    opts: &WebAppOptions,
    frameworks: &[Framework],
) -> Result<PathBuf, ForgeError> {
    use crate::backend::project_manager::{ManifestTarget, ProjectManifest};

    let normalized_url = normalize_url(&opts.url)?;
    let name = opts.name.trim();
    if name.is_empty() {
        return Err(ForgeError::ConfigInvalid(
            "Please give your app a name.".to_string(),
        ));
    }
    if frameworks.is_empty() {
        return Err(ForgeError::ConfigInvalid(
            "Pick at least one kind of app to create.".to_string(),
        ));
    }

    // Dedupe while preserving the caller's order.
    let mut chosen: Vec<Framework> = Vec::new();
    for fw in frameworks {
        if !chosen.contains(fw) {
            chosen.push(*fw);
        }
    }

    let dir_slug = slug(name);
    let project_dir = parent_dir.join(&dir_slug);
    if project_dir.exists()
        && std::fs::read_dir(&project_dir)
            .map(|mut it| it.next().is_some())
            .unwrap_or(false)
    {
        return Err(ForgeError::ConfigInvalid(format!(
            "A folder named \"{dir_slug}\" already exists here. \
             Pick a different app name or save location."
        )));
    }

    let mut manifest_targets = Vec::new();
    for fw in &chosen {
        let a = adapter(*fw);
        a.scaffold(&project_dir, opts)?;
        manifest_targets.push(ManifestTarget {
            framework: fw.id().to_string(),
            dir: a.default_dir().to_string(),
        });
    }

    let manifest = ProjectManifest {
        schema_version: 1,
        name: name.to_string(),
        identifier: Some(
            opts.identifier
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| crate::backend::web_app::derive_identifier(name)),
        ),
        source_url: Some(normalized_url.clone()),
        targets: manifest_targets,
    };
    crate::backend::project_manager::write_manifest(&project_dir, &manifest)?;

    write_project_readme(&project_dir, name, &normalized_url, &chosen)?;
    write_project_gitignore(&project_dir)?;

    Ok(project_dir)
}

/// Add one more framework target to an existing project and record it in the
/// manifest (creating the manifest for legacy single-target projects).
pub fn add_target_to_project(
    project_root: &Path,
    framework: Framework,
    opts: &WebAppOptions,
) -> Result<PathBuf, ForgeError> {
    use crate::backend::project_manager::{read_manifest, write_manifest, ManifestTarget};

    let a = adapter(framework);
    if a.detect(project_root).is_some() {
        return Err(ForgeError::ConfigInvalid(format!(
            "This project already has a {} app.",
            a.info().label
        )));
    }

    let target_dir = a.scaffold(project_root, opts)?;

    let mut manifest = read_manifest(project_root)?.unwrap_or_else(|| {
        // Legacy project without a manifest: seed it from what's on disk.
        let targets = detect_targets(project_root)
            .into_iter()
            .filter(|t| t.framework != framework.id())
            .map(|t| ManifestTarget {
                framework: t.framework,
                dir: t.dir,
            })
            .collect();
        crate::backend::project_manager::ProjectManifest {
            schema_version: 1,
            name: opts.name.trim().to_string(),
            identifier: None,
            source_url: normalize_url(&opts.url).ok(),
            targets,
        }
    });

    if !manifest
        .targets
        .iter()
        .any(|t| t.framework == framework.id())
    {
        manifest.targets.push(ManifestTarget {
            framework: framework.id().to_string(),
            dir: a.default_dir().to_string(),
        });
    }
    write_manifest(project_root, &manifest)?;

    Ok(target_dir)
}

fn write_project_readme(
    project_dir: &Path,
    name: &str,
    url: &str,
    frameworks: &[Framework],
) -> Result<(), ForgeError> {
    let mut sections = String::new();
    for fw in frameworks {
        sections.push_str(adapter(*fw).readme_section());
        sections.push('\n');
    }

    let readme = format!(
        "# {name}\n\n\
         Apps for **{url}**, created with Forge. Each app opens your website —\n\
         update the site and every app shows the new version automatically.\n\n\
         Forge can preview and build everything below from its interface; the\n\
         sections explain what happens under the hood and how to do it by hand.\n\n\
         {sections}\
         ## Change the app icon\n\n\
         Replace the icon images inside each app folder with your own, then\n\
         build again.\n",
    );
    write_atomic(&project_dir.join("README.md"), readme.as_bytes())
}

fn write_project_gitignore(project_dir: &Path) -> Result<(), ForgeError> {
    let gitignore = "# Build output\n\
        /src-tauri/target\n\
        node_modules/\n\
        /capacitor/android\n\
        /capacitor/ios\n\
        /electron/dist\n\
        /pwa/dist\n\
        /react-native/android\n\
        /react-native/ios\n\
        /react-native/.expo\n";
    write_atomic(&project_dir.join(".gitignore"), gitignore.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::project_manager::read_manifest;

    fn opts() -> WebAppOptions {
        WebAppOptions {
            name: "My Store".to_string(),
            url: "mystore.com".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn framework_ids_round_trip() {
        for fw in Framework::ALL {
            assert_eq!(Framework::from_id(fw.id()), Some(fw));
        }
        assert_eq!(
            Framework::from_id("react-native"),
            Some(Framework::ReactNative)
        );
        assert!(Framework::from_id("flutter").is_none());
    }

    #[test]
    fn scaffold_project_writes_manifest_readme_and_target() {
        let dir = tempfile::tempdir().unwrap();
        let project = scaffold_project(dir.path(), &opts(), &[Framework::Tauri]).unwrap();
        assert_eq!(project, dir.path().join("my-store"));

        for rel in [
            "forge.project.json",
            "README.md",
            ".gitignore",
            "src-tauri/tauri.conf.json",
            "dist/index.html",
        ] {
            assert!(project.join(rel).exists(), "missing {rel}");
        }

        let manifest = read_manifest(&project).unwrap().unwrap();
        assert_eq!(manifest.name, "My Store");
        assert_eq!(manifest.source_url.as_deref(), Some("https://mystore.com/"));
        assert_eq!(manifest.identifier.as_deref(), Some("com.forge.mystore"));
        assert_eq!(manifest.targets.len(), 1);
        assert_eq!(manifest.targets[0].framework, "tauri");
        assert_eq!(manifest.targets[0].dir, ".");

        let readme = std::fs::read_to_string(project.join("README.md")).unwrap();
        assert!(readme.contains("https://mystore.com/"));
        assert!(readme.contains("Desktop app (Tauri)"));
    }

    #[test]
    fn scaffold_project_refuses_non_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("my-store");
        std::fs::create_dir_all(&existing).unwrap();
        std::fs::write(existing.join("keep.txt"), b"hi").unwrap();
        assert!(scaffold_project(dir.path(), &opts(), &[Framework::Tauri]).is_err());
    }

    #[test]
    fn scaffold_project_requires_a_framework() {
        let dir = tempfile::tempdir().unwrap();
        assert!(scaffold_project(dir.path(), &opts(), &[]).is_err());
    }

    #[test]
    fn add_target_refuses_duplicates() {
        let dir = tempfile::tempdir().unwrap();
        let project = scaffold_project(dir.path(), &opts(), &[Framework::Tauri]).unwrap();
        let err = add_target_to_project(&project, Framework::Tauri, &opts());
        assert!(err.is_err());
    }

    #[test]
    fn resolve_target_dir_uses_manifest_then_detection() {
        let dir = tempfile::tempdir().unwrap();
        let project = scaffold_project(dir.path(), &opts(), &[Framework::Tauri]).unwrap();
        let resolved = resolve_target_dir(&project, Framework::Tauri).unwrap();
        assert_eq!(resolved, project);

        // Without a manifest, detection still finds the target.
        std::fs::remove_file(project.join(crate::backend::project_manager::PROJECT_MANIFEST))
            .unwrap();
        let resolved = resolve_target_dir(&project, Framework::Tauri).unwrap();
        assert_eq!(resolved, project);
    }
}
