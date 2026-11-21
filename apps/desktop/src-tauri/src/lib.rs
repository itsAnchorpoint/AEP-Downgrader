mod commands;
mod engine;
mod inspect;
mod models;
mod pattern;
mod updater;

/// Whether to draw through XWayland. The WebKitGTK an AppImage carries (built on Ubuntu 22.04)
/// cannot use an NVIDIA GPU under Wayland and paints every frame on the CPU, which lags badly;
/// through XWayland it is smooth, if a little softer at fractional scales. A GDK_BACKEND the user
/// set wins, and the .deb and builds from source use the system WebKitGTK, so they stay native.
#[cfg(target_os = "linux")]
fn wants_xwayland(
    appimage: bool,
    wayland: bool,
    xwayland: bool,
    backend_set: bool,
    nvidia: bool,
) -> bool {
    appimage && wayland && xwayland && !backend_set && nvidia
}

#[cfg(target_os = "linux")]
fn choose_display_backend() {
    use std::env::{set_var, var_os};
    if wants_xwayland(
        var_os("APPIMAGE").is_some(),
        var_os("WAYLAND_DISPLAY").is_some(),
        var_os("DISPLAY").is_some(),
        var_os("GDK_BACKEND").is_some(),
        std::path::Path::new("/sys/module/nvidia").exists(),
    ) {
        // First thing in run(), before GTK starts or any thread exists. A restart after an
        // update inherits it.
        set_var("GDK_BACKEND", "x11");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    choose_display_backend();
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

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::wants_xwayland;

    #[test]
    fn appimage_on_nvidia_under_wayland_goes_through_xwayland() {
        assert!(wants_xwayland(true, true, true, false, true));
    }

    #[test]
    fn everything_else_stays_as_it_is() {
        assert!(
            !wants_xwayland(false, true, true, false, true),
            "the .deb uses the system WebKitGTK"
        );
        assert!(
            !wants_xwayland(true, true, true, true, true),
            "the user's GDK_BACKEND wins"
        );
        assert!(
            !wants_xwayland(true, true, true, false, false),
            "not NVIDIA"
        );
        assert!(
            !wants_xwayland(true, true, false, false, true),
            "no XWayland to fall back to"
        );
        assert!(
            !wants_xwayland(true, false, true, false, true),
            "already on X11"
        );
    }
}
