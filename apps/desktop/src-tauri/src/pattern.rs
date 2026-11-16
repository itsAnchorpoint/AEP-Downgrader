//! Output file name from the user's pattern. Must behave exactly like
//! `src/lib/pattern.ts` (used for the live preview in Settings); the tests
//! below mirror `pattern.test.ts` case by case.

const DEFAULT_PATTERN: &str = "{name}_AE{year}";

fn is_blank(c: char) -> bool {
    // JavaScript's trim() also removes the byte order mark.
    c.is_whitespace() || c == '\u{feff}'
}

/// Characters Windows forbids in file names become `_`; trailing dots and spaces go.
fn clean(text: &str) -> String {
    let replaced: String = text
        .chars()
        .map(|c| if "<>:\"/\\|?*".contains(c) || c < '\u{20}' { '_' } else { c })
        .collect();
    replaced.trim_end_matches(['.', ' ']).to_string()
}

/// `{name}` is the original name without its extension, `{year}` the target version name.
pub fn output_name(pattern: &str, file_name: &str, year: u32) -> String {
    let (stem, ext) = match file_name.rfind('.') {
        Some(dot) if dot > 0 => file_name.split_at(dot),
        _ => (file_name, ""),
    };
    let fill = |p: &str| p.replace("{name}", stem).replace("{year}", &year.to_string());
    let trimmed = pattern.trim_matches(is_blank);
    let base = clean(&fill(if trimmed.is_empty() { DEFAULT_PATTERN } else { trimmed }));
    let base = if base.is_empty() { clean(&fill(DEFAULT_PATTERN)) } else { base };
    base + ext
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_name_and_year_and_keeps_the_extension() {
        assert_eq!(output_name("{name}_AE{year}", "Intro_v05.aep", 2024), "Intro_v05_AE2024.aep");
    }

    #[test]
    fn keeps_only_the_last_extension() {
        assert_eq!(output_name("{name}-{year}", "my.comp.v2.aet", 2023), "my.comp.v2-2023.aet");
    }

    #[test]
    fn empty_pattern_falls_back_to_the_default() {
        assert_eq!(output_name("   ", "A.ffx", 2025), output_name(DEFAULT_PATTERN, "A.ffx", 2025));
    }

    #[test]
    fn illegal_characters_are_replaced() {
        assert_eq!(output_name("{name}:{year}/x", "A.aep", 2024), "A_2024_x.aep");
    }

    #[test]
    fn files_without_an_extension() {
        assert_eq!(output_name("{name}_{year}", "README", 2024), "README_2024");
    }
}
