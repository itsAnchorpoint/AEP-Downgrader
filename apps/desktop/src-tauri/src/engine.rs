//! Glue between the commands and `aep-core`: targets, preflight reports and
//! the conversion of one file. Free of Tauri types so it can be tested directly.

use crate::models::*;
use crate::pattern::output_name;
use aep_core::compat;
use aep_core::convert::{convert, ConvertError, Converted, Note as CoreNote};
use aep_core::version::release_name;
use serde_json::{json, Map};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;

const MAX_PARALLEL: usize = 4;

/// The targets the app offers, newest first, straight from the core crate's table
/// so the app, the command line and the documentation cannot disagree.
pub fn list_targets() -> Vec<TargetInfo> {
    compat::TARGETS
        .iter()
        .map(|t| TargetInfo {
            major: u32::from(t.ae_major),
            name: release_name(t.ae_major).map_or_else(
                || (2000 + u32::from(t.ae_major)).to_string(),
                str::to_string,
            ),
            status: t.status.as_str(),
        })
        .collect()
}

fn run_core(bytes: &[u8], target: u32) -> Result<Converted, ConvertError> {
    let major =
        u8::try_from(target).map_err(|_| ConvertError::UnsupportedTarget { ae_major: u8::MAX })?;
    convert(bytes, major)
}

/// The code the UI translates, for every way a conversion can fail.
fn error_code(error: &ConvertError) -> &'static str {
    match error {
        ConvertError::Blocked(notes) => notes
            .iter()
            .find(|n| n.code.is_blocker())
            .map_or("structure-not-understood", |n| n.code.as_str()),
        ConvertError::NotADowngrade { .. } => "not-a-downgrade",
        ConvertError::UnsupportedTarget { .. } => "unsupported-target",
        // Format majors below 94 (After Effects 22 and older) are at or below every target.
        ConvertError::UnknownSource { format } if format.major < 94 => "not-a-downgrade",
        ConvertError::UnknownSource { .. } => "unsupported-source",
        ConvertError::NotRoundTrip => "structure-not-understood",
        ConvertError::Unreadable(_) | ConvertError::NoHead | ConvertError::NotAProject(_) => {
            "damaged"
        }
    }
}

fn core_note(note: &CoreNote) -> Note {
    let mut params = Map::new();
    params.insert("count".into(), json!(note.count));
    Note {
        severity: if note.code.is_blocker() {
            Severity::Block
        } else {
            Severity::Info
        },
        code: note.code.as_str().into(),
        params,
    }
}

fn blocked(target: u32, code: &str) -> Preflight {
    Preflight {
        target,
        blocked: true,
        notes: vec![Note {
            severity: Severity::Block,
            code: code.into(),
            params: Map::new(),
        }],
    }
}

/// Notes about effects: built-in effects the target lacks (they stay in the file and After
/// Effects shows them as missing) and third-party plug-ins the older version needs installed.
fn effect_notes(bytes: &[u8], target: u32) -> Vec<Note> {
    let (Ok(file), Ok(major)) = (aep_core::riff::parse(bytes), u8::try_from(target)) else {
        return Vec::new();
    };
    let report = aep_core::effects::report(&file, major);
    let names = |list: &[aep_core::effects::UsedEffect]| {
        json!(list.iter().map(|e| e.name.as_str()).collect::<Vec<_>>())
    };
    let mut notes = Vec::new();
    if !report.missing.is_empty() {
        let mut params = Map::new();
        params.insert("count".into(), json!(report.missing.len()));
        params.insert("names".into(), names(&report.missing));
        params.insert(
            "year".into(),
            json!(release_name(major).map_or_else(|| (2000 + target).to_string(), str::to_string)),
        );
        notes.push(Note {
            severity: Severity::Warn,
            code: "effects-removed".into(),
            params,
        });
    }
    if !report.third_party.is_empty() {
        let mut params = Map::new();
        params.insert("count".into(), json!(report.third_party.len()));
        params.insert("names".into(), names(&report.third_party));
        notes.push(Note {
            severity: Severity::Info,
            code: "plugins-kept".into(),
            params,
        });
    }
    notes
}

/// Converts in memory and throws the bytes away; only the notes matter.
pub fn preflight(path: &str, target: u32) -> Preflight {
    let Ok(bytes) = fs::read(path) else {
        return blocked(target, "unreadable");
    };
    let (blocked_by_rules, mut notes) = match run_core(&bytes, target) {
        Ok(done) => (false, done.notes.iter().map(core_note).collect::<Vec<_>>()),
        Err(ConvertError::Blocked(notes)) => (true, notes.iter().map(core_note).collect()),
        Err(error) => return blocked(target, error_code(&error)),
    };
    notes.extend(effect_notes(&bytes, target));
    // Copies for an experimental target were never opened in that After Effects: say so in the
    // report, not only on the target tile.
    let experimental = u8::try_from(target)
        .ok()
        .and_then(compat::target)
        .is_some_and(|t| t.status == compat::Status::Experimental);
    if experimental && !blocked_by_rules {
        notes.push(Note {
            severity: Severity::Warn,
            code: "experimental-target".into(),
            params: Map::new(),
        });
    }
    Preflight {
        target,
        blocked: blocked_by_rules,
        notes,
    }
}

