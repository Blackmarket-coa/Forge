use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const KEYGEN_API: &str = "https://api.keygen.sh/v1";
const OFFLINE_GRACE_SECS: u64 = 7 * 24 * 60 * 60;

/// Keygen account id baked in at compile time. Release builds get it from the
/// `KEYGEN_ACCOUNT_ID` environment variable of the build (see
/// `.github/workflows/release.yml`); a build made without it cannot validate
/// licenses at all.
const BUILD_KEYGEN_ACCOUNT_ID: Option<&str> = option_env!("KEYGEN_ACCOUNT_ID");

/// Error returned instead of an "invalid key" result when this build has no
/// Keygen account id, so users are not told a good key is bad.
pub const LICENSING_NOT_CONFIGURED: &str =
    "Licensing is not configured in this build of Forge, so license keys cannot be \
     validated. Official release builds include it; builds from source need \
     KEYGEN_ACCOUNT_ID set at compile time.";

/// Where the effective Keygen account id came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountIdSource {
    /// Runtime `KEYGEN_ACCOUNT_ID` override (debug builds only).
    RuntimeOverride,
    /// Baked in at compile time.
    Build,
}

fn normalize_account_id(raw: Option<&str>) -> Option<String> {
    let trimmed = raw?.trim();
    // "demo-account" was the old placeholder fallback; it never validates.
    if trimmed.is_empty() || trimmed == "demo-account" {
        return None;
    }
    Some(trimmed.to_string())
}

fn resolve_account_id(
    runtime: Option<&str>,
    build: Option<&str>,
    allow_runtime_override: bool,
) -> Option<(String, AccountIdSource)> {
    if allow_runtime_override {
        if let Some(id) = normalize_account_id(runtime) {
            return Some((id, AccountIdSource::RuntimeOverride));
        }
    }
    normalize_account_id(build).map(|id| (id, AccountIdSource::Build))
}

/// The Keygen account id this binary validates against, or `None` when
/// licensing is not configured. The runtime `KEYGEN_ACCOUNT_ID` override is
/// honored only in debug builds so a shipped binary cannot be pointed at a
/// different Keygen account.
pub fn keygen_account_id() -> Option<(String, AccountIdSource)> {
    let runtime = std::env::var("KEYGEN_ACCOUNT_ID").ok();
    resolve_account_id(
        runtime.as_deref(),
        BUILD_KEYGEN_ACCOUNT_ID,
        cfg!(debug_assertions),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub tier: String,
    pub valid: bool,
    pub expires_at: Option<String>,
    pub key_masked: Option<String>,
    pub checked_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LicenseFile {
    key: Option<String>,
    cached: Option<CachedValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedValidation {
    tier: String,
    valid: bool,
    expires_at: Option<String>,
    checked_at: u64,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

fn forge_home() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "unable to resolve home directory".to_string())?;
    Ok(PathBuf::from(home).join(".forge"))
}

fn license_path() -> Result<PathBuf, String> {
    Ok(forge_home()?.join("license.json"))
}

fn read_license_file() -> Result<LicenseFile, String> {
    let path = license_path()?;
    if !path.exists() {
        return Ok(LicenseFile::default());
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str::<LicenseFile>(&content).map_err(|e| e.to_string())
}

fn write_license_file(file: &LicenseFile) -> Result<(), String> {
    let path = license_path()?;
    let content = serde_json::to_string_pretty(file).map_err(|e| e.to_string())?;
    crate::backend::fs_util::write_atomic(&path, content.as_bytes()).map_err(|e| e.to_string())
}

fn mask_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 8 {
        return "****".to_string();
    }

    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{}****{}", prefix, suffix)
}

fn free_status() -> LicenseStatus {
    LicenseStatus {
        tier: "free".to_string(),
        valid: false,
        expires_at: None,
        key_masked: None,
        checked_at: None,
    }
}

fn status_from_cache(cache: &CachedValidation, key: Option<&str>) -> LicenseStatus {
    LicenseStatus {
        tier: cache.tier.clone(),
        valid: cache.valid,
        expires_at: cache.expires_at.clone(),
        key_masked: key.map(mask_key),
        checked_at: Some(cache.checked_at),
    }
}

fn parse_validation_response(value: &Value) -> (bool, String, Option<String>) {
    let valid = value
        .get("meta")
        .and_then(|v| v.get("valid"))
        .and_then(Value::as_bool)
        .or_else(|| {
            value
                .get("data")
                .and_then(|v| v.get("attributes"))
                .and_then(|v| v.get("status"))
                .and_then(Value::as_str)
                .map(|status| status.eq_ignore_ascii_case("active"))
        })
        .unwrap_or(false);

    let expires_at = value
        .pointer("/data/attributes/expiry")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            value
                .pointer("/data/attributes/expiresAt")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        });

    let mut tier = "free".to_string();
    let tier_candidates = [
        value.pointer("/meta/tier").and_then(Value::as_str),
        value
            .pointer("/data/attributes/metadata/tier")
            .and_then(Value::as_str),
        value
            .pointer("/data/attributes/policy/name")
            .and_then(Value::as_str),
        value
            .pointer("/data/attributes/name")
            .and_then(Value::as_str),
    ];

    for candidate in tier_candidates.into_iter().flatten() {
        let lower = candidate.to_lowercase();
        if lower.contains("team") {
            tier = "team".to_string();
            break;
        }
        if lower.contains("pro") {
            tier = "pro".to_string();
            break;
        }
    }

    if !valid {
        tier = "free".to_string();
    }

    (valid, tier, expires_at)
}

