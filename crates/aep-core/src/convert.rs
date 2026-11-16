//! Rewrite a project so that an older After Effects opens it.
//!
//! The rules below reproduce what After Effects itself does in
//! "Save a Copy As <older version>". They were derived by comparing files
//! that After Effects 24.5 saved from one project as 24.x, 23.x and 22.x.
//!
//! # Pipeline of [`convert`]
//!
//! 1. Parse the input ([`crate::riff::parse`]); failure → [`ConvertError::Unreadable`].
//! 2. Safety net: `parsed.to_bytes()` must equal the input, else [`ConvertError::NotRoundTrip`].
//! 3. The form must be `Egg!` (else [`ConvertError::NotAProject`]) and there must be a
//!    top-level `head` chunk that [`Head::parse`] accepts (else [`ConvertError::NoHead`]).
//! 4. [`stamp_for`]`(target_ae_major)` must exist (else [`ConvertError::UnsupportedTarget`]).
//!    The source format major must be at most 98 (else [`ConvertError::UnknownSource`]), and the
//!    stamp's format major must be lower than the source's (else [`ConvertError::NotADowngrade`],
//!    which also covers sources already below 94).
//! 5. Apply the steps for every format major from the source down to the target's + 1,
//!    highest first (table below). Collect notes.
//! 6. If any note is a blocker, return [`ConvertError::Blocked`] with all notes. Nothing is written.
//! 7. Stamp `head`: write the target stamp's format and word into the first eight bytes of the
//!    `head` chunk, keeping the source file's platform bits
//!    (`stamp.word.with_platform(source.app.platform())`). `svap` is left alone.
//! 8. Serialize. Return the bytes, the notes, and the old and new [`Head`].
//!
//! # Steps (format major → what leaving it does)
//!
//! | from | to | rule |
//! |------|----|------|
//! | 98, 97, 96 | one lower | nothing besides the stamp |
//! | 95 (AE 24) | 94 | S1 |
//! | 94 (AE 23) | 93 (AE 22) | M1 (may block), L1, E1, B1 |
//!
//! In every rule "everywhere" means: in the children of every [`Node::List`], at any depth
//! (and in the top-level children). A match name is the bytes of a `tdmn` chunk's data up
//! to the first NUL byte.
//!
//! - **S1** Everywhere: a `tdmn` chunk with match name `ADBE Shadow Color` that is
//!   immediately followed by a list of kind `tdbs` → remove both nodes.
//!   Note [`NoteCode::ShadowColorRemoved`], count = pairs removed.
//! - **M1** A *layer* is a [`Node::List`] that has a direct child chunk `ldta` of exactly 164
//!   bytes. Its id is the big-endian u32 at `ldta[0..4]`, its matte id the one at
//!   `ldta[160..164]`. For each list of siblings, walk the layers in order; a layer whose
//!   matte id is not 0 must have, as the previous *layer* among its siblings, the layer
//!   whose id equals that matte id. Each layer violating this counts once for the note
//!   [`NoteCode::MatteNotAdjacent`], which is a blocker.
//! - **L1** Everywhere: an `ldta` chunk of exactly 164 bytes → keep its first 160 bytes.
//!   Note [`NoteCode::LayerRecordsShortened`], count = chunks shortened.
//! - **E1** Everywhere: a `tdmn` chunk whose match name is one of `ADBE Light Env Atom`,
//!   `ADBE Light Backgd Visible`, `ADBE Light Backgd Opacity`, `ADBE Light Backgd Blur`,
//!   immediately followed by any list (either list variant) → remove both nodes.
//!   Note [`NoteCode::LightEnvironmentRemoved`], count = pairs removed.
//! - **B1** Everywhere: a list of kind `tdbs` whose previous sibling is a `blsi` chunk →
//!   remove that list. Note [`NoteCode::LayerStyleOptionRemoved`], count = lists removed.
//!
//! A note is only emitted when its count is greater than zero. Notes are returned in the
//! order the rules are listed here.
//!
//! Known gap: After Effects also shortens the `otln` chunk (panel outline state) when it
//! removes properties. That is not reproduced yet.

use crate::inspect::FileKind;
use crate::riff::{FourCc, Node, RiffError, parse};
use crate::version::{FormatVersion, Head, STAMPS, stamp_for};

