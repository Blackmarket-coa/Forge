//! Package an extension project (W3): compute the digests FBM's publish flow
//! consumes and write a deterministic `dist/<slug>-<version>/` bundle.
//!
//! Semantics note (per the cross-repo contract): the AUTHORITATIVE hashing +
//! signing happens server-side over the FBM-built distribution manifest —
//! Forge's digests are the local pre-check plus the inputs it submits
//! (`manifest.sha256` = declarative-payload hash for `manifest_plugin`;
//! `code_sha256` → `code_blob_sha256`, the hash of the hosted bundle).
//! `manifest_plugin` artifacts produce no archive (there are no bytes to
//! host — FBM stores and serves everything). Every other kind gets a
//! **deterministic zip** at `dist/<slug>-<version>.zip` (manifest + entry +
//! assets, sorted entries, fixed timestamps) — the file the author uploads
//! and whose hash FBM binds into the signed envelope.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::backend::errors::ForgeError;
use crate::backend::extension_manifest::{self, canonical_json};
use crate::backend::fs_util::write_atomic;
use crate::backend::web_app::slug;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDigests {
    /// Hex SHA-256 of the canonical-JSON authored manifest.
    pub manifest_sha256: String,
    /// Hex SHA-256 of the hosted bundle (`code_blob_sha256` at publish):
    /// the deterministic zip for asset-carrying kinds, absent for
    /// `manifest_plugin` (nothing is hosted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_sha256: Option<String>,
    /// Hex SHA-256 of the entry file itself when one exists (informational).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_sha256: Option<String>,
    /// Hex SHA-256 of the declarative payload (`homepageCard` + `fbm.dataSource`)
    /// — what `manifest.sha256` carries for `manifest_plugin` artifacts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_sha256: Option<String>,
    /// Per-asset hex SHA-256, keyed by path relative to `assets/`.
    pub asset_hashes: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageResult {
    pub dist_dir: String,
    pub digests: ExtensionDigests,
    /// Path of the deterministic bundle zip, for kinds that need hosting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_path: Option<String>,
    /// True when publishing requires hosting the bundle and passing its URL
    /// (every kind except `manifest_plugin`).
    pub needs_blob: bool,
    /// Human-readable validation issues (empty = valid).
    pub issues: Vec<String>,
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// The declarative payload a `manifest_plugin`'s `sha256` binds: the render
/// card plus the `fbm.dataSource` block, canonically serialized. Mirrors the
/// FBM first-party seed's construction.
fn declarative_payload(raw: &Value) -> Value {
    let mut payload = serde_json::Map::new();
    if let Some(card) = raw.get("homepageCard") {
        payload.insert("homepageCard".to_string(), card.clone());
    }
    if let Some(data_source) = raw.get("fbm").and_then(|f| f.get("dataSource")) {
        payload.insert("dataSource".to_string(), data_source.clone());
    }
    Value::Object(payload)
}

fn collect_asset_hashes(
    assets_dir: &Path,
) -> Result<std::collections::BTreeMap<String, String>, ForgeError> {
    let mut hashes = std::collections::BTreeMap::new();
    if !assets_dir.exists() {
        return Ok(hashes);
    }
    for entry in walkdir::WalkDir::new(assets_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(assets_dir)
            .map_err(|e| ForgeError::ConfigInvalid(e.to_string()))?
            .to_string_lossy()
            .replace('\\', "/");
        if rel == ".gitkeep" {
            continue;
        }
        let bytes = fs::read(entry.path())?;
        hashes.insert(rel, sha256_hex(&bytes));
    }
    Ok(hashes)
}

/// Validate, hash, stamp `manifest.sha256` for `manifest_plugin` artifacts
/// (the declarative payload hash), and write the deterministic
/// `dist/<slug>-<version>/` bundle (manifest + assets + `digests.json`).
/// Two runs over unchanged sources produce identical digests.
pub fn package_extension(project_path: &Path) -> Result<PackageResult, ForgeError> {
    let (manifest, mut raw) = extension_manifest::load(project_path)?;
    let mut issues = extension_manifest::validate(&manifest);

    // manifest_plugin: bind the declarative payload hash into the manifest
    // before hashing it, exactly like FBM's first-party seed does.
    if manifest.artifact_kind == "manifest_plugin" {
        let payload_hash = sha256_hex(canonical_json(&declarative_payload(&raw)).as_bytes());
        raw["sha256"] = Value::String(payload_hash);
        extension_manifest::write(project_path, &raw)?;
    }

    // Hash the entry file when one exists on disk (informational; the hosted
    // blob is the bundle zip below).
    let mut entry_sha256 = None;
    if let Some(entry) = &manifest.entry {
        let entry_path = project_path.join(entry);
        if entry_path.exists() {
            entry_sha256 = Some(sha256_hex(&fs::read(&entry_path)?));
        } else if manifest.artifact_kind == "code_plugin" {
            issues.push(format!("entry: \"{entry}\" does not exist in the project"));
        }
    }

    let asset_hashes = collect_asset_hashes(&project_path.join("assets"))?;
    let (reloaded, raw_final) = extension_manifest::load(project_path)?;
    let manifest_sha256 = sha256_hex(canonical_json(&raw_final).as_bytes());
    let payload_sha256 = if reloaded.artifact_kind == "manifest_plugin" {
        reloaded.sha256.clone()
    } else {
        None
    };

    let mut digests = ExtensionDigests {
        manifest_sha256,
        code_sha256: None,
        entry_sha256,
        payload_sha256,
        asset_hashes,
    };

    let dist_dir: PathBuf =
        project_path
            .join("dist")
            .join(format!("{}-{}", slug(&reloaded.name), reloaded.version));
    let manifest_pretty = serde_json::to_string_pretty(&raw_final)?;
    write_atomic(
        &dist_dir.join(extension_manifest::MANIFEST_FILE),
        format!("{manifest_pretty}\n").as_bytes(),
    )?;
    let digests_pretty = serde_json::to_string_pretty(&digests)?;
    write_atomic(
        &dist_dir.join("digests.json"),
        format!("{digests_pretty}\n").as_bytes(),
    )?;
    for rel in digests.asset_hashes.keys() {
        let src = project_path.join("assets").join(rel);
        let dst = dist_dir.join("assets").join(rel);
        write_atomic(&dst, &fs::read(&src)?)?;
    }

    // Asset-carrying kinds ship a hosted bundle: build the deterministic zip
    // and bind its hash as the blob hash the publish flow submits.
    let needs_blob = reloaded.artifact_kind != "manifest_plugin";
    let mut bundle_path = None;
    if needs_blob {
        let zip_path = project_path.join("dist").join(format!(
            "{}-{}.zip",
            slug(&reloaded.name),
            reloaded.version
        ));
        let bundle_sha256 = write_bundle_zip(
            project_path,
            &zip_path,
            &manifest_pretty,
            reloaded.entry.as_deref(),
            &digests,
        )?;
        digests.code_sha256 = Some(bundle_sha256);
        bundle_path = Some(zip_path.display().to_string());
    }

    Ok(PackageResult {
        dist_dir: dist_dir.display().to_string(),
        digests,
        bundle_path,
        needs_blob,
        issues,
    })
}

/// Write the hosted bundle: manifest.json, the entry file (when present), and
/// every asset, with sorted entry order and fixed timestamps so two runs over
/// unchanged sources produce byte-identical zips. Returns the zip's SHA-256.
fn write_bundle_zip(
    project_path: &Path,
    zip_path: &Path,
    manifest_pretty: &str,
    entry: Option<&str>,
    digests: &ExtensionDigests,
) -> Result<String, ForgeError> {
    if let Some(parent) = zip_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = fs::File::create(zip_path)?;
    let mut writer = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());

    let mut add = |name: &str, bytes: &[u8]| -> Result<(), ForgeError> {
        writer
            .start_file(name, options)
            .map_err(|e| ForgeError::ProcessError(format!("could not write bundle zip: {e}")))?;
        use std::io::Write;
        writer.write_all(bytes)?;
        Ok(())
    };

    add(
        extension_manifest::MANIFEST_FILE,
        format!("{manifest_pretty}\n").as_bytes(),
    )?;
    if let Some(entry_rel) = entry {
        let entry_path = project_path.join(entry_rel);
        if entry_path.exists() {
            add(entry_rel, &fs::read(&entry_path)?)?;
        }
    }
    for rel in digests.asset_hashes.keys() {
        let bytes = fs::read(project_path.join("assets").join(rel))?;
        add(&format!("assets/{rel}"), &bytes)?;
    }

    writer
        .finish()
        .map_err(|e| ForgeError::ProcessError(format!("could not finish bundle zip: {e}")))?;

    Ok(sha256_hex(&fs::read(zip_path)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::extension_scaffold::{scaffold_extension, ExtensionOptions};

    fn scaffold_widget(dir: &Path) -> PathBuf {
        scaffold_extension(
            dir,
            &ExtensionOptions {
                name: "Widget".into(),
                template: Some("featured-vendor-widget".into()),
            },
        )
        .unwrap()
    }

    #[test]
    fn packaging_is_deterministic_and_stamps_the_payload_hash() {
        let dir = tempfile::tempdir().unwrap();
        let project = scaffold_widget(dir.path());
        write_atomic(&project.join("assets").join("icon.svg"), b"<svg/>").unwrap();

        let first = package_extension(&project).unwrap();
        let second = package_extension(&project).unwrap();
        assert!(first.issues.is_empty());
        assert_eq!(
            first.digests.manifest_sha256,
            second.digests.manifest_sha256
        );
        assert_eq!(first.digests.payload_sha256, second.digests.payload_sha256);
        assert_eq!(first.digests.asset_hashes, second.digests.asset_hashes);
        assert_eq!(first.digests.asset_hashes.len(), 1);

        // The manifest now carries the declarative payload hash.
        let (manifest, _) = extension_manifest::load(&project).unwrap();
        assert_eq!(
            manifest.sha256.as_deref(),
            first.digests.payload_sha256.as_deref()
        );
        assert!(project
            .join("dist")
            .join("widget-0.1.0")
            .join("digests.json")
            .exists());
    }

    #[test]
    fn changing_the_payload_changes_the_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let project = scaffold_widget(dir.path());
        let before = package_extension(&project).unwrap();

        let (_m, mut raw) = extension_manifest::load(&project).unwrap();
        raw["homepageCard"]["title"] = serde_json::Value::String("Changed".into());
        extension_manifest::write(&project, &raw).unwrap();

        let after = package_extension(&project).unwrap();
        assert_ne!(before.digests.payload_sha256, after.digests.payload_sha256);
        assert_ne!(
            before.digests.manifest_sha256,
            after.digests.manifest_sha256
        );
    }

    #[test]
    fn manifest_plugin_needs_no_blob_and_ships_no_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let project = scaffold_widget(dir.path());
        let result = package_extension(&project).unwrap();
        assert!(!result.needs_blob);
        assert!(result.bundle_path.is_none());
        assert!(result.digests.code_sha256.is_none());
    }

    #[test]
    fn asset_kinds_get_a_deterministic_bundle_zip() {
        let dir = tempfile::tempdir().unwrap();
        let project = scaffold_extension(
            dir.path(),
            &ExtensionOptions {
                name: "Night Theme".into(),
                template: Some("theme".into()),
            },
        )
        .unwrap();

        let first = package_extension(&project).unwrap();
        assert!(first.issues.is_empty(), "{:?}", first.issues);
        assert!(first.needs_blob);
        let bundle = first.bundle_path.clone().expect("bundle path");
        assert!(bundle.ends_with("night-theme-0.1.0.zip"), "{bundle}");
        assert!(Path::new(&bundle).exists());
        let blob_hash = first.digests.code_sha256.clone().expect("blob hash");

        // Two runs over unchanged sources produce byte-identical bundles.
        let second = package_extension(&project).unwrap();
        assert_eq!(
            second.digests.code_sha256.as_deref(),
            Some(blob_hash.as_str())
        );
        assert_eq!(blob_hash, sha256_hex(&fs::read(&bundle).unwrap()));

        // Changing an asset changes the blob hash.
        write_atomic(&project.join("assets").join("theme.json"), b"{}").unwrap();
        let third = package_extension(&project).unwrap();
        assert_ne!(third.digests.code_sha256, second.digests.code_sha256);
    }

    #[test]
    fn code_plugin_missing_entry_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let project = scaffold_widget(dir.path());
        let (_m, mut raw) = extension_manifest::load(&project).unwrap();
        raw["artifactKind"] = serde_json::Value::String("code_plugin".into());
        raw["entry"] = serde_json::Value::String("index.js".into());
        extension_manifest::write(&project, &raw).unwrap();

        let result = package_extension(&project).unwrap();
        assert!(result.issues.iter().any(|i| i.contains("does not exist")));
    }
}
