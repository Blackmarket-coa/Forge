//! Capacitor framework adapter: real iPhone/Android apps that load the
//! user's website. The scaffold is a plain Capacitor project whose
//! `server.url` points at the live site, so the app always shows the current
//! website; the native Android/iOS projects are generated on demand by
//! `npx cap add` at preview/build time.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use url::Url;

use crate::backend::errors::ForgeError;
use crate::backend::frameworks::{
    gradlew_step, is_reverse_domain, npm_install_step, npx_program, tools, BundleKind, CommandSpec,
    Framework, FrameworkAdapter, FrameworkInfo, TargetStatus,
};
use crate::backend::fs_util::write_atomic;
use crate::backend::web_app::{html_escape, normalize_url, refuse_non_empty, slug, WebAppOptions};

pub struct CapacitorAdapter;

/// Locate the capacitor config inside a target dir; the bool is true for a
/// TypeScript config Forge can't edit (but can still detect and build).
fn capacitor_config(dir: &Path) -> Option<(PathBuf, bool)> {
    let json = dir.join("capacitor.config.json");
    if json.exists() {
        return Some((json, false));
    }
    let ts = dir.join("capacitor.config.ts");
    if ts.exists() {
        return Some((ts, true));
    }
    None
}

/// Find the target dir relative to the project root: the root itself for
/// standalone Capacitor projects, else the `capacitor/` subdir.
fn locate(project_root: &Path) -> Option<(PathBuf, String)> {
    if capacitor_config(project_root).is_some() {
        return Some((project_root.to_path_buf(), ".".to_string()));
    }
    let sub = project_root.join("capacitor");
    if capacitor_config(&sub).is_some() {
        return Some((sub, "capacitor".to_string()));
    }
    None
}

fn capacitor_core_version(target_dir: &Path) -> Option<String> {
    let content = fs::read_to_string(target_dir.join("package.json")).ok()?;
    let value: Value = serde_json::from_str(&content).ok()?;
    for section in ["dependencies", "devDependencies"] {
        if let Some(v) = value
            .get(section)
            .and_then(|d| d.get("@capacitor/core"))
            .and_then(|v| v.as_str())
        {
            return Some(v.trim_start_matches(['^', '~']).to_string());
        }
    }
    None
}

impl FrameworkAdapter for CapacitorAdapter {
    fn framework(&self) -> Framework {
        Framework::Capacitor
    }

    fn info(&self) -> FrameworkInfo {
        FrameworkInfo {
            id: "capacitor",
            label: "iPhone & Android app (Capacitor)",
            tagline: "Your website as a real mobile app for Google Play and the App Store.",
            platforms: &["Android", "iOS"],
            bundle_kinds: vec![
                BundleKind {
                    id: "apk",
                    label: "Android app (.apk) — install on devices for testing",
                    platform: "Android",
                    note: Some("Needs Node.js and the Android SDK"),
                },
                BundleKind {
                    id: "aab",
                    label: "Android bundle (.aab) — for Google Play (needs signing)",
                    platform: "Android",
                    note: Some("Needs Node.js and the Android SDK"),
                },
                BundleKind {
                    id: "ios",
                    label: "iPhone/iPad project — opens in Xcode",
                    platform: "iOS",
                    note: Some("Needs a Mac with Xcode"),
                },
            ],
            tools: vec![
                tools::NODE,
                tools::NPM,
                tools::JAVA,
                tools::ANDROID_SDK,
                tools::XCODE,
            ],
            config_file: "capacitor.config.json",
            dev_label: "Preview on Android device",
            dev_available: true,
        }
    }

