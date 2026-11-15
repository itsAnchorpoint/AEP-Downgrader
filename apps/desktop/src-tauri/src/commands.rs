use crate::inspect;
use crate::mock;
use crate::models::*;
use std::path::Path;
use std::time::Duration;
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
    mock::list_targets()
}

#[tauri::command]
pub fn preflight(path: String, target: u32) -> Result<Preflight, String> {
    mock::preflight(&path, target)
}

fn emit(app: &AppHandle, id: &str, state: ProgressState, output: Option<String>, error: Option<&str>) {
    let event = ProgressEvent {
        id: id.to_string(),
        state,
        output_path: output,
        error_code: error.map(str::to_string),
    };
    // A closed window is not an error worth reporting from a mock.
    let _ = app.emit(PROGRESS_EVENT, event);
}

#[tauri::command]
pub fn convert(app: AppHandle, jobs: Vec<Job>, options: OutputOptions) {
    for job in &jobs {
        emit(&app, &job.id, ProgressState::Queued, None, None);
    }
    tauri::async_runtime::spawn(async move {
        for job in jobs {
            if inspect::describe_path(&job.path).problem.is_some() {
                emit(&app, &job.id, ProgressState::Failed, None, Some("unreadable"));
                continue;
            }
            emit(&app, &job.id, ProgressState::Running, None, None);
            tokio::time::sleep(Duration::from_millis(mock::conversion_millis(&job.path))).await;
            if mock::should_fail(&job.path) {
                emit(&app, &job.id, ProgressState::Failed, None, Some("write-failed"));
                continue;
            }
            let output = mock::output_path(&job, &options);
            if options.on_exists == OnExists::Skip && output.exists() {
                emit(&app, &job.id, ProgressState::Skipped, None, None);
            } else {
                emit(&app, &job.id, ProgressState::Done, Some(output.to_string_lossy().into_owned()), None);
            }
        }
    });
}

#[tauri::command]
pub fn reveal_in_folder(app: AppHandle, path: String) -> Result<(), String> {
    // The mock writes nothing, so fall back to the nearest existing parent.
    let mut target = Path::new(&path);
    while !target.exists() {
        target = target.parent().ok_or("path does not exist")?;
    }
    app.opener().reveal_item_in_dir(target).map_err(|e| e.to_string())
}
