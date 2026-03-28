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
        backend::ipc::create_project,
        backend::ipc::init_tauri,
        backend::ipc::create_workspace,
        backend::ipc::get_workspaces,
        backend::ipc::update_workspace,
        backend::ipc::delete_workspace,
        backend::ipc::add_project_to_workspace,
        backend::ipc::remove_project_from_workspace,
        backend::ipc::save_build_preset,
        backend::ipc::get_build_presets,
        backend::ipc::run_build_preset,
        backend::ipc::get_build_history,
        backend::ipc::get_deploy_status,
    ])
}
