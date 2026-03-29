fn main() {
    env_logger::init();
    let app = forge_lib::register_commands(tauri::Builder::default());
    app.run(tauri::generate_context!())
        .expect("error while running Forge application");
}
