//! RIFX container: parse a file into a tree and write it back byte for byte.
//!
//! Layout (all integers big-endian):
//!
//! ```text
//! "RIFX"  u32 body_size  body[body_size]  trailer[..]
//! body  = form[4]  chunk*
//! chunk = id[4]  u32 size  data[size]  pad[size % 2]      (pad byte is 0x00)
//! list  = chunk whose id is "LIST"; its data = kind[4] chunk*
//! ```
//!
//! `trailer` is whatever follows the RIFX body. After Effects stores an XMP
//! packet there; it is kept verbatim.
//!
//! The one invariant everything else relies on:
//! `parse(bytes)?.to_bytes() == bytes` for every file `parse` accepts.

/// Four-character chunk id, list kind or form type.
pub type FourCc = [u8; 4];

/// Why a byte slice could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiffError {
    /// Fewer than 12 bytes, or the first four are not `RIFX`.
    NotRifx,
    /// `8 + body_size` is larger than the input.
    Truncated { declared: u64, available: u64 },
    /// A top-level chunk does not fit in the body, or an odd-sized top-level
    /// chunk is missing its pad byte. `offset` is the file offset of the chunk id.
    BadChunk { offset: u64 },
}

/// One node of the chunk tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// Ordinary chunk. `data` excludes the pad byte.
    Chunk { id: FourCc, data: Vec<u8> },
    /// `LIST` chunk whose content parsed cleanly as chunks.
    List { kind: FourCc, children: Vec<Node> },
    /// `LIST` chunk kept verbatim. `data` is everything after the 4-byte kind.
    /// Used for kind `btdk` (never parsed) and for any LIST whose content does
    /// not parse cleanly as chunks.
    OpaqueList { kind: FourCc, data: Vec<u8> },
}

/// A parsed RIFX file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiffFile {
    /// Form type: `Egg!` for .aep and .et.
    pub form: FourCc,
    /// Top-level chunks, in file order.
    pub children: Vec<Node>,
    /// Bytes after the RIFX body, verbatim.
    pub trailer: Vec<u8>,
}

const LIST_ID: FourCc = *b"LIST";
const BTDK: FourCc = *b"btdk";
const RIFX: &[u8; 4] = b"RIFX";

/// Read a big-endian `u32` from four bytes.
fn be32(b: &[u8]) -> u64 {
    ((b[0] as u64) << 24) | ((b[1] as u64) << 16) | ((b[2] as u64) << 8) | (b[3] as u64)
}

/// True when the slice holds a complete chunk header: 4 id bytes plus a
/// 4-byte size field. `pos` is the byte offset of the id within `s`.
fn has_header(s: &[u8], pos: u64) -> bool {
    let len = s.len() as u64;
    pos < len && len - pos >= 8
}

/// Try to parse one chunk starting at `pos` inside `s`.
///
/// Returns the id, the data slice (pad excluded) and the offset just past the
/// chunk (pad included), or `None` when the chunk does not fit, its pad byte
/// is missing or non-zero.
fn try_chunk(s: &[u8], pos: u64) -> Option<(FourCc, &[u8], u64)> {
    if !has_header(s, pos) {
        return None;
    }
    let hdr = &s[pos as usize..(pos + 8) as usize];
    let id: FourCc = hdr[0..4].try_into().unwrap_or_default();
    let size = be32(&hdr[4..8]);
    let len = s.len() as u64;
    let data_start = pos + 8;
    let data_end = data_start + size;
    let pad_end = data_end + size % 2;
    if pad_end > len {
        return None;
    }
    let data = &s[data_start as usize..data_end as usize];
    if size % 2 == 1 && s[data_end as usize] != 0 {
        return None;
    }
    Some((id, data, pad_end))
}

/// Try to parse `s` as a complete sequence of chunks ending exactly at
/// `s.len()`. Returns `None` when anything is off.
fn try_chunk_seq(s: &[u8]) -> Option<Vec<Node>> {
    let mut out = Vec::new();
    let mut pos: u64 = 0;
    let len = s.len() as u64;
    while pos < len {
        let (id, data, next) = try_chunk(s, pos)?;
        out.push(make_node(id, data));
        pos = next;
    }
    Some(out)
}

