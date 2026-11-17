//! [`aep_core::effects::report`] against what After Effects itself said.

use aep_core::effects::report;
use aep_core::riff::parse;
use std::path::Path;

const LOCAL_SHIIRU27: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures-local/shiiru27"
);

/// After Effects 24.5, opening the 27.0 template converted to 24, listed exactly these
/// 17 missing effects (dialog "This project contains references to missing effects"):
/// 15 built-in effects newer than 24, and two plug-ins that the tester has installed and the
/// AE 24.5 used for the check does not (Goodbye Greenscreen, Variable Vibrance).
#[test]
fn template_270_for_24_lists_what_after_effects_245_reported() {
    let path = Path::new(LOCAL_SHIIRU27).join("AEPD_template_27.0.aet");
    if !path.is_file() {
        eprintln!("fixtures-local/shiiru27 not present; skipping");
        return;
    }
    let file = parse(&std::fs::read(path).unwrap()).unwrap();
    let effects = report(&file, 24);
    let mut names: Vec<String> = effects.missing.into_iter().map(|e| e.name).collect();
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
        "Midtones",
        "Texture",
        "Unmult",
    ];
    expected.sort();
    assert_eq!(names, expected);
    for plug_in in ["Goodbye Greenscreen", "Variable Vibrance"] {
        assert!(
            effects
                .third_party
                .iter()
                .any(|e| e.match_name == format!("ADBE {plug_in}") && e.name == plug_in),
            "{plug_in} is not reported as a plug-in"
        );
    }
}
