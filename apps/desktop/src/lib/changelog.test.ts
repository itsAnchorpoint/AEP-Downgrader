import { describe, expect, it } from "vitest";
import changelog from "../content/whatsnew.en.md?raw";
import { changelogSince, compareVersions, hasChangesSince, headingVersion, notesFor, RU_NOTES_MARKER } from "./changelog";

const SAMPLE = `# Changelog

All notable changes.

## [2.1.0] - 2026-12-01

### Added

- Presets.

## [2.0.1] - 2026-11-25

### Fixed

- A crash.

## [2.0.0] - 2026-11-20

### Added

- Everything.
`;

describe("compareVersions", () => {
  it("orders releases and pre-releases like semver", () => {
    const sorted = ["2.0.1", "2.0.0", "2.0.0-beta.10", "2.0.0-beta.2", "2.0.0-alpha.0", "1.2.0", "2.0.0-beta"];
    sorted.sort(compareVersions);
    expect(sorted).toEqual(["1.2.0", "2.0.0-alpha.0", "2.0.0-beta", "2.0.0-beta.2", "2.0.0-beta.10", "2.0.0", "2.0.1"]);
  });

  it("ignores a leading v and treats equal versions as equal", () => {
    expect(compareVersions("v2.0.0", "2.0.0")).toBe(0);
    expect(compareVersions("10.0.0", "9.9.9")).toBe(1);
  });
});

describe("headingVersion", () => {
  it("reads released and unreleased headings", () => {
    expect(headingVersion("## [2.0.1] - 2026-11-25")).toBe("2.0.1");
    expect(headingVersion("## [Unreleased] 2.0.0")).toBe("2.0.0");
    expect(headingVersion("## [Unreleased]")).toBeNull();
    expect(headingVersion("### Added")).toBeNull();
  });
});

describe("changelogSince", () => {
  it("keeps only the newer sections, without the title and introduction", () => {
    const text = changelogSince(SAMPLE, "2.0.0");
    expect(text).not.toContain("# Changelog");
    expect(text).not.toContain("All notable changes");
    expect(text).toContain("## [2.1.0]");
    expect(text).toContain("- A crash.");
    expect(text).not.toContain("## [2.0.0]");
    expect(text).not.toContain("- Everything.");
  });

  it("returns every section without a starting version, but no introduction", () => {
    const text = changelogSince(SAMPLE, null);
    expect(text.startsWith("## ")).toBe(true);
    expect(text).toContain("- Everything.");
    expect(text).not.toContain("All notable changes");
  });

  it("picks the notes in the app language from the feed", () => {
    const notes = `- English\n\n${RU_NOTES_MARKER}\n\n- Русский`;
    expect(notesFor(notes, "ru")).toBe("- Русский");
    expect(notesFor(notes, "en")).toBe("- English");
    expect(notesFor("- Only English", "ru")).toBe("- Only English");
  });

  it("reports whether anything is newer", () => {
    expect(hasChangesSince(SAMPLE, "2.0.1")).toBe(true);
    expect(hasChangesSince(SAMPLE, "2.1.0")).toBe(false);
  });

  it("finds the current release in the bundled notes", () => {
    expect(hasChangesSince(changelog, "1.2.0")).toBe(true);
  });
});
