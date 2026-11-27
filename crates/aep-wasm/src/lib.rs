//! `aep-core` for the website: the "check a file" section identifies a dropped file in the browser,
//! with the same rules as the app (`apps/desktop/src-tauri/src/inspect.rs`). Nothing leaves the page.
//!
//! A plain C ABI, no `wasm-bindgen`, so the site needs no build tool for it:
//!
//! 1. `alloc(len)` returns a buffer in WebAssembly memory; the page copies the first bytes of the file
//!    there (64 KiB is enough, see [`aep_core::inspect`]).
//! 2. `inspect(ptr, len)` frees that buffer and describes the file as JSON (see [`describe`]);
//!    `result_ptr()` and `result_len()` say where the UTF-8 text is. It stays valid until the next call.
//! 3. For the report, the page copies the whole file into a buffer from `alloc` and calls
//!    `preflight(ptr, len, target)`: the same notes the app shows before converting (see [`report`]).
//!    The conversion runs in memory and its bytes are thrown away; only the app writes a copy.

use std::cell::RefCell;

use aep_core::compat;
use aep_core::convert::{ConvertError, Note as CoreNote, convert};
use aep_core::inspect::{FileKind, inspect as inspect_prefix};
use aep_core::version::{AppVersion, FormatVersion, Head, Platform, STAMPS, release_name};

/// Newest After Effects the converter knows (the app's `NEWEST_KNOWN_MAJOR`).
const NEWEST_KNOWN_MAJOR: u8 = 27;
/// Newest preset format the converter knows (the app's `NEWEST_KNOWN_FORMAT`).
const NEWEST_KNOWN_FORMAT: u16 = 98;

thread_local! {
    static RESULT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// A buffer of `len` bytes for the page to fill. [`inspect`] takes it back.
#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Describe the `len` bytes at `ptr` (a buffer from [`alloc`] of that size, which this frees).
///
/// # Safety
/// `ptr` must come from `alloc(len)` with the same `len`, filled with `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn inspect(ptr: *mut u8, len: usize) {
    // SAFETY: the caller passes a buffer from `alloc(len)` that it filled completely.
    let bytes = unsafe { Vec::from_raw_parts(ptr, len, len) };
    let json = describe(&bytes);
    RESULT.with(|r| *r.borrow_mut() = json.into_bytes());
}

/// The report for converting the `len` bytes at `ptr` (a buffer from [`alloc`], which this frees)
/// to After Effects `target` (a major: 22 to 26).
///
/// # Safety
/// As for [`inspect`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn preflight(ptr: *mut u8, len: usize, target: u32) {
    // SAFETY: the caller passes a buffer from `alloc(len)` that it filled completely.
    let bytes = unsafe { Vec::from_raw_parts(ptr, len, len) };
    let json = report(&bytes, target);
    RESULT.with(|r| *r.borrow_mut() = json.into_bytes());
}

/// Where the last [`inspect`] result starts.
#[unsafe(no_mangle)]
pub extern "C" fn result_ptr() -> *const u8 {
    RESULT.with(|r| r.borrow().as_ptr())
}

/// Length in bytes of the last [`inspect`] result.
#[unsafe(no_mangle)]
pub extern "C" fn result_len() -> usize {
    RESULT.with(|r| r.borrow().len())
}

