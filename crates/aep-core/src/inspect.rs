//! Identify a file from its first bytes, without parsing the whole file.
//!
//! [`inspect`] takes a prefix of the file. 64 KiB is plenty: the chunks it
//! looks for (`svap`, `head`) are the first chunks of a project.
//!
//! Decision order:
//!
//! 1. Prefix starts with `00 05 16 07` → [`FileKind::AppleDouble`] (a macOS
//!    `._name.aep` sidecar file, not a project).
//! 2. After skipping an optional UTF-8 byte-order mark (`EF BB BF`) and ASCII
//!    whitespace, the prefix starts with `<?xml` or `<AfterEffectsProject`,
//!    and the bytes `AfterEffectsProject` occur somewhere in the prefix
//!    → [`FileKind::XmlProject`] (.aepx).
//! 3. Prefix is at least 12 bytes, starts with `RIFX`, and bytes 8..12 are
//!    `Egg!` → [`FileKind::Project`]; `FaFX` → [`FileKind::Preset`].
//! 4. Anything else → [`FileKind::Unknown`].
//!
//! For `Project` and `Preset` the top-level chunks are then scanned, starting
//! at offset 12. Each chunk is `id[4]`, big-endian `u32 size`, `data[size]`,
//! plus one pad byte when `size` is odd. `LIST` chunks are skipped like any
//! other chunk (their content is not entered). The scan stops when the next
//! 8-byte chunk header does not fit in the prefix, or after 64 chunks.
//! A chunk is used only if all of its data lies inside the prefix:
//!
//! - first `svap` with at least 4 data bytes → `saved_by` (big-endian u32);
//! - first `head`, in a `Project` → `head` via [`Head::parse`];
//! - first `head`, in a `Preset`, with at least 12 data bytes →
//!   `preset_format`: major = big-endian u32 at data[4..8], minor = big-endian
//!   u32 at data[8..12], each clamped to `u16::MAX`.
//!
//! Must never panic, whatever the input. Use u64 or checked arithmetic for offsets.

use crate::version::{AppVersion, FormatVersion, Head};

/// What kind of file the bytes look like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// After Effects project or template (.aep, .aet).
    Project,
    /// Animation preset (.ffx).
    Preset,
    /// XML project (.aepx).
    XmlProject,
    /// macOS AppleDouble sidecar file.
    AppleDouble,
    Unknown,
}

/// Result of [`inspect`]. Fields that do not apply, or whose chunk was not
/// found inside the prefix, are `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inspection {
    pub kind: FileKind,
    /// `Project` only: format version and the After Effects version the file is written for.
    pub head: Option<Head>,
    /// The application that saved the file (`svap` chunk).
    pub saved_by: Option<AppVersion>,
    /// `Preset` only: format version of the preset.
    pub preset_format: Option<FormatVersion>,
}

/// Identify a file from a prefix of its bytes. See the module docs for the rules.
pub fn inspect(prefix: &[u8]) -> Inspection {
    // Rule 1: macOS AppleDouble sidecar file.
    if starts_with4(prefix, APPLE_DOUBLE_MAGIC) {
        return Inspection {
            kind: FileKind::AppleDouble,
            head: None,
            saved_by: None,
            preset_format: None,
        };
    }

    // Rule 2: XML project (.aepx).
    if is_xml_project(prefix) {
        return Inspection {
            kind: FileKind::XmlProject,
            head: None,
            saved_by: None,
            preset_format: None,
        };
    }

    // Rule 3: RIFX container that is a project or a preset.
    if starts_with4(prefix, RIFX)
        && let Some(form) = prefix.get(8..12)
    {
        if form == EGG {
            return scan_rifx(prefix, FileKind::Project);
        }
        if form == FAFX {
            return scan_rifx(prefix, FileKind::Preset);
        }
    }

    // Rule 4: anything else.
    Inspection {
        kind: FileKind::Unknown,
        head: None,
        saved_by: None,
        preset_format: None,
    }
}

/// Four-character needles, kept as `[u8]` slices so every comparison is an
/// ordinary slice equality.
const RIFX: &[u8] = b"RIFX";
const EGG: &[u8] = b"Egg!";
const FAFX: &[u8] = b"FaFX";
const SVAP: &[u8] = b"svap";
const HEAD: &[u8] = b"head";
const AFTER_EFFECTS_PROJECT: &[u8] = b"AfterEffectsProject";
const APPLE_DOUBLE_MAGIC: &[u8] = &[0x00, 0x05, 0x16, 0x07];
const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// True when `b` starts with the four-byte `needle`.
fn starts_with4(b: &[u8], needle: &[u8]) -> bool {
    b.get(..4) == Some(needle)
}

