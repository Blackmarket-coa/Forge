//! Electron framework adapter: Chromium-based desktop apps. The scaffold is
//! a minimal main-process script that opens the user's website in a hardened
//! BrowserWindow, with electron-builder configured for installers. All app
//! settings live in `package.json` (the `forgeApp` block plus the standard
//! electron-builder `build` block) so there is exactly one config file.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use url::Url;

use crate::backend::errors::ForgeError;
use crate::backend::frameworks::{
    is_reverse_domain, npm_install_step, npx_program, tools, BundleKind, CommandSpec, Framework,
    FrameworkAdapter, FrameworkInfo, TargetStatus,
};
use crate::backend::fs_util::write_atomic;
use crate::backend::web_app::{
    normalize_url, refuse_non_empty, slug, WebAppOptions, DEFAULT_HEIGHT, DEFAULT_WIDTH, ICON_PNG,
};

pub struct ElectronAdapter;

fn read_package_json(dir: &Path) -> Option<Value> {
    let content = fs::read_to_string(dir.join("package.json")).ok()?;
    serde_json::from_str(&content).ok()
}

fn electron_version(package: &Value) -> Option<String> {
    for section in ["devDependencies", "dependencies"] {
        if let Some(v) = package
            .get(section)
            .and_then(|d| d.get("electron"))
            .and_then(|v| v.as_str())
        {
            return Some(v.trim_start_matches(['^', '~']).to_string());
        }
    }
    None
}

/// A directory is an Electron target when its package.json depends on
/// electron.
fn is_electron_dir(dir: &Path) -> Option<Value> {
    let package = read_package_json(dir)?;
    electron_version(&package)?;
    Some(package)
}

fn locate(project_root: &Path) -> Option<(PathBuf, String, Value)> {
    if let Some(package) = is_electron_dir(project_root) {
        return Some((project_root.to_path_buf(), ".".to_string(), package));
    }
    let sub = project_root.join("electron");
    if let Some(package) = is_electron_dir(&sub) {
        return Some((sub, "electron".to_string(), package));
    }
    None
}

impl FrameworkAdapter for ElectronAdapter {
    fn framework(&self) -> Framework {
        Framework::Electron
    }

    fn info(&self) -> FrameworkInfo {
        FrameworkInfo {
            id: "electron",
            label: "Desktop app (Electron)",
            tagline: "The classic web-tech desktop app — bigger downloads, huge ecosystem.",
            platforms: &["macOS", "Linux", "Windows"],
            bundle_kinds: vec![
                BundleKind {
                    id: "appimage",
                    label: "Linux app (AppImage)",
                    platform: "Linux",
                    note: Some("Needs Node.js"),
                },
                BundleKind {
                    id: "deb",
                    label: "Linux app (.deb)",
                    platform: "Linux",
                    note: Some("Needs Node.js"),
                },
                BundleKind {
                    id: "dmg",
                    label: "macOS app (.dmg)",
                    platform: "macOS",
                    note: Some("Build on a Mac"),
                },
                BundleKind {
                    id: "nsis",
                    label: "Windows installer (.exe)",
                    platform: "Windows",
                    note: Some("Build on Windows"),
                },
            ],
            tools: vec![tools::NODE, tools::NPM],
            config_file: "package.json",
            dev_label: "Preview app",
            dev_available: true,
        }
    }