/// The file as JSON, from a prefix of its bytes:
///
/// ```text
/// {"kind":"project"|"preset"|"xml"|"sidecar"|"unknown",
///  "problem":null|"not-ae-file"|"xml-project"|"sidecar"|"damaged"|"too-new",
///  "version":null|{"major":27,"minor":0,"patch":0,"build":78,"platform":"mac-arm","release":true,"name":"2027"},
///  "format":null|{"major":98,"minor":6},
///  "release":null|"2027",
///  "targets":[{"major":26,"name":"2026","status":"experimental"}, …]}
/// ```
///
/// `release` names the After Effects release the file comes from (for a preset, from its format).
/// `targets` lists the versions it can be converted to, newest first, with their status from
/// [`aep_core::compat`]; it is empty when there is a problem.
pub fn describe(prefix: &[u8]) -> String {
    let found = inspect_prefix(prefix);
    let mut out = Json::default();
    match found.kind {
        FileKind::Project => {
            out.kind = "project";
            match found.head {
                Some(head) => {
                    out.version = Some(head.app);
                    out.format = Some(head.format);
                    out.release = release_name(head.app.major());
                    if head.app.major() > NEWEST_KNOWN_MAJOR {
                        out.problem = Some("too-new");
                    } else {
                        out.targets = compat::targets_for(&head)
                            .into_iter()
                            .map(|t| (t.ae_major, t.status.as_str()))
                            .collect();
                    }
                }
                None => out.problem = Some("damaged"),
            }
        }
        FileKind::Preset => {
            out.kind = "preset";
            match found.preset_format {
                Some(format) => {
                    out.format = Some(format);
                    out.release = release_for_format(format.major);
                    if format.major > NEWEST_KNOWN_FORMAT {
                        out.problem = Some("too-new");
                    } else {
                        out.targets = preset_targets(format);
                    }
                }
                None => out.problem = Some("damaged"),
            }
        }
        FileKind::XmlProject => {
            out.kind = "xml";
            out.problem = Some("xml-project");
        }
        FileKind::AppleDouble => {
            out.kind = "sidecar";
            out.problem = Some("sidecar");
        }
        FileKind::Unknown => {
            out.kind = "unknown";
            out.problem = Some("not-ae-file");
        }
    }
    out.to_string()
}

/// What converting `bytes` to After Effects `target` would change, as the app's preflight
/// (`apps/desktop/src-tauri/src/engine.rs`) reports it:
///
/// ```text
/// {"target":24,"blocked":false,
///  "notes":[{"severity":"info"|"warn"|"block","code":"shadow-color-removed","params":{"count":2}}, …]}
/// ```
///
/// Codes and params are the app's: `effects-removed` and `plugins-kept` carry `names` (and `year`),
/// an Experimental target adds `experimental-target`, a failure is a single blocking note.
pub fn report(bytes: &[u8], target: u32) -> String {
    let blocked = |code: &str| {
        format!(
            "{{\"target\":{target},\"blocked\":true,\"notes\":[{{\"severity\":\"block\",\"code\":\"{code}\",\"params\":{{}}}}]}}"
        )
    };
    let Ok(major) = u8::try_from(target) else {
        return blocked("unsupported-target");
    };
    let (is_blocked, mut notes): (bool, Vec<String>) = match convert(bytes, major) {
        Ok(done) => (false, done.notes.iter().map(core_note).collect()),
        Err(ConvertError::Blocked(notes)) => (true, notes.iter().map(core_note).collect()),
        Err(error) => return blocked(error_code(&error)),
    };
    notes.extend(effect_notes(bytes, major));
    let experimental =
        compat::target(major).is_some_and(|t| t.status == compat::Status::Experimental);
    if experimental && !is_blocked {
        notes.push(
            "{\"severity\":\"warn\",\"code\":\"experimental-target\",\"params\":{}}".to_string(),
        );
    }
    format!(
        "{{\"target\":{target},\"blocked\":{is_blocked},\"notes\":[{}]}}",
        notes.join(",")
    )
}

/// The app's `error_code`: the code the page translates for every way a conversion can fail.
fn error_code(error: &ConvertError) -> &'static str {
    match error {
        ConvertError::Blocked(notes) => notes
            .iter()
            .find(|n| n.code.is_blocker())
            .map_or("structure-not-understood", |n| n.code.as_str()),
        ConvertError::NotADowngrade { .. } => "not-a-downgrade",
        ConvertError::UnsupportedTarget { .. } => "unsupported-target",
        ConvertError::UnknownSource { format } if format.major < 94 => "not-a-downgrade",
        ConvertError::UnknownSource { .. } => "unsupported-source",
        ConvertError::NotRoundTrip => "structure-not-understood",
        ConvertError::Unreadable(_) | ConvertError::NoHead | ConvertError::NotAProject(_) => {
            "damaged"
        }
    }
}

