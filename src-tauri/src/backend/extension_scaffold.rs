//! Scaffold a BMC extension project (W3): a directory holding a
//! `manifest.json` (the shared extension manifest), an `assets/` folder, and
//! a README — the extension sibling of [`crate::backend::web_app`]'s
//! website-wrapper scaffolder, writing files directly via `write_atomic`.
//!
//! Templates cover the Blackout-hosted artifact kinds an author can publish
//! through FBM's registry today. `manifest_plugin` templates publish
//! end-to-end with nothing to host (FBM stores and serves everything); the
//! asset-carrying kinds (theme, recipes, kits, vault items, privacy tools)
//! are packaged into a deterministic zip whose address the author supplies at
//! publish. The **featured-vendor-widget** stays the W3 end-to-end
//! demonstrator (FBM ships the 1.0.0 first-party seed, so the template starts
//! at 0.1.0 for authors to iterate on).

use std::path::{Path, PathBuf};

use serde::Serialize;
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
    /// Template key from [`TEMPLATES`]; `"blank"` when absent.
    pub template: Option<String>,
}

pub const TEMPLATES: [&str; 8] = [
    "blank",
    "featured-vendor-widget",
    "pinned-nav-panel",
    "theme",
    "automation-recipe",
    "coalition-kit",
    "vault-item",
    "privacy-tool",
];

/// UI-facing description of one template — served by
/// `get_extension_templates` so the Extensions view stays in sync with the
/// backend's template registry.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub artifact_kind: &'static str,
    /// True when publishing this kind requires hosting the packaged bundle
    /// (everything except `manifest_plugin` artifacts).
    pub needs_blob: bool,
}

