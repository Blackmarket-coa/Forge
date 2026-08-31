//! React Native (Expo) framework adapter: a React Native shell whose single
//! screen is a WebView showing the user's website. Preview runs through Expo
//! (`npx expo start` + the Expo Go phone app); Android test builds go through
//! `expo prebuild` + Gradle; store builds are documented in the generated
//! README (Xcode on a Mac, or Expo's EAS cloud builds).

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
use crate::backend::web_app::{normalize_url, refuse_non_empty, slug, WebAppOptions};

pub struct ReactNativeAdapter;

fn expo_app_json(dir: &Path) -> Option<Value> {
    let content = fs::read_to_string(dir.join("app.json")).ok()?;
    let value: Value = serde_json::from_str(&content).ok()?;
    value.get("expo")?;
    Some(value)
}

fn locate(project_root: &Path) -> Option<(PathBuf, String, Value)> {
    if let Some(config) = expo_app_json(project_root) {
        return Some((project_root.to_path_buf(), ".".to_string(), config));
    }
    let sub = project_root.join("react-native");
    if let Some(config) = expo_app_json(&sub) {
        return Some((sub, "react-native".to_string(), config));
    }
    None
}

fn expo_version(target_dir: &Path) -> Option<String> {
    let content = fs::read_to_string(target_dir.join("package.json")).ok()?;
    let value: Value = serde_json::from_str(&content).ok()?;
    for section in ["dependencies", "devDependencies"] {
        if let Some(v) = value
            .get(section)
            .and_then(|d| d.get("expo"))
            .and_then(|v| v.as_str())
        {
            return Some(v.trim_start_matches(['^', '~']).to_string());
        }
    }
    None
}

impl FrameworkAdapter for ReactNativeAdapter {
    fn framework(&self) -> Framework {
        Framework::ReactNative
    }

    fn info(&self) -> FrameworkInfo {
        FrameworkInfo {
            id: "react-native",
            label: "Mobile app (React Native + Expo)",
            tagline:
                "A React Native shell around your site — preview instantly with the Expo Go app.",
            platforms: &["Android", "iOS"],
            bundle_kinds: vec![
                BundleKind {
                    id: "apk",
                    label: "Android app (.apk) — install on devices for testing",
                    platform: "Android",
                    note: Some("Needs Node.js and the Android SDK"),
                },
                BundleKind {
                    id: "ios",
                    label: "iPhone/iPad project — for Xcode or EAS",
                    platform: "iOS",
                    note: Some("Needs a Mac with Xcode (or an Expo EAS account)"),
                },
            ],
            tools: vec![
                tools::NODE,
                tools::NPM,
                tools::JAVA,
                tools::ANDROID_SDK,
                tools::XCODE,
            ],
            config_file: "app.json",
            dev_label: "Preview with Expo Go",
            dev_available: true,
        }
    }