fn core_note(note: &CoreNote) -> String {
    let severity = if note.code.is_blocker() {
        "block"
    } else {
        "info"
    };
    format!(
        "{{\"severity\":\"{severity}\",\"code\":\"{}\",\"params\":{{\"count\":{}}}}}",
        note.code.as_str(),
        note.count
    )
}

/// The app's `effect_notes`: built-in effects the target lacks, and third-party plugins.
fn effect_notes(bytes: &[u8], major: u8) -> Vec<String> {
    let Ok(file) = aep_core::riff::parse(bytes) else {
        return Vec::new();
    };
    let found = aep_core::effects::report(&file, major);
    let names = |list: &[aep_core::effects::UsedEffect]| {
        let quoted: Vec<String> = list.iter().map(|e| json_string(&e.name)).collect();
        format!("[{}]", quoted.join(","))
    };
    let mut notes = Vec::new();
    if !found.missing.is_empty() {
        let year = release_name(major)
            .map_or_else(|| (2000 + u32::from(major)).to_string(), str::to_string);
        notes.push(format!(
            "{{\"severity\":\"warn\",\"code\":\"effects-removed\",\"params\":{{\"count\":{},\"names\":{},\"year\":{}}}}}",
            found.missing.len(),
            names(&found.missing),
            json_string(&year)
        ));
    }
    if !found.third_party.is_empty() {
        notes.push(format!(
            "{{\"severity\":\"info\",\"code\":\"plugins-kept\",\"params\":{{\"count\":{},\"names\":{}}}}}",
            found.third_party.len(),
            names(&found.third_party)
        ));
    }
    notes
}

/// `text` as a JSON string literal. Effect names come from the file, so anything may be in them.
fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The targets of a preset: as for a project with the same format major ([`compat::targets_for`]
/// reads only the format of the head it is given).
fn preset_targets(format: FormatVersion) -> Vec<(u8, &'static str)> {
    let head = Head {
        format,
        app: STAMPS[0].word,
    };
    compat::targets_for(&head)
        .into_iter()
        .map(|t| (t.ae_major, t.status.as_str()))
        .collect()
}

/// The After Effects release a format major comes from: 98 is 2027 down to 94 for 2023
/// (93 is shared by several releases, so it has no single name).
fn release_for_format(major: u16) -> Option<&'static str> {
    match major {
        94..=98 => release_name((major - 71) as u8),
        _ => None,
    }
}

fn platform(app: AppVersion) -> &'static str {
    match app.platform() {
        Platform::Windows => "windows",
        Platform::MacIntel => "mac-intel",
        Platform::MacArm => "mac-arm",
        Platform::Unknown(_) => "unknown",
    }
}

