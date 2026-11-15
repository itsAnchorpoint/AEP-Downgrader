//! After Effects version numbers as stored in project files.
//!
//! A project's `head` chunk starts with eight bytes (big-endian):
//!
//! ```text
//! head[0..2]  u16  file-format major
//! head[2..4]  u16  file-format minor
//! head[4..8]  u32  version word of the After Effects release the file is written for
//! ```
//!
//! The `svap` chunk holds one version word: the application that saved the file.
//!
//! Version word, bit 31 = most significant bit:
//!
//! ```text
//! bit  31      reserved (0 in every known file)
//! bits 30..=26 major, high part (5 bits)
//! bits 25..=22 platform code (4 bits): 12 Windows, 13 macOS Intel, 14 macOS Apple silicon
//! bits 21..=19 major, low part (3 bits)      major = high * 8 + low
//! bits 18..=15 minor (4 bits)
//! bits 14..=11 patch (4 bits)
//! bit  10      reserved (1 in every known file)
//! bit   9      1 = release build, 0 = beta build
//! bit   8      reserved (0 in every known file)
//! bits  7..=0  build number
//! ```
//!
//! Example: `0x0F0B0665` is 25.6.0 build 101, Windows, release.

use std::fmt;

/// Platform a file was saved on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacIntel,
    MacArm,
    /// Any other 4-bit code.
    Unknown(u8),
}

impl Platform {
    /// 12 → Windows, 13 → MacIntel, 14 → MacArm, anything else → Unknown(code & 0x0F).
    pub fn from_code(code: u8) -> Platform {
        match code & 0x0F {
            12 => Platform::Windows,
            13 => Platform::MacIntel,
            14 => Platform::MacArm,
            other => Platform::Unknown(other),
        }
    }

    /// Inverse of [`Platform::from_code`]. Always in `0..=15`.
    pub fn code(self) -> u8 {
        match self {
            Platform::Windows => 12,
            Platform::MacIntel => 13,
            Platform::MacArm => 14,
            Platform::Unknown(code) => code & 0x0F,
        }
    }
}

/// A version word. The raw `u32` is the source of truth, so reserved bits
/// survive a read-modify-write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppVersion(pub u32);

impl AppVersion {
    pub fn major(self) -> u8 {
        let word = self.0;
        let high = ((word >> 26) & 0x1F) as u8;
        let low = ((word >> 19) & 0x07) as u8;
        high * 8 + low
    }

    pub fn minor(self) -> u8 {
        ((self.0 >> 15) & 0x0F) as u8
    }

    pub fn patch(self) -> u8 {
        ((self.0 >> 11) & 0x0F) as u8
    }

    pub fn build(self) -> u8 {
        (self.0 & 0xFF) as u8
    }

    pub fn platform(self) -> Platform {
        let code = ((self.0 >> 22) & 0x0F) as u8;
        Platform::from_code(code)
    }

    /// `true` for a release build, `false` for a beta build.
    pub fn is_release(self) -> bool {
        (self.0 >> 9) & 0x1 != 0
    }

    /// Same word with only the four platform bits replaced.
    pub fn with_platform(self, platform: Platform) -> AppVersion {
        let word = self.0;
        let code = platform.code() & 0x0F;
        let cleared = word & !(0x0F << 22);
        let set = cleared | ((code as u32) << 22);
        AppVersion(set)
    }
}

/// Formats as `<major>.<minor>.<patch>x<build>`, e.g. `25.6.0x101`.
impl fmt::Display for AppVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}x{}",
            self.major(),
            self.minor(),
            self.patch(),
            self.build()
        )
    }
}

/// File-format version. Ordered by major, then minor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FormatVersion {
    pub major: u16,
    pub minor: u16,
}

/// Formats as `<major>.<minor>`, e.g. `96.9`.
impl fmt::Display for FormatVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// The first eight bytes of a `head` chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Head {
    pub format: FormatVersion,
    pub app: AppVersion,
}

