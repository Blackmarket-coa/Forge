//! Hand-rolled SemVer parsing and precedence, mirroring FBM's
//! `plugin-registry/compat.ts` byte-for-byte in semantics (W3, shared
//! extension manifest — `free-black-market/docs/contracts/extension-manifest.md`):
//!
//! - `parse` accepts `X.Y.Z` with an optional leading `v` and ignores any
//!   `-prerelease`/`+build` suffix for the numeric core;
//! - `compare_precedence` implements full SemVer §11 (release > prerelease;
//!   numeric identifiers sort below alphanumeric; numerics numerically;
//!   a shared prefix loses to more identifiers), returning `None` when either
//!   side is unparseable — the same fail-null contract FBM keeps so fail-open
//!   bound checks stay fail-open;
//! - `is_valid` matches FBM's write-validation shape
//!   (`v?X.Y.Z(-pre)?(+build)?`, ASCII `[a-z0-9.-]` identifiers).
//!
//! The test vectors at the bottom are the SAME vectors FBM's
//! `compat.unit.spec.ts` pins, so a drift on either side fails a suite.

use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Semver {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

/// Parse the numeric core of `X.Y.Z` (optional leading `v`, ignoring any
/// `-pre`/`+build` suffix), else `None`.
pub fn parse(input: &str) -> Option<Semver> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    let no_v = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    let core = no_v.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some(Semver {
        major,
        minor,
        patch,
    })
}

fn split_prerelease(input: &str) -> Option<Vec<String>> {
    let trimmed = input.trim();
    let no_v = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    let no_build = no_v.split('+').next().unwrap_or(no_v);
    let dash = no_build.find('-')?;
    let identifiers = &no_build[dash + 1..];
    if identifiers.is_empty() {
        return None;
    }
    Some(identifiers.split('.').map(str::to_string).collect())
}

/// Full SemVer §11 precedence; `None` when either side is unparseable.
pub fn compare_precedence(a: &str, b: &str) -> Option<Ordering> {
    let pa = parse(a)?;
    let pb = parse(b)?;
    let core = (pa.major, pa.minor, pa.patch).cmp(&(pb.major, pb.minor, pb.patch));
    if core != Ordering::Equal {
        return Some(core);
    }
    match (split_prerelease(a), split_prerelease(b)) {
        (None, None) => Some(Ordering::Equal),
        (None, Some(_)) => Some(Ordering::Greater), // release > any prerelease
        (Some(_), None) => Some(Ordering::Less),
        (Some(pre_a), Some(pre_b)) => {
            for (id_a, id_b) in pre_a.iter().zip(pre_b.iter()) {
                if id_a == id_b {
                    continue;
                }
                let num_a = id_a.chars().all(|c| c.is_ascii_digit());
                let num_b = id_b.chars().all(|c| c.is_ascii_digit());
                return Some(match (num_a, num_b) {
                    (true, true) => id_a
                        .parse::<u64>()
                        .unwrap_or(u64::MAX)
                        .cmp(&id_b.parse::<u64>().unwrap_or(u64::MAX)),
                    // Numeric identifiers sort below alphanumeric.
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    (false, false) => id_a.cmp(id_b),
                });
            }
            Some(pre_a.len().cmp(&pre_b.len()))
        }
    }
}

/// Strict write-validation shape: `v?X.Y.Z(-pre)?(+build)?` with ASCII
/// `[A-Za-z0-9.-]` suffix identifiers (case-insensitive like FBM's regex).
pub fn is_valid(input: &str) -> bool {
    let trimmed = input.trim();
    if parse(trimmed).is_none() {
        return false;
    }
    let no_v = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    // Split off build metadata, then prerelease; each suffix must be
    // non-empty and use only [A-Za-z0-9.-].
    let mut build_split = no_v.splitn(2, '+');
    let before_build = build_split.next().unwrap_or("");
    if let Some(build) = build_split.next() {
        if build.is_empty()
            || !build
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return false;
        }
    }
    let mut pre_split = before_build.splitn(2, '-');
    let _core = pre_split.next();
    if let Some(pre) = pre_split.next() {
        if pre.is_empty()
            || !pre
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
        {
            return false;
        }
    }
    true
}

