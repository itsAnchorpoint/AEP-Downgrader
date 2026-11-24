# Compatibility

What AEP Downgrader can convert to, how far each target has been checked, and what a conversion changes.

## The table

| Target | Status | Basis |
|---|---|---|
| After Effects 2026 (26.x) | Experimental | Matches the file After Effects 27.0 saves for 26.x except color settings of sources. Not yet opened in After Effects 2026. |
| After Effects 2025 (25.x) | Experimental | Matches the file After Effects 27.0 saves for 25.x except one migrated effect. Not yet opened in After Effects 2025. |
| After Effects 2024 (24.x) | Verified | Projects from 25.6 and 26.0 opened in After Effects 24.5. |
| After Effects 2023 (23.x) | Experimental | Matches the file After Effects 24.5 saves for 23.x except panel state. Not yet opened in After Effects 2023. |
| After Effects 2022 (22.x) | Experimental | Matches the file After Effects 24.5 saves for 22.x. Not yet opened in After Effects 2022. |

This is the table the tool prints (`aep compat`). The app, the command line and this page read the same source, so they cannot disagree.

## What the statuses mean

**Verified.** Converted files were opened in a real After Effects of that version.

**Experimental.** Converted files were not opened in a real After Effects of that version yet. The output may still match what After Effects itself writes for that version. The "Basis" column says what was compared and where it differs. Keep your originals.

The project does not call a target Verified because the output looks right. Only opening the file in the real application counts.

What "Verified" means for 2024 in detail: projects saved by After Effects 25.6 and 26.0, among them a real working project with 25 compositions, 221 layers, 206 effects and 406 expressions, were converted and opened in After Effects 24.5 with no expression errors. A project, a template project and an animation preset saved by After Effects 27.0 were also converted and opened (the preset applied) in After Effects 24.5. After Effects 24.5 is the only After Effects the maintainer has, so that is the only version tested directly.

## Projects, templates and presets

The same targets and rules apply to all three kinds of file:

- **Projects (`.aep`)** and **template projects (`.aet`)** have the same structure. When you open a converted template, After Effects asks where to save it as a project, as it does with any template.
- **Animation presets (`.ffx`)** hold effects and their settings. The app removes from them what the target version does not have, the same as in a project. A preset carries only a format number, not a full version, so the app shows the After Effects release from the format (a preset from After Effects 27 shows as 2027).

## Which targets are offered for a file

Only versions older than the version that saved the file.

| File saved by | Targets offered |
|---|---|
| After Effects 27 | 2026, 2025, 2024, 2023, 2022 |
| After Effects 26 | 2025, 2024, 2023, 2022 |
| After Effects 25 | 2024, 2023, 2022 |
| After Effects 24 | 2023, 2022 |
| After Effects 23 | 2022 |
| After Effects 22 or older | none |

Files from versions newer than the converter knows are refused rather than guessed at.

## What each step changes

A downgrade passes through every version between the file and the target. Each step applies the changes below. The rules reproduce what After Effects itself does in **Save a Copy As**; [FORMAT.md](FORMAT.md) explains how they were found.

| Leaving | What changes |
|---|---|
| After Effects 27 (to 26) | Data that only After Effects 27 writes is removed. 3D material settings that 26 does not have (a random offset) are removed. |
| After Effects 26 (to 25) | Data that only After Effects 26 writes is removed. 3D material settings that 25 does not have are removed. |
| After Effects 25 (to 24) | Nothing besides the version stamp. |
| After Effects 24 (to 23) | The **Shadow Color** property is removed from the layers that have it. |
| After Effects 23 (to 22) | Light environment settings are removed from lights. A layer style option that 2022 does not have is removed. Layer records are stored in the older, shorter form (no visible change). A track matte that is not directly above its layer blocks the conversion (see below). |

In every case the version stamp in the file header is rewritten to the one After Effects itself writes for the target. The platform recorded in the file (Windows, macOS Intel, macOS Apple silicon) is kept.

The app shows the changes that apply to your file in its report (**Show report** next to the file), before anything is written.

## What a conversion cannot do

- **It cannot make a feature exist in an older version.** Effects and features that only exist in the newer version do not appear in the older one. Where the converter knows about such a feature (for example, the ones listed above) it removes it and tells you. It does not rebuild it.
- **Third-party plugins are kept in the file.** The older After Effects needs them installed to open the project.
- **XML projects (`.aepx`) are not supported.**
- **Some of After Effects's own changes are not reproduced yet.** They are the "except" in the Basis column:
  - 2023: when After Effects removes properties it also shortens the saved panel outline state (the `otln` chunk).
  - 2026 and 2025: After Effects 27 also rewrites the color settings of footage sources and migrates one effect (Compound Blur) when it saves for an older version.

## What blocks a conversion

- **A track matte that is not the layer directly above** (to After Effects 2022 only). If a layer uses a matte that sits somewhere else in the stack, After Effects 2022 cannot represent it. Nothing is written. Choose 2023 or newer, or move each matte layer directly above the layer that uses it, then convert again.
- **A file the converter does not fully understand.** If the file cannot be parsed and written back byte for byte, it is not converted, to avoid damaging it.
- **A file from a newer version than the converter knows**, or one that is already saved in the target version or older.
- **A damaged or incomplete file.**

## How to help verify a target

Experimental targets become Verified when someone opens converted files in that version of After Effects.

The tester kit in [`tools/ae`](../tools/ae/README.md) builds a small test project with nothing personal in it, saves the older-version copies that your After Effects offers, and checks that everything opens. You send back a zip file. About 10 to 15 minutes. Versions the maintainer lacks, such as 2022, 2023, 2025 and 2026, are the most useful, and so is After Effects on macOS.

You can also convert one of your own projects to your version and tell us whether it opens, and whether After Effects shows any warning. Open an [issue](https://github.com/itsAnchorpoint/AEP-Downgrader/issues) with the After Effects version and system you used.
