//! Integration tests: run [`aep_core::inspect::inspect`] over the first 4096
//! bytes of the real `.aep` fixtures and check the kind, head format, the
//! `head` app word and the `svap` word.

use aep_core::inspect::{FileKind, inspect};
use aep_core::version::{AppVersion, FormatVersion};
use std::fs;
use std::path::Path;

/// Workspace root, two levels above this crate's manifest directory.
const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../");

/// Read `rel` (relative to the workspace root) and return its first 4096 bytes.
fn read_prefix(rel: &str) -> Vec<u8> {
    let path = Path::new(ROOT).join(rel);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("cannot read {rel:?}: {e}"));
    data.into_iter().take(4096).collect()
}

#[test]
fn fixtures() {
    // (relative path, format major, format minor, head.app word, svap word)
    let cases = [
        (
            "tests/fixtures/TEST22-x.aep",
            93u16,
            43u16,
            0x0B33_063Bu32,
            0x0F02_8634u32,
        ),
        (
            "tests/fixtures/TEST23-x.aep",
            94,
            9,
            0x0B3B_0637,
            0x0F02_8634,
        ),
        (
            "tests/fixtures/TEST24-x.aep",
            95,
            5,
            0x0F02_8634,
            0x0F02_8634,
        ),
        (
            "tests/fixtures-local/TEST25-4.aep",
            96,
            6,
            0x0F0A_0656,
            0x0F0A_0656,
        ),
        (
            "tests/fixtures-local/TEST25-6.aep",
            96,
            9,
            0x0F0B_0665,
            0x0F0B_0665,
        ),
        (
            "tests/fixtures/TEST26-x.aep",
            97,
            2,
            0x0F10_0643,
            0x0F10_0643,
        ),
    ];
    for (rel, major, minor, head_app, saved_by) in cases {
        // Real-world projects are not in the repository; check them when present.
        if rel.starts_with("tests/fixtures-local/")
            && !std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../"))
                .join(rel)
                .exists()
        {
            continue;
        }
        let data = read_prefix(rel);
        let r = inspect(&data);
        assert_eq!(r.kind, FileKind::Project, "{rel}: kind");
        let head = r
            .head
            .unwrap_or_else(|| panic!("{rel}: head chunk not found"));
        assert_eq!(head.format, FormatVersion { major, minor }, "{rel}: format");
        assert_eq!(head.app, AppVersion(head_app), "{rel}: head.app");
        assert_eq!(r.saved_by, Some(AppVersion(saved_by)), "{rel}: saved_by");
    }
}
