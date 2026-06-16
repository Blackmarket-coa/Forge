//! Generate a minimal, build-ready Tauri project that wraps a website URL.
//!
//! This powers Forge's "turn your website into an app" flow. Unlike the
//! `create-tauri-app` scaffolder used by [`crate::backend::ipc::create_project`],
//! it writes the project files directly, so a non-technical user needs **no
//! Node.js, package manager, or frontend framework** — only the Tauri build
//! toolchain (Rust + the Tauri CLI), which Forge already detects, is required
//! to turn the generated project into an installer.
//!
//! The generated app is a single window whose `url` points straight at the
//! user's website, so it loads the live site in both `tauri dev` and a bundled
//! release. The bundled icons are reused from Forge's own icon set so the
//! project builds out of the box; users can replace them later.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::json;
use url::Url;

use crate::backend::errors::ForgeError;
use crate::backend::fs_util::write_atomic;

/// Default desktop window size for a generated app.
const DEFAULT_WIDTH: u32 = 1200;
const DEFAULT_HEIGHT: u32 = 800;

/// Bundled default icons, reused from Forge's own icon set so generated apps
/// build without the user having to supply their own. Paths are relative to
/// this source file (`src-tauri/src/backend/`).
const ICON_PNG: &[u8] = include_bytes!("../../icons/icon.png");
const ICON_32: &[u8] = include_bytes!("../../icons/32x32.png");
const ICON_128: &[u8] = include_bytes!("../../icons/128x128.png");
const ICON_128_2X: &[u8] = include_bytes!("../../icons/128x128@2x.png");
const ICON_ICNS: &[u8] = include_bytes!("../../icons/icon.icns");
const ICON_ICO: &[u8] = include_bytes!("../../icons/icon.ico");

/// Options for generating a website-wrapper app.
#[derive(Debug, Clone, Default)]
pub struct WebAppOptions {
    /// Human-friendly app name, e.g. "My Store".
    pub name: String,
    /// The website address the app should open. May be entered without a
    /// scheme; it is normalized by [`normalize_url`].
    pub url: String,
    /// Optional reverse-DNS bundle identifier. Derived from `name` when absent.
    pub identifier: Option<String>,
    /// Optional initial window width / height.
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Normalize a user-entered website address into a valid `http(s)` URL string.
///
/// Accepts input with or without a scheme (`example.com` → `https://example.com/`)
/// and rejects empty, malformed, or non-web (e.g. `ftp`/`file`) addresses with
/// a plain-language error suitable for showing directly to a non-technical
/// user.
pub fn normalize_url(input: &str) -> Result<String, ForgeError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ForgeError::ConfigInvalid(
            "Please enter your website address.".to_string(),
        ));
    }

    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };

    let parsed = Url::parse(&candidate).map_err(|_| {
        ForgeError::ConfigInvalid(format!(
            "\"{input}\" doesn't look like a valid website address."
        ))
    })?;

    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ForgeError::ConfigInvalid(
            "Website addresses must start with http:// or https://.".to_string(),
        ));
    }

    match parsed.host_str() {
        Some(host) if !host.is_empty() => {}
        _ => {
            return Err(ForgeError::ConfigInvalid(format!(
                "\"{input}\" is missing a website name (like example.com)."
            )))
        }
    }

    Ok(parsed.to_string())
}

/// Build a filesystem-safe slug from an app name (lowercase, hyphen-separated).
pub fn slug(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in name.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    let cleaned = out.trim_matches('-');
    if cleaned.is_empty() {
        "my-app".to_string()
    } else {
        cleaned.to_string()
    }
}

/// Derive a valid Cargo package name (cannot start with a digit).
pub fn crate_name(name: &str) -> String {
    let s = slug(name);
    if s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("app-{s}")
    } else {
        s
    }
}

/// Derive a reverse-DNS bundle identifier from an app name, e.g.
/// "My Store" → "com.forge.mystore".
pub fn derive_identifier(name: &str) -> String {
    let cleaned: String = name
        .to_lowercase()
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect();
    let segment = if cleaned.is_empty() || cleaned.starts_with(|c: char| c.is_ascii_digit()) {
        format!("app{cleaned}")
    } else {
        cleaned
    };
    format!("com.forge.{segment}")
}