const EGG: FourCc = *b"Egg!";
const FAFX: FourCc = *b"FaFX";
const HEAD: FourCc = *b"head";
const TDMN: FourCc = *b"tdmn";
const TDBS: FourCc = *b"tdbs";
const LDTA: FourCc = *b"ldta";
const BLSI: FourCc = *b"blsi";

const SHADOW_COLOR: &str = "ADBE Shadow Color";
const LIGHT_NAMES: [&str; 4] = [
    "ADBE Light Env Atom",
    "ADBE Light Backgd Visible",
    "ADBE Light Backgd Opacity",
    "ADBE Light Backgd Blur",
];

/// What a conversion did or why it cannot be done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteCode {
    ShadowColorRemoved,
    LightEnvironmentRemoved,
    LayerStyleOptionRemoved,
    LayerRecordsShortened,
    /// Blocker: a layer uses a track matte that is not the layer directly above it.
    /// After Effects 22 cannot express that.
    MatteNotAdjacent,
}

impl NoteCode {
    /// Stable identifier for user interfaces and logs:
    /// `shadow-color-removed`, `light-environment-removed`, `layer-style-option-removed`,
    /// `layer-records-shortened`, `matte-not-adjacent`.
    pub fn as_str(self) -> &'static str {
        match self {
            NoteCode::ShadowColorRemoved => "shadow-color-removed",
            NoteCode::LightEnvironmentRemoved => "light-environment-removed",
            NoteCode::LayerStyleOptionRemoved => "layer-style-option-removed",
            NoteCode::LayerRecordsShortened => "layer-records-shortened",
            NoteCode::MatteNotAdjacent => "matte-not-adjacent",
        }
    }

    /// `true` only for [`NoteCode::MatteNotAdjacent`].
    pub fn is_blocker(self) -> bool {
        matches!(self, NoteCode::MatteNotAdjacent)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Note {
    pub code: NoteCode,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConvertError {
    Unreadable(RiffError),
    NotRoundTrip,
    /// The file is a RIFX file of another kind; `Preset` when the form is `FaFX`, else `Unknown`.
    NotAProject(FileKind),
    NoHead,
    UnknownSource {
        format: FormatVersion,
    },
    UnsupportedTarget {
        ae_major: u8,
    },
    NotADowngrade {
        source_format: FormatVersion,
        target_format: FormatVersion,
    },
    Blocked(Vec<Note>),
}

/// A finished conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Converted {
    pub bytes: Vec<u8>,
    pub notes: Vec<Note>,
    pub from: Head,
    pub to: Head,
}

/// Convert `input` for After Effects `target_ae_major` (22, 23, 24 or 25).
/// See the module docs for the exact pipeline. Never panics on any input.
pub fn convert(input: &[u8], target_ae_major: u8) -> Result<Converted, ConvertError> {
    // 1. Parse.
    let mut file = parse(input).map_err(ConvertError::Unreadable)?;

    // 2. Safety net: the parse must round-trip exactly.
    if file.to_bytes() != input {
        return Err(ConvertError::NotRoundTrip);
    }

    // 3. Form and head.
    if file.form != EGG {
        let kind = if file.form == FAFX {
            FileKind::Preset
        } else {
            FileKind::Unknown
        };
        return Err(ConvertError::NotAProject(kind));
    }
    let head_bytes = file.chunk(HEAD).ok_or(ConvertError::NoHead)?;
    let head = Head::parse(head_bytes).ok_or(ConvertError::NoHead)?;

    // 4. Target stamp, then source format, then the downgrade check.
    let stamp = stamp_for(target_ae_major).ok_or(ConvertError::UnsupportedTarget {
        ae_major: target_ae_major,
    })?;
    if head.format.major > 98 {
        return Err(ConvertError::UnknownSource {
            format: head.format,
        });
    }
    if !(stamp.format.major < head.format.major) {
        return Err(ConvertError::NotADowngrade {
            source_format: head.format,
            target_format: stamp.format,
        });
    }

    // 5. Apply the steps for every format major from the source down to the
    //    target's + 1, highest first.
    let from = stamp.format.major + 1;
    let to = head.format.major;
    let mut counts = [0u32; 5]; // S1, M1, L1, E1, B1
    if from <= 95 && 95 <= to {
        for_each_children_mut(&mut file.children, &mut |children| {
            rule_s1(children, &mut counts[0])
        });
    }
    if from <= 94 && 94 <= to {
        for_each_children_mut(&mut file.children, &mut |children| {
            rule_m1(children, &mut counts[1])
        });
        for_each_children_mut(&mut file.children, &mut |children| {
            rule_l1(children, &mut counts[2])
        });
        for_each_children_mut(&mut file.children, &mut |children| {
            rule_e1(children, &mut counts[3])
        });
        for_each_children_mut(&mut file.children, &mut |children| {
            rule_b1(children, &mut counts[4])
        });
    }

    // 6. Assemble notes in the documented order, dropping zero counts.
    let notes = [
        (NoteCode::ShadowColorRemoved, counts[0]),
        (NoteCode::MatteNotAdjacent, counts[1]),
        (NoteCode::LayerRecordsShortened, counts[2]),
        (NoteCode::LightEnvironmentRemoved, counts[3]),
        (NoteCode::LayerStyleOptionRemoved, counts[4]),
    ]
    .into_iter()
    .filter(|(_, count)| *count > 0)
    .map(|(code, count)| Note { code, count })
    .collect::<Vec<Note>>();

    if notes.iter().any(|n| n.code.is_blocker()) {
        return Err(ConvertError::Blocked(notes));
    }

    // 7. Stamp the head, keeping the source platform bits. The head is a
    //    top-level chunk and is at least eight bytes long (Head::parse accepted
    //    it), so `write_into` always succeeds here.
    let new_head = Head {
        format: stamp.format,
        app: stamp.word.with_platform(head.app.platform()),
    };
    for node in file.children.iter_mut() {
        if let Node::Chunk { id, data } = node
            && *id == HEAD
        {
            new_head.write_into(data);
        }
    }

    // 8. Serialize.
    let bytes = file.to_bytes();
    Ok(Converted {
        bytes,
        notes,
        from: head,
        to: new_head,
    })
}

/// After Effects majors a file with this head can be converted to, newest first:
/// every entry of [`crate::version::STAMPS`] whose format major is lower than
/// `head.format.major`. Empty when `head.format.major` is not in 94..=98.
pub fn targets_for(head: &Head) -> Vec<u8> {
    if !(94..=98).contains(&head.format.major) {
        return Vec::new();
    }
    STAMPS
        .iter()
        .filter(|s| s.format.major < head.format.major)
        .map(|s| s.ae_major)
        .collect()
}

/// Visit the `children` `Vec` of every [`Node::List`] at any depth, plus the
/// top-level `children`, with mutable access. `f` is called on each `children`
/// `Vec` (including the one passed in) and then the walk recurses into every
/// `List` child of that `Vec`.
fn for_each_children_mut(nodes: &mut Vec<Node>, f: &mut dyn FnMut(&mut Vec<Node>)) {
    f(nodes);
    for node in nodes.iter_mut() {
        if let Node::List { children, .. } = node {
            for_each_children_mut(children, f);
        }
    }
}

/// The bytes of a `tdmn` chunk's data up to the first NUL byte, as UTF-8 text.
/// Returns `""` when the prefix is not valid UTF-8.
fn match_name_is(data: &[u8], name: &str) -> bool {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    &data[..end] == name.as_bytes()
}

/// `true` when the node is a list (either variant) of `kind`.
fn is_list_kind(node: &Node, kind: FourCc) -> bool {
    node.kind() == Some(kind)
}

/// A `tdmn` chunk whose match name is `ADBE Shadow Color`.
fn is_shadow_tdmn(node: &Node) -> bool {
    matches!(node, Node::Chunk { id, data } if *id == TDMN && match_name_is(data, SHADOW_COLOR))
}

/// S1: remove a `tdmn` "ADBE Shadow Color" immediately followed by a `tdbs` list.
fn rule_s1(children: &mut Vec<Node>, count: &mut u32) {
    let len = children.len();
    let mut out: Vec<Node> = Vec::with_capacity(len);
    let mut i = 0;
    while i < len {
        if is_shadow_tdmn(&children[i]) && i + 1 < len && is_list_kind(&children[i + 1], TDBS) {
            *count += 1;
            i += 2;
        } else {
            out.push(children[i].clone());
            i += 1;
        }
    }
    *children = out;
}

/// If `node` is a layer, its `(id, matte_id)`; otherwise `None`.
/// A layer is a `Node::List` with a direct `ldta` child of exactly 164 bytes.
fn layer_ids(node: &Node) -> Option<(u32, u32)> {
    let Node::List { children, .. } = node else {
        return None;
    };
    for child in children {
        let Node::Chunk { id, data } = child else {
            continue;
        };
        if *id == LDTA && data.len() == 164 {
            let id = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            let matte = u32::from_be_bytes([data[160], data[161], data[162], data[163]]);
            return Some((id, matte));
        }
    }
    None
}

/// M1: flag layers whose non-zero matte id does not refer to the previous layer.
fn rule_m1(children: &mut [Node], count: &mut u32) {
    let mut prev_id: Option<u32> = None;
    for node in children.iter() {
        if let Some((id, matte)) = layer_ids(node) {
            if matte != 0 && prev_id != Some(matte) {
                *count += 1;
            }
            prev_id = Some(id);
        }
    }
}

/// L1: shorten every 164-byte `ldta` to its first 160 bytes.
fn rule_l1(children: &mut [Node], count: &mut u32) {
    for node in children.iter_mut() {
        if let Node::Chunk { id, data } = node
            && *id == LDTA
            && data.len() == 164
        {
            data.truncate(160);
            *count += 1;
        }
    }
}

/// E1: remove a `tdmn` with a light-environment match name immediately followed
/// by any list.
fn rule_e1(children: &mut Vec<Node>, count: &mut u32) {
    let len = children.len();
    let mut out: Vec<Node> = Vec::with_capacity(len);
    let mut i = 0;
    while i < len {
        let is_light = match &children[i] {
            Node::Chunk { id, data } => {
                *id == TDMN && LIGHT_NAMES.iter().any(|name| match_name_is(data, name))
            }
            _ => false,
        };
        if is_light && i + 1 < len && children[i + 1].kind().is_some() {
            *count += 1;
            i += 2;
        } else {
            out.push(children[i].clone());
            i += 1;
        }
    }
    *children = out;
}

/// B1: remove a `tdbs` list whose previous sibling is a `blsi` chunk.
fn rule_b1(children: &mut Vec<Node>, count: &mut u32) {
    let len = children.len();
    let mut out: Vec<Node> = Vec::with_capacity(len);
    let mut i = 0;
    while i < len {
        let prev_blsi = i > 0 && matches!(&children[i - 1], Node::Chunk { id, .. } if *id == BLSI);
        if is_list_kind(&children[i], TDBS) && prev_blsi {
            *count += 1;
            i += 1;
        } else {
            out.push(children[i].clone());
            i += 1;
        }
    }
    *children = out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::riff::RiffFile;
    use crate::version::{AppVersion, Platform};

    /// Build a minimal `Egg!` RIFX file with a 20-byte `head` chunk for the
    /// given format and word, plus the given top-level extra nodes.
    fn project_bytes(format: FormatVersion, word: u32, extra_top: Vec<Node>) -> Vec<u8> {
        let mut head_data = vec![0u8; 20];
        Head {
            format,
            app: AppVersion(word),
        }
        .write_into(&mut head_data);
        let mut children = vec![Node::Chunk {
            id: HEAD,
            data: head_data,
        }];
        children.extend(extra_top);
        let file = RiffFile {
            form: EGG,
            children,
            trailer: Vec::new(),
        };
        file.to_bytes()
    }

    fn head_of(format: FormatVersion, word: u32) -> Head {
        Head {
            format,
            app: AppVersion(word),
        }
    }

    const WIN_WORD_24: u32 = 0x0F02_8634; // AE 24, Windows

    fn tdmn(name: &str) -> Node {
        let mut data = name.as_bytes().to_vec();
        data.push(0);
        Node::Chunk { id: TDMN, data }
    }

    fn tdbs_list(children: Vec<Node>) -> Node {
        Node::List {
            kind: TDBS,
            children,
        }
    }

    fn empty_chunk(id: FourCc, len: usize) -> Node {
        Node::Chunk {
            id,
            data: vec![0u8; len],
        }
    }

    // a) S1 -------------------------------------------------------------

    #[test]
    fn s1_removes_pairs_and_keeps_singles() {
        // Two shadow pairs in one list, back to back, plus a shadow tdmn not
        // followed by a tdbs list, plus a different match name.
        let list = Node::List {
            kind: *b"tdgp",
            children: vec![
                tdmn(SHADOW_COLOR),
                tdbs_list(Vec::new()),
                tdmn(SHADOW_COLOR),
                tdbs_list(Vec::new()),
                tdmn(SHADOW_COLOR), // no following list → stays
                tdmn("Other Match Name"),
                tdbs_list(Vec::new()), // different name → stays
            ],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![list],
        );
        let result = convert(&bytes, 23).expect("converts");
        assert_eq!(
            result.notes,
            vec![Note {
                code: NoteCode::ShadowColorRemoved,
                count: 2,
            }]
        );
        let parsed = parse(&result.bytes).expect("reparse");
        // The remaining tdgp list has: shadow tdmn (kept, no list after),
        // "Other" tdmn, tdbs list.
        let tdgp = parsed
            .children
            .iter()
            .find_map(|n| match n {
                Node::List { kind, children } if *kind == *b"tdgp" => Some(children),
                _ => None,
            })
            .expect("tdgp list present");
        assert_eq!(tdgp.len(), 3);
    }

    // b) L1 -------------------------------------------------------------

    fn layer_with_ldta(id: u32, matte: u32, len: usize) -> Node {
        let mut data = vec![0u8; len];
        data[0..4].copy_from_slice(&id.to_be_bytes());
        if len >= 164 {
            data[160..164].copy_from_slice(&matte.to_be_bytes());
        }
        Node::List {
            kind: *b"Layr",
            children: vec![Node::Chunk { id: LDTA, data }],
        }
    }

    #[test]
    fn l1_shortens_only_164() {
        let list = Node::List {
            kind: *b"Item",
            children: vec![
                layer_with_ldta(1, 0, 164),
                layer_with_ldta(2, 0, 160),
                layer_with_ldta(3, 0, 100),
            ],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![list],
        );
        let result = convert(&bytes, 22).expect("converts");
        assert_eq!(
            result.notes,
            vec![Note {
                code: NoteCode::LayerRecordsShortened,
                count: 1,
            }]
        );
        let parsed = parse(&result.bytes).expect("reparse");
        let item = parsed
            .children
            .iter()
            .find_map(|n| match n {
                Node::List { kind, children } if *kind == *b"Item" => Some(children),
                _ => None,
            })
            .expect("Item list present");
        let lens: Vec<usize> = item
            .iter()
            .filter_map(|n| match n {
                Node::List { children, .. } => children.iter().find_map(|c| match c {
                    Node::Chunk { id, data } if *id == LDTA => Some(data.len()),
                    _ => None,
                }),
                _ => None,
            })
            .collect();
        assert_eq!(lens, vec![160, 160, 100]);
    }

    // c) M1 -------------------------------------------------------------

    #[test]
    fn m1_adjacency_rules() {
        let list = Node::List {
            kind: *b"Item",
            children: vec![
                layer_with_ldta(1, 0, 164), // matte 0 → fine
                layer_with_ldta(2, 1, 164), // matte == previous id → fine
                Node::Chunk {
                    id: *b"othr",
                    data: vec![0; 8],
                }, // non-layer node between layers
                layer_with_ldta(3, 1, 164), // matte 1, previous layer is id 2 → blocked
                layer_with_ldta(4, 2, 164), // matte 2, previous layer is id 3 → blocked
            ],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![list],
        );
        let result = convert(&bytes, 22);
        match result {
            Err(ConvertError::Blocked(notes)) => {
                let matte = notes
                    .iter()
                    .find(|n| n.code == NoteCode::MatteNotAdjacent)
                    .expect("matte note present");
                assert_eq!(matte.count, 2);
            }
            other => panic!("expected Blocked, got {other:?}"),
        }
    }

    #[test]
    fn m1_first_layer_nonzero_matte_blocked() {
        let list = Node::List {
            kind: *b"Item",
            children: vec![layer_with_ldta(5, 9, 164)],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![list],
        );
        let result = convert(&bytes, 22);
        match result {
            Err(ConvertError::Blocked(notes)) => {
                let matte = notes
                    .iter()
                    .find(|n| n.code == NoteCode::MatteNotAdjacent)
                    .expect("matte note present");
                assert_eq!(matte.count, 1);
            }
            other => panic!("expected Blocked, got {other:?}"),
        }
    }

    // d) E1 -------------------------------------------------------------

    #[test]
    fn e1_removes_light_pairs() {
        let list = Node::List {
            kind: *b"Item",
            children: vec![
                tdmn("ADBE Light Env Atom"),
                Node::List {
                    kind: *b"tdgp",
                    children: Vec::new(),
                },
                tdmn("ADBE Light Backgd Visible"),
                Node::OpaqueList {
                    kind: *b"tdgp",
                    data: vec![0; 8],
                },
                tdmn("ADBE Light Backgd Opacity"), // no following list → stays
                tdmn("ADBE Light Backgd Blur"),
                Node::List {
                    kind: *b"tdbs",
                    children: Vec::new(),
                },
            ],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![list],
        );
        let result = convert(&bytes, 22).expect("converts");
        let light = result
            .notes
            .iter()
            .find(|n| n.code == NoteCode::LightEnvironmentRemoved)
            .expect("light note present");
        assert_eq!(light.count, 3);
        let parsed = parse(&result.bytes).expect("reparse");
        let item = parsed
            .children
            .iter()
            .find_map(|n| match n {
                Node::List { kind, children } if *kind == *b"Item" => Some(children),
                _ => None,
            })
            .expect("Item list present");
        // Only the "Opacity" tdmn (no list after) should remain.
        assert_eq!(item.len(), 1);
        assert!(matches!(&item[0], Node::Chunk { id, .. } if *id == TDMN));
    }

    // e) B1 -------------------------------------------------------------

    #[test]
    fn b1_removes_tdbs_after_blsi() {
        let list = Node::List {
            kind: *b"Item",
            children: vec![
                empty_chunk(BLSI, 16),
                Node::List {
                    kind: TDBS,
                    children: Vec::new(),
                }, // removed
                empty_chunk(*b"blsv", 16),
                Node::List {
                    kind: TDBS,
                    children: Vec::new(),
                }, // stays
            ],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![list],
        );
        let result = convert(&bytes, 22).expect("converts");
        let style = result
            .notes
            .iter()
            .find(|n| n.code == NoteCode::LayerStyleOptionRemoved)
            .expect("style note present");
        assert_eq!(style.count, 1);
        let parsed = parse(&result.bytes).expect("reparse");
        let item = parsed
            .children
            .iter()
            .find_map(|n| match n {
                Node::List { kind, children } if *kind == *b"Item" => Some(children),
                _ => None,
            })
            .expect("Item list present");
        assert_eq!(item.len(), 3);
    }

    // f) pipeline -------------------------------------------------------

    #[test]
    fn pipeline_95_to_23_s1_only_and_stamp() {
        let list = Node::List {
            kind: *b"tdgp",
            children: vec![tdmn(SHADOW_COLOR), tdbs_list(Vec::new())],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![list],
        );
        let result = convert(&bytes, 23).expect("converts");
        assert_eq!(
            result.notes,
            vec![Note {
                code: NoteCode::ShadowColorRemoved,
                count: 1,
            }]
        );
        assert_eq!(
            result.to.format,
            FormatVersion {
                major: 94,
                minor: 9
            }
        );
        assert_eq!(result.to.app.0, 0x0B3B_0637);
        assert_eq!(result.to.app.platform(), Platform::Windows);
    }

    #[test]
    fn pipeline_95_to_23_macos_keeps_platform() {
        let mac_word = 0x0F88_0614; // platform code 14 (MacArm)
        let list = Node::List {
            kind: *b"tdgp",
            children: vec![tdmn(SHADOW_COLOR), tdbs_list(Vec::new())],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            mac_word,
            vec![list],
        );
        let result = convert(&bytes, 23).expect("converts");
        // Same version numbers as the Windows case, but platform code 14.
        assert_eq!(
            result.to.format,
            FormatVersion {
                major: 94,
                minor: 9
            }
        );
        assert_eq!(result.to.app.platform(), Platform::MacArm);
        // 0x0B3B0637 with platform bits set to 14:
        let expected = 0x0B3B_0637 & !(0x0F << 22) | (14 << 22);
        assert_eq!(result.to.app.0, expected);
    }

    #[test]
    fn pipeline_95_to_22_applies_all() {
        let list = Node::List {
            kind: *b"Item",
            children: vec![
                tdmn(SHADOW_COLOR),
                tdbs_list(Vec::new()),
                layer_with_ldta(1, 0, 164),
                tdmn("ADBE Light Env Atom"),
                Node::List {
                    kind: *b"tdgp",
                    children: Vec::new(),
                },
                empty_chunk(BLSI, 16),
                Node::List {
                    kind: TDBS,
                    children: Vec::new(),
                },
            ],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![list],
        );
        let result = convert(&bytes, 22).expect("converts");
        let codes: Vec<NoteCode> = result.notes.iter().map(|n| n.code).collect();
        assert_eq!(
            codes,
            vec![
                NoteCode::ShadowColorRemoved,
                NoteCode::LayerRecordsShortened,
                NoteCode::LightEnvironmentRemoved,
                NoteCode::LayerStyleOptionRemoved,
            ]
        );
        assert_eq!(
            result.to.format,
            FormatVersion {
                major: 93,
                minor: 43
            }
        );
        assert_eq!(result.to.app.0, 0x0B33_063B);
    }

    #[test]
    fn pipeline_97_to_24_only_head_changes() {
        let bytes = project_bytes(
            FormatVersion {
                major: 97,
                minor: 0,
            },
            0x0F10_0643,
            vec![],
        );
        let result = convert(&bytes, 24).expect("converts");
        assert!(result.notes.is_empty(), "no rules apply");
        assert_eq!(
            result.to.format,
            FormatVersion {
                major: 95,
                minor: 6
            }
        );
        // The only change is the head: 8 bytes at the head chunk.
        let parsed = parse(&result.bytes).expect("reparse");
        let head = parsed.chunk(HEAD).expect("head");
        assert_eq!(&head[0..2], &95u16.to_be_bytes());
        assert_eq!(&head[2..4], &6u16.to_be_bytes());
    }

    // g) errors ---------------------------------------------------------

    #[test]
    fn error_garbage_is_unreadable() {
        let bytes = b"not a riff file at all";
        match convert(bytes, 23) {
            Err(ConvertError::Unreadable(_)) => {}
            other => panic!("expected Unreadable, got {other:?}"),
        }
    }

    #[test]
    fn error_fafx_is_preset() {
        let mut file = RiffFile {
            form: FAFX,
            children: vec![],
            trailer: Vec::new(),
        };
        let _ = &mut file;
        let bytes = file.to_bytes();
        match convert(&bytes, 23) {
            Err(ConvertError::NotAProject(FileKind::Preset)) => {}
            other => panic!("expected NotAProject(Preset), got {other:?}"),
        }
    }

    #[test]
    fn error_no_head() {
        let file = RiffFile {
            form: EGG,
            children: vec![],
            trailer: Vec::new(),
        };
        let bytes = file.to_bytes();
        match convert(&bytes, 23) {
            Err(ConvertError::NoHead) => {}
            other => panic!("expected NoHead, got {other:?}"),
        }
    }

    #[test]
    fn error_unknown_source_and_not_downgrade() {
        // A format-93 source is not unknown; it is simply already old, so the
        // lowest target (22, format 93) is not a downgrade.
        let bytes = project_bytes(
            FormatVersion {
                major: 93,
                minor: 43,
            },
            0x0B33_063B,
            vec![],
        );
        match convert(&bytes, 22) {
            Err(ConvertError::NotADowngrade {
                source_format,
                target_format,
            }) => {
                assert_eq!(
                    source_format,
                    FormatVersion {
                        major: 93,
                        minor: 43
                    }
                );
                assert_eq!(
                    target_format,
                    FormatVersion {
                        major: 93,
                        minor: 43
                    }
                );
            }
            other => panic!("expected NotADowngrade, got {other:?}"),
        }
        // A format-99 source is unknown.
        let bytes = project_bytes(
            FormatVersion {
                major: 99,
                minor: 0,
            },
            0x0F10_0643,
            vec![],
        );
        match convert(&bytes, 24) {
            Err(ConvertError::UnknownSource { format }) => {
                assert_eq!(
                    format,
                    FormatVersion {
                        major: 99,
                        minor: 0
                    }
                );
            }
            other => panic!("expected UnknownSource, got {other:?}"),
        }
    }

    #[test]
    fn error_unsupported_target() {
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![],
        );
        for target in [26u8, 21] {
            match convert(&bytes, target) {
                Err(ConvertError::UnsupportedTarget { ae_major }) => {
                    assert_eq!(ae_major, target);
                }
                other => panic!("expected UnsupportedTarget({target}), got {other:?}"),
            }
        }
    }

    #[test]
    fn error_not_a_downgrade() {
        // Source 95, target 24 (format 95.6) → not lower.
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![],
        );
        match convert(&bytes, 24) {
            Err(ConvertError::NotADowngrade { .. }) => {}
            other => panic!("expected NotADowngrade, got {other:?}"),
        }
        // Source 94, target 23 (format 94.9) → not lower.
        let bytes = project_bytes(
            FormatVersion {
                major: 94,
                minor: 9,
            },
            0x0B3B_0637,
            vec![],
        );
        match convert(&bytes, 23) {
            Err(ConvertError::NotADowngrade { .. }) => {}
            other => panic!("expected NotADowngrade, got {other:?}"),
        }
    }

    #[test]
    fn error_blocked_returns_all_notes_and_no_output() {
        let list = Node::List {
            kind: *b"Item",
            children: vec![
                tdmn(SHADOW_COLOR),
                tdbs_list(Vec::new()),
                layer_with_ldta(1, 7, 164), // blocked
                empty_chunk(BLSI, 16),
                Node::List {
                    kind: TDBS,
                    children: Vec::new(),
                },
            ],
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![list],
        );
        match convert(&bytes, 22) {
            Err(ConvertError::Blocked(notes)) => {
                let codes: Vec<NoteCode> = notes.iter().map(|n| n.code).collect();
                assert_eq!(
                    codes,
                    vec![
                        NoteCode::ShadowColorRemoved,
                        NoteCode::MatteNotAdjacent,
                        NoteCode::LayerRecordsShortened,
                        NoteCode::LayerStyleOptionRemoved,
                    ]
                );
            }
            other => panic!("expected Blocked, got {other:?}"),
        }
    }

    // h) targets_for ----------------------------------------------------

    #[test]
    fn targets_for_each_source() {
        let cases = [
            (97u16, vec![25u8, 24, 23, 22]),
            (95, vec![23, 22]),
            (94, vec![22]),
            (93, Vec::<u8>::new()),
            (99, Vec::<u8>::new()),
        ];
        for (major, expected) in cases {
            let head = head_of(FormatVersion { major, minor: 0 }, 0);
            assert_eq!(targets_for(&head), expected, "for major {major}");
        }
    }

    // i) head tail and svap preserved -----------------------------------

    #[test]
    fn head_tail_and_svap_unchanged() {
        let mut head_data = vec![0u8; 20];
        Head {
            format: FormatVersion {
                major: 95,
                minor: 5,
            },
            app: AppVersion(WIN_WORD_24),
        }
        .write_into(&mut head_data);
        let original_tail = head_data[8..].to_vec();
        let svap_data = vec![0xAB, 0xCD, 0xEF, 0x12];
        let svap = Node::Chunk {
            id: *b"svap",
            data: svap_data.clone(),
        };
        let bytes = project_bytes(
            FormatVersion {
                major: 95,
                minor: 5,
            },
            WIN_WORD_24,
            vec![svap],
        );
        let result = convert(&bytes, 23).expect("converts");
        let parsed = parse(&result.bytes).expect("reparse");
        let head = parsed.chunk(HEAD).expect("head");
        assert_eq!(&head[8..], &original_tail, "head bytes beyond 8 unchanged");
        let svap_out = parsed.chunk(*b"svap").expect("svap");
        assert_eq!(svap_out, svap_data.as_slice(), "svap unchanged");
    }
}