/// Build the tree node for a chunk with the given id and data (pad excluded).
fn make_node(id: FourCc, data: &[u8]) -> Node {
    if id == LIST_ID && data.len() >= 4 {
        let kind: FourCc = data[0..4].try_into().unwrap_or_default();
        if kind == BTDK {
            return Node::OpaqueList {
                kind,
                data: data[4..].to_vec(),
            };
        }
        if let Some(children) = try_chunk_seq(&data[4..]) {
            return Node::List { kind, children };
        }
        return Node::OpaqueList {
            kind,
            data: data[4..].to_vec(),
        };
    }
    Node::Chunk {
        id,
        data: data.to_vec(),
    }
}

/// Parse `bytes` into a tree.
///
/// Rules:
/// - A `LIST` with kind `btdk` always becomes [`Node::OpaqueList`].
/// - Any other `LIST` becomes [`Node::List`] if its content parses as chunks
///   that end exactly at the end of the LIST data (an odd-sized last child
///   must have its pad byte inside the LIST). Otherwise it becomes
///   [`Node::OpaqueList`]. A LIST shorter than 4 bytes is an ordinary
///   [`Node::Chunk`] with id `LIST`.
/// - A pad byte that is not `0x00` counts as "does not parse cleanly": inside
///   a LIST it makes that LIST opaque, at top level it is `BadChunk`.
/// - Top-level chunks must fill the body exactly, else `BadChunk`.
/// - Must never panic and never index out of bounds, whatever the input.
pub fn parse(bytes: &[u8]) -> Result<RiffFile, RiffError> {
    if bytes.len() < 12 || bytes[0..4] != *RIFX {
        return Err(RiffError::NotRifx);
    }
    let body_size = be32(&bytes[4..8]);
    let avail = bytes.len() as u64;
    let declared = 8 + body_size;
    if declared > avail {
        return Err(RiffError::Truncated {
            declared,
            available: avail,
        });
    }
    let body_end = 8 + body_size as usize;
    let body = &bytes[8..body_end];
    if body.len() < 4 {
        // Not even room for the 4-byte form type.
        return Err(RiffError::BadChunk { offset: 8 });
    }
    let form: FourCc = body[0..4].try_into().unwrap_or_default();
    let trailer = bytes[body_end..].to_vec();

    let mut children = Vec::new();
    let mut pos: u64 = 4; // relative to the start of the body
    let body_len = body.len() as u64;
    while pos < body_len {
        let offset = 8 + pos; // file offset of the chunk id
        let (id, data, next) = match try_chunk(body, pos) {
            Some(c) => c,
            None => return Err(RiffError::BadChunk { offset }),
        };
        children.push(make_node(id, data));
        pos = next;
    }

    Ok(RiffFile {
        form,
        children,
        trailer,
    })
}

impl RiffFile {
    /// Serialize the tree. Sizes are recomputed from the content, so an edited
    /// tree comes out consistent.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut body: Vec<u8> = Vec::with_capacity(4 + self.children.len() * 16);
        body.extend_from_slice(&self.form);
        for child in &self.children {
            encode_node(child, &mut body);
        }
        let mut out: Vec<u8> = Vec::with_capacity(8 + body.len() + self.trailer.len());
        out.extend_from_slice(RIFX);
        let body_size = body.len() as u64;
        out.extend_from_slice(&(body_size.min(u32::MAX as u64) as u32).to_be_bytes());
        out.extend_from_slice(&body);
        out.extend_from_slice(&self.trailer);
        out
    }

    /// Data of the first top-level [`Node::Chunk`] with this id.
    pub fn chunk(&self, id: FourCc) -> Option<&[u8]> {
        self.children.iter().find_map(|c| match c {
            Node::Chunk { id: cid, data, .. } if *cid == id => Some(data.as_slice()),
            _ => None,
        })
    }

    /// Call `f` for every node, depth first, parents before children.
    /// `path` holds the kinds of the enclosing lists, outermost first
    /// (empty for top-level nodes).
    pub fn walk<'a>(&'a self, f: &mut dyn FnMut(&[FourCc], &'a Node)) {
        let path: Vec<FourCc> = Vec::new();
        for child in &self.children {
            child.walk_rec(&path, f);
        }
    }
}

