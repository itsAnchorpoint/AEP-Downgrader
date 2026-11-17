//! Real `inspect_paths`: identifies files with `aep-core` from their first 64 KiB.

use crate::models::*;
use aep_core::inspect::{inspect, FileKind as CoreKind};
use aep_core::version::{release_name, AppVersion, FormatVersion, Platform as CorePlatform};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const PREFIX_LEN: u64 = 64 * 1024;
const FOLDER_EXTENSIONS: [&str; 3] = ["aep", "aet", "ffx"];
/// Newest After Effects major this app knows about.
const NEWEST_KNOWN_MAJOR: u8 = 27;
/// Newest file format major the converter knows (After Effects 27 writes 98).
const NEWEST_KNOWN_FORMAT: u16 = 98;

fn extension(path: &Path) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

fn fnv(text: &str) -> u64 {
    text.bytes()
        .fold(0xcbf29ce484222325, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100000001b3))
}

/// Folders become their .aep/.aet/.ffx files (non-recursive, sorted by name);
/// explicit files stay whatever their extension. Each canonical path appears once.
fn expand(paths: &[String]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    let mut push = |path: PathBuf| {
        let canonical = fs::canonicalize(&path).unwrap_or(path);
        if seen.insert(canonical.clone()) {
            out.push(canonical);
        }
    };
    for raw in paths {
        let path = PathBuf::from(raw);
        if path.is_dir() {
            let mut inside: Vec<PathBuf> = fs::read_dir(&path)
                .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
                .unwrap_or_default();
            inside.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
            for file in inside {
                if file.is_file() && FOLDER_EXTENSIONS.contains(&extension(&file).as_str()) {
                    push(file);
                }
            }
        } else {
            push(path);
        }
    }
    out
}

fn read_prefix(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    File::open(path)?.take(PREFIX_LEN).read_to_end(&mut buf)?;
    Ok(buf)
}

fn platform(p: CorePlatform) -> Platform {
    match p {
        CorePlatform::Windows => Platform::Windows,
        CorePlatform::MacIntel => Platform::MacIntel,
        CorePlatform::MacArm => Platform::MacArm,
        _ => Platform::Unknown,
    }
}

fn version(app: AppVersion) -> Version {
    Version {
        major: u32::from(app.major()),
        minor: u32::from(app.minor()),
        patch: u32::from(app.patch()),
        build: u32::from(app.build()),
        platform: platform(app.platform()),
        release: app.is_release(),
        name: release_name(app.major()).map(str::to_string),
    }
}

fn format(f: FormatVersion) -> Format {
    Format { major: u32::from(f.major), minor: u32::from(f.minor) }
}

fn describe(path: &Path) -> FileInfo {
    let key = path.to_string_lossy().into_owned();
    let mut info = FileInfo {
        id: format!("{:016x}", fnv(&key)),
        name: path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        path: key,
        size: fs::metadata(path).map(|m| m.len()).unwrap_or(0),
        kind: FileKind::Unknown,
        version: None,
        format: None,
        problem: None,
    };
    let prefix = match read_prefix(path) {
        Ok(bytes) => bytes,
        Err(_) => {
            info.problem = Some(Problem::Unreadable);
            return info;
        }
    };
    let found = inspect(&prefix);
    match found.kind {
        CoreKind::Project => {
            info.kind = if extension(path) == "aet" { FileKind::Template } else { FileKind::Project };
            match found.head {
                Some(head) => {
                    info.version = Some(version(head.app));
                    info.format = Some(format(head.format));
                    if head.app.major() > NEWEST_KNOWN_MAJOR {
                        info.problem = Some(Problem::TooNew);
                    }
                }
                None => info.problem = Some(Problem::Damaged),
            }
        }
        CoreKind::Preset => {
            info.kind = FileKind::Preset;
            info.format = found.preset_format.map(format);
            match found.preset_format {
                Some(f) if f.major > NEWEST_KNOWN_FORMAT => info.problem = Some(Problem::TooNew),
                Some(_) => {}
                None => info.problem = Some(Problem::Damaged),
            }
        }
        CoreKind::XmlProject => {
            info.kind = FileKind::Xml;
            info.problem = Some(Problem::XmlProject);
        }
        CoreKind::AppleDouble => {
            info.kind = FileKind::Sidecar;
            info.problem = Some(Problem::Sidecar);
        }
        _ => info.problem = Some(Problem::NotAeFile),
    }
    info
}

