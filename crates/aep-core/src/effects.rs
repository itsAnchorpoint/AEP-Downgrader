//! Which effects a file uses that the target After Effects does not have, and which effects
//! come from third-party plug-ins. A conversion cannot add effects: an effect newer than the
//! target stays in the file and After Effects shows it as missing.
//!
//! # Where the data comes from
//!
//! - [`ADDED`]: built-in effects added after After Effects 22, with the major version that
//!   added them. Derived by comparing the built-in effect lists of After Effects 24.5, 26.2 and
//!   27.0 (dumped by `tools/ae/dump_effects.jsx` on testers' machines) and checked against
//!   Adobe's feature history (adobedocs.github.io/after-effects-feature-history): OCIO Look and
//!   CDL Transform in 24.0; Unmult and the Compressor, Distortion and Gate audio effects in 26.0;
//!   Curl Noise in 26.3; Contour Paint and De-esser in 27.0 (release coverage of 27.0). The other
//!   effects in the 27.0 list but not in the 26.2 list were added somewhere between 26.3 and
//!   27.0; until a list from the last 26.x exists they are entered as 26, so that the app never
//!   tells a 26.x user an effect is missing when it may be there. No lists from 22, 23 or 25
//!   exist yet, so effects added in 23 or 25 are not known (none appeared between 24.5 and 26.2).
//! - [`BUILT_IN_OTHER`]: built-in effects whose match name does not start with `ADBE ` or `CC `,
//!   from the effect list of After Effects 27.0 on macOS.
//! - [`PLUG_INS_NAMED_ADBE`]: third-party plug-ins whose match name starts with `ADBE ` anyway,
//!   found in testers' effect lists by their plug-in category (Goodbye Greenscreen is listed
//!   under "Blace Plugins" and sold by Blace on aescripts.com).
//!
//! # How effects are found
//!
//! A project lists every effect it uses once, in `LIST:EfdG` > `LIST:EfDf` > `tdmn` (the
//! effect's match name). A preset has no such list; there every `tdmn` match name is checked
//! against [`ADDED`] for missing effects, and third-party effects are not reported.

use crate::riff::{FourCc, Node, RiffFile};

const TDMN: FourCc = *b"tdmn";
const EFDG: FourCc = *b"EfdG";
const EFDF: FourCc = *b"EfDf";
const FAFX: FourCc = *b"FaFX";
const SSPC: FourCc = *b"sspc";
const FNAM: FourCc = *b"fnam";

/// Built-in effect added after After Effects 22: (match name, display name, AE major).
pub const ADDED: &[(&str, &str, u8)] = &[
    ("ADBE OCIO CDL Transform", "OCIO CDL Transform", 24),
    ("ADBE OCIO Look Transform", "OCIO Look Transform", 24),
    ("ADBE Unmult", "Unmult", 26),
    ("ADBE Aud Compressor", "Compressor", 26),
    ("ADBE Aud Distortion", "Distortion", 26),
    ("ADBE Aud Gate", "Gate", 26),
    ("ADBE Curl Noise", "Curl Noise", 26),
    ("ADBE Aud Deesser", "De-esser", 27),
    ("ADBE Contour Paint", "Contour Paint", 27),
    ("ADBE Variable Vibrance", "Variable Vibrance", 26), // after 26.2, by 27.0
    ("ADBE Color And Contrast", "Color And Contrast", 26), // after 26.2, by 27.0
    ("ADBE Color Film", "Film Color", 26),               // after 26.2, by 27.0
    ("ADBE Color Shift", "Color Shift", 26),             // after 26.2, by 27.0
    ("ADBE Color Texture", "Texture", 26),               // after 26.2, by 27.0
    ("ADBE CompoundEffect", "Compound Effect", 26),      // after 26.2, by 27.0
    ("ADBE Contrast", "Contrast", 26),                   // after 26.2, by 27.0
    ("ADBE Flare", "Flare", 26),                         // after 26.2, by 27.0
    ("ADBE LUT Transform", "LUT Transform", 26),         // after 26.2, by 27.0
    ("ADBE Midtones", "Midtones", 26),                   // after 26.2, by 27.0
];

