mod commands;
mod engine;
mod inspect;
mod models;
mod pattern;
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    updater::remember_startup_exe();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(updater::Pending::default())
        .invoke_handler(tauri::generate_handler![
            commands::inspect_paths,
            commands::list_targets,
            commands::preflight,
            commands::convert,
            commands::cancel_conversion,
            commands::reveal_in_folder,
            commands::system_info,
            updater::app_info,
            updater::check_update,
            updater::install_update,
            updater::restart_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AEP Downgrader");
}
