//! Structural comparison of two parsed files.
//!
//! Used to learn what After Effects changes when it saves a copy for an older
//! version, and to check our own output against a file written by After Effects.
//!
//! How two child lists are compared:
//!
//! 1. Every node has a key: its 4-byte id for an ordinary chunk, or
//!    `LIST:<kind>` for both list variants.
//! 2. The two child lists are aligned by the longest common subsequence of
//!    their keys (standard dynamic-programming LCS; on ties prefer matching
//!    earlier nodes).
//! 3. A node of the first file with no partner is [`Change::Removed`].
//!    A node of the second file with no partner is [`Change::Added`].
//!    Removed/added lists are reported once, not once per descendant.
//! 4. Aligned pair, two ordinary chunks: equal data → nothing; otherwise
//!    [`Change::Changed`].
//! 5. Aligned pair, two [`Node::List`]: compare their children recursively.
//! 6. Aligned pair, any other combination of list variants: compare the
//!    serialized data (kind excluded); if different → [`Change::Changed`].
//!
//! File-level differences are reported as `Changed` with an empty `path` and
//! `index_path`, and the ids `(form)` and `(trailer)`.
//!
//! Changes are returned in file order (depth first, in the order of the first
//! file; an `Added` node comes where it sits in the second file relative to
//! the aligned nodes around it).

use crate::riff::{FourCc, Node, RiffFile};

/// One difference between two files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Removed(Lone),
    Added(Lone),
    Changed(Pair),
}

/// A node present in only one of the files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lone {
    /// Kinds of the enclosing lists, outermost first, joined by `/`.
    /// Example: `Fold/Item/Layr/tdgp`. Empty for top-level nodes.
    pub path: String,
    /// Index of each enclosing list in its parent's children, then the index
    /// of the node itself. In the first file for `Removed`, in the second for `Added`.
    pub index_path: Vec<usize>,
    /// The node's key: `tdmn`, `LIST:tdbs`, ...
    pub key: String,
    /// The node's data length (see `Node::data_len`).
    pub len: u64,
    /// Ordinary chunk: the first 40 data bytes, with every byte outside
    /// 0x20..=0x7e shown as `.`, and trailing `.` characters trimmed.
    /// List: `(<n> nodes)`, counting all descendants.
    pub preview: String,
}

/// Two aligned nodes whose content differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair {
    pub path: String,
    /// Index path in the first file.
    pub index_path: Vec<usize>,
    pub key: String,
    pub len_a: u64,
    pub len_b: u64,
    /// Offset of the first differing byte (the shorter length if one is a prefix of the other).
    pub first_diff: u64,
    /// Number of differing positions when `len_a == len_b`, otherwise 0.
    pub diff_count: u64,
    /// Up to 8 `(offset, byte_a, byte_b)` triples, in order, when `len_a == len_b`. Otherwise empty.
    pub samples: Vec<(u64, u8, u8)>,
}

/// Printable form of a four-character code: bytes outside 0x20..=0x7e become `?`.
pub fn fourcc_text(code: FourCc) -> String {
    code.iter()
        .map(|&c| {
            if (0x20..=0x7e).contains(&c) {
                c as char
            } else {
                '?'
            }
        })
        .collect()
}

/// The key of a node as described in the module docs.
pub fn node_key(node: &Node) -> String {
    match node {
        Node::Chunk { id, .. } => fourcc_text(*id),
        Node::List { kind, .. } | Node::OpaqueList { kind, .. } => {
            format!("LIST:{}", fourcc_text(*kind))
        }
    }
}

/// Compare two files. See the module docs for the rules.
pub fn diff(a: &RiffFile, b: &RiffFile) -> Vec<Change> {
    let mut out: Vec<Change> = Vec::new();

    // File-level: form type (4 bytes at the front of the body).
    if a.form != b.form {
        let (first_diff, diff_count, samples) = compare_data(&a.form, &b.form);
        out.push(Change::Changed(Pair {
            path: String::new(),
            index_path: Vec::new(),
            key: "(form)".to_string(),
            len_a: 4,
            len_b: 4,
            first_diff,
            diff_count,
            samples,
        }));
    }

    // Top-level children.
    diff_lists("", &Vec::new(), &a.children, &b.children, &mut out);

    // File-level: trailer (bytes after the body).
    if a.trailer != b.trailer {
        let (first_diff, diff_count, samples) = compare_data(&a.trailer, &b.trailer);
        out.push(Change::Changed(Pair {
            path: String::new(),
            index_path: Vec::new(),
            key: "(trailer)".to_string(),
            len_a: a.trailer.len() as u64,
            len_b: b.trailer.len() as u64,
            first_diff,
            diff_count,
            samples,
        }));
    }

    out
}