/// Built-in effects whose match name does not start with `ADBE ` or `CC `. Match names that
/// start with one of these entries ending in a space (`"APC "`) cover a whole family.
pub const BUILT_IN_OTHER: &[&str] = &[
    "APC ",
    "CINEMA 4D Effect",
    "CS ",
    "CSRainfall",
    "CSSnowfall",
    "Cryptomatte",
    "EXtractoR",
    "IDentifier",
    "Keylight 906",
    "Mettle SkyBox ",
    "VISINF ",
    "mochaAECC",
];

/// Third-party plug-ins whose match name starts with `ADBE ` like a built-in effect.
pub const PLUG_INS_NAMED_ADBE: &[&str] = &["ADBE Goodbye Greenscreen"];

/// An effect the file uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsedEffect {
    pub match_name: String,
    /// Display name: from [`ADDED`] for built-in effects, from the project's effect
    /// definition for plug-ins, else the match name.
    pub name: String,
}

/// What a file's effects mean for one target.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectReport {
    /// Built-in effects that the target After Effects does not have, in file order.
    pub missing: Vec<UsedEffect>,
    /// Effects from third-party plug-ins (projects only), in file order.
    pub third_party: Vec<UsedEffect>,
}

/// Match name of a `tdmn` chunk: its data up to the first NUL, as UTF-8 (lossy).
fn match_name(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).into_owned()
}

/// `true` for match names of built-in effects. Pseudo effects (`Pseudo/...`, expression
/// controls saved with presets) are not plug-ins and count as built-in.
pub fn is_built_in(match_name: &str) -> bool {
    if PLUG_INS_NAMED_ADBE.contains(&match_name) {
        return false;
    }
    match_name.starts_with("ADBE ")
        || match_name.starts_with("CC ")
        || match_name.starts_with("Pseudo/")
        || BUILT_IN_OTHER.iter().any(|b| {
            if b.ends_with(' ') {
                match_name.starts_with(b)
            } else {
                match_name == *b
            }
        })
}

/// The After Effects major that added a built-in effect, if it was added after 22.
pub fn added_in(match_name: &str) -> Option<(&'static str, u8)> {
    ADDED
        .iter()
        .find(|(m, _, _)| *m == match_name)
        .map(|(_, name, major)| (*name, *major))
}

/// Display name stored with an effect definition: `LIST:sspc` > `fnam`, whose data is a
/// nested `Utf8` chunk (id, big-endian u32 length, text).
fn definition_display_name(children: &[Node]) -> Option<String> {
    let sspc = children.iter().find_map(|c| match c {
        Node::List { kind, children } if *kind == SSPC => Some(children),
        _ => None,
    })?;
    let fnam = sspc.iter().find_map(|c| match c {
        Node::Chunk { id, data } if *id == FNAM => Some(data),
        _ => None,
    })?;
    if fnam.get(0..4)? != b"Utf8" {
        return None;
    }
    let len = u32::from_be_bytes(fnam.get(4..8)?.try_into().ok()?) as usize;
    let text = String::from_utf8_lossy(fnam.get(8..8usize.checked_add(len)?)?)
        .trim()
        .to_owned();
    (!text.is_empty()).then_some(text)
}

/// Effects a project declares in `LIST:EfdG` > `LIST:EfDf`, in order, without repeats:
/// (match name, display name if stored).
fn declared_effects(file: &RiffFile) -> Vec<(String, Option<String>)> {
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    fn visit(nodes: &[Node], in_efdg: bool, out: &mut Vec<(String, Option<String>)>) {
        for node in nodes {
            if let Node::List { kind, children } = node {
                if in_efdg && *kind == EFDF {
                    if let Some(Node::Chunk { data, .. }) = children
                        .iter()
                        .find(|c| matches!(c, Node::Chunk { id, .. } if *id == TDMN))
                    {
                        let name = match_name(data);
                        if !name.is_empty() && !out.iter().any(|(m, _)| *m == name) {
                            out.push((name, definition_display_name(children)));
                        }
                    }
                } else {
                    visit(children, in_efdg || *kind == EFDG, out);
                }
            }
        }
    }
    visit(&file.children, false, &mut out);
    out
}

/// Every distinct `tdmn` match name in the file, in order.
fn all_match_names(file: &RiffFile) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    file.walk(&mut |_, node| {
        if let Node::Chunk { id, data } = node
            && *id == TDMN
        {
            let name = match_name(data);
            if !out.contains(&name) {
                out.push(name);
            }
        }
    });
    out
}