/// True when the bytes look like an XML project: after an optional UTF-8 BOM
/// and ASCII whitespace they start with `<?xml` or `<AfterEffectsProject`, and
/// `AfterEffectsProject` occurs somewhere in the prefix.
fn is_xml_project(prefix: &[u8]) -> bool {
    let mut i = 0usize;
    if prefix.get(..3) == Some(UTF8_BOM) {
        i = 3;
    }
    while i < prefix.len() && prefix[i].is_ascii_whitespace() {
        i += 1;
    }
    let rest = &prefix[i..];
    let starts = rest.starts_with(b"<?xml") || rest.starts_with(b"<AfterEffectsProject");
    starts
        && prefix
            .windows(AFTER_EFFECTS_PROJECT.len())
            .any(|w| w == AFTER_EFFECTS_PROJECT)
}

/// Read a big-endian 32-bit value as a `u64` from four bytes.
fn be32(b: &[u8]) -> u64 {
    (u64::from(b[0]) << 24) | (u64::from(b[1]) << 16) | (u64::from(b[2]) << 8) | u64::from(b[3])
}

/// Scan the top-level chunks of a RIFX project or preset and fill in the
/// `svap`, `head`/`preset_format` fields. Never panics on a truncated prefix.
fn scan_rifx(prefix: &[u8], kind: FileKind) -> Inspection {
    let mut head: Option<Head> = None;
    let mut saved_by: Option<AppVersion> = None;
    let mut preset_format: Option<FormatVersion> = None;

    let len = prefix.len() as u64;
    let mut pos: u64 = 12;
    let mut chunks = 0u32;

    while chunks < 64 && pos + 8 <= len {
        let id = &prefix[pos as usize..(pos + 4) as usize];
        let size = be32(&prefix[(pos + 4) as usize..(pos + 8) as usize]);
        let data_start = pos + 8;
        let data_end = data_start + size;
        let pad_end = data_end + size % 2;
        // A chunk is usable only if all of its data (plus pad) fits.
        if pad_end > len {
            break;
        }
        let data = &prefix[data_start as usize..data_end as usize];

        if id == SVAP && saved_by.is_none() && data.len() >= 4 {
            saved_by = Some(AppVersion(be32(&data[0..4]) as u32));
        } else if id == HEAD {
            if kind == FileKind::Project && head.is_none() {
                head = Head::parse(data);
            } else if kind == FileKind::Preset && preset_format.is_none() && data.len() >= 12 {
                let major = (be32(&data[4..8]) & u64::from(u16::MAX)) as u16;
                let minor = (be32(&data[8..12]) & u64::from(u16::MAX)) as u16;
                preset_format = Some(FormatVersion { major, minor });
            }
        }

        pos = pad_end;
        chunks += 1;
    }

    Inspection {
        kind,
        head,
        saved_by,
        preset_format,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-made project from spec test (f): 52 bytes.
    fn project_f() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFX");
        b.extend_from_slice(&44u32.to_be_bytes());
        b.extend_from_slice(b"Egg!");
        // svap, size 4
        b.extend_from_slice(b"svap");
        b.extend_from_slice(&4u32.to_be_bytes());
        b.extend_from_slice(&[0x0F, 0x02, 0x86, 0x34]);
        // head, size 20
        b.extend_from_slice(b"head");
        b.extend_from_slice(&20u32.to_be_bytes());
        b.extend_from_slice(&[
            0x00, 0x5F, 0x00, 0x05, 0x0F, 0x02, 0x86, 0x34, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x20,
        ]);
        b
    }

    /// Project (f) with an odd-sized `abcd` chunk (size 3 + 1 pad byte) inserted
    /// between `svap` and `head`.
    fn project_with_abcd() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFX");
        b.extend_from_slice(&56u32.to_be_bytes()); // form + svap + abcd + head = 4+12+12+28
        b.extend_from_slice(b"Egg!");
        // svap, size 4
        b.extend_from_slice(b"svap");
        b.extend_from_slice(&4u32.to_be_bytes());
        b.extend_from_slice(&[0x0F, 0x02, 0x86, 0x34]);
        // abcd, size 3 (odd -> 1 pad byte)
        b.extend_from_slice(b"abcd");
        b.extend_from_slice(&3u32.to_be_bytes());
        b.extend_from_slice(&[0x11, 0x22, 0x33]);
        b.push(0);
        // head, size 20
        b.extend_from_slice(b"head");
        b.extend_from_slice(&20u32.to_be_bytes());
        b.extend_from_slice(&[
            0x00, 0x5F, 0x00, 0x05, 0x0F, 0x02, 0x86, 0x34, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x10, 0x00, 0x00, 0x00, 0x20,
        ]);
        b
    }

    /// Preset from spec test (e). The 36 bytes of
    /// `52494658 0000001c 46614658 68656164 00000010 00000003 0000005d
    ///  0000002b 00000000`, written as a literal.
    fn preset_e() -> Vec<u8> {
        vec![
            0x52, 0x49, 0x46, 0x58, 0x00, 0x00, 0x00, 0x1c, 0x46, 0x61, 0x46, 0x58, 0x68, 0x65,
            0x61, 0x64, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x5d,
            0x00, 0x00, 0x00, 0x2b, 0x00, 0x00, 0x00, 0x00,
        ]
    }

    // (a) empty input
    #[test]
    fn a_empty() {
        let r = inspect(&[]);
        assert_eq!(r.kind, FileKind::Unknown);
        assert_eq!(r.head, None);
        assert_eq!(r.saved_by, None);
        assert_eq!(r.preset_format, None);
    }

    // (b) AppleDouble
    #[test]
    fn b_apple_double() {
        let mut b: Vec<u8> = vec![0x00, 0x05, 0x16, 0x07];
        b.resize(64, 0);
        let r = inspect(&b);
        assert_eq!(r.kind, FileKind::AppleDouble);
        assert_eq!(r.head, None);
        assert_eq!(r.saved_by, None);
        assert_eq!(r.preset_format, None);
    }

    // (c) XML project
    #[test]
    fn c_xml_project() {
        let xml = b"<?xml version=\"1.0\"?>\n<AfterEffectsProject xmlns=\"x\">";
        let r = inspect(xml);
        assert_eq!(r.kind, FileKind::XmlProject);
        assert_eq!(r.head, None);
        assert_eq!(r.saved_by, None);
        assert_eq!(r.preset_format, None);

        let mut with_bom: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
        with_bom.extend_from_slice(xml);
        let r = inspect(&with_bom);
        assert_eq!(r.kind, FileKind::XmlProject);

        let not_aep = b"<?xml version=\"1.0\"?><svg/>";
        let r = inspect(not_aep);
        assert_eq!(r.kind, FileKind::Unknown);
    }

    // (d) not a RIFX project
    #[test]
    fn d_not_project() {
        let mut riff: Vec<u8> = b"RIFF".to_vec();
        riff.resize(24, 0);
        assert_eq!(inspect(&riff).kind, FileKind::Unknown);

        let r = inspect(b"RIFX\0\0\0\x04WAVE");
        assert_eq!(r.kind, FileKind::Unknown);
    }

    // (e) preset
    #[test]
    fn e_preset() {
        let p = preset_e();
        assert_eq!(p.len(), 36);
        let r = inspect(&p);
        assert_eq!(r.kind, FileKind::Preset);
        assert_eq!(
            r.preset_format,
            Some(FormatVersion {
                major: 93,
                minor: 43
            })
        );
        assert_eq!(r.head, None);
        assert_eq!(r.saved_by, None);
    }

    // (f) hand-made project
    #[test]
    fn f_project() {
        let p = project_f();
        let r = inspect(&p);
        assert_eq!(r.kind, FileKind::Project);
        assert_eq!(r.saved_by, Some(AppVersion(0x0F02_8634)));
        let head = r.head.expect("head chunk parses");
        assert_eq!(
            head.format,
            FormatVersion {
                major: 95,
                minor: 5
            }
        );
        assert_eq!(head.app, AppVersion(0x0F02_8634));
        assert_eq!(r.preset_format, None);
    }

    // (g) project cut off after 30 bytes (inside the head chunk)
    #[test]
    fn g_truncated() {
        let p = project_f();
        assert!(p.len() >= 30);
        let r = inspect(&p[..30]);
        assert_eq!(r.kind, FileKind::Project);
        assert_eq!(r.saved_by, Some(AppVersion(0x0F02_8634)));
        assert_eq!(r.head, None);
        assert_eq!(r.preset_format, None);
    }

    // (h) odd-sized `abcd` chunk between `svap` and `head`
    #[test]
    fn h_odd_chunk() {
        let p = project_with_abcd();
        let r = inspect(&p);
        assert_eq!(r.kind, FileKind::Project);
        assert_eq!(r.saved_by, Some(AppVersion(0x0F02_8634)));
        let head = r.head.expect("head chunk is found past the odd chunk");
        assert_eq!(
            head.format,
            FormatVersion {
                major: 95,
                minor: 5
            }
        );
        assert_eq!(head.app, AppVersion(0x0F02_8634));
    }

    // (i) every prefix length of the (f) project must not panic
    #[test]
    fn i_all_prefixes_no_panic() {
        let p = project_f();
        for len in 0..=p.len() {
            let _ = inspect(&p[..len]);
        }
    }
}
