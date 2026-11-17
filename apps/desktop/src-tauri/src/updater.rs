//! Update check and install: the only network access of the app.
//!
//! The update feed is `latest.json` in GitHub Releases, signed with the project's updater key
//! (public key in `tauri.conf.json`). Stable builds read the feed of the latest release; builds
//! with a pre-release version (`2.0.0-beta.1`) read a separate beta feed, so testers get betas
//! and stable users never do.
//!
//! How an update is applied depends on how the app was installed ([`InstallKind`]):
//! - Windows installer, macOS app, Linux AppImage: the updater plugin replaces the installation.
//! - Windows portable exe (no uninstaller next to it): the signed exe is downloaded and swapped
//!   for the running one with `self-replace`. It has its own entry in the feed.
//! - Everything else (a `.deb`, a build from source, an app started from the disk image): the
//!   app only says that a new version exists and links to the release page.

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tauri::ipc::Channel;
use tauri::{AppHandle, Runtime, State, Url};
use tauri_plugin_updater::{Update, UpdaterExt};

/// Feed for builds with a pre-release version. A pre-release on GitHub, so the "latest release"
/// link (and the update check of 1.x) never points to it.
const BETA_FEED: &str =
    "https://github.com/itsAnchorpoint/AEP-Downgrader/releases/download/updater-beta/latest.json";
/// Feed entry of the Windows portable exe.
const PORTABLE_TARGET: &str = "windows-x86_64-portable";
/// Settings of the portable app live next to its exe under this name.
const PORTABLE_SETTINGS: &str = "AEP Downgrader settings.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallKind {
    /// Updated in place by the updater plugin.
    Installer,
    /// Windows exe without an installer: updated by replacing the exe.
    Portable,
    /// Not updated by the app: it links to the release page instead.
    Manual,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub install_kind: InstallKind,
    /// Where the settings store lives: a bare name (the app data folder) or an absolute path.
    pub settings_file: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    /// Release notes from the feed (Markdown), if any.
    pub notes: Option<String>,
    /// Release date as `YYYY-MM-DD`, if the feed has one.
    pub date: Option<String>,
    pub install_kind: InstallKind,
}

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "camelCase")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    Started {
        total: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    Progress {
        received: u64,
    },
    Finished,
}

/// The update found by the last check, kept for `install_update`.
#[derive(Default)]
pub struct Pending(Mutex<Option<Update>>);

/// Path of the running exe as it was at startup. A portable update renames the running file
/// on Windows, so the path is captured before that to relaunch the new exe.
static STARTUP_EXE: OnceLock<Option<PathBuf>> = OnceLock::new();

pub fn remember_startup_exe() {
    STARTUP_EXE.get_or_init(|| std::env::current_exe().ok());
}

fn startup_exe() -> Option<PathBuf> {
    STARTUP_EXE
        .get_or_init(|| std::env::current_exe().ok())
        .clone()
}

/// Facts about the running app that decide how it is updated.
struct Environment<'a> {
    os: &'a str,
    debug: bool,
    exe_dir: Option<&'a Path>,
    /// The Windows installer puts `uninstall.exe` next to the app.
    has_uninstaller: bool,
    /// Set by the AppImage runtime.
    appimage: bool,
}

fn install_kind_for(env: &Environment) -> InstallKind {
    if env.debug || env.exe_dir.is_none() {
        return InstallKind::Manual;
    }
    match env.os {
        "windows" if env.has_uninstaller => InstallKind::Installer,
        "windows" => InstallKind::Portable,
        "macos" => match env.exe_dir.map(|d| d.to_string_lossy().into_owned()) {
            // Started from the mounted disk image, or from a randomised read-only copy
            // (App Translocation): the update could not replace the app.
            Some(dir) if dir.starts_with("/Volumes/") || dir.contains("/AppTranslocation/") => {
                InstallKind::Manual
            }
            _ => InstallKind::Installer,
        },
        "linux" if env.appimage => InstallKind::Installer,
        _ => InstallKind::Manual,
    }
}