/// The effects of `file` that After Effects `target_major` lacks, and its third-party effects.
pub fn report(file: &RiffFile, target_major: u8) -> EffectReport {
    let preset = file.form == FAFX;
    let names = if preset {
        all_match_names(file)
            .into_iter()
            .map(|m| (m, None))
            .collect()
    } else {
        declared_effects(file)
    };
    let mut out = EffectReport::default();
    for (match_name, display) in names {
        if let Some((name, major)) = added_in(&match_name) {
            if major > target_major {
                out.missing.push(UsedEffect {
                    match_name,
                    name: name.to_owned(),
                });
            }
        } else if !preset && !is_built_in(&match_name) {
            out.third_party.push(UsedEffect {
                name: display.unwrap_or_else(|| match_name.clone()),
                match_name,
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tdmn(name: &str) -> Node {
        let mut data = name.as_bytes().to_vec();
        data.resize(40, 0);
        Node::Chunk { id: TDMN, data }
    }

    fn efdf(name: &str) -> Node {
        Node::List {
            kind: EFDF,
            children: vec![
                tdmn(name),
                Node::List {
                    kind: *b"sspc",
                    children: vec![],
                },
            ],
        }
    }

    fn project(effects: &[&str]) -> RiffFile {
        RiffFile {
            form: *b"Egg!",
            children: vec![Node::List {
                kind: EFDG,
                children: effects.iter().map(|e| efdf(e)).collect(),
            }],
            trailer: Vec::new(),
        }
    }

    #[test]
    fn built_in_families_and_plug_ins() {
        assert!(is_built_in("ADBE Gaussian Blur 2"));
        assert!(is_built_in("CC Toner"));
        assert!(is_built_in("APC Colorama"));
        assert!(is_built_in("Keylight 906"));
        assert!(is_built_in("Mettle SkyBox Glow"));
        assert!(is_built_in("Pseudo/ADBE Timer Controls"));
        assert!(!is_built_in("tc Particular"));
        assert!(!is_built_in("Keylight 906 Pro"));
        assert!(!is_built_in("VideoCopilot OpticalFlares"));
        assert!(!is_built_in("ADBE Goodbye Greenscreen"));
    }

    #[test]
    fn missing_depends_on_the_target() {
        let file = project(&[
            "ADBE Gaussian Blur 2",
            "ADBE Curl Noise",
            "ADBE Contour Paint",
            "ADBE Unmult",
        ]);
        let names = |t: u8| {
            report(&file, t)
                .missing
                .into_iter()
                .map(|e| e.name)
                .collect::<Vec<_>>()
        };
        assert_eq!(names(26), vec!["Contour Paint"]);
        assert_eq!(names(25), vec!["Curl Noise", "Contour Paint", "Unmult"]);
        assert_eq!(names(24), names(25));
    }

    #[test]
    fn third_party_effects_of_a_project() {
        let file = project(&[
            "tc Particular",
            "ADBE Glo2",
            "tc Particular",
            "VIDEOCOPILOT OpticalFlares",
        ]);
        let r = report(&file, 24);
        assert!(r.missing.is_empty());
        let names: Vec<_> = r.third_party.into_iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["tc Particular", "VIDEOCOPILOT OpticalFlares"]);
    }

    #[test]
    fn presets_report_missing_effects_from_any_match_name() {
        let file = RiffFile {
            form: FAFX,
            children: vec![Node::List {
                kind: *b"besc",
                children: vec![
                    tdmn("ADBE Effect Parade"),
                    tdmn("ADBE Variable Vibrance"),
                    tdmn("tc Particular"),
                ],
            }],
            trailer: Vec::new(),
        };
        let r = report(&file, 25);
        assert_eq!(
            r.missing
                .iter()
                .map(|e| e.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Variable Vibrance"]
        );
        assert!(r.third_party.is_empty());
    }

    #[test]
    fn effects_outside_efdg_are_not_declared() {
        let file = RiffFile {
            form: *b"Egg!",
            children: vec![efdf("tc Particular")],
            trailer: Vec::new(),
        };
        assert_eq!(report(&file, 24), EffectReport::default());
    }
}