/// Group changes for a compact report. Two changes fall in the same group
/// when they have the same variant, `path` and `key`, and additionally:
/// `Removed`/`Added` — the same `len` and `preview`; `Changed` — the same
/// `len_a` and `len_b`. Groups keep the order of first appearance.
/// Each returned line has one of these forms (count is the group size):
///
/// ```text
/// removed  <path>  <key> <len> "<preview>"  x<count>
/// added    <path>  <key> <len> "<preview>"  x<count>
/// changed  <path>  <key> <len_a>-><len_b>  x<count>
/// ```
///
/// An empty path is printed as `/`.
pub fn summarize(changes: &[Change]) -> Vec<String> {
    // A group is identified by (tag, path, key, rest). `rest` is the variant-
    // specific middle field: `<len> "<preview>"` for Removed/Added and
    // `<len_a>-><len_b>` for Changed.
    struct Group {
        tag: &'static str,
        path: String,
        key: String,
        rest: String,
        line: String,
        count: usize,
    }

    let mut groups: Vec<Group> = Vec::new();

    for change in changes {
        let (tag, path, key, rest) = match change {
            Change::Removed(l) => (
                "removed",
                l.path.clone(),
                l.key.clone(),
                format!("{} \"{}\"", l.len, l.preview),
            ),
            Change::Added(l) => (
                "added",
                l.path.clone(),
                l.key.clone(),
                format!("{} \"{}\"", l.len, l.preview),
            ),
            Change::Changed(p) => (
                "changed",
                p.path.clone(),
                p.key.clone(),
                format!("{}->{}", p.len_a, p.len_b),
            ),
        };

        if let Some(g) = groups
            .iter_mut()
            .find(|g| g.tag == tag && g.path == path && g.key == key && g.rest == rest)
        {
            g.count += 1;
            continue;
        }

        let path_text = if path.is_empty() {
            "/".to_string()
        } else {
            path.clone()
        };
        let line = format!("{tag}  {path_text}  {key} {rest}");
        groups.push(Group {
            tag,
            path,
            key,
            rest,
            line,
            count: 1,
        });
    }

    groups
        .into_iter()
        .map(|g| format!("{}  x{}", g.line, g.count))
        .collect()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Compare two byte slices the way [`Pair`] wants: the first differing
/// offset (the shorter length when one is a prefix of the other), the count
/// of differing positions (only when the lengths are equal), and up to 8
/// `(offset, byte_a, byte_b)` samples (only when the lengths are equal).
fn compare_data(a: &[u8], b: &[u8]) -> (u64, u64, Vec<(u64, u8, u8)>) {
    let min_len = a.len().min(b.len());
    let same_len = a.len() == b.len();

    let mut first: Option<usize> = None;
    for i in 0..min_len {
        if a[i] != b[i] {
            first = Some(i);
            break;
        }
    }

    match first {
        Some(f) => {
            let first_diff = f as u64;
            if same_len {
                let mut diff_count = 0u64;
                let mut samples: Vec<(u64, u8, u8)> = Vec::new();
                for i in 0..a.len() {
                    if a[i] != b[i] {
                        diff_count += 1;
                        if samples.len() < 8 {
                            samples.push((i as u64, a[i], b[i]));
                        }
                    }
                }
                (first_diff, diff_count, samples)
            } else {
                (first_diff, 0, Vec::new())
            }
        }
        // Common prefix identical; the difference is only the length.
        None => (min_len as u64, 0, Vec::new()),
    }
}

/// The preview text for a lone node (see [`Lone::preview`]).
fn node_preview(node: &Node) -> String {
    match node {
        Node::Chunk { data, .. } => {
            let mut s: String = data
                .iter()
                .take(40)
                .map(|&c| {
                    if (0x20..=0x7e).contains(&c) {
                        c as char
                    } else {
                        '.'
                    }
                })
                .collect();
            while s.ends_with('.') {
                s.pop();
            }
            s
        }
        Node::List { .. } | Node::OpaqueList { .. } => {
            let n = descendant_count(node);
            format!("({n} nodes)")
        }
    }
}

/// Number of nodes strictly below `node`.
fn descendant_count(node: &Node) -> u64 {
    match node {
        Node::Chunk { .. } => 0,
        Node::List { children, .. } => children.iter().map(|c| 1 + descendant_count(c)).sum(),
        Node::OpaqueList { .. } => 0,
    }
}

/// Build a [`Lone`] for a node that has no partner.
fn make_lone(path: &str, index_prefix: &[usize], own_index: usize, node: &Node) -> Lone {
    let mut index_path: Vec<usize> = index_prefix.to_vec();
    index_path.push(own_index);
    Lone {
        path: path.to_string(),
        index_path,
        key: node_key(node),
        len: node.data_len(),
        preview: node_preview(node),
    }
}

/// The "serialized data (kind excluded)" of a list-variant node.
/// For [`Node::List`] this re-encodes the children; for [`Node::OpaqueList`]
/// it is the stored `data`.
fn list_content(node: &Node) -> Vec<u8> {
    match node {
        Node::List { children, .. } => {
            let mut out: Vec<u8> = Vec::new();
            for child in children {
                encode_child(child, &mut out);
            }
            out
        }
        Node::OpaqueList { data, .. } => data.clone(),
        Node::Chunk { data, .. } => data.clone(),
    }
}

/// Encode one child node (id, size, data, pad) into `out`, exactly as the
/// container format stores it.
fn encode_child(node: &Node, out: &mut Vec<u8>) {
    match node {
        Node::Chunk { id, data } => {
            out.extend_from_slice(id);
            push_size(out, data.len() as u64);
            out.extend_from_slice(data);
            if data.len() % 2 == 1 {
                out.push(0);
            }
        }
        Node::List { kind, children } => {
            out.extend_from_slice(b"LIST");
            let size_at = out.len();
            out.extend_from_slice(&[0, 0, 0, 0]);
            let start = out.len();
            out.extend_from_slice(kind);
            for child in children {
                encode_child(child, out);
            }
            let size: u64 = out.len() as u64 - start as u64;
            out[size_at..size_at + 4]
                .copy_from_slice(&(size.min(u32::MAX as u64) as u32).to_be_bytes());
            if size % 2 == 1 {
                out.push(0);
            }
        }
        Node::OpaqueList { kind, data } => {
            out.extend_from_slice(b"LIST");
            let size_at = out.len();
            out.extend_from_slice(&[0, 0, 0, 0]);
            let start = out.len();
            out.extend_from_slice(kind);
            out.extend_from_slice(data);
            let size: u64 = out.len() as u64 - start as u64;
            out[size_at..size_at + 4]
                .copy_from_slice(&(size.min(u32::MAX as u64) as u32).to_be_bytes());
            if size % 2 == 1 {
                out.push(0);
            }
        }
    }
}

fn push_size(out: &mut Vec<u8>, size: u64) {
    out.extend_from_slice(&(size.min(u32::MAX as u64) as u32).to_be_bytes());
}

/// True when `node` is a parsed [`Node::List`] (as opposed to opaque).
fn is_list(node: &Node) -> bool {
    matches!(node, Node::List { .. })
}

/// The data to compare for an aligned pair that is not two parsed lists:
/// the chunk data for Chunk+Chunk, otherwise the serialized list content.
fn pair_data(a: &Node, b: &Node) -> (Vec<u8>, Vec<u8>) {
    match (a, b) {
        (Node::Chunk { data: x, .. }, Node::Chunk { data: y, .. }) => (x.clone(), y.clone()),
        _ => (list_content(a), list_content(b)),
    }
}

/// Compare two child lists and append the resulting changes to `out`.
///
/// `path` holds the kinds of the enclosing lists, outermost first (empty at
/// the top level). `index_prefix` holds the index of each enclosing list in
/// its parent's children, outermost first.
fn diff_lists(path: &str, index_prefix: &[usize], a: &[Node], b: &[Node], out: &mut Vec<Change>) {
    let n = a.len();
    let m = b.len();

    let keys_a: Vec<String> = a.iter().map(node_key).collect();
    let keys_b: Vec<String> = b.iter().map(node_key).collect();

    // LCS table, allocated once for this pair of lists: (n+1) x (m+1) flat.
    // lcs[i][j] = length of the LCS of keys_a[..i] and keys_b[..j].
    let width = m + 1;
    let mut table = vec![0u32; (n + 1) * width];
    for i in 1..=n {
        let row = i * width;
        let prev_row = (i - 1) * width;
        for j in 1..=m {
            if keys_a[i - 1] == keys_b[j - 1] {
                table[row + j] = table[prev_row + j - 1] + 1;
            } else {
                table[row + j] = table[prev_row + j].max(table[row + j - 1]);
            }
        }
    }

    // Backtrack from the bottom-right to recover the aligned (i, j) pairs,
    // 0-based, then reverse them into increasing order. On a key match the
    // pair belongs to the LCS. On a mismatch, prefer moving up (drop the
    // later `a` node) when that is no worse, so earlier `a` nodes stay
    // available to match — "on ties prefer matching earlier nodes".
    let mut matches: Vec<(usize, usize)> = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 && j > 0 {
        if keys_a[i - 1] == keys_b[j - 1] {
            matches.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if table[(i - 1) * width + j] >= table[i * width + (j - 1)] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    matches.reverse();

    // Walk the alignment left to right.
    let mut ai = 0usize;
    let mut bi = 0usize;
    for (mi, mj) in matches {
        while ai < mi {
            out.push(Change::Removed(make_lone(path, index_prefix, ai, &a[ai])));
            ai += 1;
        }
        while bi < mj {
            out.push(Change::Added(make_lone(path, index_prefix, bi, &b[bi])));
            bi += 1;
        }
        ai = mi + 1;
        bi = mj + 1;

        let anode = &a[mi];
        let bnode = &b[mj];
        if is_list(anode) && is_list(bnode) {
            // Rule 5: recurse into both lists.
            if let (Node::List { kind, children: ac }, Node::List { children: bc, .. }) =
                (anode, bnode)
            {
                let child_path = if path.is_empty() {
                    fourcc_text(*kind)
                } else {
                    format!("{path}/{}", fourcc_text(*kind))
                };
                let mut child_prefix: Vec<usize> = index_prefix.to_vec();
                child_prefix.push(mi);
                diff_lists(&child_path, &child_prefix, ac, bc, out);
            }
        } else {
            // Rule 4 (two ordinary chunks) or Rule 6 (a mix of list variants):
            // compare the content directly from the node fields.
            let (da, db) = pair_data(anode, bnode);
            if da != db {
                let (first_diff, diff_count, samples) = compare_data(&da, &db);
                emit_changed(
                    &mut DiffCtx {
                        path,
                        index_prefix,
                        out,
                    },
                    mi,
                    anode,
                    bnode,
                    first_diff,
                    diff_count,
                    samples,
                );
            }
        }
    }

    while ai < n {
        out.push(Change::Removed(make_lone(path, index_prefix, ai, &a[ai])));
        ai += 1;
    }
    while bi < m {
        out.push(Change::Added(make_lone(path, index_prefix, bi, &b[bi])));
        bi += 1;
    }
}

/// The context shared by the recursive list walk and [`emit_changed`]:
/// the enclosing path, the enclosing index path and the output being built.
struct DiffCtx<'a> {
    path: &'a str,
    index_prefix: &'a [usize],
    out: &'a mut Vec<Change>,
}

/// Push a [`Change::Changed`] for an aligned pair whose content differs.
fn emit_changed(
    ctx: &mut DiffCtx<'_>,
    own_index: usize,
    anode: &Node,
    bnode: &Node,
    first_diff: u64,
    diff_count: u64,
    samples: Vec<(u64, u8, u8)>,
) {
    let mut index_path: Vec<usize> = ctx.index_prefix.to_vec();
    index_path.push(own_index);
    ctx.out.push(Change::Changed(Pair {
        path: ctx.path.to_string(),
        index_path,
        key: node_key(anode),
        len_a: anode.data_len(),
        len_b: bnode.data_len(),
        first_diff,
        diff_count,
        samples,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::riff::{Node, RiffFile};

    fn chunk(id: &str, data: &[u8]) -> Node {
        Node::Chunk {
            id: id.as_bytes().try_into().unwrap(),
            data: data.to_vec(),
        }
    }

    fn list(kind: &str, children: Vec<Node>) -> Node {
        Node::List {
            kind: kind.as_bytes().try_into().unwrap(),
            children,
        }
    }

    fn file(children: Vec<Node>) -> RiffFile {
        RiffFile {
            form: *b"Egg!",
            children,
            trailer: Vec::new(),
        }
    }

    fn only_change(changes: &[Change]) -> &Change {
        assert_eq!(changes.len(), 1, "expected exactly one change: {changes:?}");
        &changes[0]
    }

    // a) two identical files -> empty Vec
    #[test]
    fn identical_files_empty() {
        let a = file(vec![
            chunk("aaaa", &[1, 2, 3]),
            list("Fold", vec![chunk("bbbb", &[9, 8])]),
        ]);
        let b = a.clone();
        assert_eq!(diff(&a, &b), Vec::<Change>::new());
    }

    // b) second file lacks one top-level chunk "bbbb" between "aaaa" and "cccc"
    #[test]
    fn removed_top_level_chunk() {
        let a = file(vec![
            chunk("aaaa", &[1]),
            chunk("bbbb", &[2]),
            chunk("cccc", &[3]),
        ]);
        let b = file(vec![chunk("aaaa", &[1]), chunk("cccc", &[3])]);
        let changes = diff(&a, &b);
        let c = only_change(&changes);
        match c {
            Change::Removed(l) => {
                assert_eq!(l.key, "bbbb");
                assert_eq!(l.path, "");
                assert_eq!(l.index_path, vec![1]);
            }
            other => panic!("expected Removed, got {other:?}"),
        }
    }

    // c) second file has an extra chunk
    #[test]
    fn added_top_level_chunk() {
        let a = file(vec![chunk("aaaa", &[1]), chunk("cccc", &[3])]);
        let b = file(vec![
            chunk("aaaa", &[1]),
            chunk("eeee", &[5]),
            chunk("cccc", &[3]),
        ]);
        let changes = diff(&a, &b);
        let c = only_change(&changes);
        match c {
            Change::Added(l) => {
                assert_eq!(l.key, "eeee");
                assert_eq!(l.path, "");
                // index of the extra chunk in the second file
                assert_eq!(l.index_path, vec![1]);
            }
            other => panic!("expected Added, got {other:?}"),
        }
    }

    // d) same id, same length, 3 bytes differ
    #[test]
    fn changed_same_length_three_bytes() {
        let a = file(vec![chunk("xxxx", &[0, 1, 2, 3, 4, 5, 6, 7])]);
        let b = file(vec![chunk("xxxx", &[0, 9, 2, 8, 8, 5, 6, 7])]);
        let changes = diff(&a, &b);
        let c = only_change(&changes);
        match c {
            Change::Changed(p) => {
                assert_eq!(p.key, "xxxx");
                assert_eq!(p.diff_count, 3);
                assert_eq!(p.first_diff, 1);
                // differing offsets are 1, 3 and 4
                assert_eq!(p.samples, vec![(1, 1, 9), (3, 3, 8), (4, 4, 8)]);
            }
            other => panic!("expected Changed, got {other:?}"),
        }
    }

    // e) lengths 164 and 160, shorter is a prefix
    #[test]
    fn changed_prefix_lengths() {
        let data_a: Vec<u8> = (0..164).map(|i| (i % 251) as u8).collect();
        let data_b: Vec<u8> = data_a[..160].to_vec();
        let a = file(vec![chunk("ldta", &data_a)]);
        let b = file(vec![chunk("ldta", &data_b)]);
        let changes = diff(&a, &b);
        let c = only_change(&changes);
        match c {
            Change::Changed(p) => {
                assert_eq!(p.len_a, 164);
                assert_eq!(p.len_b, 160);
                assert_eq!(p.first_diff, 160);
                assert_eq!(p.diff_count, 0);
                assert!(p.samples.is_empty());
            }
            other => panic!("expected Changed, got {other:?}"),
        }
    }

    // f) nested change
    #[test]
    fn nested_change() {
        let a = file(vec![list(
            "Fold",
            vec![list(
                "Item",
                vec![list("Layr", vec![chunk("ldta", &[1, 2, 3])])],
            )],
        )]);
        let b = file(vec![list(
            "Fold",
            vec![list(
                "Item",
                vec![list("Layr", vec![chunk("ldta", &[1, 9, 3])])],
            )],
        )]);
        let changes = diff(&a, &b);
        let c = only_change(&changes);
        match c {
            Change::Changed(p) => {
                assert_eq!(p.path, "Fold/Item/Layr");
                assert_eq!(p.key, "ldta");
                // Fold[0] / Item[0] / Layr[0] / ldta[0]
                assert_eq!(p.index_path, vec![0, 0, 0, 0]);
            }
            other => panic!("expected Changed, got {other:?}"),
        }
    }

    // g) a whole inner List removed
    #[test]
    fn removed_inner_list() {
        let inner = vec![chunk("aaaa", &[1, 2]), chunk("bbbb", &[3])];
        let a = file(vec![list("Fold", vec![list("Item", inner)])]);
        let b = file(vec![list("Fold", vec![])]);
        let changes = diff(&a, &b);
        let c = only_change(&changes);
        match c {
            Change::Removed(l) => {
                assert_eq!(l.key, "LIST:Item");
                // two descendants (aaaa, bbbb)
                assert_eq!(l.preview, "(2 nodes)");
            }
            other => panic!("expected Removed, got {other:?}"),
        }
    }

    // h) repeated keys [x, y, x, y] vs [x, y, y]
    #[test]
    fn repeated_keys_remove_second_x() {
        let a = file(vec![
            chunk("aaaa", &[1]),
            chunk("bbbb", &[2]),
            chunk("aaaa", &[3]),
            chunk("bbbb", &[4]),
        ]);
        let b = file(vec![
            chunk("aaaa", &[1]),
            chunk("bbbb", &[2]),
            chunk("bbbb", &[4]),
        ]);
        let changes = diff(&a, &b);
        let c = only_change(&changes);
        match c {
            Change::Removed(l) => {
                assert_eq!(l.key, "aaaa");
                assert_eq!(l.index_path, vec![2]); // the second x
            }
            other => panic!("expected Removed, got {other:?}"),
        }
    }

    // i) different form and different trailer
    #[test]
    fn different_form_and_trailer() {
        let mut a = file(vec![chunk("aaaa", &[1])]);
        a.trailer = vec![1, 2, 3];
        let mut b = a.clone();
        b.form = *b"Zzz!";
        b.trailer = vec![1, 2, 3, 4, 5];
        let changes = diff(&a, &b);
        assert_eq!(changes.len(), 2);
        match &changes[0] {
            Change::Changed(p) => {
                assert_eq!(p.key, "(form)");
                assert_eq!(p.path, "");
                assert_eq!(p.index_path, Vec::new());
            }
            other => panic!("expected form Changed, got {other:?}"),
        }
        match &changes[1] {
            Change::Changed(p) => {
                assert_eq!(p.key, "(trailer)");
                assert_eq!(p.len_a, 3);
                assert_eq!(p.len_b, 5);
            }
            other => panic!("expected trailer Changed, got {other:?}"),
        }
    }

    // j) summarize collapses groups
    #[test]
    fn summarize_collapses() {
        let mut changes: Vec<Change> = Vec::new();
        for _ in 0..12 {
            changes.push(Change::Changed(Pair {
                path: "Fold/Item/Layr".to_string(),
                index_path: vec![0, 0, 0, 0],
                key: "ldta".to_string(),
                len_a: 164,
                len_b: 160,
                first_diff: 160,
                diff_count: 0,
                samples: Vec::new(),
            }));
        }
        changes.push(Change::Removed(Lone {
            path: String::new(),
            index_path: vec![0],
            key: "zzzz".to_string(),
            len: 8,
            preview: "hello".to_string(),
        }));
        let lines = summarize(&changes);
        assert_eq!(lines[0], "changed  Fold/Item/Layr  ldta 164->160  x12");
        assert!(lines[1].starts_with("removed  /  "), "got {:?}", lines[1]);
    }

    // k) preview
    #[test]
    fn preview_chunk_text() {
        let node = chunk("name", b"ADBE Shadow Color\0\0\0");
        assert_eq!(node_preview(&node), "ADBE Shadow Color");
    }
}
