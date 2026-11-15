//! Every fixture must parse and serialize back byte for byte.

use aep_core::riff::{Node, parse};
use std::path::PathBuf;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures");
/// Real-world projects that are not in the repository. Used when present.
const LOCAL_FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures-local");

fn count_nodes(file: &aep_core::riff::RiffFile) -> u64 {
    let mut n = 0u64;
    file.walk(&mut |_path, _node: &Node| n += 1);
    n
}

#[test]
fn fixtures_roundtrip() {
    let dir = PathBuf::from(FIXTURES);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("fixtures folder exists")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "aep"))
        .collect();
    assert!(!files.is_empty(), "no .aep fixture found in {FIXTURES}");
    if let Ok(local) = std::fs::read_dir(LOCAL_FIXTURES) {
        files.extend(
            local
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "aep")),
        );
    }
    files.sort();
    assert!(!files.is_empty(), "no .aep fixture found in {FIXTURES}");
    for path in &files {
        let bytes =
            std::fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let file =
            parse(&bytes).unwrap_or_else(|e| panic!("{}: parse failed: {e:?}", path.display()));
        let out = file.to_bytes();
        assert_eq!(out, bytes, "{}: round-trip differs", path.display());
        let n = count_nodes(&file);
        assert!(
            n > 100,
            "{}: only {n} nodes, expected more than 100",
            path.display()
        );
    }
}