/// Pre-publish host-compat warning, mirroring FBM's fail-open
/// `isInstallable`: an absent or unparseable bound is "no bound", so a bad
/// value never blocks authoring — the registry re-checks at install time.
pub fn host_compat_warning(
    min_host_version: Option<&str>,
    max_host_version: Option<&str>,
    host_version: &str,
) -> Option<String> {
    if let Some(min) = min_host_version {
        if let Some(Ordering::Less) = compare_precedence(host_version, min) {
            return Some(format!(
                "Requires host version >= {min} (host is {host_version})"
            ));
        }
    }
    if let Some(max) = max_host_version {
        if let Some(Ordering::Greater) = compare_precedence(host_version, max) {
            return Some(format!(
                "Supports host version <= {max} (host is {host_version})"
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Shared vectors with FBM compat.unit.spec.ts — keep in sync.

    #[test]
    fn parses_core_with_v_prefix_and_suffixes() {
        assert_eq!(
            parse("1.2.3"),
            Some(Semver {
                major: 1,
                minor: 2,
                patch: 3
            })
        );
        assert_eq!(
            parse("v2.0.0"),
            Some(Semver {
                major: 2,
                minor: 0,
                patch: 0
            })
        );
        assert_eq!(
            parse("1.4.0-beta.1"),
            Some(Semver {
                major: 1,
                minor: 4,
                patch: 0
            })
        );
        assert_eq!(
            parse("1.4.0+build9"),
            Some(Semver {
                major: 1,
                minor: 4,
                patch: 0
            })
        );
        for bad in ["", "1.0", "1", "latest", "1.0.0.0", "garbage"] {
            assert!(parse(bad).is_none(), "{bad} should not parse");
        }
    }

    #[test]
    fn precedence_matches_release_ordering() {
        assert_eq!(compare_precedence("1.2.3", "1.2.3"), Some(Ordering::Equal));
        assert_eq!(compare_precedence("1.2.3", "1.3.0"), Some(Ordering::Less));
        assert_eq!(
            compare_precedence("2.0.0", "1.9.9"),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn prereleases_sort_below_their_release() {
        assert_eq!(
            compare_precedence("1.0.0-rc.1", "1.0.0"),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_precedence("1.0.0", "1.0.0-rc.1"),
            Some(Ordering::Greater)
        );
    }

    #[test]
    fn prerelease_identifier_ordering_per_semver_11() {
        assert_eq!(
            compare_precedence("1.0.0-alpha", "1.0.0-alpha.1"),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_precedence("1.0.0-alpha.1", "1.0.0-alpha.beta"),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_precedence("1.0.0-alpha.beta", "1.0.0-beta"),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_precedence("1.0.0-beta.2", "1.0.0-beta.11"),
            Some(Ordering::Less)
        );
        assert_eq!(
            compare_precedence("1.0.0-rc.1", "1.0.0-rc.1"),
            Some(Ordering::Equal)
        );
    }

    #[test]
    fn build_metadata_ignored_and_fail_null_contract() {
        assert_eq!(
            compare_precedence("1.0.0+build5", "1.0.0"),
            Some(Ordering::Equal)
        );
        assert_eq!(compare_precedence("garbage", "1.0.0"), None);
    }

    #[test]
    fn validity_shape_matches_fbm() {
        for good in [
            "1.0.0",
            "v1.0.0",
            "1.0.0-rc.1",
            "1.0.0+build.5",
            "1.0.0-rc.1+b2",
            "1.0.0 ",
        ] {
            assert!(is_valid(good), "{good} should be valid");
        }
        for bad in ["", "1.0", "1", "latest", "1.0.0.0", "1.0.0-"] {
            assert!(!is_valid(bad), "{bad} should be invalid");
        }
    }

    #[test]
    fn host_compat_is_fail_open() {
        assert!(host_compat_warning(Some("2.0.0"), None, "1.5.0").is_some());
        assert!(host_compat_warning(None, Some("1.0.0"), "2.0.0").is_some());
        assert!(host_compat_warning(Some("1.0.0"), Some("2.0.0"), "1.5.0").is_none());
        // Unparseable bound = no bound (registry re-checks at install time).
        assert!(host_compat_warning(Some("garbage"), None, "1.0.0").is_none());
        // Prerelease host vs exact release bound: blocked (matches FBM's
        // prerelease-aware gate).
        assert!(host_compat_warning(Some("1.0.0"), None, "1.0.0-rc.1").is_some());
    }
}