#[derive(Default)]
struct Json {
    kind: &'static str,
    problem: Option<&'static str>,
    version: Option<AppVersion>,
    format: Option<FormatVersion>,
    release: Option<&'static str>,
    targets: Vec<(u8, &'static str)>,
}

/// A string from the fixed vocabulary above or a release name: no characters that need escaping.
fn quoted(s: Option<&str>) -> String {
    s.map_or_else(|| "null".to_string(), |s| format!("\"{s}\""))
}

impl std::fmt::Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version = self.version.map_or_else(
            || "null".to_string(),
            |v| {
                format!(
                    "{{\"major\":{},\"minor\":{},\"patch\":{},\"build\":{},\"platform\":\"{}\",\"release\":{},\"name\":{}}}",
                    v.major(),
                    v.minor(),
                    v.patch(),
                    v.build(),
                    platform(v),
                    v.is_release(),
                    quoted(release_name(v.major())),
                )
            },
        );
        let format = self.format.map_or_else(
            || "null".to_string(),
            |v| format!("{{\"major\":{},\"minor\":{}}}", v.major, v.minor),
        );
        let targets: Vec<String> = self
            .targets
            .iter()
            .map(|(major, status)| {
                format!(
                    "{{\"major\":{major},\"name\":{},\"status\":\"{status}\"}}",
                    quoted(release_name(*major))
                )
            })
            .collect();
        write!(
            f,
            "{{\"kind\":\"{}\",\"problem\":{},\"version\":{version},\"format\":{format},\"release\":{},\"targets\":[{}]}}",
            self.kind,
            quoted(self.problem),
            quoted(self.release),
            targets.join(",")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(name);
        let bytes = std::fs::read(path).expect("fixture");
        bytes[..bytes.len().min(64 * 1024)].to_vec()
    }

    #[test]
    fn a_2026_project_lists_older_targets() {
        let json = describe(&fixture("TEST26-x.aep"));
        assert!(
            json.starts_with("{\"kind\":\"project\",\"problem\":null,"),
            "{json}"
        );
        assert!(json.contains("\"name\":\"2026\""), "{json}");
        assert!(json.contains("\"targets\":[{\"major\":25,"), "{json}");
        assert!(
            json.contains("{\"major\":24,\"name\":\"2024\",\"status\":\"verified\"}"),
            "{json}"
        );
        assert!(
            json.contains("{\"major\":22,\"name\":\"2022\",\"status\":\"experimental\"}"),
            "{json}"
        );
    }

    #[test]
    fn a_2022_project_has_no_targets() {
        let json = describe(&fixture("TEST22-x.aep"));
        assert!(json.contains("\"problem\":null"), "{json}");
        assert!(json.ends_with("\"targets\":[]}"), "{json}");
    }

    #[test]
    fn other_files_say_what_they_are() {
        assert!(describe(b"hello").contains("\"problem\":\"not-ae-file\""));
        assert!(
            describe(&[0x00, 0x05, 0x16, 0x07, 0, 2, 0, 0]).contains("\"problem\":\"sidecar\"")
        );
        assert!(
            describe(b"<?xml version=\"1.0\"?><AfterEffectsProject").contains("\"kind\":\"xml\"")
        );
    }

    #[test]
    fn the_report_matches_the_app() {
        let full = |name: &str| {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures")
                .join(name);
            std::fs::read(path).expect("fixture")
        };
        let to24 = report(&full("TEST26-x.aep"), 24);
        assert!(
            to24.starts_with("{\"target\":24,\"blocked\":false,"),
            "{to24}"
        );
        assert!(!to24.contains("experimental-target"), "{to24}");
        let to23 = report(&full("TEST26-x.aep"), 23);
        assert!(to23.contains("\"code\":\"experimental-target\""), "{to23}");
        let same = report(&full("TEST24-x.aep"), 24);
        assert!(same.contains("\"code\":\"not-a-downgrade\""), "{same}");
        assert!(report(b"nope", 24).contains("\"blocked\":true"));
    }

    #[test]
    fn names_are_escaped() {
        assert_eq!(json_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\u000a\"");
    }

    #[test]
    fn the_abi_round_trips() {
        let bytes = fixture("TEST24-x.aep");
        let ptr = alloc(bytes.len());
        // SAFETY: `ptr` has room for `bytes.len()` bytes and `inspect` takes it back.
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
            inspect(ptr, bytes.len());
        }
        // SAFETY: the result lives until the next call.
        let out = unsafe { std::slice::from_raw_parts(result_ptr(), result_len()) };
        let text = std::str::from_utf8(out).expect("utf-8");
        assert!(text.contains("\"name\":\"2024\""), "{text}");
    }
}