pub fn install_kind() -> InstallKind {
    let exe = startup_exe();
    let exe_dir = exe.as_deref().and_then(Path::parent);
    install_kind_for(&Environment {
        os: std::env::consts::OS,
        debug: cfg!(debug_assertions),
        exe_dir,
        has_uninstaller: exe_dir.is_some_and(|d| d.join("uninstall.exe").is_file()),
        appimage: std::env::var_os("APPIMAGE").is_some(),
    })
}

/// True if a file can be created in `dir` (a portable app on a read-only stick cannot keep its
/// settings next to the exe).
fn is_writable(dir: &Path) -> bool {
    let probe = dir.join(".aepd-write-test");
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

fn settings_file(kind: InstallKind) -> String {
    if kind == InstallKind::Portable {
        if let Some(dir) = startup_exe().as_deref().and_then(Path::parent) {
            if is_writable(dir) {
                return dir.join(PORTABLE_SETTINGS).to_string_lossy().into_owned();
            }
        }
    }
    "settings.json".to_owned()
}

fn is_prerelease(version: &str) -> bool {
    version.contains('-')
}

#[tauri::command]
pub fn app_info<R: Runtime>(app: AppHandle<R>) -> AppInfo {
    let kind = install_kind();
    AppInfo {
        version: app.package_info().version.to_string(),
        install_kind: kind,
        settings_file: settings_file(kind),
    }
}

#[tauri::command]
pub async fn check_update<R: Runtime>(
    app: AppHandle<R>,
    pending: State<'_, Pending>,
) -> Result<Option<UpdateInfo>, String> {
    let kind = install_kind();
    let mut builder = app.updater_builder();
    if is_prerelease(&app.package_info().version.to_string()) {
        let feed = Url::parse(BETA_FEED).map_err(|e| e.to_string())?;
        builder = builder.endpoints(vec![feed]).map_err(|e| e.to_string())?;
    }
    if kind == InstallKind::Portable {
        builder = builder.target(PORTABLE_TARGET);
    }
    let update = builder
        .build()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| e.to_string())?;

    let info = update.as_ref().map(|u| UpdateInfo {
        version: u.version.clone(),
        notes: u.body.clone().filter(|b| !b.trim().is_empty()),
        date: u.date.map(|d| d.date().to_string()),
        install_kind: kind,
    });
    *pending.0.lock().map_err(|e| e.to_string())? = update;
    Ok(info)
}