pub enum Outcome {
    Done(PathBuf),
    Skipped,
    Failed(&'static str),
}

/// "name (2).aep", "name (3).aep", ... until a free name is found.
fn first_free(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    (2..)
        .map(|n| path.with_file_name(format!("{stem} ({n}){ext}")))
        .find(|candidate| !candidate.exists())
        .expect("an unbounded range always yields a free name")
}

fn is_same_file(a: &Path, b: &Path) -> bool {
    matches!((fs::canonicalize(a), fs::canonicalize(b)), (Ok(a), Ok(b)) if a == b)
}

/// Writes next to the destination and renames into place; the temporary file never outlives an error.
fn write_atomic(dest: &Path, bytes: &[u8], tag: &str) -> std::io::Result<()> {
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temp = dest.with_file_name(format!(".{name}.{}.{tag}.tmp", std::process::id()));
    let result = fs::write(&temp, bytes).and_then(|()| fs::rename(&temp, dest));
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

pub fn run_job(job: &Job, options: &OutputOptions) -> Outcome {
    let source = Path::new(&job.path);
    let Some(file_name) = source.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return Outcome::Failed("unreadable");
    };
    let folder = match &options.location {
        Location::Folder { folder } => PathBuf::from(folder),
        Location::Keyword(_) => source.parent().map(Path::to_path_buf).unwrap_or_default(),
    };
    let wanted = folder.join(output_name(&options.pattern, &file_name, 2000 + job.target));
    if options.on_exists == OnExists::Skip && wanted.exists() {
        return Outcome::Skipped;
    }
    let Ok(bytes) = fs::read(source) else {
        return Outcome::Failed("unreadable");
    };
    let converted = match run_core(&bytes, job.target) {
        Ok(done) => done,
        Err(error) => return Outcome::Failed(error_code(&error)),
    };
    let dest = if options.on_exists == OnExists::Rename {
        first_free(&wanted)
    } else {
        wanted
    };
    if is_same_file(&dest, source) {
        return Outcome::Failed("would-overwrite-source");
    }
    match write_atomic(&dest, &converted.bytes, &job.id) {
        Ok(()) => Outcome::Done(dest),
        Err(_) => Outcome::Failed("write-failed"),
    }
}

/// Runs the jobs on up to four threads and reports each state change through `emit`. Once
/// `cancel` is set, files already converting finish (no half-written copies) and the rest are
/// reported as cancelled.
pub fn run_batch(
    jobs: Vec<Job>,
    options: &OutputOptions,
    cancel: &AtomicBool,
    emit: &(dyn Fn(ProgressEvent) + Sync),
) {
    let workers = MAX_PARALLEL.min(jobs.len());
    let queue = Mutex::new(VecDeque::from(jobs));
    let event = |id: &str, state, output: Option<&Path>, error: Option<&str>| {
        emit(ProgressEvent {
            id: id.to_string(),
            state,
            output_path: output.map(|p| p.to_string_lossy().into_owned()),
            error_code: error.map(str::to_string),
        });
    };
    thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let next = queue.lock().expect("queue lock").pop_front();
                let Some(job) = next else { break };
                if cancel.load(Ordering::Relaxed) {
                    event(&job.id, ProgressState::Cancelled, None, None);
                    continue;
                }
                event(&job.id, ProgressState::Running, None, None);
                match run_job(&job, options) {
                    Outcome::Done(path) => event(&job.id, ProgressState::Done, Some(&path), None),
                    Outcome::Skipped => event(&job.id, ProgressState::Skipped, None, None),
                    Outcome::Failed(code) => {
                        event(&job.id, ProgressState::Failed, None, Some(code))
                    }
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use aep_core::inspect::inspect;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../tests/fixtures");

    fn project_using(effects: &[&str]) -> Vec<u8> {
        use aep_core::riff::{Node, RiffFile};
        let tdmn = |name: &str| {
            let mut data = name.as_bytes().to_vec();
            data.resize(40, 0);
            Node::Chunk { id: *b"tdmn", data }
        };
        let defs = effects
            .iter()
            .map(|e| Node::List {
                kind: *b"EfDf",
                children: vec![tdmn(e)],
            })
            .collect();
        RiffFile {
            form: *b"Egg!",
            children: vec![Node::List {
                kind: *b"EfdG",
                children: defs,
            }],
            trailer: Vec::new(),
        }
        .to_bytes()
    }

    #[test]
    fn effect_notes_name_missing_effects_and_plug_ins() {
        let notes = effect_notes(
            &project_using(&["ADBE Unmult", "tc Particular", "ADBE Glo2"]),
            24,
        );
        let codes: Vec<_> = notes.iter().map(|n| n.code.as_str()).collect();
        assert_eq!(codes, vec!["effects-removed", "plugins-kept"]);
        assert_eq!(notes[0].params["names"], json!(["Unmult"]));
        assert_eq!(notes[0].params["year"], json!("2024"));
        assert_eq!(notes[1].params["names"], json!(["tc Particular"]));
        assert!(effect_notes(&project_using(&["ADBE Glo2"]), 24).is_empty());
    }

    fn fixture(name: &str) -> String {
        format!("{FIXTURES}/{name}")
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aep-engine-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn options(folder: &Path, on_exists: OnExists) -> OutputOptions {
        OutputOptions {
            location: Location::Folder {
                folder: folder.to_string_lossy().into_owned(),
            },
            pattern: "{name}_AE{year}".into(),
            on_exists,
        }
    }

    fn job(path: String, target: u32) -> Job {
        Job {
            id: "job1".into(),
            path,
            target,
        }
    }

    fn done_path(outcome: Outcome) -> PathBuf {
        match outcome {
            Outcome::Done(p) => p,
            Outcome::Skipped => panic!("skipped"),
            Outcome::Failed(code) => panic!("failed: {code}"),
        }
    }

    #[test]
    fn targets_follow_the_core_table() {
        let targets = list_targets();
        let majors: Vec<_> = targets.iter().map(|t| t.major).collect();
        let table: Vec<_> = compat::TARGETS
            .iter()
            .map(|t| u32::from(t.ae_major))
            .collect();
        assert_eq!(majors, table);
        assert!(
            targets.windows(2).all(|w| w[0].major > w[1].major),
            "newest first"
        );
        assert!(targets
            .iter()
            .all(|t| t.status == "verified" || t.status == "experimental"));
        let verified: Vec<_> = targets.iter().filter(|t| t.status == "verified").collect();
        assert_eq!(verified.len(), 1);
        assert_eq!(verified[0].major, 24);
        assert_eq!(
            targets
                .iter()
                .find(|t| t.major == 25)
                .map(|t| t.name.as_str()),
            Some("2025")
        );
    }

    #[test]
    fn targets_for_a_file_are_a_subset_of_the_offered_ones() {
        let bytes = fs::read(fixture("TEST24-x.aep")).unwrap();
        let head = inspect(&bytes[..bytes.len().min(4096)])
            .head
            .expect("a project has a head");
        let offered: Vec<_> = list_targets().iter().map(|t| t.major).collect();
        let for_file = compat::targets_for(&head);
        assert!(!for_file.is_empty());
        assert!(for_file
            .iter()
            .all(|t| offered.contains(&u32::from(t.ae_major))));
    }

    #[test]
    fn preflight_reports_the_shadow_color_note() {
        let report = preflight(&fixture("TEST24-x.aep"), 23);
        assert!(!report.blocked);
        // 2023 is experimental, so its note follows the shadow colour one.
        assert_eq!(report.notes.len(), 2);
        assert_eq!(report.notes[0].code, "shadow-color-removed");
        assert_eq!(report.notes[1].code, "experimental-target");
        assert_eq!(report.notes[0].params["count"], 2);
    }

    #[test]
    fn preflight_with_nothing_to_report() {
        let report = preflight(&fixture("TEST26-x.aep"), 24);
        assert!(!report.blocked);
        assert!(report.notes.is_empty());
    }

    #[test]
    fn preflight_for_an_experimental_target_says_so() {
        let report = preflight(&fixture("TEST26-x.aep"), 25);
        assert!(!report.blocked);
        assert!(report
            .notes
            .iter()
            .any(|n| n.code == "experimental-target" && matches!(n.severity, Severity::Warn)));
        // Verified targets carry no such note.
        assert!(!preflight(&fixture("TEST26-x.aep"), 24)
            .notes
            .iter()
            .any(|n| n.code == "experimental-target"));
    }

    #[test]
    fn preflight_blocks_what_is_not_a_downgrade() {
        let report = preflight(&fixture("TEST22-x.aep"), 24);
        assert!(report.blocked);
        assert_eq!(report.notes[0].code, "not-a-downgrade");
    }

    #[test]
    fn preflight_of_a_missing_file_is_unreadable() {
        let report = preflight("/no/such/file.aep", 24);
        assert!(report.blocked);
        assert_eq!(report.notes[0].code, "unreadable");
    }

    #[test]
    fn preflight_of_garbage_is_damaged_and_unsupported_target_is_reported() {
        let dir = temp_dir("garbage");
        let file = dir.join("x.aep");
        fs::write(&file, "not a project").unwrap();
        assert_eq!(
            preflight(&file.to_string_lossy(), 24).notes[0].code,
            "damaged"
        );
        assert_eq!(
            preflight(&fixture("TEST26-x.aep"), 21).notes[0].code,
            "unsupported-target"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn conversion_writes_a_new_file_and_nothing_else() {
        let dir = temp_dir("convert");
        let source = fixture("TEST24-x.aep");
        let before = fs::read(&source).unwrap();

        let out = done_path(run_job(
            &job(source.clone(), 23),
            &options(&dir, OnExists::Rename),
        ));

        assert_eq!(out, dir.join("TEST24-x_AE2023.aep"));
        let written = fs::read(&out).unwrap();
        let format = inspect(&written).head.unwrap().format;
        assert_eq!((format.major, format.minor), (94, 9));
        assert_eq!(fs::read(&source).unwrap(), before);
        let files: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(files.len(), 1, "no temporary file may remain: {files:?}");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn on_exists_policies() {
        let dir = temp_dir("exists");
        let source = fixture("TEST24-x.aep");
        let existing = dir.join("TEST24-x_AE2023.aep");
        fs::write(&existing, "old").unwrap();

        let renamed = done_path(run_job(
            &job(source.clone(), 23),
            &options(&dir, OnExists::Rename),
        ));
        assert_eq!(renamed, dir.join("TEST24-x_AE2023 (2).aep"));
        let again = done_path(run_job(
            &job(source.clone(), 23),
            &options(&dir, OnExists::Rename),
        ));
        assert_eq!(again, dir.join("TEST24-x_AE2023 (3).aep"));
        assert_eq!(fs::read(&existing).unwrap(), b"old");

        assert!(matches!(
            run_job(&job(source.clone(), 23), &options(&dir, OnExists::Skip)),
            Outcome::Skipped
        ));
        assert_eq!(fs::read(&existing).unwrap(), b"old");

        let replaced = done_path(run_job(
            &job(source, 23),
            &options(&dir, OnExists::Overwrite),
        ));
        assert_eq!(replaced, existing);
        assert_ne!(fs::read(&existing).unwrap(), b"old");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn never_writes_over_the_source() {
        let dir = temp_dir("same");
        let source = dir.join("a.aep");
        fs::copy(fixture("TEST24-x.aep"), &source).unwrap();
        let before = fs::read(&source).unwrap();
        let mut opts = options(&dir, OnExists::Overwrite);
        opts.pattern = "{name}".into();
        let outcome = run_job(&job(source.to_string_lossy().into_owned(), 23), &opts);
        assert!(matches!(outcome, Outcome::Failed("would-overwrite-source")));
        assert_eq!(fs::read(&source).unwrap(), before);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_missing_source_fails_as_unreadable() {
        let dir = temp_dir("missing");
        let outcome = run_job(
            &job("/no/such/file.aep".into(), 23),
            &options(&dir, OnExists::Rename),
        );
        assert!(matches!(outcome, Outcome::Failed("unreadable")));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_batch_reports_running_then_a_final_state_per_job() {
        let dir = temp_dir("batch");
        let jobs = vec![
            Job {
                id: "a".into(),
                path: fixture("TEST24-x.aep"),
                target: 23,
            },
            Job {
                id: "b".into(),
                path: "/no/such/file.aep".into(),
                target: 23,
            },
        ];
        let events = Mutex::new(Vec::new());
        run_batch(
            jobs,
            &options(&dir, OnExists::Rename),
            &AtomicBool::new(false),
            &|e| events.lock().unwrap().push(e),
        );
        let events = events.into_inner().unwrap();
        let last = |id: &str| events.iter().rev().find(|e| e.id == id).unwrap().state;
        assert!(matches!(last("a"), ProgressState::Done));
        assert!(matches!(last("b"), ProgressState::Failed));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e.state, ProgressState::Running))
                .count(),
            2
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_cancelled_batch_starts_nothing_more() {
        let dir = temp_dir("cancelled");
        let jobs = vec![
            Job {
                id: "a".into(),
                path: fixture("TEST24-x.aep"),
                target: 23,
            },
            Job {
                id: "b".into(),
                path: fixture("TEST24-x.aep"),
                target: 22,
            },
        ];
        let events = Mutex::new(Vec::new());
        run_batch(
            jobs,
            &options(&dir, OnExists::Rename),
            &AtomicBool::new(true),
            &|e| events.lock().unwrap().push(e),
        );
        let events = events.into_inner().unwrap();
        assert!(events
            .iter()
            .all(|e| matches!(e.state, ProgressState::Cancelled)));
        assert_eq!(events.len(), 2);
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0, "no copy written");
        fs::remove_dir_all(dir).unwrap();
    }
}
