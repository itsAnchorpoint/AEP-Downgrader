# Changelog

All notable changes to AEP Downgrader. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased] 2.0.0

AEP Downgrader 2.0 is a rewrite. It no longer just edits the file header. It makes the same changes After Effects makes when you choose **Save a Copy As** an older version.

### Added

- A new desktop app for Windows, macOS and Linux. Drop files or whole folders, see which After Effects version saved each one, and convert several at once.
- A **Before you convert** panel. It lists what will change for each file and target, and what will be lost, before anything is written.
- Conversion to After Effects 2026, 2025, 2024, 2023 and 2022 (see the compatibility table in the README).
- After Effects 27 projects can be converted.
- Templates (`.aet`) and animation presets (`.ffx`) are converted too, with the same rules as projects.
- Every target is labelled **Verified** or **Experimental**. Verified means converted files were opened in a real After Effects of that version. Today that is 2024 only.
- Settings: where to save copies, the file name pattern, what to do when a file already exists (add a number, overwrite or skip), and automatic update checks on or off.
- Updates inside the app. It checks GitHub at most once a day, shows the release notes, and installs a new version when you click **Update**, after checking its signature. After an update it shows what changed. Works for the Windows installer and portable app, the macOS app and the Linux AppImage.
- A portable Windows version: one `.exe`, no installation, settings kept next to it.
- A Windows installer in the app's design: one window and one click, in English or Russian, with no administrator rights needed. Updates and uninstalling use the same window.
- An activity log of what the app did in this session, and **Copy report** for bug reports, with file names hidden by default.
- Light, dark and After Effects-style themes, five accent colours, English and Russian.
- A check for track mattes that are not the layer directly above their layer. After Effects 2022 cannot represent them, so the conversion to 2022 is stopped instead of producing a file that may not open correctly.
- A command-line tool, `aep`, to convert, inspect and compare projects.
- A tester kit for After Effects users who want to help check the older versions.
- Documentation of the file format, the compatibility rules and privacy.

### Changed

- Converted copies now follow what After Effects itself writes, not only a changed version number. Removed features are listed before you convert.
- Copies are named like `project_AE2024.aep`. In 1.x the name looked like `project_AE24x.aep`. You can change the pattern in Settings.
- If the copy name is taken, the app adds a number instead of overwriting the earlier copy. This is the default and can be changed.
- Files are identified by reading their structure, not a fixed position in the file.
- Windows installer and portable app, macOS disk image, Linux AppImage and `.deb` package replace the 1.x zip and tar.gz archives. Each release lists SHA-256 checksums and carries build provenance attestations.
- Everything is processed on your computer. There is no account and no telemetry. The only network request is the update check, which can be turned off.

### Fixed

- Targets "After Effects 21.x" and "20.x" in 1.2.0 reported success but saved a copy without any change. They are gone (see Removed).
- Copies are written under a temporary name and moved into place, so a failed conversion does not leave a half-written file.

### Removed

- The Python and PyQt5 application of version 1.x.
- The 1.x targets "After Effects 21.x" and "20.x". There are no such After Effects releases, and the conversion did nothing.
- The 1.x **Debug** menu and log viewer.

## [1.2.0] - 2026-04-07

### Added

- Target versions down to 20.x, with Stable and Experimental labels. Source versions from 20.x to 33.x are detected.
- Notification when a newer release is available on GitHub.

### Changed

- Better file picker, and a fixed window size.

### Fixed

- macOS app packaging and icon in the build.

## [1.1.0] - 2026-02-25

### Added

- Detection of any After Effects version from 22 up, including 25.x and 26.x files.
- A target choice for After Effects 25.x.
- Debug mode with a log viewer and report export.

### Changed

- The licence changed from MIT to GPLv3.
- macOS builds are a `.app` bundle in a `.dmg`.

## [1.0.0] - 2026-01-19

First release. A PyQt5 app with a dark interface.

### Added

- Conversion of `.aep` projects to older After Effects versions (23.x, 24.x, 25.x), by changing the version in the file header.
- Detection of the source version.
- Batch conversion of several files.
- Builds for Windows, macOS and Linux.

[Unreleased]: https://github.com/itsAnchorpoint/AEP-Downgrader/compare/v1.2.0...v2
[1.2.0]: https://github.com/itsAnchorpoint/AEP-Downgrader/releases/tag/v1.2.0
[1.1.0]: https://github.com/itsAnchorpoint/AEP-Downgrader/commit/5658445
[1.0.0]: https://github.com/itsAnchorpoint/AEP-Downgrader/releases/tag/v1.0.0