pub fn inspect_paths(paths: &[String]) -> Vec<FileInfo> {
    expand(paths).iter().map(|p| describe(p)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../tests/fixtures");

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aep-inspect-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn one(path: &Path) -> FileInfo {
        let mut rows = inspect_paths(&[path.to_string_lossy().into_owned()]);
        assert_eq!(rows.len(), 1);
        rows.remove(0)
    }

    #[test]
    fn fixture_22() {
        let info = one(&Path::new(FIXTURES).join("TEST22-x.aep"));
        assert!(info.problem.is_none());
        assert!(info.kind == FileKind::Project);
        let v = info.version.unwrap();
        assert_eq!((v.major, v.minor, v.patch, v.build), (22, 6, 0, 59));
        assert!(matches!(v.platform, Platform::Windows));
        assert!(v.release);
        assert_eq!(v.name.as_deref(), Some("2022"));
        let f = info.format.unwrap();
        assert_eq!((f.major, f.minor), (93, 43));
    }

    #[test]
    fn fixture_26() {
        let info = one(&Path::new(FIXTURES).join("TEST26-x.aep"));
        let v = info.version.unwrap();
        assert_eq!((v.major, v.minor, v.patch, v.build), (26, 0, 0, 67));
        let f = info.format.unwrap();
        assert_eq!((f.major, f.minor), (97, 2));
    }

    #[test]
    fn aet_extension_gives_a_template() {
        let dir = temp_dir("aet");
        let file = dir.join("t.aet");
        fs::copy(Path::new(FIXTURES).join("TEST22-x.aep"), &file).unwrap();
        assert!(one(&file).kind == FileKind::Template);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn sidecar_text_empty_and_missing() {
        let dir = temp_dir("bad");
        let sidecar = dir.join("._x.aep");
        fs::write(&sidecar, [0x00, 0x05, 0x16, 0x07, 1, 2, 3, 4]).unwrap();
        let info = one(&sidecar);
        assert!(info.kind == FileKind::Sidecar && matches!(info.problem, Some(Problem::Sidecar)));

        let text = dir.join("notes.txt");
        fs::write(&text, "hello").unwrap();
        assert!(matches!(one(&text).problem, Some(Problem::NotAeFile)));

        let empty = dir.join("empty.aep");
        fs::write(&empty, "").unwrap();
        assert!(matches!(one(&empty).problem, Some(Problem::NotAeFile)));

        assert!(matches!(one(&dir.join("missing.aep")).problem, Some(Problem::Unreadable)));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn folder_expands_to_sorted_aep_files_only() {
        let dir = temp_dir("folder");
        fs::copy(Path::new(FIXTURES).join("TEST26-x.aep"), dir.join("b.AEP")).unwrap();
        fs::copy(Path::new(FIXTURES).join("TEST22-x.aep"), dir.join("a.aep")).unwrap();
        fs::write(dir.join("readme.txt"), "x").unwrap();
        let rows = inspect_paths(&[dir.to_string_lossy().into_owned()]);
        let names: Vec<_> = rows.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["a.aep", "b.AEP"]);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn same_file_twice_is_one_row_with_a_stable_id() {
        let file = Path::new(FIXTURES).join("TEST22-x.aep");
        let p = file.to_string_lossy().into_owned();
        let rows = inspect_paths(&[p.clone(), p.clone()]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, inspect_paths(&[p])[0].id);
    }
}
