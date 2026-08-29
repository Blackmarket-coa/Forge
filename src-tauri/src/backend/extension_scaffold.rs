//! Scaffold a BMC extension project (W3): a directory holding a
//! `manifest.json` (the shared extension manifest), an `assets/` folder, and
//! a README — the extension sibling of [`crate::backend::web_app`]'s
//! website-wrapper scaffolder, writing files directly via `write_atomic`.
//!
//! The first template is the **featured-vendor-widget** — a `manifest_plugin`
//! home card spotlighting FBM's promoted vendors (the W3 end-to-end
//! demonstrator; FBM ships the 1.0.0 first-party seed, so the template
//! starts at 0.1.0 for authors to iterate on).

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::backend::errors::ForgeError;
use crate::backend::extension_manifest::{self, ExtensionManifest};
use crate::backend::fs_util::write_atomic;
use crate::backend::web_app::slug;

/// Options for scaffolding an extension project.
#[derive(Debug, Clone, Default)]
pub struct ExtensionOptions {
    /// Human-friendly extension name, e.g. "My Vendor Widget".
    pub name: String,
    /// Template key; `"featured-vendor-widget"` or `"blank"` (default).
    pub template: Option<String>,
}

pub const TEMPLATES: [&str; 2] = ["blank", "featured-vendor-widget"];

fn template_manifest(name: &str, template: &str) -> Value {
    let ext_slug = slug(name);
    let id = format!("coop.forge.{ext_slug}");
    match template {
        "featured-vendor-widget" => json!({
            "id": id,
            "name": name,
            "version": "0.1.0",
            "protocolVersion": 2,
            "artifactKind": "manifest_plugin",
            "capabilities": ["http.fetch"],
            "description": "A home-surface card spotlighting currently promoted Black Market vendors.",
            "homepageCard": {
                "title": name,
                "subtitle": "Today's promoted Black Market vendors",
                "to": "/marketplace/featured-vendors",
                "order": 30
            },
            "fbm": {
                "minHostVersion": "1.0.0",
                "dataSource": {
                    "vendorsUrl": "/store/vendors?featured=true",
                    "entitlementFeatureKey": "vendor.promoted_listing"
                }
            }
        }),
        _ => json!({
            "id": id,
            "name": name,
            "version": "0.1.0",
            "protocolVersion": 2,
            "artifactKind": "manifest_plugin",
            "capabilities": [],
            "description": "",
            "homepageCard": { "title": name, "order": 100 }
        }),
    }
}

/// Write the extension project. Fails if the target directory already exists
/// (never overwrite user work). Returns the project path.
pub fn scaffold_extension(
    parent_dir: &Path,
    opts: &ExtensionOptions,
) -> Result<PathBuf, ForgeError> {
    let name = opts.name.trim();
    if name.is_empty() {
        return Err(ForgeError::ConfigInvalid(
            "Please enter a name for your extension.".to_string(),
        ));
    }
    let template = opts.template.as_deref().unwrap_or("blank");
    if !TEMPLATES.contains(&template) {
        return Err(ForgeError::ConfigInvalid(format!(
            "Unknown extension template \"{template}\"."
        )));
    }

    let project_dir = parent_dir.join(slug(name));
    if project_dir.exists() {
        return Err(ForgeError::ConfigInvalid(format!(
            "\"{}\" already exists — pick a different name or location.",
            project_dir.display()
        )));
    }

    let manifest = template_manifest(name, template);
    // Belt-and-braces: a template that fails its own schema is a Forge bug.
    let typed: ExtensionManifest = serde_json::from_value(manifest.clone())?;
    let issues = extension_manifest::validate(&typed);
    if !issues.is_empty() {
        return Err(ForgeError::ConfigInvalid(format!(
            "Template produced an invalid manifest: {}",
            issues.join("; ")
        )));
    }

    extension_manifest::write(&project_dir, &manifest)?;
    write_atomic(&project_dir.join("assets").join(".gitkeep"), b"")?;
    write_atomic(
        &project_dir.join("README.md"),
        format!(
            "# {name}\n\nA BMC extension ({template} template). Edit `manifest.json`, then use\nForge's Extensions view to package and publish it to the FreeBlackMarket\nregistry. Contract: the `extension-manifest` doc in free-black-market\n`docs/contracts/`.\n"
        )
        .as_bytes(),
    )?;
    Ok(project_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffolds_the_widget_template_with_a_valid_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let path = scaffold_extension(
            dir.path(),
            &ExtensionOptions {
                name: "My Vendor Widget".into(),
                template: Some("featured-vendor-widget".into()),
            },
        )
        .unwrap();
        assert!(path.ends_with("my-vendor-widget"));
        let (manifest, raw) = extension_manifest::load(&path).unwrap();
        assert!(extension_manifest::validate(&manifest).is_empty());
        assert_eq!(manifest.artifact_kind, "manifest_plugin");
        assert_eq!(
            raw["fbm"]["dataSource"]["entitlementFeatureKey"],
            "vendor.promoted_listing"
        );
        assert!(path.join("assets").exists());
        assert!(path.join("README.md").exists());
    }

    #[test]
    fn refuses_existing_dirs_empty_names_and_unknown_templates() {
        let dir = tempfile::tempdir().unwrap();
        let opts = ExtensionOptions {
            name: "Twice".into(),
            template: None,
        };
        scaffold_extension(dir.path(), &opts).unwrap();
        assert!(scaffold_extension(dir.path(), &opts).is_err());
        assert!(scaffold_extension(
            dir.path(),
            &ExtensionOptions {
                name: "  ".into(),
                template: None
            }
        )
        .is_err());
        assert!(scaffold_extension(
            dir.path(),
            &ExtensionOptions {
                name: "X".into(),
                template: Some("nope".into())
            }
        )
        .is_err());
    }
}