impl Head {
    /// Read from `head` chunk data. `None` if `data` is shorter than 8 bytes.
    pub fn parse(data: &[u8]) -> Option<Head> {
        if data.len() < 8 {
            return None;
        }
        let major = u16::from_be_bytes([data[0], data[1]]);
        let minor = u16::from_be_bytes([data[2], data[3]]);
        let word = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        Some(Head {
            format: FormatVersion { major, minor },
            app: AppVersion(word),
        })
    }

    /// Overwrite the first eight bytes of `data` with this head and return
    /// `true`. If `data` is shorter than 8 bytes, change nothing and return
    /// `false`. Bytes from index 8 on are never touched.
    pub fn write_into(&self, data: &mut [u8]) -> bool {
        if data.len() < 8 {
            return false;
        }
        data[0..2].copy_from_slice(&self.format.major.to_be_bytes());
        data[2..4].copy_from_slice(&self.format.minor.to_be_bytes());
        data[4..8].copy_from_slice(&self.app.0.to_be_bytes());
        true
    }
}

/// Marketing name of an After Effects major version, or `None` if unknown.
///
/// 13 → "CC 2014/2015", 14 → "CC 2017", 15 → "CC 2018", 16 → "CC 2019",
/// 17 → "2020", 18 → "2021", 22 → "2022", 23 → "2023", 24 → "2024",
/// 25 → "2025", 26 → "2026". Everything else (including 19, 20, 21 and 27) → `None`.
pub fn release_name(major: u8) -> Option<&'static str> {
    match major {
        13 => Some("CC 2014/2015"),
        14 => Some("CC 2017"),
        15 => Some("CC 2018"),
        16 => Some("CC 2019"),
        17 => Some("2020"),
        18 => Some("2021"),
        22 => Some("2022"),
        23 => Some("2023"),
        24 => Some("2024"),
        25 => Some("2025"),
        26 => Some("2026"),
        _ => None,
    }
}

/// What After Effects itself writes into `head` when it saves a copy for an
/// older version. `word` carries the Windows platform code; use
/// [`AppVersion::with_platform`] to keep the source file's platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stamp {
    pub ae_major: u8,
    pub format: FormatVersion,
    pub word: AppVersion,
}

/// Known stamps, newest first:
///
/// | ae_major | format | word         |
/// |----------|--------|--------------|
/// | 25       | 96.9   | `0x0F0B0661` |
/// | 24       | 95.6   | `0x0F030641` |
/// | 23       | 94.9   | `0x0B3B0637` |
/// | 22       | 93.43  | `0x0B33063B` |
pub const STAMPS: [Stamp; 4] = [
    Stamp {
        ae_major: 25,
        format: FormatVersion {
            major: 96,
            minor: 9,
        },
        word: AppVersion(0x0F0B_0661),
    },
    Stamp {
        ae_major: 24,
        format: FormatVersion {
            major: 95,
            minor: 6,
        },
        word: AppVersion(0x0F03_0641),
    },
    Stamp {
        ae_major: 23,
        format: FormatVersion {
            major: 94,
            minor: 9,
        },
        word: AppVersion(0x0B3B_0637),
    },
    Stamp {
        ae_major: 22,
        format: FormatVersion {
            major: 93,
            minor: 43,
        },
        word: AppVersion(0x0B33_063B),
    },
];

