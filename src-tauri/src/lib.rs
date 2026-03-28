pub mod app_state;
pub mod backend;
pub mod project;

pub fn register_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        backend::ipc::register_project,
        backend::ipc::get_projects,
        backend::ipc::detect_tauri_status,
        backend::ipc::scan_directory,
        backend::ipc::read_config,
        backend::ipc::write_config,
        backend::ipc::validate_config,
        backend::ipc::run_dev,
        backend::ipc::run_build,
        backend::ipc::kill_process,
        backend::ipc::check_environment,
        backend::ipc::collect_artifacts,
    ])
}
