//! Mock backend: plausible data derived from file names and sizes.
//! Replaced by `aep-core` later. Never writes to disk.

use crate::models::*;
use serde_json::{json, Map, Value};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const SIDECAR_MAGIC: [u8; 4] = [0x00, 0x05, 0x16, 0x07];
const CONVERTIBLE_EXTENSIONS: [&str; 3] = ["aep", "aet", "ffx"];
const LATEST_MAJOR: u32 = 27;

fn hash(text: &str) -> u64 {
    text.bytes()
        .fold(0xcbf29ce484222325, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100000001b3))
}

fn extension(path: &Path) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

/// Expands folders (non-recursive) into the files AE can produce.
fn expand(paths: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for raw in paths {
        let path = PathBuf::from(raw);
        if path.is_dir() {
            let mut inside: Vec<PathBuf> = fs::read_dir(&path)
                .map(|entries| entries.filter_map(|e| e.ok().map(|e| e.path())).collect())
                .unwrap_or_default();
            inside.sort();
            out.extend(
                inside
                    .into_iter()
                    .filter(|p| p.is_file() && CONVERTIBLE_EXTENSIONS.contains(&extension(p).as_str())),
            );
        } else {
            out.push(path);
        }
    }
    out
}

fn detect_problem(path: &Path, name: &str, kind: FileKind) -> Option<Problem> {
    let lower = name.to_lowercase();
    if lower.contains("damaged") {
        return Some(Problem::Damaged);
    }
    if lower.contains("truncated") {
        return Some(Problem::Truncated);
    }
    if lower.contains("toonew") {
        return Some(Problem::TooNew);
    }
    let mut head = [0u8; 64];
    let read = File::open(path).and_then(|mut f| f.read(&mut head));
    let on_disk = path.exists();
    match read {
        Ok(n) if n >= 4 && head[..4] == SIDECAR_MAGIC => Some(Problem::Sidecar),
        Ok(n) if n < 12 => Some(Problem::Truncated),
        Ok(n) if head[..4] != *b"RIFX" => {
            if head[..n].starts_with(b"<?xml") {
                Some(Problem::XmlProject)
            } else {
                Some(Problem::NotAeFile)
            }
        }
        Ok(_) => None,
        // Not on disk at all (a demo path): fall back to the name.
        Err(_) if !on_disk => match kind {
            FileKind::Sidecar => Some(Problem::Sidecar),
            FileKind::Xml => Some(Problem::XmlProject),
            FileKind::Unknown => Some(Problem::NotAeFile),
            _ => None,
        },
        Err(_) => Some(Problem::Unreadable),
    }
}

fn describe(path: &Path) -> FileInfo {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let key = path.to_string_lossy().into_owned();
    let h = hash(&name);
    let mut kind = match extension(path).as_str() {
        "aep" => FileKind::Project,
        "aet" => FileKind::Template,
        "ffx" => FileKind::Preset,
        "aepx" => FileKind::Xml,
        _ => FileKind::Unknown,
    };
    if name.starts_with("._") {
        kind = FileKind::Sidecar;
    }
    let size = fs::metadata(path)
        .map(|m| m.len())
        .unwrap_or(40_000 + h % 60_000_000);
    let problem = detect_problem(path, &name, kind);
    if matches!(problem, Some(Problem::Sidecar)) {
        kind = FileKind::Sidecar;
    }

    let mut info = FileInfo {
        id: format!("{:016x}", hash(&key)),
        path: key,
        name,
        size,
        kind,
        version: None,
        format: None,
        problem,
    };
    if problem.is_none() {
        let majors = [22, 23, 24, 25, 25, 26, 26, 27];
        let major = majors[(h % majors.len() as u64) as usize];
        info.version = Some(Version {
            major,
            minor: ((h >> 8) % 7) as u32,
            patch: 0,
            build: 40 + ((h >> 16) % 160) as u32,
            platform: match (h >> 4) % 4 {
                0 => Platform::Windows,
                1 => Platform::MacArm,
                2 => Platform::MacIntel,
                _ => Platform::Unknown,
            },
            release: major != LATEST_MAJOR,
            name: Some((2000 + major).to_string()),
        });
        info.format = Some(Format { major: 2, minor: major });
    }
    info
}

