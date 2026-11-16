use crate::engine;
use crate::inspect;
use crate::models::*;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use tauri_plugin_opener::OpenerExt;

const PROGRESS_EVENT: &str = "convert://progress";

#[tauri::command]
pub async fn inspect_paths(paths: Vec<String>) -> Result<Vec<FileInfo>, String> {
    // File reads can be slow on network or spinning disks: keep them off the main thread.
    tauri::async_runtime::spawn_blocking(move || inspect::inspect_paths(&paths))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_targets() -> Vec<TargetInfo> {
    engine::list_targets()
}

#[tauri::command]
pub async fn preflight(path: String, target: u32) -> Result<Preflight, String> {
    tauri::async_runtime::spawn_blocking(move || engine::preflight(&path, target))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn convert(app: AppHandle, jobs: Vec<Job>, options: OutputOptions) {
    let emit = move |event: ProgressEvent| {
        // A closed window is not worth failing a conversion over.
        let _ = app.emit(PROGRESS_EVENT, event);
    };
    for job in &jobs {
        emit(ProgressEvent { id: job.id.clone(), state: ProgressState::Queued, output_path: None, error_code: None });
    }
    std::thread::spawn(move || engine::run_batch(jobs, &options, &emit));
}

#[tauri::command]
pub fn reveal_in_folder(app: AppHandle, path: String) -> Result<(), String> {
    // Fall back to the nearest existing parent if the file has since been moved or deleted.
    let mut target = Path::new(&path);
    while !target.exists() {
        target = target.parent().ok_or("path does not exist")?;
    }
    app.opener().reveal_item_in_dir(target).map_err(|e| e.to_string())
}
