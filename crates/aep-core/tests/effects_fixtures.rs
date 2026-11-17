//! [`aep_core::effects::report`] against what After Effects itself said.

use aep_core::effects::report;
use aep_core::riff::parse;
use std::path::Path;

const LOCAL_SHIIRU27: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures-local/shiiru27"
);

/// After Effects 24.5, opening the 27.0 template converted to 24, listed exactly these
/// 17 missing effects (dialog "This project contains references to missing effects").
#[test]
fn template_270_for_24_lists_what_after_effects_245_reported() {
    let path = Path::new(LOCAL_SHIIRU27).join("AEPD_template_27.0.aet");
    if !path.is_file() {
        eprintln!("fixtures-local/shiiru27 not present; skipping");
        return;
    }
    let file = parse(&std::fs::read(path).unwrap()).unwrap();
    let mut names: Vec<String> = report(&file, 24)
        .missing
        .into_iter()
        .map(|e| e.name)
        .collect();
    names.sort();
    let mut expected = vec![
        "Color And Contrast",
        "Color Shift",
        "Compound Effect",
        "Compressor",
        "Contour Paint",
        "Contrast",
        "Curl Noise",
        "De-esser",
        "Distortion",
        "Film Color",
        "Flare",
        "Gate",
        "Goodbye Greenscreen",
        "Midtones",
        "Texture",
        "Unmult",
        "Variable Vibrance",
    ];
    expected.sort();
    assert_eq!(names, expected);
}
