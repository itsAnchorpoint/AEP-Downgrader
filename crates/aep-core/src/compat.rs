//! Which target versions are offered, and how far each has been verified.
//!
//! The project promises only what was checked. A target is [`Status::Verified`]
//! when converted files were opened in a real After Effects of that version.
//! Otherwise it is [`Status::Experimental`], even if the output matches what
//! After Effects itself writes.
//!
//! One table feeds the app, the command line and the documentation.

use crate::version::{Head, release_name};

/// How far a target has been verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Verified,
    Experimental,
}

impl Status {
    /// `verified` or `experimental`.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Verified => "verified",
            Status::Experimental => "experimental",
        }
    }
}

/// One target version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub ae_major: u8,
    pub status: Status,
    /// One sentence saying what the status rests on.
    pub evidence: &'static str,
}

/// All targets, newest first.
///
/// | ae_major | status | evidence |
/// |----------|--------|----------|
/// | 26 | Experimental | `Matches the file After Effects 27.0 saves for 26.x except color settings of sources. Not yet opened in After Effects 2026.` |
/// | 25 | Experimental | `Matches the file After Effects 27.0 saves for 25.x except one migrated effect. Not yet opened in After Effects 2025.` |
/// | 24 | Verified | `Projects from 25.6 and 26.0 opened in After Effects 24.5.` |
/// | 23 | Experimental | `Matches the file After Effects 24.5 saves for 23.x except panel state. Not yet opened in After Effects 2023.` |
/// | 22 | Experimental | `Matches the file After Effects 24.5 saves for 22.x. Not yet opened in After Effects 2022.` |
pub const TARGETS: [Target; 5] = [
    Target {
        ae_major: 26,
        status: Status::Experimental,
        evidence: "Matches the file After Effects 27.0 saves for 26.x except color settings of sources. Not yet opened in After Effects 2026.",
    },
    Target {
        ae_major: 25,
        status: Status::Experimental,
        evidence: "Matches the file After Effects 27.0 saves for 25.x except one migrated effect. Not yet opened in After Effects 2025.",
    },
    Target {
        ae_major: 24,
        status: Status::Verified,
        evidence: "Projects from 25.6 and 26.0 opened in After Effects 24.5.",
    },
    Target {
        ae_major: 23,
        status: Status::Experimental,
        evidence: "Matches the file After Effects 24.5 saves for 23.x except panel state. Not yet opened in After Effects 2023.",
    },
    Target {
        ae_major: 22,
        status: Status::Experimental,
        evidence: "Matches the file After Effects 24.5 saves for 22.x. Not yet opened in After Effects 2022.",
    },
];

/// The entry of [`TARGETS`] for this After Effects major, if any.
pub fn target(ae_major: u8) -> Option<Target> {
    TARGETS.iter().find(|t| t.ae_major == ae_major).copied()
}

/// The targets a file with this head can be converted to, newest first:
/// the entries of [`TARGETS`] whose `ae_major` is in
/// [`crate::convert::targets_for`]`(head)`.
pub fn targets_for(head: &Head) -> Vec<Target> {
    let majors = crate::convert::targets_for(head);
    TARGETS
        .iter()
        .filter(|t| majors.contains(&t.ae_major))
        .copied()
        .collect()
}

/// The compatibility table as Markdown, for the documentation:
/// a header row `| Target | Status | Basis |`, the separator row
/// `|---|---|---|`, then one row per entry of [`TARGETS`] in order, formatted
/// `| After Effects <name> (<major>.x) | <Status, capitalized> | <evidence> |`,
/// where `<name>` comes from [`crate::version::release_name`] (use the major
/// number itself if there is no name). Lines are joined with `\n` and the
/// text ends with a single `\n`.
pub fn markdown_table() -> String {
    let mut lines = Vec::new();
    lines.push("| Target | Status | Basis |".to_string());
    lines.push("|---|---|---|".to_string());
    for target in TARGETS {
        let name = match release_name(target.ae_major) {
            Some(name) => name.to_string(),
            None => target.ae_major.to_string(),
        };
        let status = match target.status {
            Status::Verified => "Verified",
            Status::Experimental => "Experimental",
        };
        lines.push(format!(
            "| After Effects {} ({}.x) | {} | {} |",
            name, target.ae_major, status, target.evidence
        ));
    }
    let mut table = lines.join("\n");
    table.push('\n');
    table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version::{AppVersion, FormatVersion};

    #[test]
    fn status_as_str() {
        assert_eq!(Status::Verified.as_str(), "verified");
        assert_eq!(Status::Experimental.as_str(), "experimental");
    }

    #[test]
    fn target_lookup() {
        let t24 = target(24).expect("24 is known");
        assert_eq!(t24.status, Status::Verified);
        assert!(target(26).is_some());
        assert_eq!(target(27), None);
        assert_eq!(target(21), None);
        assert_eq!(
            TARGETS
                .iter()
                .filter(|t| t.status == Status::Verified)
                .count(),
            1,
            "exactly one entry of TARGETS is Verified"
        );
    }

    #[test]
    fn targets_for_by_format_major() {
        let head = |major: u16| Head {
            format: FormatVersion { major, minor: 0 },
            app: AppVersion(0x0F0B_0665),
        };
        let majors = |ts: Vec<Target>| -> Vec<u8> { ts.into_iter().map(|t| t.ae_major).collect() };
        assert_eq!(majors(targets_for(&head(98))), vec![26, 25, 24, 23, 22]);
        assert_eq!(majors(targets_for(&head(97))), vec![25, 24, 23, 22]);
        assert_eq!(majors(targets_for(&head(95))), vec![23, 22]);
        assert!(
            targets_for(&head(93)).is_empty(),
            "format major 93 has no targets"
        );
    }

    #[test]
    fn markdown_table_shape() {
        let table = markdown_table();
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 7, "table has 7 lines");
        assert_eq!(lines[0], "| Target | Status | Basis |");
        assert_eq!(lines[1], "|---|---|---|");
        assert_eq!(
            lines[4],
            "| After Effects 2024 (24.x) | Verified | Projects from 25.6 and 26.0 opened in After Effects 24.5. |"
        );
        assert!(table.ends_with('\n'), "ends with a newline");
        assert!(!table.ends_with("\n\n"), "ends with exactly one newline");
    }
}
