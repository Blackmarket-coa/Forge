//! The shared BMC extension manifest (W3): the Rust mirror of the schema
//! FBM validates (`free-black-market/backend/src/modules/plugin-registry/manifest.ts`)
//! and the Blackout host consumes (`blackout/packages/blackout-protocol/src/plugins`).
//! The cross-repo contract is
//! `free-black-market/docs/contracts/extension-manifest.md`; the artifact-kind
//! and capability literals below are transcribed from it and kept in sync
//! manually.
//!
//! Forge authors the manifest; FBM injects the listing ref + signs at
//! publish. Host-compat bounds ride in the FBM-namespaced `fbm` block.
//! Unknown fields round-trip untouched (serde `flatten` here, zod
//! `.passthrough()` there) — the canonical hash covers what was authored,
//! never a stripped copy.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::backend::errors::ForgeError;
use crate::backend::fs_util::write_atomic;
use crate::backend::semver;

/// Transcribed from blackout `PluginArtifactKind` (14 kinds).
pub const ARTIFACT_KINDS: [&str; 14] = [
    "theme",
    "manifest_plugin",
    "code_plugin",
    "asset_bundle",
    "coalition_kit",
    "profile_cosmetic",
    "sound_pack",
    "community_template",
    "stream_asset",
    "vault_item",
    "ai_persona",
    "automation_recipe",
    "privacy_tool",
    "twitch_extension_compat",
];

/// Transcribed from blackout `PluginCapability` (10 capabilities).
pub const CAPABILITIES: [&str; 10] = [
    "shell.panel.read",
    "shell.panel.write",
    "message.read",
    "message.compose",
    "storage.read",
    "storage.write",
    "http.fetch",
    "ai.inference",
    "twitch.ext.identityShare",
    "twitch.ext.subscriptionStatus",
];

pub const MANIFEST_FILE: &str = "manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HomepageCard {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FbmBlock {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_host_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_host_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Free-form extras (e.g. `dataSource`) — round-trip untouched.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<u32>,
    pub artifact_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage_card: Option<HomepageCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fbm: Option<FbmBlock>,
    /// Everything else (pinnedNav, rightPanel, mobileTab, pluginDens, the
    /// server-injected `listing`, future fields) — preserved verbatim.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn is_hex_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn is_manifest_id(value: &str) -> bool {
    let len_ok = (3..=128).contains(&value.len());
    len_ok
        && value
            .chars()
            .next()
            .map(|c| c.is_ascii_alphanumeric())
            .unwrap_or(false)
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}

/// Validate an authored manifest the way FBM's authoring-mode validator does,
/// returning human-readable issues (the `validate_config` idiom — empty means
/// valid). `manifest_plugin` is validated fully; other kinds get the base
/// checks and publish-side rules stay FBM's job.
pub fn validate(manifest: &ExtensionManifest) -> Vec<String> {
    let mut issues = Vec::new();
    if !is_manifest_id(&manifest.id) {
        issues.push(format!(
            "id: \"{}\" must be reverse-DNS-ish (a-z0-9.-, 3-128 chars)",
            manifest.id
        ));
    }
    if manifest.name.trim().is_empty() || manifest.name.len() > 120 {
        issues.push("name: required, 1-120 characters".to_string());
    }
    if !semver::is_valid(&manifest.version) {
        issues.push(format!(
            "version: \"{}\" is not valid semver",
            manifest.version
        ));
    }
    if let Some(pv) = manifest.protocol_version {
        if pv != 1 && pv != 2 {
            issues.push(format!("protocolVersion: {pv} must be 1 or 2"));
        }
    }
    if !ARTIFACT_KINDS.contains(&manifest.artifact_kind.as_str()) {
        issues.push(format!(
            "artifactKind: \"{}\" is not a known artifact kind",
            manifest.artifact_kind
        ));
    }
    for capability in &manifest.capabilities {
        if !CAPABILITIES.contains(&capability.as_str()) {
            issues.push(format!(
                "capabilities: \"{capability}\" is not a known capability"
            ));
        }
    }
    if let Some(sha) = &manifest.sha256 {
        if !is_hex_sha256(sha) {
            issues.push("sha256: must be 64 hex characters".to_string());
        }
    }
    if manifest.artifact_kind == "code_plugin" && manifest.entry.is_none() {
        issues.push("entry: code_plugin artifacts need an entrypoint".to_string());
    }
    if let Some(fbm) = &manifest.fbm {
        for (label, bound) in [
            ("fbm.minHostVersion", &fbm.min_host_version),
            ("fbm.maxHostVersion", &fbm.max_host_version),
        ] {
            if let Some(value) = bound {
                if !semver::is_valid(value) {
                    issues.push(format!("{label}: \"{value}\" is not valid semver"));
                }
            }
        }
        if let Some(category) = &fbm.category {
            if !["MARKETPLACE_EXTENSION", "ANALYTICS", "AUTOMATION"].contains(&category.as_str()) {
                issues.push(format!(
                    "fbm.category: \"{category}\" is not a registry category"
                ));
            }
        }
    }
    if let Some(card) = &manifest.homepage_card {
        if card.title.trim().is_empty() || card.title.len() > 120 {
            issues.push("homepageCard.title: required, 1-120 characters".to_string());
        }
    }
    issues
}

