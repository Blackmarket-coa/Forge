fn main() {
    // Default to a sensible log level so operator-facing diagnostics are
    // visible out of the box; RUST_LOG still overrides this when set.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("forge=info,warn"))
        .init();

    // Initialise Sentry before anything else so that panics and errors during
    // startup are captured.  When SENTRY_DSN is unset this is a no-op.
    let _sentry = std::env::var("SENTRY_DSN").ok().map(|dsn| {
        sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ))
    });

    forge_lib::backend::config::validate_startup_env();

    let app = forge_lib::register_commands(tauri::Builder::default());
    app.run(tauri::generate_context!())
        .expect("error while running Forge application");
}