/// Downloads, verifies and installs the update found by the last check. The app is not
/// restarted here (except by the Windows installer, which restarts it itself): the UI asks
/// the user and calls `restart_app`.
#[tauri::command]
pub async fn install_update(
    pending: State<'_, Pending>,
    on_event: Channel<DownloadEvent>,
) -> Result<(), String> {
    let kind = install_kind();
    if kind == InstallKind::Manual {
        return Err("this installation is updated by downloading the new version".into());
    }
    let update = pending
        .0
        .lock()
        .map_err(|e| e.to_string())?
        .clone()
        .ok_or("no update to install")?;

    let mut started = false;
    let mut received: u64 = 0;
    let mut reported: u64 = 0;
    // `download` checks the signature against the public key before returning the bytes.
    let bytes = update
        .download(
            |chunk, total| {
                if !started {
                    started = true;
                    let _ = on_event.send(DownloadEvent::Started { total });
                }
                received += chunk as u64;
                // About a hundred progress events for a typical download, not thousands.
                if received - reported >= 64 * 1024 {
                    reported = received;
                    let _ = on_event.send(DownloadEvent::Progress { received });
                }
            },
            || {
                let _ = on_event.send(DownloadEvent::Finished);
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    match kind {
        InstallKind::Portable => replace_portable_exe(&bytes),
        _ => update.install(&bytes).map_err(|e| e.to_string()),
    }
}

/// Swaps the running portable exe for the downloaded one (already verified).
fn replace_portable_exe(bytes: &[u8]) -> Result<(), String> {
    if !bytes.starts_with(b"MZ") {
        return Err("the update is not a Windows program".into());
    }
    let exe = startup_exe().ok_or("cannot find the running program")?;
    let dir = exe.parent().ok_or("cannot find the program folder")?;
    let staged = dir.join(".aepd-update.exe");
    std::fs::write(&staged, bytes).map_err(|e| format!("cannot write next to the program: {e}"))?;
    let result = replace_exe(&staged);
    let _ = std::fs::remove_file(&staged);
    result
}

#[cfg(windows)]
fn replace_exe(staged: &Path) -> Result<(), String> {
    self_replace::self_replace(staged).map_err(|e| e.to_string())
}

#[cfg(not(windows))]
fn replace_exe(_staged: &Path) -> Result<(), String> {
    Err("portable updates exist only on Windows".into())
}

#[tauri::command]
pub fn restart_app<R: Runtime>(app: AppHandle<R>) {
    if install_kind() == InstallKind::Portable {
        // Start the exe at the original path, which now holds the new version.
        if let Some(exe) = startup_exe() {
            if std::process::Command::new(exe).spawn().is_ok() {
                app.exit(0);
                return;
            }
        }
    }
    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(os: &'a str, dir: Option<&'a Path>) -> Environment<'a> {
        Environment {
            os,
            debug: false,
            exe_dir: dir,
            has_uninstaller: false,
            appimage: false,
        }
    }

    #[test]
    fn windows_with_uninstaller_is_installed_without_is_portable() {
        let dir = Path::new("C:/Users/me/AppData/Local/AEP Downgrader");
        assert_eq!(
            install_kind_for(&Environment {
                has_uninstaller: true,
                ..env("windows", Some(dir))
            }),
            InstallKind::Installer
        );
        assert_eq!(
            install_kind_for(&env("windows", Some(Path::new("D:/Tools")))),
            InstallKind::Portable
        );
    }

    #[test]
    fn macos_from_disk_image_or_translocated_is_manual() {
        let installed = Path::new("/Applications/AEP Downgrader.app/Contents/MacOS");
        assert_eq!(
            install_kind_for(&env("macos", Some(installed))),
            InstallKind::Installer
        );
        let dmg = Path::new("/Volumes/AEP Downgrader/AEP Downgrader.app/Contents/MacOS");
        assert_eq!(
            install_kind_for(&env("macos", Some(dmg))),
            InstallKind::Manual
        );
        let moved = Path::new(
            "/private/var/folders/x/AppTranslocation/ABC/d/AEP Downgrader.app/Contents/MacOS",
        );
        assert_eq!(
            install_kind_for(&env("macos", Some(moved))),
            InstallKind::Manual
        );
    }

    #[test]
    fn linux_updates_only_the_appimage() {
        let dir = Path::new("/usr/bin");
        assert_eq!(
            install_kind_for(&env("linux", Some(dir))),
            InstallKind::Manual
        );
        assert_eq!(
            install_kind_for(&Environment {
                appimage: true,
                ..env("linux", Some(dir))
            }),
            InstallKind::Installer
        );
    }

    #[test]
    fn debug_builds_and_unknown_paths_are_manual() {
        let dir = Path::new("D:/Tools");
        assert_eq!(
            install_kind_for(&Environment {
                debug: true,
                ..env("windows", Some(dir))
            }),
            InstallKind::Manual
        );
        assert_eq!(install_kind_for(&env("windows", None)), InstallKind::Manual);
    }

    #[test]
    fn prerelease_versions_use_the_beta_feed() {
        assert!(is_prerelease("2.0.0-beta.1"));
        assert!(!is_prerelease("2.0.0"));
    }

    #[test]
    fn portable_replacement_rejects_non_windows_programs() {
        assert!(replace_portable_exe(b"\x7fELF").is_err());
    }

    #[test]
    fn download_events_serialize_for_the_ui() {
        let started = serde_json::to_value(DownloadEvent::Started { total: Some(10) }).unwrap();
        assert_eq!(
            started,
            serde_json::json!({ "event": "started", "data": { "total": 10 } })
        );
        let finished = serde_json::to_value(DownloadEvent::Finished).unwrap();
        assert_eq!(finished, serde_json::json!({ "event": "finished" }));
    }
}