    fn default_dir(&self) -> &'static str {
        "electron"
    }

    fn detect(&self, project_root: &Path) -> Option<TargetStatus> {
        let (target_dir, dir, package) = locate(project_root)?;

        let product_name = package
            .get("build")
            .and_then(|b| b.get("productName"))
            .and_then(|v| v.as_str())
            .or_else(|| package.get("name").and_then(|v| v.as_str()))
            .map(str::to_string);
        let identifier = package
            .get("build")
            .and_then(|b| b.get("appId"))
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let source_url = package
            .get("forgeApp")
            .and_then(|f| f.get("url"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let config_issues = self.validate_config(&target_dir, &package);
        let config_ok = config_issues.is_empty();
        let status = if config_ok { "ready" } else { "needs_config" };

        Some(TargetStatus {
            framework: "electron".to_string(),
            dir,
            product_name,
            identifier,
            source_url,
            version: electron_version(&package),
            config_ok,
            config_issues,
            status: status.to_string(),
        })
    }

    fn scaffold(&self, project_root: &Path, opts: &WebAppOptions) -> Result<PathBuf, ForgeError> {
        let normalized_url = normalize_url(&opts.url)?;
        let name = opts.name.trim();
        if name.is_empty() {
            return Err(ForgeError::ConfigInvalid(
                "Please give your app a name.".to_string(),
            ));
        }

        let target = project_root.join("electron");
        refuse_non_empty(&target)?;

        let identifier = opts.effective_identifier();
        let width = opts.width.unwrap_or(DEFAULT_WIDTH);
        let height = opts.height.unwrap_or(DEFAULT_HEIGHT);
        let app_slug = slug(name);

        let package_json = json!({
            "name": format!("{app_slug}-desktop"),
            "private": true,
            "version": "0.1.0",
            "description": format!("{name} desktop app"),
            "main": "main.js",
            "scripts": {
                "start": "electron .",
                "dist": "electron-builder"
            },
            "forgeApp": {
                "url": normalized_url,
                "width": width,
                "height": height
            },
            "build": {
                "appId": identifier,
                "productName": name,
                "directories": { "output": "dist" },
                "files": ["main.js", "package.json", "icon.png"],
                "linux": { "target": ["AppImage", "deb"], "icon": "icon.png", "category": "Utility" },
                "mac": { "target": ["dmg"], "icon": "icon.png" },
                "win": { "target": ["nsis"], "icon": "icon.png" }
            },
            "devDependencies": {
                "electron": "^44.0.0",
                "electron-builder": "^26.0.0"
            }
        });

        // The main process reads its settings from package.json's forgeApp
        // block, so the config editor has a single file to manage. The window
        // is hardened: no Node access from the remote site, and new windows
        // open in the user's browser instead.
        let main_js = "\
// Generated by Forge. Opens your website in its own desktop window.\n\
const { app, BrowserWindow, shell } = require('electron')\n\
const { forgeApp } = require('./package.json')\n\
\n\
function createWindow() {\n\
  const win = new BrowserWindow({\n\
    width: forgeApp.width || 1200,\n\
    height: forgeApp.height || 800,\n\
    webPreferences: {\n\
      contextIsolation: true,\n\
      nodeIntegration: false,\n\
      sandbox: true,\n\
    },\n\
  })\n\
\n\
  win.loadURL(forgeApp.url)\n\
\n\
  // Links that ask for a new window open in the user's browser.\n\
  win.webContents.setWindowOpenHandler(({ url }) => {\n\
    shell.openExternal(url)\n\
    return { action: 'deny' }\n\
  })\n\
}\n\
\n\
app.whenReady().then(() => {\n\
  createWindow()\n\
  app.on('activate', () => {\n\
    if (BrowserWindow.getAllWindows().length === 0) createWindow()\n\
  })\n\
})\n\
\n\
app.on('window-all-closed', () => {\n\
  if (process.platform !== 'darwin') app.quit()\n\
})\n";

        let readme = format!(
            "# {name} — desktop app (Electron)\n\n\
             An Electron app that opens **{normalized_url}**. Forge can preview\n\
             and build it for you; by hand you only need Node.js:\n\n\
             ```\nnpm install\nnpx electron .          # preview\nnpx electron-builder    # build installers\n```\n\n\
             Installers appear in `dist/`. Build on the platform you are\n\
             targeting (a Mac for .dmg, Windows for the .exe installer).\n"
        );

        write_atomic(
            &target.join("package.json"),
            serde_json::to_string_pretty(&package_json)?.as_bytes(),
        )?;
        write_atomic(&target.join("main.js"), main_js.as_bytes())?;
        write_atomic(&target.join("icon.png"), ICON_PNG)?;
        write_atomic(&target.join("README.md"), readme.as_bytes())?;

        Ok(target)
    }

    fn readme_section(&self) -> &'static str {
        "## Desktop app (Electron)\n\n\
         Lives in `electron/`. Needs Node.js only. Preview with `npm install`\n\
         then `npx electron .`; build installers with `npx electron-builder`\n\
         (they appear in `electron/dist/`). See `electron/README.md`.\n"
    }

    fn dev_steps(&self, target_dir: &Path) -> Vec<CommandSpec> {
        let mut steps = Vec::new();
        if let Some(install) = npm_install_step(target_dir) {
            steps.push(install);
        }
        steps.push(CommandSpec::new(
            npx_program(),
            vec!["electron", "."],
            target_dir,
        ));
        steps
    }

    fn build_steps(
        &self,
        target_dir: &Path,
        bundle_kind: &str,
    ) -> Result<Vec<CommandSpec>, ForgeError> {
        let builder_args: Vec<&str> = match bundle_kind {
            "appimage" => vec!["electron-builder", "--linux", "appimage"],
            "deb" => vec!["electron-builder", "--linux", "deb"],
            "dmg" => vec!["electron-builder", "--mac", "dmg"],
            "nsis" => vec!["electron-builder", "--win", "nsis"],
            other => {
                return Err(ForgeError::ConfigInvalid(format!(
                    "unknown Electron build target: {other}"
                )))
            }
        };

        let mut steps = Vec::new();
        if let Some(install) = npm_install_step(target_dir) {
            steps.push(install);
        }
        steps.push(CommandSpec::new(npx_program(), builder_args, target_dir));
        Ok(steps)
    }

    fn artifact_dirs(&self, target_dir: &Path) -> Vec<PathBuf> {
        vec![target_dir.join("dist")]
    }

    fn config_path(&self, target_dir: &Path) -> Result<PathBuf, ForgeError> {
        let path = target_dir.join("package.json");
        if path.exists() {
            Ok(path)
        } else {
            Err(ForgeError::ConfigNotFound(target_dir.display().to_string()))
        }
    }

    fn validate_config(&self, _target_dir: &Path, config: &Value) -> Vec<String> {
        let mut issues = Vec::new();

        let app_id = config
            .get("build")
            .and_then(|b| b.get("appId"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if app_id.is_empty() {
            issues.push("Missing build.appId".to_string());
        } else if !is_reverse_domain(app_id) {
            issues.push(
                "build.appId should match reverse-domain format (e.g. com.example.app)".to_string(),
            );
        }

        let has_name = config
            .get("build")
            .and_then(|b| b.get("productName"))
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
            || config
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
        if !has_name {
            issues.push("Missing build.productName".to_string());
        }

        if let Some(forge_app) = config.get("forgeApp") {
            if let Some(url) = forge_app.get("url").and_then(|v| v.as_str()) {
                if Url::parse(url).is_err() {
                    issues.push("forgeApp.url must be a valid URL".to_string());
                }
            } else {
                issues.push("Missing forgeApp.url (the website this app opens)".to_string());
            }
            for key in ["width", "height"] {
                if let Some(v) = forge_app.get(key).and_then(|v| v.as_f64()) {
                    if v <= 0.0 {
                        issues.push(format!("forgeApp.{key} must be positive"));
                    }
                }
            }
        }

        issues
    }

    fn platform_for_artifact(&self, extension: &str) -> Option<&'static str> {
        match extension {
            "AppImage" | "appimage" | "deb" | "rpm" => Some("Linux"),
            "dmg" => Some("macOS"),
            "exe" => Some("Windows"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> WebAppOptions {
        WebAppOptions {
            name: "My Store".to_string(),
            url: "mystore.com".to_string(),
            width: Some(1000),
            height: Some(700),
            ..Default::default()
        }
    }

    #[test]
    fn scaffold_writes_electron_target() {
        let dir = tempfile::tempdir().unwrap();
        let target = ElectronAdapter.scaffold(dir.path(), &opts()).unwrap();
        assert_eq!(target, dir.path().join("electron"));

        let package: Value =
            serde_json::from_str(&fs::read_to_string(target.join("package.json")).unwrap())
                .unwrap();
        assert_eq!(package["build"]["appId"], "com.forge.mystore");
        assert_eq!(package["build"]["productName"], "My Store");
        assert_eq!(package["forgeApp"]["url"], "https://mystore.com/");
        assert_eq!(package["forgeApp"]["width"], 1000);
        assert!(target.join("main.js").exists());
        assert!(target.join("icon.png").exists());

        let main_js = fs::read_to_string(target.join("main.js")).unwrap();
        assert!(main_js.contains("contextIsolation: true"));
        assert!(main_js.contains("nodeIntegration: false"));
    }

    #[test]
    fn detect_finds_scaffolded_target() {
        let dir = tempfile::tempdir().unwrap();
        ElectronAdapter.scaffold(dir.path(), &opts()).unwrap();

        let status = ElectronAdapter.detect(dir.path()).unwrap();
        assert_eq!(status.framework, "electron");
        assert_eq!(status.dir, "electron");
        assert_eq!(status.product_name.as_deref(), Some("My Store"));
        assert_eq!(status.source_url.as_deref(), Some("https://mystore.com/"));
        assert_eq!(status.version.as_deref(), Some("44.0.0"));
        assert_eq!(status.status, "ready");
    }

    #[test]
    fn detect_ignores_non_electron_package_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("package.json"),
            r#"{ "name": "site", "dependencies": { "react": "^18.0.0" } }"#,
        )
        .unwrap();
        assert!(ElectronAdapter.detect(dir.path()).is_none());
    }

    #[test]
    fn build_steps_map_bundle_kinds() {
        let dir = tempfile::tempdir().unwrap();
        ElectronAdapter.scaffold(dir.path(), &opts()).unwrap();
        let target = dir.path().join("electron");

        let steps = ElectronAdapter.build_steps(&target, "appimage").unwrap();
        assert_eq!(steps.len(), 2); // npm install + electron-builder
        assert!(steps[1].args.contains(&"--linux".to_string()));
        assert!(ElectronAdapter.build_steps(&target, "nope").is_err());
    }

    #[test]
    fn validate_flags_issues() {
        let issues = ElectronAdapter.validate_config(
            Path::new("."),
            &json!({ "build": { "appId": "bad" }, "forgeApp": { "width": -1 } }),
        );
        assert!(issues.iter().any(|i| i.contains("reverse-domain")));
        assert!(issues.iter().any(|i| i.contains("productName")));
        assert!(issues.iter().any(|i| i.contains("forgeApp.url")));
        assert!(issues.iter().any(|i| i.contains("width")));
    }
}
