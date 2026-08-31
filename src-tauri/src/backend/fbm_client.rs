//! FreeBlackMarket registry client (W3): the publish half of
//! build → sign → publish → install. Forge holds NO signing keys — it
//! submits the authored manifest + digests through the seller API and FBM
//! signs at publish (`POST /v1/seller/listings/:id/publish`, per
//! `free-black-market/docs/contracts/extension-manifest.md`).
//!
//! Configuration lives per-machine in `~/.forge/fbm.json`
//! (`{ "api_base_url": "...", "seller_token": "...", "publishable_key": "..." }`)
//! — never in project files; the seller token is masked in every log/status
//! surface like license keys (the publishable key is public by design).
//! Dark by default: with no config present, publish/browse return a typed,
//! plain-language error pointing at Settings instead of making any request.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::backend::errors::ForgeError;
use crate::backend::extension_manifest::ExtensionManifest;
use crate::backend::extension_package::ExtensionDigests;
use crate::backend::fs_util::write_atomic;
use crate::backend::web_app::slug;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FbmConfig {
    pub api_base_url: Option<String>,
    pub seller_token: Option<String>,
    /// FBM storefront publishable key — public by design (it ships in
    /// storefronts), but required: Medusa rejects every `/store/*` request
    /// without one, and registry browsing reads `/store/plugins`.
    pub publishable_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FbmStatus {
    pub configured: bool,
    pub api_base_url: Option<String>,
    /// Masked (`first6...last4`) — the raw token never leaves the backend.
    pub seller_token_masked: Option<String>,
    /// Shown in full — the publishable key is public by design.
    pub publishable_key: Option<String>,
}

fn config_path() -> Result<PathBuf, ForgeError> {
    let home = dirs_home().ok_or_else(|| {
        ForgeError::ConfigInvalid("Could not determine your home directory.".to_string())
    })?;
    Ok(home.join(".forge").join("fbm.json"))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

pub fn read_config() -> Result<FbmConfig, ForgeError> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(FbmConfig::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn write_config(config: &FbmConfig) -> Result<(), ForgeError> {
    let path = config_path()?;
    let pretty = serde_json::to_string_pretty(config)?;
    write_atomic(&path, format!("{pretty}\n").as_bytes())?;
    Ok(())
}

/// Every artifact kind except `manifest_plugin` ships a hosted bundle, so
/// publishing it needs the bundle's public address (`code_blob_url`).
pub fn blob_required(artifact_kind: &str) -> bool {
    artifact_kind != "manifest_plugin"
}

/// Pre-flight for asset-carrying kinds: refuse to publish without the hosted
/// bundle's address + hash, with a plain-language pointer at the fix. Pure,
/// so the rule is unit-testable without any network or config.
fn require_bundle_inputs(
    artifact_kind: &str,
    code_blob_url: Option<&str>,
    code_sha256: Option<&str>,
) -> Result<(), ForgeError> {
    if !blob_required(artifact_kind) {
        return Ok(());
    }
    if code_blob_url
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .is_none()
    {
        return Err(ForgeError::ConfigInvalid(format!(
            "\"{artifact_kind}\" extensions ship an asset bundle. Upload the \
             packaged dist/<name>-<version>.zip somewhere public (your website \
             or a release page) and paste its address before publishing."
        )));
    }
    if code_sha256.is_none() {
        return Err(ForgeError::ConfigInvalid(
            "Package the extension first so the bundle hash is computed.".to_string(),
        ));
    }
    Ok(())
}

pub fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        return "***".to_string();
    }
    format!("{}...{}", &token[..6], &token[token.len() - 4..])
}

pub fn status() -> Result<FbmStatus, ForgeError> {
    let config = read_config()?;
    let configured = config
        .api_base_url
        .as_deref()
        .is_some_and(|v| !v.is_empty())
        && config
            .seller_token
            .as_deref()
            .is_some_and(|v| !v.is_empty());
    Ok(FbmStatus {
        configured,
        api_base_url: config.api_base_url.clone(),
        seller_token_masked: config.seller_token.as_deref().map(mask_token),
        publishable_key: config.publishable_key.clone(),
    })
}

fn require_base_url(config: &FbmConfig) -> Result<String, ForgeError> {
    match config.api_base_url.as_deref() {
        Some(url) if !url.trim().is_empty() => Ok(url.trim().trim_end_matches('/').to_string()),
        _ => Err(ForgeError::ConfigInvalid(
            "FreeBlackMarket isn't configured yet — set the API address in Settings.".to_string(),
        )),
    }
}

fn require_token(config: &FbmConfig) -> Result<String, ForgeError> {
    match config.seller_token.as_deref() {
        Some(token) if !token.trim().is_empty() => Ok(token.trim().to_string()),
        _ => Err(ForgeError::ConfigInvalid(
            "FreeBlackMarket isn't configured yet — add your seller token in Settings.".to_string(),
        )),
    }
}

fn require_publishable_key(config: &FbmConfig) -> Result<String, ForgeError> {
    match config.publishable_key.as_deref() {
        Some(key) if !key.trim().is_empty() => Ok(key.trim().to_string()),
        _ => Err(ForgeError::ConfigInvalid(
            "Registry browsing needs the FreeBlackMarket publishable key — add it in Settings."
                .to_string(),
        )),
    }
}

async fn error_detail(response: reqwest::Response) -> String {
    let status = response.status();
    let body: Value = response.json().await.unwrap_or(Value::Null);
    let type_field = body.get("type").and_then(Value::as_str).unwrap_or("");
    let message = body
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("request failed");
    if type_field.is_empty() {
        format!("{status}: {message}")
    } else {
        format!("{status} {type_field}: {message}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishOutcome {
    pub listing_id: String,
    pub plugin_slug: Option<String>,
    pub plugin_version: Option<String>,
    pub envelope: Value,
}

/// Create-or-update the seller listing for this extension, then publish it.
/// `code_blob_url`/`code_blob_sha256` are only sent when a blob exists —
/// `manifest_plugin` needs none per the contract's relaxation.
pub async fn publish_extension(
    manifest: &ExtensionManifest,
    raw_manifest: &Value,
    digests: &ExtensionDigests,
    code_blob_url: Option<&str>,
) -> Result<PublishOutcome, ForgeError> {
    require_bundle_inputs(
        &manifest.artifact_kind,
        code_blob_url,
        digests.code_sha256.as_deref(),
    )?;
    let config = read_config()?;
    let base = require_base_url(&config)?;
    let token = require_token(&config)?;
    let client = reqwest::Client::new();
    let plugin_slug = slug(&manifest.name);

    // 1. Find an existing draft by slug, else create one.
    let listing_url = format!("{base}/v1/seller/listings");
    let mut body = json!({
        "slug": plugin_slug,
        "title": manifest.name,
        "description": manifest.description,
        "manifest": raw_manifest,
        "version": manifest.version,
        "plugin_slug": plugin_slug,
    });
    if let (Some(url), Some(sha)) = (code_blob_url, digests.code_sha256.as_deref()) {
        body["code_blob_url"] = json!(url);
        body["code_blob_sha256"] = json!(sha);
    }

    let create = client
        .post(&listing_url)
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .map_err(|e| ForgeError::ProcessError(format!("publish request failed: {e}")))?;

    let listing_id = if create.status().is_success() {
        let created: Value = create
            .json()
            .await
            .map_err(|e| ForgeError::ProcessError(format!("invalid create response: {e}")))?;
        created
            .pointer("/listing/id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ForgeError::ProcessError("create response carried no listing id".to_string())
            })?
            .to_string()
    } else if create.status().as_u16() == 409 {
        // duplicate_slug: find the existing listing and update the draft.
        let list: Value = client
            .get(&listing_url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| ForgeError::ProcessError(format!("listing lookup failed: {e}")))?
            .json()
            .await
            .map_err(|e| ForgeError::ProcessError(format!("invalid listings response: {e}")))?;
        let id = list
            .pointer("/listings")
            .and_then(Value::as_array)
            .and_then(|rows| {
                rows.iter().find(|row| {
                    row.get("slug").and_then(Value::as_str) == Some(plugin_slug.as_str())
                })
            })
            .and_then(|row| row.get("id"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                ForgeError::ProcessError(
                    "a listing with this slug exists but could not be found".to_string(),
                )
            })?;
        let mut patch = body.clone();
        patch.as_object_mut().map(|obj| obj.remove("slug"));
        let update = client
            .patch(format!("{base}/v1/seller/listings/{id}"))
            .bearer_auth(&token)
            .json(&patch)
            .send()
            .await
            .map_err(|e| ForgeError::ProcessError(format!("draft update failed: {e}")))?;
        if !update.status().is_success() {
            return Err(ForgeError::ProcessError(error_detail(update).await));
        }
        id
    } else {
        return Err(ForgeError::ProcessError(error_detail(create).await));
    };

    // 2. Publish — FBM validates, signs, and bridges into the registry.
    let publish = client
        .post(format!("{base}/v1/seller/listings/{listing_id}/publish"))
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| ForgeError::ProcessError(format!("publish failed: {e}")))?;
    if !publish.status().is_success() {
        return Err(ForgeError::ProcessError(error_detail(publish).await));
    }
    let published: Value = publish
        .json()
        .await
        .map_err(|e| ForgeError::ProcessError(format!("invalid publish response: {e}")))?;

    Ok(PublishOutcome {
        listing_id,
        plugin_slug: published
            .pointer("/plugin/slug")
            .and_then(Value::as_str)
            .map(str::to_string),
        plugin_version: published
            .pointer("/plugin/version")
            .and_then(Value::as_str)
            .map(str::to_string),
        envelope: published.get("envelope").cloned().unwrap_or(Value::Null),
    })
}

/// Read-only registry browse against the public list route — the real
/// implementation of the long-dead `plugin_browser` paywall entry. Needs the
/// base URL plus the storefront publishable key (Medusa gates `/store/*`
/// behind one; it is a public key, not a secret).
pub async fn browse_plugins(category: Option<&str>) -> Result<Value, ForgeError> {
    let config = read_config()?;
    let base = require_base_url(&config)?;
    let publishable_key = require_publishable_key(&config)?;
    let mut url = format!("{base}/store/plugins");
    if let Some(cat) = category {
        url.push_str(&format!("?category={cat}"));
    }
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("x-publishable-api-key", publishable_key)
        .send()
        .await
        .map_err(|e| ForgeError::ProcessError(format!("registry browse failed: {e}")))?;
    if !response.status().is_success() {
        return Err(ForgeError::ProcessError(error_detail(response).await));
    }
    response
        .json()
        .await
        .map_err(|e| ForgeError::ProcessError(format!("invalid registry response: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_tokens_and_reports_configuration_state() {
        assert_eq!(mask_token("short"), "***");
        assert_eq!(mask_token("seller-token-123456"), "seller...3456");
    }

    #[test]
    fn unconfigured_client_fails_closed_with_a_settings_pointer() {
        let config = FbmConfig::default();
        assert!(require_base_url(&config).is_err());
        assert!(require_token(&config).is_err());
        assert!(require_publishable_key(&config).is_err());
        let config = FbmConfig {
            api_base_url: Some("https://api.fbm.test/".into()),
            seller_token: Some(" tok ".into()),
            publishable_key: Some(" pk_test ".into()),
        };
        assert_eq!(require_base_url(&config).unwrap(), "https://api.fbm.test");
        assert_eq!(require_token(&config).unwrap(), "tok");
        assert_eq!(require_publishable_key(&config).unwrap(), "pk_test");
    }

    #[test]
    fn bundle_inputs_are_required_for_asset_kinds_only() {
        // manifest_plugin publishes with nothing hosted.
        assert!(require_bundle_inputs("manifest_plugin", None, None).is_ok());

        // Asset kinds need both the address and the packaged hash.
        let err = require_bundle_inputs("theme", None, Some("ab".repeat(32).as_str()))
            .unwrap_err()
            .to_string();
        assert!(err.contains("asset bundle"), "{err}");
        let err = require_bundle_inputs("theme", Some("   "), Some("hash"))
            .unwrap_err()
            .to_string();
        assert!(err.contains("asset bundle"), "{err}");
        let err = require_bundle_inputs("vault_item", Some("https://x/kit.zip"), None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("Package the extension"), "{err}");
        assert!(
            require_bundle_inputs("vault_item", Some("https://x/kit.zip"), Some("hash")).is_ok()
        );
    }

    #[test]
    fn configs_written_before_the_publishable_key_still_load() {
        let config: FbmConfig = serde_json::from_str(
            r#"{ "api_base_url": "https://api.fbm.test", "seller_token": "tok" }"#,
        )
        .unwrap();
        assert_eq!(config.publishable_key, None);
    }
}