/// The stamp for an After Effects major version, if known.
pub fn stamp_for(ae_major: u8) -> Option<Stamp> {
    STAMPS
        .iter()
        .find(|stamp| stamp.ae_major == ae_major)
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_rows() {
        // word, major, minor, patch, build, platform, is_release
        let rows: [(u32, u8, u8, u8, u8, Platform, bool); 9] = [
            (0x0B33_063B, 22, 6, 0, 59, Platform::Windows, true),
            (0x0B3B_0637, 23, 6, 0, 55, Platform::Windows, true),
            (0x0F02_8634, 24, 5, 0, 52, Platform::Windows, true),
            (0x0F0A_0656, 25, 4, 0, 86, Platform::Windows, true),
            (0x0F0B_0665, 25, 6, 0, 101, Platform::Windows, true),
            (0x0F10_0643, 26, 0, 0, 67, Platform::Windows, true),
            (0x0B39_0E03, 23, 2, 1, 3, Platform::Windows, true),
            (0x0F88_0614, 25, 0, 0, 20, Platform::MacArm, true),
            (0x0F08_0404, 25, 0, 0, 4, Platform::Windows, false),
        ];
        for (word, major, minor, patch, build, platform, release) in rows {
            let v = AppVersion(word);
            assert_eq!(v.major(), major, "word {word:#010x}");
            assert_eq!(v.minor(), minor, "word {word:#010x}");
            assert_eq!(v.patch(), patch, "word {word:#010x}");
            assert_eq!(v.build(), build, "word {word:#010x}");
            assert_eq!(v.platform(), platform, "word {word:#010x}");
            assert_eq!(v.is_release(), release, "word {word:#010x}");
        }
    }

    #[test]
    fn display_impls() {
        assert_eq!(AppVersion(0x0F0B_0665).to_string(), "25.6.0x101");
        let fv = FormatVersion {
            major: 96,
            minor: 9,
        };
        assert_eq!(fv.to_string(), "96.9");
    }

    #[test]
    fn with_platform_roundtrip() {
        let original = AppVersion(0x0F0B_0665);
        let as_mac = original.with_platform(Platform::MacArm);
        assert_eq!(as_mac, AppVersion(0x0F8B_0665));
        assert_eq!(as_mac.with_platform(Platform::Windows), original);
        for code in 0..=15u8 {
            assert_eq!(Platform::from_code(code).code(), code);
        }
    }

    #[test]
    fn head_parse() {
        let data = [
            0x00, 0x60, 0x00, 0x09, 0x0F, 0x0B, 0x06, 0x65, 0x80, 0, 0, 0, 0, 0, 0x06, 0xD6, 0x07,
            0xB0, 0xB3, 0x3D,
        ];
        let head = Head::parse(&data).expect("20 bytes should parse");
        assert_eq!(
            head.format,
            FormatVersion {
                major: 96,
                minor: 9
            }
        );
        assert_eq!(head.app, AppVersion(0x0F0B_0665));
        assert_eq!(Head::parse(&data[..7]), None);
    }

    #[test]
    fn head_write_into() {
        let mut data = [
            0x00, 0x60, 0x00, 0x09, 0x0F, 0x0B, 0x06, 0x65, 0x80, 0, 0, 0, 0, 0, 0x06, 0xD6, 0x07,
            0xB0, 0xB3, 0x3D,
        ];
        let tail = data[8..].to_vec();
        let head = Head {
            format: FormatVersion {
                major: 95,
                minor: 6,
            },
            app: AppVersion(0x0F03_0641),
        };
        assert!(head.write_into(&mut data));
        let expected_prefix = [0x00, 0x5F, 0x00, 0x06, 0x0F, 0x03, 0x06, 0x41];
        assert_eq!(&data[..8], &expected_prefix);
        assert_eq!(&data[8..], &tail);

        let short = [1u8, 2, 3, 4, 5, 6, 7];
        let mut short_mut = short;
        assert!(!head.write_into(&mut short_mut));
        assert_eq!(short_mut, short);
    }

    #[test]
    fn release_names() {
        assert_eq!(release_name(24), Some("2024"));
        assert_eq!(release_name(18), Some("2021"));
        assert_eq!(release_name(16), Some("CC 2019"));
        assert_eq!(release_name(21), None);
        assert_eq!(release_name(27), None);
    }

    #[test]
    fn stamps() {
        let s24 = stamp_for(24).expect("24 should be known");
        assert_eq!(
            s24.format,
            FormatVersion {
                major: 95,
                minor: 6
            }
        );
        assert_eq!(s24.word, AppVersion(0x0F03_0641));
        for stamp in STAMPS {
            assert_eq!(stamp.word.major(), stamp.ae_major);
        }
        assert_eq!(stamp_for(26), None);
    }

    #[test]
    fn format_version_ordering() {
        let a = FormatVersion {
            major: 93,
            minor: 43,
        };
        let b = FormatVersion {
            major: 94,
            minor: 1,
        };
        let c = FormatVersion {
            major: 96,
            minor: 6,
        };
        let d = FormatVersion {
            major: 96,
            minor: 9,
        };
        assert!(a < b);
        assert!(c < d);
    }
}
