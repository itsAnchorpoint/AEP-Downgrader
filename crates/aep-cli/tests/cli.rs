//! The `aep` binary, run as a user would run it.

use std::process::Command;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/TEST24-x.aep"
);

/// Run `aep` and return its exit code and stdout.
fn aep(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aep"))
        .args(args)
        .output()
        .expect("aep runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

#[test]
fn tree_prints_one_line_per_node() {
    let (code, stdout) = aep(&["tree", FIXTURE]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1609);
    assert_eq!(lines[0], "svap 4");
    assert_eq!(lines[1], "head 20");
}

#[test]
fn roundtrip_reports_ok() {
    let (code, stdout) = aep(&["roundtrip", FIXTURE]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "OK 63976 bytes, 1609 nodes\n");
}

#[test]
fn bad_usage_and_missing_file() {
    assert_eq!(aep(&[]).0, 2);
    assert_eq!(aep(&["nope", "x"]).0, 2);
    assert_eq!(aep(&["tree", "/no/such/file"]).0, 1);
}