    fn default_dir(&self) -> &'static str {
        "react-native"
    }

    fn detect(&self, project_root: &Path) -> Option<TargetStatus> {
        let (target_dir, dir, config) = locate(project_root)?;
        let expo = config.get("expo")?;

        let product_name = expo
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let identifier = expo
            .get("android")
            .and_then(|a| a.get("package"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                expo.get("ios")
                    .and_then(|i| i.get("bundleIdentifier"))
                    .and_then(|v| v.as_str())
            })
            .map(str::to_string);
        let source_url = expo
            .get("extra")
            .and_then(|e| e.get("forgeUrl"))
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let config_issues = self.validate_config(&target_dir, &config);
        let config_ok = config_issues.is_empty();
        let status = if config_ok { "ready" } else { "needs_config" };

        Some(TargetStatus {
            framework: "react-native".to_string(),
            dir,
            product_name,
            identifier,
            source_url,
            version: expo_version(&target_dir),
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

        let target = project_root.join("react-native");
        refuse_non_empty(&target)?;

        let identifier = opts.effective_identifier();
        let app_slug = slug(name);

        // Versions follow Expo SDK 57's published template pairing; the
        // generated README points at `npx expo install --fix` if they drift.
        let package_json = json!({
            "name": format!("{app_slug}-mobile-rn"),
            "private": true,
            "version": "0.1.0",
            "main": "node_modules/expo/AppEntry.js",
            "scripts": {
                "start": "expo start",
                "android": "expo run:android",
                "ios": "expo run:ios"
            },
            "dependencies": {
                "expo": "~57.0.0",
                "react": "19.2.3",
                "react-native": "0.86.3",
                "react-native-webview": "^14.0.0"
            }
        });

        let app_json = json!({
            "expo": {
                "name": name,
                "slug": app_slug,
                "version": "0.1.0",
                "orientation": "default",
                "userInterfaceStyle": "automatic",
                "ios": {
                    "bundleIdentifier": identifier,
                    "supportsTablet": true
                },
                "android": {
                    "package": identifier
                },
                "extra": {
                    "forgeUrl": normalized_url
                }
            }
        });

        // The website address lives in app.json (expo.extra.forgeUrl) so the
        // config editor can change it without touching code.
        let app_js = "\
// Generated by Forge. Shows your website inside a React Native app.\n\
import React from 'react'\n\
import { SafeAreaView, StyleSheet } from 'react-native'\n\
import { WebView } from 'react-native-webview'\n\
import appConfig from './app.json'\n\
\n\
const url = appConfig.expo.extra.forgeUrl\n\
\n\
export default function App() {\n\
  return (\n\
    <SafeAreaView style={styles.container}>\n\
      <WebView\n\
        source={{ uri: url }}\n\
        style={styles.web}\n\
        allowsBackForwardNavigationGestures\n\
      />\n\
    </SafeAreaView>\n\
  )\n\
}\n\
\n\
const styles = StyleSheet.create({\n\
  container: { flex: 1 },\n\
  web: { flex: 1 },\n\
})\n";

        let readme = format!(
            "# {name} — mobile app (React Native + Expo)\n\n\
             A React Native app that shows **{normalized_url}**. The quickest\n\
             preview needs only Node.js and the free Expo Go app on your phone:\n\n\
             ```\nnpm install\nnpx expo start\n```\n\n\
             Scan the QR code with Expo Go and your app opens on the phone.\n\n\
             ## Installable Android build (.apk)\n\n\
             Needs the Android SDK (install Android Studio):\n\n\
             ```\nnpx expo prebuild --platform android\ncd android && ./gradlew assembleDebug\n```\n\n\
             The .apk appears under `android/app/build/outputs/apk/`.\n\n\
             ## iPhone/iPad and app stores\n\n\
             Run `npx expo prebuild --platform ios` on a Mac with Xcode, or use\n\
             Expo's cloud build service (EAS): <https://docs.expo.dev/build/introduction/>.\n\n\
             If package versions ever get out of sync, `npx expo install --fix`\n\
             repairs them.\n"
        );

        write_atomic(
            &target.join("package.json"),
            serde_json::to_string_pretty(&package_json)?.as_bytes(),
        )?;
        write_atomic(
            &target.join("app.json"),
            serde_json::to_string_pretty(&app_json)?.as_bytes(),
        )?;
        write_atomic(&target.join("App.js"), app_js.as_bytes())?;
        write_atomic(&target.join("README.md"), readme.as_bytes())?;

        Ok(target)
    }

    fn readme_section(&self) -> &'static str {
        "## Mobile app (React Native + Expo)\n\n\
         Lives in `react-native/`. Preview needs only Node.js and the free\n\
         Expo Go app on your phone: `npm install` then `npx expo start`, and\n\
         scan the QR code. Android/iOS builds are covered in\n\
         `react-native/README.md`.\n"
    }

    fn dev_steps(&self, target_dir: &Path) -> Vec<CommandSpec> {
        let mut steps = Vec::new();
        if let Some(install) = npm_install_step(target_dir) {
            steps.push(install);
        }
        steps.push(CommandSpec::new(
            npx_program(),
            vec!["expo", "start"],
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
            "apk" => {
                if !target_dir.join("android").exists() {
                    steps.push(CommandSpec::new(
                        npx_program(),
                        vec!["expo", "prebuild", "--platform", "android"],
                        target_dir,
                    ));
                }
                steps.push(gradlew_step(&target_dir.join("android"), "assembleDebug"));
            }
            "ios" => {
                if !target_dir.join("ios").exists() {
                    steps.push(CommandSpec::new(
                        npx_program(),
                        vec!["expo", "prebuild", "--platform", "ios"],
                        target_dir,
                    ));
                } else {
                    // Nothing to regenerate; report that via a harmless step.
                    steps.push(CommandSpec::new(
                        npx_program(),
                        vec!["expo", "prebuild", "--platform", "ios", "--no-install"],
                        target_dir,
                    ));
                }
            }
            other => {
                return Err(ForgeError::ConfigInvalid(format!(
                    "unknown React Native build target: {other}"
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
        let path = target_dir.join("app.json");
        if path.exists() {
            Ok(path)
        } else {
            Err(ForgeError::ConfigNotFound(target_dir.display().to_string()))
        }
    }

    fn validate_config(&self, _target_dir: &Path, config: &Value) -> Vec<String> {
        let mut issues = Vec::new();
        let Some(expo) = config.get("expo") else {
            issues.push("Missing top-level \"expo\" section".to_string());
            return issues;
        };

        if expo
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            != Some(true)
        {
            issues.push("Missing or empty expo.name".to_string());
        }

        if expo
            .get("slug")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            != Some(true)
        {
            issues.push("Missing or empty expo.slug".to_string());
        }

        for (label, value) in [
            (
                "expo.android.package",
                expo.get("android").and_then(|a| a.get("package")),
            ),
            (
                "expo.ios.bundleIdentifier",
                expo.get("ios").and_then(|i| i.get("bundleIdentifier")),
            ),
        ] {
            if let Some(id) = value.and_then(|v| v.as_str()) {
                if !is_reverse_domain(id) {
                    issues.push(format!(
                        "{label} should match reverse-domain format (e.g. com.example.app)"
                    ));
                }
            }
        }

        if let Some(url) = expo
            .get("extra")
            .and_then(|e| e.get("forgeUrl"))
            .and_then(|v| v.as_str())
        {
            if Url::parse(url).is_err() {
                issues.push("expo.extra.forgeUrl must be a valid URL".to_string());
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
    fn scaffold_writes_expo_target() {
        let dir = tempfile::tempdir().unwrap();
        let target = ReactNativeAdapter.scaffold(dir.path(), &opts()).unwrap();
        assert_eq!(target, dir.path().join("react-native"));

        let app: Value =
            serde_json::from_str(&fs::read_to_string(target.join("app.json")).unwrap()).unwrap();
        assert_eq!(app["expo"]["name"], "My Store");
        assert_eq!(app["expo"]["slug"], "my-store");
        assert_eq!(app["expo"]["android"]["package"], "com.forge.mystore");
        assert_eq!(app["expo"]["extra"]["forgeUrl"], "https://mystore.com/");
        assert!(target.join("App.js").exists());
        assert!(target.join("package.json").exists());

        let app_js = fs::read_to_string(target.join("App.js")).unwrap();
        assert!(app_js.contains("react-native-webview"));
        assert!(app_js.contains("expo.extra.forgeUrl"));
    }

    #[test]
    fn detect_finds_scaffolded_target() {
        let dir = tempfile::tempdir().unwrap();
        ReactNativeAdapter.scaffold(dir.path(), &opts()).unwrap();

        let status = ReactNativeAdapter.detect(dir.path()).unwrap();
        assert_eq!(status.framework, "react-native");
        assert_eq!(status.dir, "react-native");
        assert_eq!(status.product_name.as_deref(), Some("My Store"));
        assert_eq!(status.identifier.as_deref(), Some("com.forge.mystore"));
        assert_eq!(status.source_url.as_deref(), Some("https://mystore.com/"));
        assert_eq!(status.version.as_deref(), Some("57.0.0"));
        assert_eq!(status.status, "ready");
    }

    #[test]
    fn detect_ignores_non_expo_app_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("app.json"), r#"{ "name": "not expo" }"#).unwrap();
        assert!(ReactNativeAdapter.detect(dir.path()).is_none());
    }

    #[test]
    fn build_steps_for_apk_end_with_gradle() {
        let dir = tempfile::tempdir().unwrap();
        ReactNativeAdapter.scaffold(dir.path(), &opts()).unwrap();
        let target = dir.path().join("react-native");

        let steps = ReactNativeAdapter.build_steps(&target, "apk").unwrap();
        // npm install, expo prebuild, gradle.
        assert_eq!(steps.len(), 3);
        assert!(steps[2].program.ends_with("gradlew"));
        assert!(ReactNativeAdapter.build_steps(&target, "nope").is_err());
    }

    #[test]
    fn validate_flags_issues() {
        let issues = ReactNativeAdapter.validate_config(
            Path::new("."),
            &json!({ "expo": { "android": { "package": "bad" }, "extra": { "forgeUrl": "nope" } } }),
        );
        assert!(issues.iter().any(|i| i.contains("expo.name")));
        assert!(issues.iter().any(|i| i.contains("expo.slug")));
        assert!(issues.iter().any(|i| i.contains("reverse-domain")));
        assert!(issues.iter().any(|i| i.contains("forgeUrl")));
    }
}
