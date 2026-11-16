//! Structural comparison of two parsed files.
//!
//! Used to learn what After Effects changes when it saves a copy for an older
//! version, and to check our own output against a file written by After Effects.
//!
//! How two child lists are compared:
//!
//! 1. Every node has a key: its 4-byte id for an ordinary chunk, or
//!    `LIST:<kind>` for both list variants.
//! 2. Each node is also fingerprinted with a FNV-1a hash of its serialized
//!    content. The two lists are first aligned by the longest common
//!    subsequence of these hashes. Candidate pairs whose full content is not
//!    actually equal (hash collision) are discarded. The remaining pairs are
//!    *anchors*: identical nodes that are frozen in place (no change emitted,
//!    no recursion).
//! 3. Each gap between anchors (including the head and tail) is aligned by
//!    the longest common subsequence of the node keys (standard
//!    dynamic-programming LCS; on ties prefer matching earlier nodes).
//! 4. A node of the first file with no partner is [`Change::Removed`].
//!    A node of the second file with no partner is [`Change::Added`].
//!    Removed/added lists are reported once, not once per descendant.
//! 5. Aligned pair, two ordinary chunks: equal data → nothing; otherwise
//!    [`Change::Changed`].
//! 6. Aligned pair, two [`Node::List`]: compare their children recursively.
//! 7. Aligned pair, any other combination of list variants: compare the
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
    /// The `node_preview` of the immediately preceding sibling in the same
    /// list, when that sibling is an ordinary [`Node::Chunk`]. Empty string
    /// otherwise (no preceding sibling, or preceding sibling is a list).
    pub prev: String,
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
/// `Removed`/`Added` — the same `len`, `preview` and `prev`; `Changed` — the
/// same `len_a` and `len_b`. Groups keep the order of first appearance.
/// Each returned line has one of these forms (count is the group size):
///
/// ```text
/// removed  <path>  <key> <len> "<preview>"  x<count>
/// removed  <path>  <key> <len> "<preview>" after "<prev>"  x<count>
/// added    <path>  <key> <len> "<preview>"  x<count>
/// added    <path>  <key> <len> "<preview>" after "<prev>"  x<count>
/// changed  <path>  <key> <len_a>-><len_b>  x<count>
/// ```
///
/// An empty path is printed as `/`.
pub fn summarize(changes: &[Change]) -> Vec<String> {
    // A group is identified by (tag, path, key, rest). `rest` is the variant-
    // specific middle field: `<len> "<preview>"[ after "<prev>"]` for
    // Removed/Added and `<len_a>-><len_b>` for Changed.
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
            Change::Removed(l) => {
                let prev = if l.prev.is_empty() {
                    String::new()
                } else {
                    format!(" after \"{}\"", l.prev)
                };
                (
                    "removed",
                    l.path.clone(),
                    l.key.clone(),
                    format!("{} \"{}\"{}", l.len, l.preview, prev),
                )
            }
            Change::Added(l) => {
                let prev = if l.prev.is_empty() {
                    String::new()
                } else {
                    format!(" after \"{}\"", l.prev)
                };
                (
                    "added",
                    l.path.clone(),
                    l.key.clone(),
                    format!("{} \"{}\"{}", l.len, l.preview, prev),
                )
            }
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

/// True when a change is "save noise": a difference that After Effects
/// introduces on every save regardless of the actual edit, and that users
/// typically want to hide.
///
/// Rules (only [`Change::Changed`] can be noise; `Removed`/`Added` never are):
///
/// * key is `"(trailer)"`, or
/// * key is `"head"` and `first_diff >= 8`, or
/// * `len_a == len_b` and the key is one of
///   `sdat`, `aRbp`, `ftts`, `pard`, `mkif`, `ewin`, `fips`, `Rhed`, `nhed`,
///   `nnhd`, or
/// * `len_a == len_b`, key is `"tdb4"`, `diff_count == 1` and
///   `first_diff == 119` (byte 119 of a property record is session state:
///   After Effects sets it while a project is open and clears it after
///   reopening; it is not tied to the file version).
pub fn is_save_noise(change: &Change) -> bool {
    match change {
        Change::Changed(p) => {
            if p.key == "(trailer)" {
                return true;
            }
            if p.key == "head" && p.first_diff >= 8 {
                return true;
            }
            if p.key == "tdb4" && p.len_a == p.len_b && p.diff_count == 1 && p.first_diff == 119 {
                return true;
            }
            if p.len_a == p.len_b {
                matches!(
                    p.key.as_str(),
                    "sdat"
                        | "aRbp"
                        | "ftts"
                        | "pard"
                        | "mkif"
                        | "ewin"
                        | "fips"
                        | "Rhed"
                        | "nhed"
                        | "nnhd"
                )
            } else {
                false
            }
        }
        Change::Removed(_) | Change::Added(_) => false,
    }
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

/// FNV-1a 64-bit hash of a node's serialized content.
///
/// Two nodes with the same hash *may* still differ (collision); the caller
/// must confirm with a full `==` comparison before treating them as anchors.
fn node_hash(node: &Node) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut buf: Vec<u8> = Vec::new();
    encode_child(node, &mut buf);

    let mut hash = FNV_OFFSET;
    for &byte in &buf {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Build a [`Lone`] for a node that has no partner.
///
/// `prev` is the [`node_preview`] of the immediately preceding sibling when
/// that sibling is an ordinary [`Node::Chunk`], otherwise the empty string.
fn make_lone(
    path: &str,
    index_prefix: &[usize],
    own_index: usize,
    node: &Node,
    prev: &str,
) -> Lone {
    let mut index_path: Vec<usize> = index_prefix.to_vec();
    index_path.push(own_index);
    Lone {
        path: path.to_string(),
        index_path,
        key: node_key(node),
        len: node.data_len(),
        preview: node_preview(node),
        prev: prev.to_string(),
    }
}

/// Compute the `prev` field for a node at index `idx` in list `list`.
fn prev_sibling_preview(list: &[Node], idx: usize) -> String {
    if idx == 0 {
        return String::new();
    }
    match &list[idx - 1] {
        Node::Chunk { .. } => node_preview(&list[idx - 1]),
        _ => String::new(),
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
///
/// Alignment strategy: first find *anchors* (identical nodes) via a longest
/// common subsequence on FNV-1a content hashes, confirming each candidate
/// pair with full `Node` equality. Anchors are frozen (no change emitted, no
/// recursion). Each gap between anchors is aligned by key-LCS.
fn diff_lists(path: &str, index_prefix: &[usize], a: &[Node], b: &[Node], out: &mut Vec<Change>) {
    let n = a.len();
    let m = b.len();

    if n == 0 && m == 0 {
        return;
    }

    // Step 1: compute a content hash for every node in both lists.
    let hashes_a: Vec<u64> = a.iter().map(node_hash).collect();
    let hashes_b: Vec<u64> = b.iter().map(node_hash).collect();

    // Early exit: if both lists contain the same multiset of node contents
    // (only order differs), there is nothing to report.
    if n == m {
        let mut sorted_a = hashes_a.clone();
        let mut sorted_b = hashes_b.clone();
        sorted_a.sort_unstable();
        sorted_b.sort_unstable();
        if sorted_a == sorted_b {
            return;
        }
    }

    // Step 2: LCS on the hash arrays to find candidate anchor pairs.
    // lcs[i][j] = length of the LCS of hashes_a[..i] and hashes_b[..j].
    let width = m + 1;
    let mut table = vec![0u32; (n + 1) * width];
    for i in 1..=n {
        let row = i * width;
        let prev_row = (i - 1) * width;
        for j in 1..=m {
            if hashes_a[i - 1] == hashes_b[j - 1] {
                table[row + j] = table[prev_row + j - 1] + 1;
            } else {
                table[row + j] = table[prev_row + j].max(table[row + j - 1]);
            }
        }
    }

    // Step 3: backtrack to recover candidate (i, j) pairs.
    let mut candidates: Vec<(usize, usize)> = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 && j > 0 {
        if hashes_a[i - 1] == hashes_b[j - 1] {
            candidates.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if table[(i - 1) * width + j] >= table[i * width + (j - 1)] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    candidates.reverse();

    // Step 4: confirm real equality for each candidate (hash collision check).
    let anchors: Vec<(usize, usize)> = candidates
        .into_iter()
        .filter(|&(ai, bi)| a[ai] == b[bi])
        .collect();

    // Step 5: walk the anchors left-to-right, processing each gap.
    let mut ai = 0usize;
    let mut bi = 0usize;
    for &(anchor_a, anchor_b) in &anchors {
        if ai < anchor_a || bi < anchor_b {
            let mut gap = Gap {
                path,
                index_prefix,
                a_full: a,
                b_full: b,
                a_start: ai,
                a_end: anchor_a,
                b_start: bi,
                b_end: anchor_b,
                out,
            };
            diff_gap(&mut gap);
        }
        ai = anchor_a + 1;
        bi = anchor_b + 1;
    }
    // Tail gap after the last anchor.
    if ai < n || bi < m {
        let mut gap = Gap {
            path,
            index_prefix,
            a_full: a,
            b_full: b,
            a_start: ai,
            a_end: n,
            b_start: bi,
            b_end: m,
            out,
        };
        diff_gap(&mut gap);
    }
}

/// A slice of each list between two anchors, plus the context needed to
/// report changes: the enclosing path, the enclosing index path, the complete
/// lists and the gap ranges within them, and the output being built.
struct Gap<'a> {
    path: &'a str,
    index_prefix: &'a [usize],
    a_full: &'a [Node],
    b_full: &'a [Node],
    a_start: usize,
    a_end: usize,
    b_start: usize,
    b_end: usize,
    out: &'a mut Vec<Change>,
}

/// Align a gap (a slice of each list between two anchors) by key-LCS and emit
/// changes. `g.a_full`/`g.b_full` are the complete lists;
/// `[g.a_start..g.a_end)` and `[g.b_start..g.b_end)` are the gap ranges
/// within them.
fn diff_gap(g: &mut Gap<'_>) {
    let n = g.a_end - g.a_start;
    let m = g.b_end - g.b_start;

    if n == 0 && m == 0 {
        return;
    }

    let keys_a: Vec<String> = (g.a_start..g.a_end)
        .map(|i| node_key(&g.a_full[i]))
        .collect();
    let keys_b: Vec<String> = (g.b_start..g.b_end)
        .map(|j| node_key(&g.b_full[j]))
        .collect();

    // Key-LCS table.
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

    // Backtrack to get aligned pairs within the gap.
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
            let idx = ai + g.a_start;
            let prev = prev_sibling_preview(g.a_full, idx);
            g.out.push(Change::Removed(make_lone(
                g.path,
                g.index_prefix,
                idx,
                &g.a_full[idx],
                &prev,
            )));
            ai += 1;
        }
        while bi < mj {
            let idx = bi + g.b_start;
            let prev = prev_sibling_preview(g.b_full, idx);
            g.out.push(Change::Added(make_lone(
                g.path,
                g.index_prefix,
                idx,
                &g.b_full[idx],
                &prev,
            )));
            bi += 1;
        }
        ai = mi + 1;
        bi = mj + 1;

        let a_idx = mi + g.a_start;
        let b_idx = mj + g.b_start;
        let anode = &g.a_full[a_idx];
        let bnode = &g.b_full[b_idx];
        if is_list(anode) && is_list(bnode) {
            // Rule 5: recurse into both lists.
            if let (Node::List { kind, children: ac }, Node::List { children: bc, .. }) =
                (anode, bnode)
            {
                let child_path = if g.path.is_empty() {
                    fourcc_text(*kind)
                } else {
                    format!("{}/{}", g.path, fourcc_text(*kind))
                };
                let mut child_prefix: Vec<usize> = g.index_prefix.to_vec();
                child_prefix.push(a_idx);
                diff_lists(&child_path, &child_prefix, ac, bc, g.out);
            }
        } else {
            // Rule 4 (two ordinary chunks) or Rule 6 (a mix of list variants).
            let (da, db) = pair_data(anode, bnode);
            if da != db {
                let (first_diff, diff_count, samples) = compare_data(&da, &db);
                emit_changed(
                    &mut DiffCtx {
                        path: g.path,
                        index_prefix: g.index_prefix,
                        out: g.out,
                    },
                    a_idx,
                    anode,
                    bnode,
                    first_diff,
                    diff_count,
                    samples,
                );
            }
        }
    }

    // Remaining nodes after the last aligned pair.
    while ai < n {
        let idx = ai + g.a_start;
        let prev = prev_sibling_preview(g.a_full, idx);
        g.out.push(Change::Removed(make_lone(
            g.path,
            g.index_prefix,
            idx,
            &g.a_full[idx],
            &prev,
        )));
        ai += 1;
    }
    while bi < m {
        let idx = bi + g.b_start;
        let prev = prev_sibling_preview(g.b_full, idx);
        g.out.push(Change::Added(make_lone(
            g.path,
            g.index_prefix,
            idx,
            &g.b_full[idx],
            &prev,
        )));
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
            prev: String::new(),
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

    // l) anchored alignment: identical nodes are frozen, no spurious changes.
    #[test]
    fn anchored_identical_nodes_no_change() {
        // Two files with the same nodes but in different order.
        // Anchors should prevent spurious Removed/Added pairs.
        let a = file(vec![
            chunk("aaaa", &[1, 2, 3]),
            chunk("bbbb", &[4, 5, 6]),
            chunk("cccc", &[7, 8, 9]),
        ]);
        let b = file(vec![
            chunk("aaaa", &[1, 2, 3]),
            chunk("cccc", &[7, 8, 9]),
            chunk("bbbb", &[4, 5, 6]),
        ]);
        // aaaa and cccc are anchors (identical content).
        // bbbb is in a different position but is also identical, so it should
        // also be an anchor. With all three as anchors, no changes expected.
        let changes = diff(&a, &b);
        assert!(changes.is_empty(), "expected no changes, got {changes:?}");
    }

    // m) anchored alignment: one node differs, others are anchors.
    #[test]
    fn anchored_one_differs_others_frozen() {
        let a = file(vec![
            chunk("aaaa", &[1, 2, 3]),
            chunk("bbbb", &[4, 5, 6]),
            chunk("cccc", &[7, 8, 9]),
        ]);
        let b = file(vec![
            chunk("aaaa", &[1, 2, 3]),
            chunk("bbbb", &[4, 5, 7]), // different
            chunk("cccc", &[7, 8, 9]),
        ]);
        let changes = diff(&a, &b);
        assert_eq!(changes.len(), 1, "expected 1 change, got {changes:?}");
        match &changes[0] {
            Change::Changed(p) => {
                assert_eq!(p.key, "bbbb");
                assert_eq!(p.first_diff, 2);
            }
            other => panic!("expected Changed, got {other:?}"),
        }
    }

    // n) prev field: removed node has prev set to the preceding chunk preview.
    #[test]
    fn prev_field_set_for_removed() {
        let a = file(vec![
            chunk("aaaa", b"hello world"),
            chunk("bbbb", &[2]),
            chunk("cccc", &[3]),
        ]);
        let b = file(vec![chunk("aaaa", b"hello world"), chunk("cccc", &[3])]);
        let changes = diff(&a, &b);
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            Change::Removed(l) => {
                assert_eq!(l.key, "bbbb");
                assert_eq!(l.prev, "hello world");
            }
            other => panic!("expected Removed, got {other:?}"),
        }
    }

    // o) prev field: first node in list has empty prev.
    #[test]
    fn prev_field_empty_for_first_node() {
        let a = file(vec![chunk("aaaa", &[1]), chunk("cccc", &[3])]);
        let b = file(vec![chunk("cccc", &[3])]);
        let changes = diff(&a, &b);
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            Change::Removed(l) => {
                assert_eq!(l.key, "aaaa");
                assert_eq!(l.prev, "");
            }
            other => panic!("expected Removed, got {other:?}"),
        }
    }

    // p) prev field: preceding sibling is a list, so prev is empty.
    #[test]
    fn prev_field_empty_when_prev_sibling_is_list() {
        let a = file(vec![
            list("Fold", vec![chunk("bbbb", &[9])]),
            chunk("cccc", &[3]),
        ]);
        let b = file(vec![chunk("cccc", &[3])]);
        let changes = diff(&a, &b);
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            Change::Removed(l) => {
                assert_eq!(l.key, "LIST:Fold");
                assert_eq!(l.prev, "");
            }
            other => panic!("expected Removed, got {other:?}"),
        }
    }

    // q) is_save_noise: (trailer) is always noise.
    #[test]
    fn noise_trailer() {
        let change = Change::Changed(Pair {
            path: String::new(),
            index_path: Vec::new(),
            key: "(trailer)".to_string(),
            len_a: 3,
            len_b: 5,
            first_diff: 0,
            diff_count: 0,
            samples: Vec::new(),
        });
        assert!(is_save_noise(&change));
    }

    // r) is_save_noise: head with first_diff >= 8 is noise.
    #[test]
    fn noise_head_first_diff_ge_8() {
        let change = Change::Changed(Pair {
            path: String::new(),
            index_path: Vec::new(),
            key: "head".to_string(),
            len_a: 20,
            len_b: 20,
            first_diff: 8,
            diff_count: 2,
            samples: Vec::new(),
        });
        assert!(is_save_noise(&change));
    }

    // s) is_save_noise: head with first_diff < 8 is NOT noise.
    #[test]
    fn not_noise_head_first_diff_lt_8() {
        let change = Change::Changed(Pair {
            path: String::new(),
            index_path: Vec::new(),
            key: "head".to_string(),
            len_a: 20,
            len_b: 20,
            first_diff: 1,
            diff_count: 2,
            samples: Vec::new(),
        });
        assert!(!is_save_noise(&change));
    }

    // t) is_save_noise: same-length known key is noise.
    #[test]
    fn noise_same_length_known_key() {
        for key in [
            "sdat", "aRbp", "ftts", "pard", "mkif", "ewin", "fips", "Rhed", "nhed", "nnhd",
        ] {
            let change = Change::Changed(Pair {
                path: String::new(),
                index_path: Vec::new(),
                key: key.to_string(),
                len_a: 100,
                len_b: 100,
                first_diff: 0,
                diff_count: 1,
                samples: Vec::new(),
            });
            assert!(is_save_noise(&change), "expected noise for key {key}");
        }
    }

    // u) is_save_noise: different-length known key is NOT noise.
    #[test]
    fn not_noise_diff_length_known_key() {
        let change = Change::Changed(Pair {
            path: String::new(),
            index_path: Vec::new(),
            key: "sdat".to_string(),
            len_a: 100,
            len_b: 200,
            first_diff: 0,
            diff_count: 0,
            samples: Vec::new(),
        });
        assert!(!is_save_noise(&change));
    }

    // v) is_save_noise: Removed/Added are never noise.
    #[test]
    fn not_noise_removed_or_added() {
        let removed = Change::Removed(Lone {
            path: String::new(),
            index_path: vec![0],
            key: "sdat".to_string(),
            len: 10,
            preview: "x".to_string(),
            prev: String::new(),
        });
        assert!(!is_save_noise(&removed));

        let added = Change::Added(Lone {
            path: String::new(),
            index_path: vec![0],
            key: "sdat".to_string(),
            len: 10,
            preview: "x".to_string(),
            prev: String::new(),
        });
        assert!(!is_save_noise(&added));
    }

    // w) is_save_noise: a tdb4 differing only in its session-state byte
    // (byte 119) is noise; any other difference is not.
    #[test]
    fn noise_tdb4_session_state_byte() {
        let make = |diff_count: u64, first_diff: u64| {
            Change::Changed(Pair {
                path: String::new(),
                index_path: Vec::new(),
                key: "tdb4".to_string(),
                len_a: 500,
                len_b: 500,
                first_diff,
                diff_count,
                samples: Vec::new(),
            })
        };

        assert!(
            is_save_noise(&make(1, 119)),
            "single difference at byte 119 must be noise"
        );
        assert!(!is_save_noise(&make(2, 119)));
        assert!(!is_save_noise(&make(1, 118)));
    }
}