/// Escape a string for safe inclusion in HTML text/attribute content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Generate a complete Tauri project under `parent_dir` that opens `opts.url`.
///
/// Returns the path to the new project directory. Fails with a friendly error
/// if the inputs are invalid or a non-empty folder of the same name already
/// exists at the destination.
pub fn scaffold_web_app(parent_dir: &Path, opts: &WebAppOptions) -> Result<PathBuf, ForgeError> {
    let normalized_url = normalize_url(&opts.url)?;

    let name = opts.name.trim();
    if name.is_empty() {
        return Err(ForgeError::ConfigInvalid(
            "Please give your app a name.".to_string(),
        ));
    }

    let dir_slug = slug(name);
    let project_dir = parent_dir.join(&dir_slug);

    // Refuse to clobber an existing non-empty directory.
    if project_dir.exists()
        && fs::read_dir(&project_dir)
            .map(|mut it| it.next().is_some())
            .unwrap_or(false)
    {
        return Err(ForgeError::ConfigInvalid(format!(
            "A folder named \"{dir_slug}\" already exists here. \
             Pick a different app name or save location."
        )));
    }

    let identifier = opts
        .identifier
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| derive_identifier(name));

    let width = opts.width.unwrap_or(DEFAULT_WIDTH);
    let height = opts.height.unwrap_or(DEFAULT_HEIGHT);

    let src_tauri = project_dir.join("src-tauri");
    let icons_dir = src_tauri.join("icons");

    // tauri.conf.json — the single window points directly at the remote site.
    let conf = json!({
        "$schema": "https://schema.tauri.app/config/2",
        "productName": name,
        "version": "0.1.0",
        "identifier": identifier,
        "build": { "frontendDist": "../dist" },
        "app": {
            "windows": [{
                "label": "main",
                "title": name,
                "url": normalized_url,
                "width": width,
                "height": height,
                "resizable": true
            }],
            "security": { "csp": null }
        },
        "bundle": {
            "active": true,
            "targets": "all",
            "icon": [
                "icons/32x32.png",
                "icons/128x128.png",
                "icons/128x128@2x.png",
                "icons/icon.icns",
                "icons/icon.ico"
            ]
        }
    });
    let conf_str = serde_json::to_string_pretty(&conf)?;

    let cargo_toml = format!(
        "[package]\n\
         name = \"{crate_name}\"\n\
         version = \"0.1.0\"\n\
         description = \"{desc}\"\n\
         edition = \"2021\"\n\
         \n\
         [build-dependencies]\n\
         tauri-build = {{ version = \"2\", features = [] }}\n\
         \n\
         [dependencies]\n\
         tauri = {{ version = \"2\", features = [] }}\n",
        crate_name = crate_name(name),
        desc = name.replace('\\', " ").replace('"', "'"),
    );

    let build_rs = "fn main() {\n    tauri_build::build()\n}\n";

    let main_rs =
        "// Prevents an extra console window from opening on Windows in release builds.\n\
         #![cfg_attr(not(debug_assertions), windows_subsystem = \"windows\")]\n\
         \n\
         fn main() {\n    \
             tauri::Builder::default()\n        \
                 .run(tauri::generate_context!())\n        \
                 .expect(\"error while running the application\");\n\
         }\n";

    // A minimal capability so the app runs; the remote site has no Tauri/IPC
    // access by default (global Tauri injection stays off), so this is safe.
    let capability = json!({
        "identifier": "default",
        "description": "Default capability for the main window.",
        "windows": ["main"],
        "permissions": ["core:default"]
    });
    let capability_str = serde_json::to_string_pretty(&capability)?;

    // Tauri requires `frontendDist` to exist at build time. The window loads the
    // remote URL, so this placeholder is never actually shown.
    let safe_name = html_escape(name);
    let index_html = format!(
        "<!doctype html>\n\
         <html lang=\"en\">\n  \
           <head>\n    \
             <meta charset=\"utf-8\" />\n    \
             <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n    \
             <title>{safe_name}</title>\n  \
           </head>\n  \
           <body>\n    \
             <p>Opening {safe_name}…</p>\n  \
           </body>\n\
         </html>\n"
    );

    let gitignore = "# Build output\n/src-tauri/target\n";

    let readme = format!(
        "# {name}\n\n\
         A desktop app for **{url}**, created with Forge. When you open it, your \
         website loads in its own app window.\n\n\
         ## Turn this into an installable app\n\n\
         You need the free Tauri build tools on your computer. Forge can check \
         this for you under **Settings → Tools on your computer**, or install them \
         manually:\n\n\
         1. Install Rust: <https://www.rust-lang.org/tools/install>\n\
         2. Install the Tauri CLI: `cargo install tauri-cli --version \"^2\"`\n\n\
         Then build the installer from this folder:\n\n\
         ```\ncargo tauri build\n```\n\n\
         Your finished installer appears in `src-tauri/target/release/bundle/`.\n\n\
         ## Change the app icon\n\n\
         Replace the images in `src-tauri/icons/` with your own, then build again.\n",
        name = name,
        url = normalized_url,
    );

    // Write project files. `write_atomic` creates parent directories as needed.
    write_atomic(&src_tauri.join("tauri.conf.json"), conf_str.as_bytes())?;
    write_atomic(&src_tauri.join("Cargo.toml"), cargo_toml.as_bytes())?;
    write_atomic(&src_tauri.join("build.rs"), build_rs.as_bytes())?;
    write_atomic(&src_tauri.join("src").join("main.rs"), main_rs.as_bytes())?;
    write_atomic(
        &src_tauri.join("capabilities").join("default.json"),
        capability_str.as_bytes(),
    )?;
    write_atomic(
        &project_dir.join("dist").join("index.html"),
        index_html.as_bytes(),
    )?;
    write_atomic(&project_dir.join(".gitignore"), gitignore.as_bytes())?;
    write_atomic(&project_dir.join("README.md"), readme.as_bytes())?;

    // Default icons.
    write_atomic(&icons_dir.join("icon.png"), ICON_PNG)?;
    write_atomic(&icons_dir.join("32x32.png"), ICON_32)?;
    write_atomic(&icons_dir.join("128x128.png"), ICON_128)?;
    write_atomic(&icons_dir.join("128x128@2x.png"), ICON_128_2X)?;
    write_atomic(&icons_dir.join("icon.icns"), ICON_ICNS)?;
    write_atomic(&icons_dir.join("icon.ico"), ICON_ICO)?;

    Ok(project_dir)
}

