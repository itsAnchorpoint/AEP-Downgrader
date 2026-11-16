//! Integration tests: compare [`aep_core::convert`] output against files
//! written by After Effects itself. The expected results below were measured
//! on the real fixture files and must not be weakened.

use aep_core::convert::{ConvertError, NoteCode, convert};
use aep_core::diff::{self, Change};
use aep_core::riff::{RiffFile, parse};
use std::path::Path;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/");
const LOCAL_KIT24: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures-local/kit24"
);

/// Read a fixture from the shared `tests/fixtures` folder.
fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(FIXTURES).join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Read a file from the local `tests/fixtures-local/kit24` folder.
fn local(name: &str) -> Vec<u8> {
    let path = Path::new(LOCAL_KIT24).join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn parse_bytes(bytes: &[u8]) -> RiffFile {
    parse(bytes).expect("parses the bytes")
}

/// First 8 bytes of the data of the first top-level `head` chunk.
fn head_first8(file: &RiffFile) -> [u8; 8] {
    let data = file
        .chunk(*b"head")
        .expect("file has a top-level head chunk");
    let mut out = [0u8; 8];
    out.copy_from_slice(&data[..8]);
    out
}

/// The key of a change (`tdmn`, `LIST:tdbs`, `otln`, ...).
fn key_of(change: &Change) -> &str {
    match change {
        Change::Removed(l) => &l.key,
        Change::Added(l) => &l.key,
        Change::Changed(p) => &p.key,
    }
}

/// Diff two parsed files and drop every change that is save noise.
fn diff_filtered(ours: &RiffFile, theirs: &RiffFile) -> Vec<Change> {
    diff::diff(ours, theirs)
        .into_iter()
        .filter(|c| !diff::is_save_noise(c))
        .collect()
}

/// Convert `input` to `target`, parse the result, diff it against the parsed
/// AE-written `reference`, and drop every change that is save noise.
fn remaining(input: &[u8], target: u8, reference: &[u8]) -> Vec<Change> {
    let out = convert(input, target).expect("convert succeeds");
    diff_filtered(&parse_bytes(&out.bytes), &parse_bytes(reference))
}

/// One line per group of remaining changes, for failure messages.
fn report(changes: &[Change]) -> String {
    diff::summarize(changes).join("\n")
}

/// The keys that may legitimately remain after a conversion (they differ
/// between separately-saved files or are version-dependent bookkeeping).
const ALLOWED: &[&str] = &["otln", "fdta", "fovi", "LIST:Item"];

#[test]
fn test_a_24_to_23() {
    let input = fixture("TEST24-x.aep");
    let reference = fixture("TEST23-x.aep");
    let out = convert(&input, 23).expect("convert succeeds");
    let ours = parse_bytes(&out.bytes);
    let changes = diff_filtered(&ours, &parse_bytes(&reference));

    for c in &changes {
        assert!(
            ALLOWED.contains(&key_of(c)),
            "unexpected remaining change with key {}; remaining:\n{}",
            key_of(c),
            report(&changes)
        );
    }
    for c in &changes {
        match c {
            Change::Removed(l) | Change::Added(l) => {
                assert_ne!(
                    l.preview,
                    "ADBE Shadow Color",
                    "remaining Removed/Added has preview shadow color; remaining:\n{}",
                    report(&changes)
                );
                assert_ne!(
                    l.prev,
                    "ADBE Shadow Color",
                    "remaining Removed/Added has prev shadow color; remaining:\n{}",
                    report(&changes)
                );
            }
            Change::Changed(_) => {}
        }
    }

    assert_eq!(
        head_first8(&ours),
        head_first8(&parse_bytes(&reference)),
        "output head bytes 0..8 must equal TEST23-x.aep's"
    );
}

#[test]
fn test_b_23_to_22() {
    let input = fixture("TEST23-x.aep");
    let reference = fixture("TEST22-x.aep");
    let out = convert(&input, 22).expect("convert succeeds");
    let ours = parse_bytes(&out.bytes);
    let changes = diff_filtered(&ours, &parse_bytes(&reference));

    assert!(
        changes.is_empty(),
        "expected no remaining changes, got {}:\n{}",
        changes.len(),
        report(&changes)
    );

    assert_eq!(
        head_first8(&ours),
        head_first8(&parse_bytes(&reference)),
        "output head bytes 0..8 must equal TEST22-x.aep's"
    );
}

#[test]
fn test_c_24_to_22() {
    let input = fixture("TEST24-x.aep");
    let reference = fixture("TEST22-x.aep");
    let changes = remaining(&input, 22, &reference);

    for c in &changes {
        assert!(
            ALLOWED.contains(&key_of(c)),
            "unexpected remaining change with key {}; remaining:\n{}",
            key_of(c),
            report(&changes)
        );
    }
    for c in &changes {
        if let Change::Changed(p) = c {
            assert_ne!(
                p.key,
                "ldta",
                "unexpected remaining Changed ldta; remaining:\n{}",
                report(&changes)
            );
        }
    }
    for c in &changes {
        if let Change::Removed(l) = c {
            assert_ne!(
                l.key,
                "LIST:tdbs",
                "unexpected remaining Removed LIST:tdbs; remaining:\n{}",
                report(&changes)
            );
        }
    }
}

#[test]
fn test_d_26_to_24() {
    let input = fixture("TEST26-x.aep");
    let out = convert(&input, 24).expect("convert succeeds");
    let out_bytes = &out.bytes;

    assert_eq!(
        out_bytes.len(),
        input.len(),
        "output length must equal input length"
    );

    let mut differing: Vec<usize> = Vec::new();
    for (i, (a, b)) in input.iter().zip(out_bytes).enumerate() {
        if a != b {
            differing.push(i);
        }
    }
    assert!(
        differing.len() <= 8,
        "at most 8 bytes may differ, got {}: {:?}",
        differing.len(),
        differing
    );
    for &pos in &differing {
        assert!(
            (32..40).contains(&pos),
            "differing byte at file offset {} is outside 32..40: {:?}",
            pos,
            differing
        );
    }
}

#[test]
fn test_e_local_245_to_23() {
    if !Path::new(LOCAL_KIT24).is_dir() {
        eprintln!("fixtures-local/kit24 not present; skipping");
        return;
    }
    let input = local("AEPD_test_24.5_win.aep");
    let reference = local("AEPD_test_24.5_win_as_23.aep");
    let out = convert(&input, 23).expect("convert succeeds");
    let changes = diff_filtered(&parse_bytes(&out.bytes), &parse_bytes(&reference));

    assert_eq!(
        changes.len(),
        2,
        "expected exactly 2 remaining changes, got {}:\n{}",
        changes.len(),
        report(&changes)
    );
    for c in &changes {
        match c {
            Change::Changed(p) => assert_eq!(
                p.key,
                "otln",
                "expected Changed otl, got {}; remaining:\n{}",
                p.key,
                report(&changes)
            ),
            other => panic!(
                "expected only Changed(otln), got: {}\n{}",
                report(std::slice::from_ref(other)),
                report(&changes)
            ),
        }
    }

    assert!(
        out.notes
            .iter()
            .any(|n| n.code == NoteCode::ShadowColorRemoved && n.count == 108),
        "expected ShadowColorRemoved x108 in notes, got: {:?}",
        out.notes
    );
}

#[test]
fn test_e_local_245_23_to_22_blocked() {
    if !Path::new(LOCAL_KIT24).is_dir() {
        eprintln!("fixtures-local/kit24 not present; skipping");
        return;
    }
    let input = local("AEPD_test_24.5_win_as_23.aep");
    match convert(&input, 22) {
        Err(ConvertError::Blocked(notes)) => assert!(
            notes
                .iter()
                .any(|n| n.code == NoteCode::MatteNotAdjacent && n.count == 2),
            "expected MatteNotAdjacent x2 in notes, got: {:?}",
            notes
        ),
        other => panic!("expected Err(Blocked), got: {:?}", other),
    }
}

const LOCAL_SHIIRU27: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures-local/shiiru27"
);

