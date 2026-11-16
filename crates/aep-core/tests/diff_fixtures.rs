//! Integration tests: diff real fixture files.

use aep_core::diff::{self, Change};
use aep_core::riff::parse;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn read_and_parse(name: &str) -> aep_core::riff::RiffFile {
    let bytes = std::fs::read(fixture(name)).expect("reads fixture");
    parse(&bytes).expect("parses fixture")
}

#[test]
fn test23_vs_test22() {
    let a = read_and_parse("TEST23-x.aep");
    let b = read_and_parse("TEST22-x.aep");
    let changes = diff::diff(&a, &b);

    // Exactly 12 Changed with key "ldta" (all 164->160).
    let ldta_changed: Vec<&Change> = changes
        .iter()
        .filter(|c| matches!(c, Change::Changed(p) if p.key == "ldta"))
        .collect();
    assert_eq!(
        ldta_changed.len(),
        12,
        "expected 12 ldta Changed, got {}",
        ldta_changed.len()
    );
    for c in &ldta_changed {
        if let Change::Changed(p) = c {
            assert_eq!(p.len_a, 164, "len_a = {}", p.len_a);
            assert_eq!(p.len_b, 160, "len_b = {}", p.len_b);
        }
    }

    // Exactly 1 Removed with key "LIST:tdbs".
    let removed_tdbs: Vec<&Change> = changes
        .iter()
        .filter(|c| matches!(c, Change::Removed(l) if l.key == "LIST:tdbs"))
        .collect();
    assert_eq!(
        removed_tdbs.len(),
        1,
        "expected 1 Removed LIST:tdbs, got {}",
        removed_tdbs.len()
    );
}

#[test]
fn test24_vs_test23() {
    let a = read_and_parse("TEST24-x.aep");
    let b = read_and_parse("TEST23-x.aep");
    let changes = diff::diff(&a, &b);

    // Exactly 2 Removed with key "tdmn" and preview "ADBE Shadow Color".
    let removed_tdmn: Vec<&Change> = changes
        .iter()
        .filter(|c| matches!(c, Change::Removed(l) if l.key == "tdmn"))
        .collect();
    assert_eq!(
        removed_tdmn.len(),
        2,
        "expected 2 Removed tdmn, got {}",
        removed_tdmn.len()
    );
    for c in &removed_tdmn {
        if let Change::Removed(l) = c {
            assert_eq!(l.preview, "ADBE Shadow Color", "preview = {:?}", l.preview);
        }
    }

    // Exactly 1 Changed with key "otln", len_a 344, len_b 332.
    let changed_otln: Vec<&Change> = changes
        .iter()
        .filter(|c| matches!(c, Change::Changed(p) if p.key == "otln"))
        .collect();
    assert_eq!(
        changed_otln.len(),
        1,
        "expected 1 Changed otl, got {}",
        changed_otln.len()
    );
    for c in &changed_otln {
        if let Change::Changed(p) = c {
            assert_eq!(p.len_a, 344, "len_a = {}", p.len_a);
            assert_eq!(p.len_b, 332, "len_b = {}", p.len_b);
        }
    }
}

#[test]
fn test24_vs_test23_removed_tdbs_prev() {
    let a = read_and_parse("TEST24-x.aep");
    let b = read_and_parse("TEST23-x.aep");
    let changes = diff::diff(&a, &b);

    // Every Removed with key "LIST:tdbs" has prev == "ADBE Shadow Color".
    for c in &changes {
        let Change::Removed(l) = c else { continue };
        if l.key != "LIST:tdbs" {
            continue;
        }
        assert_eq!(
            l.prev, "ADBE Shadow Color",
            "expected prev \"ADBE Shadow Color\" for LIST:tdbs, got {:?}",
            l.prev
        );
    }

    // No Changed entries with key "tdmn".
    for c in &changes {
        if let Change::Changed(p) = c {
            assert_ne!(p.key, "tdmn", "unexpected Changed with key tdmn");
        }
    }
}

#[test]
fn self_diff_is_empty() {
    let f = read_and_parse("TEST24-x.aep");
    let changes = diff::diff(&f, &f);
    assert!(
        changes.is_empty(),
        "self-diff should be empty, got {} changes",
        changes.len()
    );
}