/// A friendly default location for generated apps: `~/Forge Apps`.
pub fn default_app_dir() -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("Forge Apps")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_url_adds_https_scheme() {
        assert_eq!(
            normalize_url("example.com").unwrap(),
            "https://example.com/"
        );
        assert_eq!(
            normalize_url("  example.com/path  ").unwrap(),
            "https://example.com/path"
        );
    }

    #[test]
    fn normalize_url_keeps_existing_scheme() {
        assert_eq!(
            normalize_url("http://example.com").unwrap(),
            "http://example.com/"
        );
        assert_eq!(
            normalize_url("https://shop.example.com/store").unwrap(),
            "https://shop.example.com/store"
        );
    }

    #[test]
    fn normalize_url_rejects_bad_input() {
        assert!(normalize_url("").is_err());
        assert!(normalize_url("   ").is_err());
        assert!(normalize_url("ftp://example.com").is_err());
        assert!(normalize_url("https://").is_err());
    }

    #[test]
    fn slug_and_identifier_are_safe() {
        assert_eq!(slug("My Cool Store!!"), "my-cool-store");
        assert_eq!(slug("   "), "my-app");
        assert_eq!(crate_name("123 app"), "app-123-app");
        assert_eq!(derive_identifier("My Store"), "com.forge.mystore");
        assert_eq!(derive_identifier("123"), "com.forge.app123");
        assert_eq!(derive_identifier("!!!"), "com.forge.app");
    }

    #[test]
    fn scaffold_writes_a_buildable_project() {
        let dir = tempfile::tempdir().unwrap();
        let opts = WebAppOptions {
            name: "My Store".to_string(),
            url: "mystore.com".to_string(),
            ..Default::default()
        };

        let project = scaffold_web_app(dir.path(), &opts).unwrap();
        assert_eq!(project, dir.path().join("my-store"));

        let conf_path = project.join("src-tauri").join("tauri.conf.json");
        let conf: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&conf_path).unwrap()).unwrap();
        assert_eq!(conf["productName"], "My Store");
        assert_eq!(conf["identifier"], "com.forge.mystore");
        assert_eq!(conf["app"]["windows"][0]["url"], "https://mystore.com/");
        assert_eq!(conf["app"]["windows"][0]["label"], "main");

        // Core project files exist.
        for rel in [
            "src-tauri/Cargo.toml",
            "src-tauri/build.rs",
            "src-tauri/src/main.rs",
            "src-tauri/capabilities/default.json",
            "dist/index.html",
            "README.md",
            ".gitignore",
        ] {
            assert!(project.join(rel).exists(), "missing {rel}");
        }

        // Icons are written and non-empty.
        for icon in ["32x32.png", "128x128.png", "icon.icns", "icon.ico"] {
            let p = project.join("src-tauri").join("icons").join(icon);
            assert!(p.exists(), "missing icon {icon}");
            assert!(fs::metadata(&p).unwrap().len() > 0, "empty icon {icon}");
        }
    }

    #[test]
    fn scaffold_refuses_non_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let existing = dir.path().join("my-store");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("keep.txt"), b"hi").unwrap();

        let opts = WebAppOptions {
            name: "My Store".to_string(),
            url: "mystore.com".to_string(),
            ..Default::default()
        };
        assert!(scaffold_web_app(dir.path(), &opts).is_err());
    }

    #[test]
    fn scaffold_rejects_invalid_url() {
        let dir = tempfile::tempdir().unwrap();
        let opts = WebAppOptions {
            name: "Broken".to_string(),
            url: "not a url".to_string(),
            ..Default::default()
        };
        // "not a url" has a space; without a scheme it becomes
        // "https://not a url" which fails to parse.
        assert!(scaffold_web_app(dir.path(), &opts).is_err());
    }
}