    fn default_dir(&self) -> &'static str {
        "capacitor"
    }

    fn detect(&self, project_root: &Path) -> Option<TargetStatus> {
        let (target_dir, dir) = locate(project_root)?;
        let (config_path, is_ts) = capacitor_config(&target_dir)?;

        let mut product_name = None;
        let mut identifier = None;
        let mut source_url = None;
        let mut config_ok = true;
        let mut config_issues = Vec::new();

        if is_ts {
            // A TypeScript config is fine for building, but Forge's editor
            // only handles the JSON form.
            config_issues.push(
                "capacitor.config.ts is TypeScript — edit it in your code editor".to_string(),
            );
        } else {
            match fs::read_to_string(&config_path)
                .map_err(ForgeError::from)
                .and_then(|c| serde_json::from_str::<Value>(&c).map_err(ForgeError::from))
            {
                Ok(value) => {
                    product_name = value
                        .get("appName")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    identifier = value
                        .get("appId")
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    source_url = value
                        .get("server")
                        .and_then(|v| v.get("url"))
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                    config_issues = self.validate_config(&target_dir, &value);
                    config_ok = config_issues.is_empty();
                }
                Err(e) => {
                    config_ok = false;
                    config_issues.push(format!("capacitor config could not be read: {e}"));
                }
            }
        }

        let status = if config_ok { "ready" } else { "needs_config" };
        Some(TargetStatus {
            framework: "capacitor".to_string(),
            dir,
            product_name,
            identifier,
            source_url,
            version: capacitor_core_version(&target_dir),
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

        let target = project_root.join("capacitor");
        refuse_non_empty(&target)?;

        let identifier = opts.effective_identifier();
        let app_slug = slug(name);

        let package_json = json!({
            "name": format!("{app_slug}-mobile"),
            "private": true,
            "version": "0.1.0",
            "description": format!("{name} mobile app"),
            "dependencies": {
                "@capacitor/android": "^8.0.0",
                "@capacitor/core": "^8.0.0",
                "@capacitor/ios": "^8.0.0"
            },
            "devDependencies": {
                "@capacitor/cli": "^8.0.0"
            }
        });

        let config = json!({
            "appId": identifier,
            "appName": name,
            "webDir": "www",
            "server": { "url": normalized_url }
        });

        // Capacitor requires webDir to exist; the app loads the remote site,
        // so this placeholder is never shown.
        let safe_name = html_escape(name);
        let index_html = format!(
            "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"utf-8\" />\n    \
             <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    \
             <title>{safe_name}</title>\n  </head>\n  <body>\n    <p>Opening {safe_name}…</p>\n  \
             </body>\n</html>\n"
        );

        let readme = format!(
            "# {name} — mobile app (Capacitor)\n\n\
             A Capacitor app that opens **{normalized_url}**. Forge can preview and\n\
             build it for you; to do it by hand you need Node.js (and the Android\n\
             SDK or Xcode):\n\n\
             ```\nnpm install\nnpx cap add android   # first time only\nnpx cap run android\n```\n\n\
             Build an installable test .apk:\n\n\
             ```\nnpx cap sync android\ncd android && ./gradlew assembleDebug\n```\n\n\
             The .apk appears under `android/app/build/outputs/apk/`. For Google\n\
             Play, build `bundleRelease` and sign it; for iPhone/iPad run\n\
             `npx cap add ios && npx cap open ios` on a Mac with Xcode.\n"
        );

        write_atomic(
            &target.join("package.json"),
            serde_json::to_string_pretty(&package_json)?.as_bytes(),
        )?;
        write_atomic(
            &target.join("capacitor.config.json"),
            serde_json::to_string_pretty(&config)?.as_bytes(),
        )?;
        write_atomic(
            &target.join("www").join("index.html"),
            index_html.as_bytes(),
        )?;
        write_atomic(&target.join("README.md"), readme.as_bytes())?;

        Ok(target)
    }

    fn readme_section(&self) -> &'static str {
        "## iPhone & Android app (Capacitor)\n\n\
         Lives in `capacitor/`. Needs Node.js, plus the Android SDK (Android)\n\
         or a Mac with Xcode (iPhone/iPad). See `capacitor/README.md` for the\n\
         exact commands — the short version is `npm install`, `npx cap add\n\
         android`, then `npx cap run android` with a device or emulator\n\
         connected.\n"
    }

    fn dev_steps(&self, target_dir: &Path) -> Vec<CommandSpec> {
        let mut steps = Vec::new();
        if let Some(install) = npm_install_step(target_dir) {
            steps.push(install);
        }
        if !target_dir.join("android").exists() {
            steps.push(CommandSpec::new(
                npx_program(),
                vec!["cap", "add", "android"],
                target_dir,
            ));
        }
        steps.push(CommandSpec::new(
            npx_program(),
            vec!["cap", "run", "android"],
            target_dir,
        ));
        steps
    }

    fn build_steps(
        &self,
        target_dir: &Path,
        bundle_kind: &str,
    ) -> Result<Vec<CommandSpec>, ForgeError> {
        let mut steps = Vec::new();
        if let Some(install) = npm_install_step(target_dir) {
            steps.push(install);
        }

        match bundle_kind {
            "apk" | "aab" => {
                if !target_dir.join("android").exists() {
                    steps.push(CommandSpec::new(
                        npx_program(),
                        vec!["cap", "add", "android"],
                        target_dir,
                    ));
                }
                steps.push(CommandSpec::new(
                    npx_program(),
                    vec!["cap", "sync", "android"],
                    target_dir,
                ));
                let task = if bundle_kind == "apk" {
                    "assembleDebug"
                } else {
                    "bundleRelease"
                };
                steps.push(gradlew_step(&target_dir.join("android"), task));
            }
            "ios" => {
                if !target_dir.join("ios").exists() {
                    steps.push(CommandSpec::new(
                        npx_program(),
                        vec!["cap", "add", "ios"],
                        target_dir,
                    ));
                }
                steps.push(CommandSpec::new(
                    npx_program(),
                    vec!["cap", "sync", "ios"],
                    target_dir,
                ));
                steps.push(CommandSpec::new(
                    npx_program(),
                    vec!["cap", "open", "ios"],
                    target_dir,
                ));
            }
            other => {
                return Err(ForgeError::ConfigInvalid(format!(
                    "unknown Capacitor build target: {other}"
                )))
            }
        }

        Ok(steps)
    }

    fn artifact_dirs(&self, target_dir: &Path) -> Vec<PathBuf> {
        vec![target_dir
            .join("android")
            .join("app")
            .join("build")
            .join("outputs")]
    }

    fn config_path(&self, target_dir: &Path) -> Result<PathBuf, ForgeError> {
        match capacitor_config(target_dir) {
            Some((path, false)) => Ok(path),
            Some((_, true)) => Err(ForgeError::ConfigInvalid(
                "This app uses capacitor.config.ts — edit it in your code editor.".to_string(),
            )),
            None => Err(ForgeError::ConfigNotFound(target_dir.display().to_string())),
        }
    }

    fn validate_config(&self, _target_dir: &Path, config: &Value) -> Vec<String> {
        let mut issues = Vec::new();

        let app_id = config
            .get("appId")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        if app_id.is_empty() {
            issues.push("Missing appId".to_string());
        } else if !is_reverse_domain(app_id) {
            issues.push(
                "appId should match reverse-domain format (e.g. com.example.app)".to_string(),
            );
        }

        if config
            .get("appName")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            != Some(true)
        {
            issues.push("Missing or empty appName".to_string());
        }

        if let Some(url) = config
            .get("server")
            .and_then(|v| v.get("url"))
            .and_then(|v| v.as_str())
        {
            if Url::parse(url).is_err() {
                issues.push("server.url must be a valid URL".to_string());
            }
        }

        issues
    }

    fn platform_for_artifact(&self, extension: &str) -> Option<&'static str> {
        match extension {
            "apk" | "aab" => Some("Android"),
            "ipa" => Some("iOS"),
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
            ..Default::default()
        }
    }

    #[test]
    fn scaffold_writes_capacitor_target() {
        let dir = tempfile::tempdir().unwrap();
        let target = CapacitorAdapter.scaffold(dir.path(), &opts()).unwrap();
        assert_eq!(target, dir.path().join("capacitor"));

        let config: Value = serde_json::from_str(
            &fs::read_to_string(target.join("capacitor.config.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(config["appId"], "com.forge.mystore");
        assert_eq!(config["appName"], "My Store");
        assert_eq!(config["server"]["url"], "https://mystore.com/");
        assert!(target.join("www/index.html").exists());
        assert!(target.join("package.json").exists());
        assert!(target.join("README.md").exists());
    }

    #[test]
    fn detect_finds_scaffolded_target() {
        let dir = tempfile::tempdir().unwrap();
        CapacitorAdapter.scaffold(dir.path(), &opts()).unwrap();

        let status = CapacitorAdapter.detect(dir.path()).unwrap();
        assert_eq!(status.framework, "capacitor");
        assert_eq!(status.dir, "capacitor");
        assert_eq!(status.product_name.as_deref(), Some("My Store"));
        assert_eq!(status.identifier.as_deref(), Some("com.forge.mystore"));
        assert_eq!(status.source_url.as_deref(), Some("https://mystore.com/"));
        assert_eq!(status.version.as_deref(), Some("8.0.0"));
        assert_eq!(status.status, "ready");
    }

    #[test]
    fn detect_standalone_project_at_root() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("capacitor.config.json"),
            r#"{ "appId": "com.example.solo", "appName": "Solo", "webDir": "www" }"#,
        )
        .unwrap();
        let status = CapacitorAdapter.detect(dir.path()).unwrap();
        assert_eq!(status.dir, ".");
        assert_eq!(status.product_name.as_deref(), Some("Solo"));
    }

    #[test]
    fn build_steps_for_apk_end_with_gradle() {
        let dir = tempfile::tempdir().unwrap();
        CapacitorAdapter.scaffold(dir.path(), &opts()).unwrap();
        let target = dir.path().join("capacitor");

        let steps = CapacitorAdapter.build_steps(&target, "apk").unwrap();
        // npm install (no node_modules), cap add android, cap sync, gradle.
        assert_eq!(steps.len(), 4);
        assert!(steps[3].program.ends_with("gradlew"));
        assert_eq!(steps[3].args, vec!["assembleDebug"]);
        assert!(CapacitorAdapter.build_steps(&target, "nope").is_err());
    }

    #[test]
    fn validate_flags_issues() {
        let issues = CapacitorAdapter.validate_config(
            Path::new("."),
            &json!({ "appId": "bad", "server": { "url": "not a url" } }),
        );
        assert!(issues.iter().any(|i| i.contains("reverse-domain")));
        assert!(issues.iter().any(|i| i.contains("appName")));
        assert!(issues.iter().any(|i| i.contains("server.url")));
    }
}