impl Node {
    /// Walk this node and its (parsed) list children.
    fn walk_rec<'a>(&'a self, path: &[FourCc], f: &mut dyn FnMut(&[FourCc], &'a Node)) {
        f(path, self);
        if let Node::List { kind, children } = self {
            let mut child_path = path.to_vec();
            child_path.push(*kind);
            for child in children {
                child.walk_rec(&child_path, f);
            }
        }
    }

    /// Chunk id; `LIST` for both list variants.
    pub fn id(&self) -> FourCc {
        match self {
            Node::Chunk { id, .. } => *id,
            Node::List { .. } | Node::OpaqueList { .. } => LIST_ID,
        }
    }

    /// List kind, or `None` for an ordinary chunk.
    pub fn kind(&self) -> Option<FourCc> {
        match self {
            Node::Chunk { .. } => None,
            Node::List { kind, .. } | Node::OpaqueList { kind, .. } => Some(*kind),
        }
    }

    /// Size of the node's data as written in its size field
    /// (for lists: 4 + encoded size of all children).
    pub fn data_len(&self) -> u64 {
        match self {
            Node::Chunk { data, .. } => data.len() as u64,
            Node::List { children, .. } => {
                4 + children
                    .iter()
                    .map(|c| {
                        let n = c.data_len();
                        8 + n + n % 2
                    })
                    .sum::<u64>()
            }
            Node::OpaqueList { data, .. } => 4 + data.len() as u64,
        }
    }
}

