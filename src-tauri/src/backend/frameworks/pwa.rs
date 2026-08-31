//! PWA framework adapter: make the user's existing website installable from
//! the browser. Unlike the other frameworks there is no separate app to run —
//! the scaffold is a small kit (web manifest, service worker, icons, and a
//! copy-paste snippet) the user uploads to their website, and the "build" is
//! packaging that kit into a zip. No toolchain is required at all.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use url::Url;

use crate::backend::errors::ForgeError;
use crate::backend::frameworks::{
    BundleKind, CommandSpec, Framework, FrameworkAdapter, FrameworkInfo, TargetStatus,
};
use crate::backend::fs_util::write_atomic;
use crate::backend::web_app::{
    normalize_url, refuse_non_empty, slug, WebAppOptions, ICON_128_2X, ICON_PNG,
};

pub struct PwaAdapter;

fn manifest_path(dir: &Path) -> Option<PathBuf> {
    let path = dir.join("manifest.webmanifest");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn locate(project_root: &Path) -> Option<(PathBuf, String)> {
    if manifest_path(project_root).is_some() {
        return Some((project_root.to_path_buf(), ".".to_string()));
    }
    let sub = project_root.join("pwa");
    if manifest_path(&sub).is_some() {
        return Some((sub, "pwa".to_string()));
    }
    None
}

/// Files that make up the uploadable kit, relative to the target dir.
const KIT_FILES: [&str; 5] = [
    "manifest.webmanifest",
    "sw.js",
    "snippet.html",
    "icons/icon-256.png",
    "icons/icon-512.png",
];

impl FrameworkAdapter for PwaAdapter {
    fn framework(&self) -> Framework {
        Framework::Pwa
    }

    fn info(&self) -> FrameworkInfo {
        FrameworkInfo {
            id: "pwa",
            label: "Install from the browser (PWA)",
            tagline: "Visitors install your site straight from the browser — no app store, no tools needed.",
            platforms: &["Web"],
            bundle_kinds: vec![BundleKind {
                id: "zip",
                label: "Web-app kit (.zip) — upload to your website",
                platform: "Web",
                note: None,
            }],
            tools: vec![],
            config_file: "manifest.webmanifest",
            dev_label: "Open site in browser",
            dev_available: false,
        }
    }

    fn default_dir(&self) -> &'static str {
        "pwa"
    }

    fn detect(&self, project_root: &Path) -> Option<TargetStatus> {
        let (target_dir, dir) = locate(project_root)?;
        let path = manifest_path(&target_dir)?;

        let mut product_name = None;
        let mut source_url = None;
        let mut config_ok = false;
        let mut config_issues = Vec::new();

        match fs::read_to_string(&path)
            .map_err(ForgeError::from)
            .and_then(|c| serde_json::from_str::<Value>(&c).map_err(ForgeError::from))
        {
            Ok(value) => {
                product_name = value
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                source_url = value
                    .get("start_url")
                    .and_then(|v| v.as_str())
                    .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
                    .map(str::to_string);
                config_issues = self.validate_config(&target_dir, &value);
                config_ok = config_issues.is_empty();
            }
            Err(e) => config_issues.push(format!("manifest.webmanifest could not be read: {e}")),
        }

        let status = if config_ok { "ready" } else { "needs_config" };
        Some(TargetStatus {
            framework: "pwa".to_string(),
            dir,
            product_name,
            identifier: None,
            source_url,
            version: None,
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

        let target = project_root.join("pwa");
        refuse_non_empty(&target)?;

        let manifest = json!({
            "name": name,
            "short_name": name,
            "start_url": normalized_url,
            "scope": "/",
            "display": "standalone",
            "background_color": "#ffffff",
            "theme_color": "#111111",
            "icons": [
                { "src": "icons/icon-256.png", "sizes": "256x256", "type": "image/png" },
                { "src": "icons/icon-512.png", "sizes": "512x512", "type": "image/png", "purpose": "any" }
            ]
        });

        let sw_js = "\
// Minimal service worker so browsers offer to install this site as an app.\n\
// It passes every request straight to the network.\n\
self.addEventListener('install', () => self.skipWaiting())\n\
self.addEventListener('activate', (event) => event.waitUntil(self.clients.claim()))\n\
self.addEventListener('fetch', () => {\n\
  // Network passthrough. Add caching here later if you want offline support.\n\
})\n";

        let snippet = "\
<!-- Add these lines inside the <head> of your website's pages. -->\n\
<link rel=\"manifest\" href=\"/manifest.webmanifest\" />\n\
<meta name=\"theme-color\" content=\"#111111\" />\n\
<script>\n\
  if ('serviceWorker' in navigator) {\n\
    navigator.serviceWorker.register('/sw.js')\n\
  }\n\
</script>\n";

        let readme = format!(
            "# {name} — install from the browser (PWA)\n\n\
             This kit makes **{normalized_url}** installable straight from the\n\
             browser (an \"Install app\" option on desktop and Android). No app\n\
             store and no build tools involved.\n\n\
             ## How to use it\n\n\
             1. Upload `manifest.webmanifest`, `sw.js`, and the `icons/` folder\n\
                to the **top level** of your website (so they're at\n\
                `/manifest.webmanifest` and `/sw.js`).\n\
             2. Copy the lines from `snippet.html` into the `<head>` of your\n\
                site's pages.\n\
             3. Visit your site over https and check for the install option.\n\n\
             Replace the images in `icons/` with your own logo (same sizes)\n\
             whenever you're ready.\n"
        );

        write_atomic(
            &target.join("manifest.webmanifest"),
            serde_json::to_string_pretty(&manifest)?.as_bytes(),
        )?;
        write_atomic(&target.join("sw.js"), sw_js.as_bytes())?;
        write_atomic(&target.join("snippet.html"), snippet.as_bytes())?;
        write_atomic(&target.join("icons").join("icon-256.png"), ICON_128_2X)?;
        write_atomic(&target.join("icons").join("icon-512.png"), ICON_PNG)?;
        write_atomic(&target.join("README.md"), readme.as_bytes())?;

        Ok(target)
    }

    fn readme_section(&self) -> &'static str {
        "## Install from the browser (PWA)\n\n\
         Lives in `pwa/`. Nothing to build or install — upload the kit files\n\
         to your website and add the snippet to your pages, and browsers will\n\
         offer to install your site as an app. Full steps in `pwa/README.md`.\n"
    }

    fn dev_steps(&self, _target_dir: &Path) -> Vec<CommandSpec> {
        Vec::new()
    }

    fn build_steps(
        &self,
        _target_dir: &Path,
        bundle_kind: &str,
    ) -> Result<Vec<CommandSpec>, ForgeError> {
        Err(ForgeError::ConfigInvalid(format!(
            "unknown PWA build target: {bundle_kind}"
        )))
    }

    fn build_in_process(&self, target_dir: &Path, bundle_kind: &str) -> Result<bool, ForgeError> {
        if bundle_kind != "zip" {
            return Ok(false);
        }

        let manifest = manifest_path(target_dir)
            .ok_or_else(|| ForgeError::ConfigNotFound(target_dir.display().to_string()))?;
        let manifest_value: Value = serde_json::from_str(&fs::read_to_string(&manifest)?)?;
        let app_name = manifest_value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("my-app");

        let dist = target_dir.join("dist");
        fs::create_dir_all(&dist)?;
        let zip_path = dist.join(format!("{}-pwa-kit.zip", slug(app_name)));

        let file = fs::File::create(&zip_path)?;
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let mut kit: Vec<&str> = KIT_FILES.to_vec();
        kit.push("README.md");
        for rel in kit {
            let path = target_dir.join(rel);
            if !path.exists() {
                continue;
            }
            let bytes = fs::read(&path)?;
            writer
                .start_file(rel, options)
                .map_err(|e| ForgeError::ProcessError(format!("could not write zip: {e}")))?;
            writer.write_all(&bytes)?;
        }
        writer
            .finish()
            .map_err(|e| ForgeError::ProcessError(format!("could not finish zip: {e}")))?;

        Ok(true)
    }

    fn artifact_dirs(&self, target_dir: &Path) -> Vec<PathBuf> {
        vec![target_dir.join("dist")]
    }

    fn config_path(&self, target_dir: &Path) -> Result<PathBuf, ForgeError> {
        manifest_path(target_dir)
            .ok_or_else(|| ForgeError::ConfigNotFound(target_dir.display().to_string()))
    }

    fn validate_config(&self, _target_dir: &Path, config: &Value) -> Vec<String> {
        let mut issues = Vec::new();

        if config
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| !s.trim().is_empty())
            != Some(true)
        {
            issues.push("Missing or empty name".to_string());
        }

        match config.get("start_url").and_then(|v| v.as_str()) {
            None => issues.push("Missing start_url".to_string()),
            Some(url) if url.starts_with("http") => {
                if Url::parse(url).is_err() {
                    issues.push("start_url must be a valid URL".to_string());
                }
            }
            Some(_relative) => {}
        }

        let icons_ok = config
            .get("icons")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);
        if !icons_ok {
            issues.push("icons must list at least one icon".to_string());
        }

        if let Some(display) = config.get("display").and_then(|v| v.as_str()) {
            if !matches!(
                display,
                "standalone" | "fullscreen" | "minimal-ui" | "browser"
            ) {
                issues.push(
                    "display should be standalone, fullscreen, minimal-ui, or browser".to_string(),
                );
            }
        }

        issues
    }

    fn platform_for_artifact(&self, extension: &str) -> Option<&'static str> {
        match extension {
            "zip" => Some("Web"),
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
    fn scaffold_writes_pwa_kit() {
        let dir = tempfile::tempdir().unwrap();
        let target = PwaAdapter.scaffold(dir.path(), &opts()).unwrap();
        assert_eq!(target, dir.path().join("pwa"));

        for rel in KIT_FILES {
            assert!(target.join(rel).exists(), "missing {rel}");
        }

        let manifest: Value =
            serde_json::from_str(&fs::read_to_string(target.join("manifest.webmanifest")).unwrap())
                .unwrap();
        assert_eq!(manifest["name"], "My Store");
        assert_eq!(manifest["start_url"], "https://mystore.com/");
        assert_eq!(manifest["display"], "standalone");
    }

    #[test]
    fn detect_finds_scaffolded_kit() {
        let dir = tempfile::tempdir().unwrap();
        PwaAdapter.scaffold(dir.path(), &opts()).unwrap();

        let status = PwaAdapter.detect(dir.path()).unwrap();
        assert_eq!(status.framework, "pwa");
        assert_eq!(status.dir, "pwa");
        assert_eq!(status.product_name.as_deref(), Some("My Store"));
        assert_eq!(status.source_url.as_deref(), Some("https://mystore.com/"));
        assert_eq!(status.status, "ready");
    }

    #[test]
    fn zip_build_produces_an_artifact() {
        let dir = tempfile::tempdir().unwrap();
        let target = PwaAdapter.scaffold(dir.path(), &opts()).unwrap();

        let handled = PwaAdapter.build_in_process(&target, "zip").unwrap();
        assert!(handled);

        let zip_path = target.join("dist").join("my-store-pwa-kit.zip");
        assert!(zip_path.exists());
        assert!(fs::metadata(&zip_path).unwrap().len() > 0);

        // Unknown kinds are not handled in-process.
        assert!(!PwaAdapter.build_in_process(&target, "apk").unwrap());
    }

    #[test]
    fn validate_flags_issues() {
        let issues =
            PwaAdapter.validate_config(Path::new("."), &json!({ "display": "weird", "icons": [] }));
        assert!(issues.iter().any(|i| i.contains("name")));
        assert!(issues.iter().any(|i| i.contains("start_url")));
        assert!(issues.iter().any(|i| i.contains("icons")));
        assert!(issues.iter().any(|i| i.contains("display")));
    }
}
