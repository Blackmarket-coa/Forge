fn main() {
    env_logger::init();

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