/// Append the encoded form of `node` (id, size, data, pad) to `out`.
fn encode_node(node: &Node, out: &mut Vec<u8>) {
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
            out.extend_from_slice(&LIST_ID);
            let size_at = out.len();
            out.extend_from_slice(&[0, 0, 0, 0]);
            let start = out.len();
            out.extend_from_slice(kind);
            for child in children {
                encode_node(child, out);
            }
            let size: u64 = out.len() as u64 - start as u64;
            out[size_at..size_at + 4]
                .copy_from_slice(&(size.min(u32::MAX as u64) as u32).to_be_bytes());
            if size % 2 == 1 {
                out.push(0);
            }
        }
        Node::OpaqueList { kind, data } => {
            out.extend_from_slice(&LIST_ID);
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

/// Append a size field, saturating at the 32-bit maximum.
fn push_size(out: &mut Vec<u8>, size: u64) {
    out.extend_from_slice(&(size.min(u32::MAX as u64) as u32).to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "RIFX" + body size + form "Egg!" + one chunk "abcd" with 3 data bytes
    /// plus one 0x00 pad byte.
    fn minimal() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFX");
        b.extend_from_slice(&16u32.to_be_bytes()); // form + "abcd" chunk incl. pad
        b.extend_from_slice(b"Egg!");
        b.extend_from_slice(b"abcd");
        b.extend_from_slice(&3u32.to_be_bytes());
        b.extend_from_slice(&[1, 2, 3]);
        b.push(0);
        b
    }

    /// Helper: build a RIFX file with one top-level LIST of the given kind and
    /// raw content after the kind.
    fn one_list(kind: FourCc, content: Vec<u8>) -> Vec<u8> {
        let data_len = 4 + content.len(); // kind + content
        let body_size = 4 + 8 + data_len + data_len % 2; // form + LIST hdr + data + pad
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFX");
        b.extend_from_slice(&(body_size as u32).to_be_bytes());
        b.extend_from_slice(b"Egg!");
        b.extend_from_slice(b"LIST");
        b.extend_from_slice(&(data_len as u32).to_be_bytes());
        b.extend_from_slice(&kind);
        b.extend_from_slice(&content);
        if data_len % 2 == 1 {
            b.push(0);
        }
        b
    }

    /// "RIFX" + form "Egg!" + one LIST of kind "Fold" holding two chunks.
    fn fold_list() -> Vec<u8> {
        let mut content = Vec::new();
        content.extend_from_slice(b"aaaa");
        content.extend_from_slice(&2u32.to_be_bytes());
        content.extend_from_slice(&[9, 8]);
        content.extend_from_slice(b"bbbb");
        content.extend_from_slice(&3u32.to_be_bytes());
        content.extend_from_slice(&[7, 6, 5]);
        content.push(0); // pad for the odd-sized bbbb chunk, inside the LIST
        one_list(*b"Fold", content)
    }

    // a) minimal file
    #[test]
    fn minimal_roundtrips() {
        let input = minimal();
        let f = parse(&input).expect("parses");
        assert_eq!(f.children.len(), 1);
        match &f.children[0] {
            Node::Chunk { id, data } => {
                assert_eq!(*id, *b"abcd");
                assert_eq!(data.len(), 3);
            }
            other => panic!("expected chunk, got {other:?}"),
        }
        assert_eq!(f.to_bytes(), input);
    }

    // b) LIST "Fold" with two children
    #[test]
    fn fold_list_roundtrips() {
        let input = fold_list();
        let f = parse(&input).expect("parses");
        assert_eq!(f.children.len(), 1);
        match &f.children[0] {
            Node::List { kind, children } => {
                assert_eq!(*kind, *b"Fold");
                assert_eq!(children.len(), 2);
            }
            other => panic!("expected list, got {other:?}"),
        }
        assert_eq!(f.to_bytes(), input);
    }

    // c) LIST "btdk" stays opaque
    #[test]
    fn btdk_is_opaque() {
        let content = b"cccc".to_vec();
        let input = one_list(*b"btdk", content);
        let f = parse(&input).expect("parses");
        assert_eq!(f.children.len(), 1);
        match &f.children[0] {
            Node::OpaqueList { kind, data } => {
                assert_eq!(*kind, *b"btdk");
                assert_eq!(data, b"cccc");
            }
            other => panic!("expected opaque list, got {other:?}"),
        }
        assert_eq!(f.to_bytes(), input);
    }

    // d) LIST "Item" with broken content becomes opaque and round-trips
    #[test]
    fn broken_list_is_opaque() {
        // A child whose size (0x7FFFFFFF) is far larger than the LIST.
        let mut content = Vec::new();
        content.extend_from_slice(b"xxxx");
        content.extend_from_slice(&0x7FFFFFFFu32.to_be_bytes());
        content.extend_from_slice(&[1, 2]);
        let input = one_list(*b"Item", content);
        let f = parse(&input).expect("parses");
        assert_eq!(f.children.len(), 1);
        match &f.children[0] {
            Node::OpaqueList { kind, .. } => assert_eq!(*kind, *b"Item"),
            other => panic!("expected opaque list, got {other:?}"),
        }
        assert_eq!(f.to_bytes(), input);
    }

    // e) 5 extra bytes after the body land in the trailer
    #[test]
    fn trailer_roundtrips() {
        let mut input = minimal();
        input.extend_from_slice(&[255, 254, 253, 252, 251]);
        let f = parse(&input).expect("parses");
        assert_eq!(f.trailer, vec![255, 254, 253, 252, 251]);
        assert_eq!(f.to_bytes(), input);
    }

    // f) errors
    #[test]
    fn error_cases() {
        // "RIFF" magic
        let mut riff = minimal();
        riff[3] = b'F';
        assert_eq!(parse(&riff), Err(RiffError::NotRifx));
        // too short
        assert_eq!(parse(b"RIFX\x00\x00"), Err(RiffError::NotRifx));
        assert_eq!(parse(&minimal()[..5]), Err(RiffError::NotRifx));
        // body size larger than input
        let mut big = minimal();
        big[4..8].copy_from_slice(&0x10000000u32.to_be_bytes());
        assert_eq!(
            parse(&big),
            Err(RiffError::Truncated {
                declared: 8 + 0x10000000,
                available: big.len() as u64,
            })
        );
        // top-level chunk larger than the body
        let mut small_body = minimal();
        small_body[4..8].copy_from_slice(&10u32.to_be_bytes()); // body = "Egg!" + 6 bytes
        assert_eq!(parse(&small_body), Err(RiffError::BadChunk { offset: 12 }));
    }

    // g) pad bytes
    #[test]
    fn pad_byte_rules() {
        // non-zero pad at top level is BadChunk
        let mut bad = minimal();
        *bad.last_mut().expect("has pad") = 7;
        assert_eq!(parse(&bad), Err(RiffError::BadChunk { offset: 12 }));
        // non-zero pad inside a LIST makes that LIST opaque and round-trips
        let mut content = Vec::new();
        content.extend_from_slice(b"aaaa");
        content.extend_from_slice(&3u32.to_be_bytes());
        content.extend_from_slice(&[1, 2, 3]);
        content.push(9); // bad pad
        let content_len = content.len();
        let input = one_list(*b"Fold", content);
        let f = parse(&input).expect("parses");
        match &f.children[0] {
            Node::OpaqueList { kind, data } => {
                assert_eq!(*kind, *b"Fold");
                assert_eq!(data.len(), content_len);
            }
            other => panic!("expected opaque list, got {other:?}"),
        }
        assert_eq!(f.to_bytes(), input);
    }

    // h) walk()
    #[test]
    fn walk_paths() {
        let f = parse(&fold_list()).expect("parses");
        let mut seen: Vec<(Vec<FourCc>, FourCc)> = Vec::new();
        f.walk(&mut |path, node| seen.push((path.to_vec(), node.id())));
        assert_eq!(seen.len(), 3);
        // the LIST itself is top-level: empty path
        assert!(seen[0].0.is_empty());
        assert_eq!(seen[0].1, *b"LIST");
        // the two chunks are inside the "Fold" list
        assert_eq!(seen[1].0.as_slice(), &[*b"Fold"][..]);
        assert_eq!(seen[1].1, *b"aaaa");
        assert_eq!(seen[2].0.as_slice(), &[*b"Fold"][..]);
        assert_eq!(seen[2].1, *b"bbbb");
    }

    // i) every prefix parses without panicking
    #[test]
    fn all_prefixes_no_panic() {
        let input = minimal();
        for len in 0..=input.len() {
            let _ = parse(&input[..len]);
        }
        // and a longer, nastier file
        let fold = fold_list();
        for len in 0..=fold.len() {
            let _ = parse(&fold[..len]);
        }
    }

    // j) chunk()
    #[test]
    fn chunk_lookup() {
        let f = parse(&minimal()).expect("parses");
        assert_eq!(f.chunk(*b"abcd"), Some(&[1, 2, 3][..]));
        assert_eq!(f.chunk(*b"zzzz"), None);
    }

    // k) 40 levels of nested "Fold" lists, each holding one "abcd" chunk (3
    // data bytes + pad) followed by the next list; the innermost list holds
    // only the chunk. Built by hand, inside out, so every size is correct.
    //
    // This would double the work at every level if data_len were called twice
    // per child and if encode_node recomputed data_len. The fix keeps it
    // linear, so this finishes in well under a second.
    #[test]
    fn deep_nesting_is_fast() {
        // 40 nested LISTs; every level holds one odd-sized chunk, then the next LIST.
        // With exponential size computation this never finishes.
        let chunk = |out: &mut Vec<u8>| {
            out.extend_from_slice(b"abcd");
            out.extend_from_slice(&3u32.to_be_bytes());
            out.extend_from_slice(&[1, 2, 3, 0]);
        };
        let list = |out: &mut Vec<u8>, content: &[u8]| {
            out.extend_from_slice(b"LIST");
            out.extend_from_slice(&((content.len() + 4) as u32).to_be_bytes());
            out.extend_from_slice(b"Fold");
            out.extend_from_slice(content);
        };
        let mut inner: Vec<u8> = Vec::new();
        chunk(&mut inner);
        for _ in 0..39 {
            let mut level = Vec::new();
            chunk(&mut level);
            list(&mut level, &inner);
            inner = level;
        }
        let mut body = Vec::new();
        body.extend_from_slice(b"Egg!");
        list(&mut body, &inner);
        let mut input = Vec::new();
        input.extend_from_slice(b"RIFX");
        input.extend_from_slice(&(body.len() as u32).to_be_bytes());
        input.extend_from_slice(&body);

        let file = parse(&input).expect("parses");
        assert_eq!(file.to_bytes(), input);
        let mut nodes = 0;
        file.walk(&mut |_, _| nodes += 1);
        assert_eq!(nodes, 80);
    }

    // data_len sanity
    #[test]
    fn data_lens() {
        let f = parse(&fold_list()).expect("parses");
        let list = &f.children[0];
        assert_eq!(list.data_len(), 4 + 8 + 2 + 8 + 3 + 1); // kind + 2 children incl. pad
    }
}