/// Read + parse `<project>/manifest.json`. Returns the typed manifest AND the
/// raw `Value` (the raw form is what gets canonically hashed and published, so
/// nothing Forge doesn't model can be lost).
pub fn load(project_path: &Path) -> Result<(ExtensionManifest, Value), ForgeError> {
    let path = project_path.join(MANIFEST_FILE);
    if !path.exists() {
        return Err(ForgeError::ConfigNotFound(path.display().to_string()));
    }
    let raw = fs::read_to_string(&path)?;
    let value: Value = serde_json::from_str(&raw)?;
    let manifest: ExtensionManifest = serde_json::from_value(value.clone())?;
    Ok((manifest, value))
}

/// Write `<project>/manifest.json` atomically, pretty-printed for humans.
pub fn write(project_path: &Path, manifest: &Value) -> Result<(), ForgeError> {
    let path = project_path.join(MANIFEST_FILE);
    let pretty = serde_json::to_string_pretty(manifest)?;
    write_atomic(&path, format!("{pretty}\n").as_bytes())?;
    Ok(())
}

/// Canonical JSON: recursively key-sorted, compact — byte-compatible with
/// FBM's `canonicalJson` (JSON.stringify over sorted keys) for JSON-clean
/// values, which is what signing and verification hash on the other side.
pub fn canonical_json(value: &Value) -> String {
    fn sort(value: &Value) -> Value {
        match value {
            Value::Array(items) => Value::Array(items.iter().map(sort).collect()),
            Value::Object(map) => {
                let sorted: BTreeMap<&String, &Value> = map.iter().collect();
                let mut out = Map::new();
                for (key, val) in sorted {
                    out.insert(key.clone(), sort(val));
                }
                Value::Object(out)
            }
            other => other.clone(),
        }
    }
    serde_json::to_string(&sort(value)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn widget() -> Value {
        json!({
            "id": "coop.fbm.featured-vendor-widget",
            "name": "Featured Vendor Widget",
            "version": "0.1.0",
            "protocolVersion": 2,
            "artifactKind": "manifest_plugin",
            "capabilities": ["http.fetch"],
            "homepageCard": { "title": "Featured Vendors", "to": "/marketplace/featured-vendors", "order": 30 },
            "fbm": { "minHostVersion": "1.0.0", "dataSource": { "vendorsUrl": "/store/vendors?featured=true" } },
            "futureField": "kept"
        })
    }

    #[test]
    fn parses_validates_and_round_trips_unknown_fields() {
        let manifest: ExtensionManifest = serde_json::from_value(widget()).unwrap();
        assert!(validate(&manifest).is_empty());
        assert_eq!(manifest.artifact_kind, "manifest_plugin");
        assert!(manifest.extra.contains_key("futureField"));
        let back = serde_json::to_value(&manifest).unwrap();
        assert_eq!(back["futureField"], "kept");
        assert_eq!(
            back["fbm"]["dataSource"]["vendorsUrl"],
            "/store/vendors?featured=true"
        );
    }

    #[test]
    fn rejects_bad_fields_with_readable_issues() {
        let mut manifest: ExtensionManifest = serde_json::from_value(widget()).unwrap();
        manifest.id = "!".into();
        manifest.version = "latest".into();
        manifest.artifact_kind = "nonsense".into();
        manifest.capabilities = vec!["root.everything".into()];
        manifest.sha256 = Some("zz".into());
        manifest.fbm = Some(FbmBlock {
            min_host_version: Some("garbage".into()),
            category: Some("WRONG".into()),
            ..Default::default()
        });
        let issues = validate(&manifest);
        for needle in [
            "id:",
            "version:",
            "artifactKind:",
            "capabilities:",
            "sha256:",
            "fbm.minHostVersion:",
            "fbm.category:",
        ] {
            assert!(
                issues.iter().any(|i| i.starts_with(needle)),
                "missing issue {needle} in {issues:?}"
            );
        }
    }

    #[test]
    fn code_plugins_need_an_entry() {
        let mut manifest: ExtensionManifest = serde_json::from_value(widget()).unwrap();
        manifest.artifact_kind = "code_plugin".into();
        assert!(validate(&manifest).iter().any(|i| i.starts_with("entry:")));
    }

    #[test]
    fn canonical_json_sorts_keys_recursively() {
        let a = json!({ "b": 1, "a": { "d": [ { "y": 2, "x": 1 } ], "c": 2 } });
        assert_eq!(
            canonical_json(&a),
            r#"{"a":{"c":2,"d":[{"x":1,"y":2}]},"b":1}"#
        );
        // Order-independent: the same content serialized differently hashes alike.
        let b = json!({ "a": { "c": 2, "d": [ { "x": 1, "y": 2 } ] }, "b": 1 });
        assert_eq!(canonical_json(&a), canonical_json(&b));
    }

    #[test]
    fn load_and_write_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), &widget()).unwrap();
        let (manifest, raw) = load(dir.path()).unwrap();
        assert_eq!(manifest.id, "coop.fbm.featured-vendor-widget");
        assert_eq!(raw["futureField"], "kept");
    }
}