async fn validate_key_remote(key: &str) -> Result<CachedValidation, String> {
    let Some((account_id, _)) = keygen_account_id() else {
        return Err(LICENSING_NOT_CONFIGURED.to_string());
    };
    let endpoint = format!("{KEYGEN_API}/accounts/{account_id}/licenses/actions/validate-key");

    let client = reqwest::Client::new();
    let response = client
        .post(&endpoint)
        .header("Content-Type", "application/vnd.api+json")
        .json(&json!({"meta": {"key": key}}))
        .send()
        .await
        .map_err(|e| format!("license validation request failed: {e}"))?;

    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("invalid validation response: {e}"))?;

    if !status.is_success() {
        let detail = body
            .pointer("/errors/0/detail")
            .and_then(Value::as_str)
            .unwrap_or("license validation failed");
        return Err(detail.to_string());
    }

    let (valid, tier, expires_at) = parse_validation_response(&body);

    Ok(CachedValidation {
        tier,
        valid,
        expires_at,
        checked_at: now_unix(),
    })
}

pub async fn bootstrap_license_check() -> Result<LicenseStatus, String> {
    let file = read_license_file()?;
    let Some(key) = file.key.clone() else {
        return Ok(free_status());
    };

    if let Some(cache) = file.cached.as_ref() {
        let age = now_unix().saturating_sub(cache.checked_at);
        if cache.valid && age < OFFLINE_GRACE_SECS {
            return Ok(status_from_cache(cache, Some(key.as_str())));
        }
    }

    let cache = validate_key_remote(&key).await?;
    let mut next = file;
    next.cached = Some(cache.clone());
    write_license_file(&next)?;

    Ok(status_from_cache(&cache, Some(key.as_str())))
}

pub fn get_license_status() -> Result<LicenseStatus, String> {
    let file = read_license_file()?;
    match (file.key.as_deref(), file.cached.as_ref()) {
        (Some(key), Some(cache)) => Ok(status_from_cache(cache, Some(key))),
        _ => Ok(free_status()),
    }
}

pub async fn validate_and_store_license(key: String) -> Result<LicenseStatus, String> {
    let cache = validate_key_remote(&key).await?;

    let file = LicenseFile {
        key: Some(key.clone()),
        cached: Some(cache.clone()),
    };
    write_license_file(&file)?;

    Ok(status_from_cache(&cache, Some(&key)))
}

pub fn clear_license() -> Result<LicenseStatus, String> {
    let path = license_path()?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(free_status())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_id_missing_or_placeholder_is_not_configured() {
        assert_eq!(resolve_account_id(None, None, true), None);
        assert_eq!(resolve_account_id(Some(""), Some("  "), true), None);
        assert_eq!(
            resolve_account_id(Some("demo-account"), Some("demo-account"), true),
            None
        );
    }

    #[test]
    fn account_id_uses_build_value() {
        assert_eq!(
            resolve_account_id(None, Some("acct-123"), false),
            Some(("acct-123".to_string(), AccountIdSource::Build))
        );
    }

    #[test]
    fn runtime_override_only_when_allowed() {
        assert_eq!(
            resolve_account_id(Some("dev-acct"), Some("acct-123"), true),
            Some(("dev-acct".to_string(), AccountIdSource::RuntimeOverride))
        );
        assert_eq!(
            resolve_account_id(Some("dev-acct"), Some("acct-123"), false),
            Some(("acct-123".to_string(), AccountIdSource::Build))
        );
        assert_eq!(resolve_account_id(Some("dev-acct"), None, false), None);
    }

    #[test]
    fn mask_key_hides_middle() {
        assert_eq!(mask_key("FORGE-ABCD-1234-WXYZ"), "FORG****WXYZ");
        assert_eq!(mask_key("short"), "****");
    }

    #[test]
    fn parse_uses_meta_valid_and_tier() {
        let body = json!({ "meta": { "valid": true, "tier": "pro" } });
        let (valid, tier, _) = parse_validation_response(&body);
        assert!(valid);
        assert_eq!(tier, "pro");
    }

    #[test]
    fn parse_invalid_falls_back_to_free() {
        let body = json!({ "meta": { "valid": false, "tier": "team" } });
        let (valid, tier, _) = parse_validation_response(&body);
        assert!(!valid);
        assert_eq!(tier, "free");
    }

    #[test]
    fn parse_detects_team_from_policy_name() {
        let body = json!({
            "meta": { "valid": true },
            "data": { "attributes": { "policy": { "name": "Team Annual" } } }
        });
        let (_, tier, _) = parse_validation_response(&body);
        assert_eq!(tier, "team");
    }

    #[test]
    fn cached_status_masks_key_and_keeps_tier() {
        let cache = CachedValidation {
            tier: "pro".to_string(),
            valid: true,
            expires_at: Some("2030-01-01".to_string()),
            checked_at: 42,
        };
        let status = status_from_cache(&cache, Some("FORGE-ABCD-1234-WXYZ"));
        assert_eq!(status.tier, "pro");
        assert!(status.valid);
        assert_eq!(status.key_masked.as_deref(), Some("FORG****WXYZ"));
        assert_eq!(status.checked_at, Some(42));
    }
}
