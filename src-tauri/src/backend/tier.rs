//! Server-side tier gating. Mirrors `web/src/lib/tier.ts` exactly so the IPC
//! commands enforce the same Pro features the UI hides — keep the two lists
//! in sync.

use crate::backend::license;

/// Features that require a Pro or Team license (same list as `tier.ts`).
const PRO_FEATURES: [&str; 6] = [
    "workspaces",
    "build_presets",
    "build_history",
    "deploy_dashboard",
    "plugin_browser",
    "extension_publish",
];

pub fn is_feature_available(feature: &str, tier: &str) -> bool {
    if PRO_FEATURES.contains(&feature) {
        return tier != "free";
    }
    true
}

fn effective_tier(status: &license::LicenseStatus) -> &str {
    if status.valid {
        status.tier.as_str()
    } else {
        "free"
    }
}

/// Fail unless the cached license status unlocks `feature`.
pub fn require_feature(feature: &str) -> Result<(), String> {
    let status = license::get_license_status()?;
    check_feature(feature, &status)
}

fn check_feature(feature: &str, status: &license::LicenseStatus) -> Result<(), String> {
    if is_feature_available(feature, effective_tier(status)) {
        Ok(())
    } else {
        Err(format!(
            "This feature requires Forge Pro ({feature}). Activate a Pro or Team license to use it."
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(tier: &str, valid: bool) -> license::LicenseStatus {
        license::LicenseStatus {
            tier: tier.to_string(),
            valid,
            expires_at: None,
            key_masked: None,
            checked_at: None,
        }
    }

    #[test]
    fn pro_features_blocked_on_free() {
        for feature in PRO_FEATURES {
            assert!(!is_feature_available(feature, "free"));
            assert!(is_feature_available(feature, "pro"));
            assert!(is_feature_available(feature, "team"));
        }
    }

    #[test]
    fn other_features_are_free() {
        assert!(is_feature_available("scaffold_extension", "free"));
    }

    #[test]
    fn publish_and_browse_require_valid_paid_license() {
        for feature in ["extension_publish", "plugin_browser"] {
            assert!(check_feature(feature, &status("free", false)).is_err());
            assert!(check_feature(feature, &status("pro", false)).is_err());
            assert!(check_feature(feature, &status("pro", true)).is_ok());
            assert!(check_feature(feature, &status("team", true)).is_ok());
        }
    }
}
