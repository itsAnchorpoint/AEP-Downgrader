//! Mock backend: plausible data derived from file names and sizes.
//! Replaced by `aep-core` later. Never writes to disk.

use crate::inspect::describe_path;
use crate::models::*;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

fn hash(text: &str) -> u64 {
    text.bytes()
        .fold(0xcbf29ce484222325, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100000001b3))
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

/// Rows with a problem are refused: they never get a report.
pub fn preflight(path: &str, target: u32) -> Result<Preflight, String> {
    let info = describe_path(path);
    if info.problem.is_some() {
        return Err("file has a problem".into());
    }
    let year = (2000 + target).to_string();
    let mut notes = Vec::new();

    if let Some(version) = &info.version {
        if target >= version.major {
            let from = version.name.clone().unwrap_or_default();
            notes.push(note(Severity::Block, "target-not-older", json!({ "year": year, "from": from })));
            return Ok(Preflight { target, blocked: true, notes });
        }
    }

    if info.kind == FileKind::Preset {
        notes.push(note(Severity::Info, "preset-no-changes", json!({})));
    } else {
        let major = info.version.as_ref().map_or(0, |v| v.major);
        if major >= 25 && target <= 24 {
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
        if hash(&info.name).is_multiple_of(3) {
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
    Ok(Preflight { target, blocked: false, notes })
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
