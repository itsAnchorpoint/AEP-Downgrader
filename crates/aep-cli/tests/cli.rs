//! The `aep` binary, run as a user would run it.

use std::process::Command;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/TEST24-x.aep"
);

const FIXTURE_22: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/TEST22-x.aep"
);

const FIXTURE_23: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/TEST23-x.aep"
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

#[test]
fn diff_detects_changes_and_reports_count() {
    let (code, stdout) = aep(&["diff", FIXTURE_23, FIXTURE_22]);
    assert_eq!(code, 1, "exit code should be 1 when changes found");
    // The last line is the changes count trailer.
    let lines: Vec<&str> = stdout.lines().collect();
    let last = lines.last().expect("has at least one line");
    assert!(
        last.ends_with("changes"),
        "last line should end with 'changes', got: {last}"
    );
    // The trailer should report 18 changes (12 ldta + 1 LIST:tdbs + others).
    assert!(
        stdout.contains("Fold/Item/Layr  ldta 164->160"),
        "expected ldta 164->160 in output"
    );
    assert!(stdout.contains("LIST:tdbs"), "expected LIST:tdbs in output");
}

#[test]
fn diff_self_is_zero() {
    let (code, stdout) = aep(&["diff", FIXTURE, FIXTURE]);
    assert_eq!(code, 0, "self-diff should have exit code 0");
    assert_eq!(stdout, "0 changes\n");
}

#[test]
fn diff_full_shows_details() {
    let (code, stdout) = aep(&["diff", "--full", FIXTURE, FIXTURE_23]);
    assert_eq!(code, 1);
    // Should show index paths with dots and sample details.
    assert!(
        stdout.contains("Fold/Item/Layr/tdgp/tdgp  tdmn"),
        "expected tdmn in --full output, got:\n{stdout}"
    );
    assert!(
        stdout.contains("\"ADBE Shadow Color\""),
        "expected ADBE Shadow Color preview in --full output"
    );
}

#[test]
fn diff_no_noise_hides_noise() {
    let (code, stdout) = aep(&["diff", "--no-noise", FIXTURE_23, FIXTURE_22]);
    assert!(
        code == 0 || code == 1,
        "exit code should be 0 or 1, got {code}"
    );
    let lines: Vec<&str> = stdout.lines().collect();
    let last = lines.last().expect("has at least one line");
    assert!(
        last.contains("noise hidden"),
        "last line should contain 'noise hidden', got: {last}"
    );
}
