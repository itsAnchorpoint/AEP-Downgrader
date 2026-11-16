mod commands;
mod engine;
mod inspect;
mod models;
mod pattern;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::inspect_paths,
            commands::list_targets,
            commands::preflight,
            commands::convert,
            commands::reveal_in_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AEP Downgrader");
}