pub fn templates() -> Vec<TemplateInfo> {
    vec![
        TemplateInfo {
            id: "featured-vendor-widget",
            label: "Featured Vendor Widget (home card)",
            description:
                "A Blackout home-surface card spotlighting promoted vendors — the end-to-end demo path. Publishes with nothing to host.",
            artifact_kind: "manifest_plugin",
            needs_blob: false,
        },
        TemplateInfo {
            id: "pinned-nav-panel",
            label: "Pinned navigation panel",
            description:
                "Adds an entry to Blackout's sidebar/nav rail. Publishes with nothing to host.",
            artifact_kind: "manifest_plugin",
            needs_blob: false,
        },
        TemplateInfo {
            id: "blank",
            label: "Blank manifest plugin",
            description: "A minimal manifest to build on. Publishes with nothing to host.",
            artifact_kind: "manifest_plugin",
            needs_blob: false,
        },
        TemplateInfo {
            id: "theme",
            label: "Theme",
            description:
                "Color/typography tokens restyling the Blackout client. Packaged as a zip you host at publish.",
            artifact_kind: "theme",
            needs_blob: true,
        },
        TemplateInfo {
            id: "automation-recipe",
            label: "Automation recipe",
            description:
                "A declarative trigger→steps recipe (lands in the registry's Automation category). Packaged as a zip you host at publish.",
            artifact_kind: "automation_recipe",
            needs_blob: true,
        },
        TemplateInfo {
            id: "coalition-kit",
            label: "Coalition kit",
            description:
                "A starter kit for a Blackout coalition: den layout, roles, and companion dens. Packaged as a zip you host at publish.",
            artifact_kind: "coalition_kit",
            needs_blob: true,
        },
        TemplateInfo {
            id: "vault-item",
            label: "Vault item",
            description:
                "Reusable secure-record templates (Black Mask-style personas/credentials, distributed through the shared registry). Packaged as a zip you host at publish.",
            artifact_kind: "vault_item",
            needs_blob: true,
        },
        TemplateInfo {
            id: "privacy-tool",
            label: "Privacy tool",
            description:
                "A privacy helper's settings + payload consumed by the Blackout host. Packaged as a zip you host at publish.",
            artifact_kind: "privacy_tool",
            needs_blob: true,
        },
    ]
}

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
        "pinned-nav-panel" => json!({
            "id": id,
            "name": name,
            "version": "0.1.0",
            "protocolVersion": 2,
            "artifactKind": "manifest_plugin",
            "capabilities": ["shell.panel.read"],
            "description": "A pinned entry in the Blackout sidebar that opens this extension's page.",
            "pinnedNav": {
                "label": name,
                "order": 40
            },
            "fbm": {
                "minHostVersion": "1.0.0"
            }
        }),
        "theme" => json!({
            "id": id,
            "name": name,
            "version": "0.1.0",
            "protocolVersion": 2,
            "artifactKind": "theme",
            "capabilities": [],
            "description": "A color and typography theme for the Blackout client.",
            "fbm": {
                "minHostVersion": "1.0.0"
            }
        }),
        "automation-recipe" => json!({
            "id": id,
            "name": name,
            "version": "0.1.0",
            "protocolVersion": 2,
            "artifactKind": "automation_recipe",
            "capabilities": ["http.fetch"],
            "description": "A declarative automation: when the trigger fires, run the steps.",
            "fbm": {
                "minHostVersion": "1.0.0"
            }
        }),
        "coalition-kit" => json!({
            "id": id,
            "name": name,
            "version": "0.1.0",
            "protocolVersion": 2,
            "artifactKind": "coalition_kit",
            "capabilities": [],
            "description": "A starter kit that provisions a coalition's spaces and roles.",
            "pluginDens": [
                { "purpose": "collaboration", "name": format!("{name} workroom") }
            ],
            "fbm": {
                "minHostVersion": "1.0.0"
            }
        }),
        "vault-item" => json!({
            "id": id,
            "name": name,
            "version": "0.1.0",
            "protocolVersion": 2,
            "artifactKind": "vault_item",
            "capabilities": [],
            "description": "Reusable secure-record templates installed through the shared registry.",
            "fbm": {
                "minHostVersion": "1.0.0"
            }
        }),
        "privacy-tool" => json!({
            "id": id,
            "name": name,
            "version": "0.1.0",
            "protocolVersion": 2,
            "artifactKind": "privacy_tool",
            "capabilities": ["storage.read", "storage.write"],
            "description": "A privacy helper the Blackout host surfaces to its users.",
            "fbm": {
                "minHostVersion": "1.0.0"
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

/// Starter asset files per template, written under `assets/`.
fn template_assets(name: &str, template: &str) -> Vec<(&'static str, String)> {
    match template {
        "theme" => vec![(
            "theme.json",
            serde_json::to_string_pretty(&json!({
                "name": name,
                "colors": {
                    "background": "#111111",
                    "surface": "#1b1b1b",
                    "accent": "#f5853f",
                    "text": "#f2f2f2",
                    "textMuted": "#9a9a9a"
                },
                "typography": {
                    "fontFamily": "inherit",
                    "monoFontFamily": "inherit"
                }
            }))
            .unwrap_or_default(),
        )],
        "automation-recipe" => vec![(
            "recipe.json",
            serde_json::to_string_pretty(&json!({
                "name": name,
                "trigger": {
                    "kind": "schedule",
                    "cron": "0 9 * * 1"
                },
                "steps": [
                    {
                        "kind": "http.fetch",
                        "url": "https://example.com/hook",
                        "method": "POST"
                    }
                ]
            }))
            .unwrap_or_default(),
        )],
        "coalition-kit" => vec![(
            "kit.json",
            serde_json::to_string_pretty(&json!({
                "name": name,
                "dens": [
                    { "name": "Announcements", "purpose": "update" },
                    { "name": "Workroom", "purpose": "collaboration" }
                ],
                "roles": [
                    { "name": "Steward", "permissions": ["invite", "moderate"] },
                    { "name": "Member", "permissions": [] }
                ]
            }))
            .unwrap_or_default(),
        )],
        "vault-item" => vec![(
            "vault-item.json",
            serde_json::to_string_pretty(&json!({
                "type": "secure_note",
                "title": name,
                "fields": [
                    { "name": "Notes", "kind": "textarea", "value": "" }
                ]
            }))
            .unwrap_or_default(),
        )],
        "privacy-tool" => vec![(
            "tool.json",
            serde_json::to_string_pretty(&json!({
                "name": name,
                "settings": [
                    {
                        "key": "enabled",
                        "label": "Enabled",
                        "kind": "boolean",
                        "default": true
                    }
                ]
            }))
            .unwrap_or_default(),
        )],
        _ => vec![],
    }
}

fn template_readme_section(template: &str) -> &'static str {
    match template {
        "featured-vendor-widget" | "blank" | "pinned-nav-panel" => {
            "\n## Publishing\n\nThis is a `manifest_plugin` — there is nothing to host. Package it in\nForge's Extensions view and publish: FBM validates, signs, stores, and\nsells it under entitlements; Blackout renders the declared surface.\n"
        }
        _ => {
            "\n## Publishing\n\nThis kind of extension ships an asset bundle. Packaging in Forge's\nExtensions view produces `dist/<slug>-<version>.zip` and its SHA-256.\nUpload that zip anywhere public (your website, a release page), then paste\nits address into the publish step — FBM binds the hash into the signed\nenvelope, so the bytes can live anywhere without being tamperable. The\nlisting, signature, versions, and sales all live on FreeBlackMarket.\n"
        }
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

    let assets = template_assets(name, template);
    if assets.is_empty() {
        write_atomic(&project_dir.join("assets").join(".gitkeep"), b"")?;
    } else {
        for (file, content) in assets {
            write_atomic(
                &project_dir.join("assets").join(file),
                format!("{content}\n").as_bytes(),
            )?;
        }
    }

    write_atomic(
        &project_dir.join("README.md"),
        format!(
            "# {name}\n\nA BMC extension ({template} template). Edit `manifest.json` (and the\nfiles under `assets/`), then use Forge's Extensions view to package and\npublish it to the FreeBlackMarket registry. Contract: the\n`extension-manifest` doc in free-black-market `docs/contracts/`.\n{section}",
            section = template_readme_section(template)
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
    fn every_template_scaffolds_a_valid_manifest_of_its_kind() {
        let infos = templates();
        for template in TEMPLATES {
            let dir = tempfile::tempdir().unwrap();
            let path = scaffold_extension(
                dir.path(),
                &ExtensionOptions {
                    name: format!("Test {template}"),
                    template: Some(template.into()),
                },
            )
            .unwrap_or_else(|e| panic!("template {template} failed to scaffold: {e}"));

            let (manifest, _raw) = extension_manifest::load(&path).unwrap();
            let issues = extension_manifest::validate(&manifest);
            assert!(issues.is_empty(), "{template}: {issues:?}");

            if let Some(info) = infos.iter().find(|i| i.id == template) {
                assert_eq!(
                    manifest.artifact_kind, info.artifact_kind,
                    "{template} kind mismatch"
                );
                // Asset-carrying templates ship a starter payload.
                if info.needs_blob && template != "blank" {
                    let has_assets = std::fs::read_dir(path.join("assets"))
                        .map(|it| it.count() > 0)
                        .unwrap_or(false);
                    assert!(has_assets, "{template} should scaffold starter assets");
                }
            }
        }
    }

    #[test]
    fn template_registry_covers_every_template_id() {
        let infos = templates();
        for template in TEMPLATES {
            assert!(
                infos.iter().any(|i| i.id == template),
                "missing TemplateInfo for {template}"
            );
        }
        assert_eq!(infos.len(), TEMPLATES.len());
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