pub fn inspect_paths(paths: &[String]) -> Vec<FileInfo> {
    expand(paths).iter().map(|p| describe(p)).collect()
}

pub fn list_targets() -> Vec<TargetInfo> {
    (22..=26)
        .rev()
        .map(|major| TargetInfo {
            major,
            name: (2000 + major).to_string(),
            status: if major >= 24 {
                TargetStatus::Verified
            } else {
                TargetStatus::Experimental
            },
        })
        .collect()
}

fn note(severity: Severity, code: &str, params: Value) -> Note {
    let params = match params {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    Note { severity, code: code.into(), params }
}

pub fn preflight(path: &str, target: u32) -> Preflight {
    let info = describe(Path::new(path));
    let year = (2000 + target).to_string();
    let mut notes = Vec::new();

    let Some(version) = info.version else {
        notes.push(note(Severity::Block, "file-unreadable", json!({})));
        return Preflight { target, blocked: true, notes };
    };
    if target >= version.major {
        let from = version.name.unwrap_or_default();
        notes.push(note(Severity::Block, "target-not-older", json!({ "year": year, "from": from })));
        return Preflight { target, blocked: true, notes };
    }

    if info.kind == FileKind::Preset {
        notes.push(note(Severity::Info, "preset-no-changes", json!({})));
    } else {
        if version.major >= 25 && target <= 24 {
            notes.push(note(Severity::Warn, "shadow-color-removed", json!({})));
            notes.push(note(
                Severity::Warn,
                "effects-removed",
                json!({ "count": 2, "names": ["Curl Noise", "Unmult"], "year": year }),
            ));
        }
        if target <= 23 {
            notes.push(note(Severity::Warn, "text-animator-ranges", json!({})));
        }
        if hash(&info.name) % 3 == 0 {
            notes.push(note(
                Severity::Info,
                "plugins-kept",
                json!({ "names": ["Deep Glow", "Trapcode Particular"] }),
            ));
        }
        if notes.is_empty() {
            notes.push(note(Severity::Info, "no-changes", json!({ "year": year })));
        }
    }
    if target <= 23 {
        notes.push(note(Severity::Warn, "experimental-target", json!({})));
    }
    Preflight { target, blocked: false, notes }
}

/// Mock of the name pattern: `{name}` and `{year}` placeholders.
pub fn output_path(job: &Job, options: &OutputOptions) -> PathBuf {
    let source = Path::new(&job.path);
    let stem = source.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let pattern = if options.pattern.trim().is_empty() { "{name}_AE{year}" } else { &options.pattern };
    let mut file = pattern
        .replace("{name}", &stem)
        .replace("{year}", &(2000 + job.target).to_string());
    if let Some(ext) = source.extension() {
        file.push('.');
        file.push_str(&ext.to_string_lossy());
    }
    let dir = match &options.location {
        Location::Folder { folder } => PathBuf::from(folder),
        Location::Keyword(_) => source.parent().map(Path::to_path_buf).unwrap_or_default(),
    };
    dir.join(file)
}

pub fn conversion_millis(path: &str) -> u64 {
    500 + hash(path) % 900
}

pub fn should_fail(path: &str) -> bool {
    path.to_lowercase().contains("fail")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_is_detected_by_magic_bytes() {
        let dir = std::env::temp_dir().join("aep-mock-test-sidecar");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("scene.aep");
        fs::write(&file, [0x00, 0x05, 0x16, 0x07, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        let info = describe(&file);
        assert!(matches!(info.problem, Some(Problem::Sidecar)));
        assert!(info.kind == FileKind::Sidecar);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rifx_file_has_a_version() {
        let dir = std::env::temp_dir().join("aep-mock-test-rifx");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("scene.aep");
        fs::write(&file, b"RIFX\0\0\0\x10Egg!head\0\0\0\0").unwrap();
        let info = describe(&file);
        assert!(info.problem.is_none());
        assert!(info.version.is_some());
        fs::remove_dir_all(dir).unwrap();
    }
}
