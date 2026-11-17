<div align="center">
  <img src="assets/icon.png" alt="AEP Downgrader logo" width="160" height="160">
  <h1>AEP Downgrader</h1>
  <p><strong>Open After Effects projects in an older version of After Effects.</strong></p>
</div>

---

AEP Downgrader makes a copy of an After Effects project (`.aep`) that an older After Effects can open. It does what After Effects does in **File > Save As > Save a Copy As > (older version)**, but without needing the newer version installed.

Free, open source ([GPLv3](LICENSE)), for Windows, macOS and Linux.

## Your projects stay on your computer

- Files are read and written on your machine only. There is no upload.
- There is no account and no telemetry.
- The only network request the app makes is the update check: at most once a day it reads a small file about the latest release from GitHub. You can turn it off in Settings. Updates are installed only when you click **Update**, and only if their signature matches.
- The source code is open, so you can check all of this. Details: [docs/PRIVACY.md](docs/PRIVACY.md).

## Download

> **Status.** Version 2.0 is not released yet. It is in development on the [`v2` branch](https://github.com/itsAnchorpoint/AEP-Downgrader/tree/v2). The current release is **1.2.0** (the older Python app). Get it from the [Releases page](https://github.com/itsAnchorpoint/AEP-Downgrader/releases). Version 1.x changes one byte in the file header and nothing else; 2.0 replaces that approach. See the [changelog](CHANGELOG.md).

When 2.0 is released, the Releases page will carry:

- Windows: an installer (`…_x64-setup.exe`), or a portable app (`…_x64-portable.exe`) that needs no installation and keeps its settings next to itself
- macOS: a disk image (`.dmg`), for Intel and Apple silicon
- Linux: an AppImage and a `.deb` package

Each release also has `SHA256SUMS.txt` and [build provenance attestations](https://github.com/itsAnchorpoint/AEP-Downgrader/attestations), so you can check that a file was built from this repository by GitHub Actions.

## Updates

The app checks for a new version at most once a day (you can turn this off, or check by hand, in Settings). When there is one, a bar at the top of the window says so, and **What's new** shows the release notes. **Update** downloads it, checks its signature and installs it; then restart the app. After an update the app shows what changed since the version you had.

The Windows installer and portable app, the macOS app in Applications and the Linux AppImage update themselves. The `.deb` package does not: the app links to the release page instead.

## How to use it

1. Open the app and drop your `.aep` files (or a folder of them) onto the window. You can also click to browse.
2. The app shows which After Effects version saved each file.
3. Pick the version you want to open the file in. Only versions older than the file are offered.
4. Read the **Before you convert** panel. It lists what will change, and what will be lost.
5. Click **Convert**. A copy is saved next to the original, named like `project_AE2024.aep`. Originals are never modified.

You can change where copies go, the file name pattern, and what happens when the name already exists, in Settings.

## Compatibility

A target is **Verified** only if converted files were opened in a real After Effects of that version. Otherwise it is **Experimental**, even when the output matches what After Effects itself writes.

| Target | Status | Basis |
|---|---|---|
| After Effects 2026 (26.x) | Experimental | Matches the file After Effects 27.0 saves for 26.x except colour settings of sources. Not yet opened in After Effects 2026. |
| After Effects 2025 (25.x) | Experimental | Matches the file After Effects 27.0 saves for 25.x except one migrated effect. Not yet opened in After Effects 2025. |
| After Effects 2024 (24.x) | Verified | Projects from 25.6 and 26.0 opened in After Effects 24.5. |
| After Effects 2023 (23.x) | Experimental | Matches the file After Effects 24.5 saves for 23.x except panel state. Not yet opened in After Effects 2023. |
| After Effects 2022 (22.x) | Experimental | Matches the file After Effects 24.5 saves for 22.x. Not yet opened in After Effects 2022. |

Full explanation, and what blocks a conversion: [docs/COMPATIBILITY.md](docs/COMPATIBILITY.md).

## What changes, and what can be lost

- A conversion cannot add anything. Effects and features that exist only in the newer version do not appear in the older one. The app lists what it will remove before you convert.
- Third-party plugins stay in the file, but must be installed in the older After Effects to open the project.
- After Effects 2022 cannot express a track matte that is not the layer directly above the layer that uses it. Such a file is not converted to 2022. Choose 2023 or newer, or move each matte layer directly above its target.
- Presets (`.ffx`) are not converted yet.
- Keep your originals. Experimental targets in particular may not open.

## First launch

The apps are not code-signed, so each system warns you the first time.

- **Windows:** SmartScreen says "Windows protected your PC". Click **More info**, then **Run anyway**.
- **macOS:** the system refuses to open the app. Open **System Settings > Privacy & Security**, scroll down to the message about AEP Downgrader and click **Open Anyway**.
- **Linux:** an AppImage must be executable. In a terminal: `chmod +x AEP-Downgrader*.AppImage`, then run it. Or right-click the file, open Properties, and allow executing it.

## Command line

The `aep` tool does the same work from a terminal. Build it with `cargo build --release -p aep-cli`, or run it through `cargo run -p aep-cli --`.

```text
aep targets <file>                      which versions this file can be downgraded to
aep convert <in> <out> --to <major>     convert (24 means After Effects 2024); refuses to overwrite <out>
aep compat                              print the compatibility table
aep tree <file>                         print the chunk tree of a project
aep roundtrip <file>                    check that the file parses and writes back byte for byte
aep diff [--full] [--no-noise] <a> <b>  structural difference between two projects
```

## How to help

- **Run the tester kit.** Only After Effects 2024 is available to the maintainer, so the other targets are Experimental until someone opens converted files in the real version. The kit is a few scripts that build a small test project with nothing personal in it: [tools/ae/README.md](tools/ae/README.md).
- **Report a file that is not recognised** ("Unknown version", or a conversion that fails) by [opening an issue](https://github.com/itsAnchorpoint/AEP-Downgrader/issues). Say which After Effects version and system saved it. Do **not** attach a confidential project. The issue only needs to say that you have such a file, and a private way to send it can be arranged. Project files contain the paths of your footage, so treat them as private.
- **Contribute code.** See [CONTRIBUTING.md](CONTRIBUTING.md).

Thank you to the volunteers who tested. Special thanks to Shiiru, who supplied files from After Effects 27.0 on macOS, and to Павел Бизонов, who supplied files from After Effects 26.2 on macOS.

## Build from source

You need Rust (stable), Node.js 22 and pnpm. On Linux you also need the WebKitGTK development packages (see `.github/workflows/ci.yml` for the list).

```bash
git clone https://github.com/itsAnchorpoint/AEP-Downgrader.git
cd AEP-Downgrader
git checkout v2

# core and command line
cargo test

# desktop app
cd apps/desktop
pnpm install
pnpm tauri dev      # run
pnpm tauri build    # installers for your system
```

## For developers

How the file format works and how the conversion rules were found: [docs/FORMAT.md](docs/FORMAT.md). How the app and its assets should look: [docs/DESIGN.md](docs/DESIGN.md).

## Licence and trademarks

GNU General Public License v3.0. See [LICENSE](LICENSE).

Adobe and After Effects are trademarks of Adobe. This project is not affiliated with, endorsed by or sponsored by Adobe.
