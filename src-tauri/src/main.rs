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

    // GUI launches (Finder/Dock, desktop entries) don't inherit the shell
    // PATH, so rustup/nvm-installed tools would look missing and build
    // spawns would fail. Copy the login shell's PATH into the process and
    // make sure ~/.cargo/bin is reachable either way.
    if let Err(e) = fix_path_env::fix() {
        log::warn!("failed to inherit shell PATH: {e}");
    }
    forge_lib::backend::env_path::ensure_cargo_bin_in_path();

    forge_lib::backend::config::validate_startup_env();

    let app = forge_lib::register_commands(tauri::Builder::default());
    app.run(tauri::generate_context!())
        .expect("error while running Forge application");
}