/// Read a file from the local `tests/fixtures-local/shiiru27` folder.
fn local27(name: &str) -> Vec<u8> {
    let path = Path::new(LOCAL_SHIIRU27).join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[test]
fn test_f_local_270_to_26() {
    if !Path::new(LOCAL_SHIIRU27).is_dir() {
        eprintln!("fixtures-local/shiiru27 not present; skipping");
        return;
    }
    let input = local27("AEPD_test_27.0_mac.aep");
    let reference = local27("AEPD_test_27.0_as_26.x.aep");
    let out = convert(&input, 26).expect("convert succeeds");
    let ours = parse_bytes(&out.bytes);
    let theirs = parse_bytes(&reference);
    let changes = diff_filtered(&ours, &theirs);

    for c in &changes {
        if let Change::Removed(l) = c {
            assert!(
                l.key != "ofl2" && l.key != "mtsf",
                "remaining Removed {}; remaining:\n{}",
                l.key,
                report(&changes)
            );
            assert!(
                !l.preview.starts_with("ADBE3D Mat Rand Offset"),
                "remaining Removed Mat Rand Offset; remaining:\n{}",
                report(&changes)
            );
        }
        assert!(
            ["Utf8", "apid", "dcui", "tdmn", "fdta"].contains(&key_of(c)),
            "unexpected remaining change with key {}; remaining:\n{}",
            key_of(c),
            report(&changes)
        );
    }
    assert_eq!(
        head_first8(&ours)[..4],
        head_first8(&theirs)[..4],
        "output head bytes 0..4 must equal the reference's"
    );
}

#[test]
fn test_g_local_26_to_25() {
    if !Path::new(LOCAL_SHIIRU27).is_dir() {
        eprintln!("fixtures-local/shiiru27 not present; skipping");
        return;
    }
    let input = local27("AEPD_test_27.0_as_26.x.aep");
    let reference = local27("AEPD_test_27.0_as_25.x.aep");
    let changes = remaining(&input, 25, &reference);

    for c in &changes {
        if let Change::Removed(l) = c {
            assert!(
                l.key != "elab" && l.key != "LIST:GdV2" && l.key != "cety",
                "remaining Removed {}; remaining:\n{}",
                l.key,
                report(&changes)
            );
            assert_ne!(
                l.preview,
                "ADBE3D Material Parade",
                "remaining Removed Material Parade; remaining:\n{}",
                report(&changes)
            );
        }
    }
    assert!(
        changes.len() <= 25,
        "expected at most 25 remaining changes, got {}:\n{}",
        changes.len(),
        report(&changes)
    );
}
