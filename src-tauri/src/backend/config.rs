/// Startup environment validation.
///
/// Call [`validate_startup_env`] once at the very beginning of `main` (before
/// the Tauri runtime starts).  It logs actionable warnings for every missing or
/// suspicious configuration value so that operators notice problems immediately
/// rather than discovering them at runtime.
use log::{info, warn};

struct EnvCheck {
    var: &'static str,
    required: bool,
    sentinel: Option<&'static str>,
    hint: &'static str,
}

pub fn validate_startup_env() {
    let checks = [
        EnvCheck {
            var: "KEYGEN_ACCOUNT_ID",
            required: false,
            sentinel: Some("demo-account"),
            hint: "license validation will use the Keygen demo sandbox and always return invalid; \
                   set KEYGEN_ACCOUNT_ID to your real account ID in production",
        },
        EnvCheck {
            var: "SENTRY_DSN",
            required: false,
            sentinel: None,
            hint: "crash reporting is disabled; set SENTRY_DSN to enable Sentry in production",
        },
    ];

    let mut all_ok = true;

    for check in &checks {
        match std::env::var(check.var) {
            Ok(val) if check.sentinel.map(|s| val == s).unwrap_or(false) => {
                warn!(
                    "config: {} is set to the placeholder value '{}' — {}",
                    check.var,
                    val,
                    check.hint
                );
                all_ok = false;
            }
            Err(_) if check.required => {
                warn!(
                    "config: required env var {} is not set — {}",
                    check.var, check.hint
                );
                all_ok = false;
            }
            Err(_) => {
                // Optional and unset — emit a lower-priority notice.
                warn!("config: {} is not set — {}", check.var, check.hint);
                all_ok = false;
            }
            Ok(_) => {}
        }
    }

    if all_ok {
        info!("config: all environment checks passed");
    }
}
