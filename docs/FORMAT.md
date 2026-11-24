# The After Effects project format, as far as we know it

For developers. This page records what AEP Downgrader relies on, how the conversion rules were found, and how they were tested. It replaces the technical document of version 1.x.

Each section says whether a statement is **measured** (seen in real files saved by After Effects, or in a real After Effects) or **not understood**. Adobe has not published the format. Everything here comes from looking at files.

The code that matches this page is in `crates/aep-core/src`: `riff.rs`, `version.rs`, `inspect.rs`, `diff.rs`, `convert.rs`, `compat.rs`. Their module comments are the exact specification.

## 1. The container (measured)

An `.aep` file is a **RIFX** file: RIFF with big-endian integers.

```text
"RIFX"  u32 body_size  body[body_size]  trailer[..]
body  = form[4]  chunk*
chunk = id[4]  u32 size  data[size]  pad[size % 2]      (pad byte is 0x00)
list  = chunk whose id is "LIST"; its data = kind[4] chunk*
```

- The form of a project is `Egg!`, and so is the form of a template project (`.aet`). The form of an animation preset (`.ffx`) is `FaFX`.
- A chunk with an odd size is followed by one zero byte that is not counted in the size.
- `LIST` chunks nest, at any depth.
- After the body there can be a trailer. After Effects stores an XMP packet there. We keep it as it is.

The parser keeps a tree and writes it back byte for byte: `parse(bytes).to_bytes() == bytes` for every file it accepts. The converter checks this before it changes anything, and refuses a file that does not round-trip. `aep roundtrip <file>` runs the same check.

Two kinds of `LIST` are kept verbatim instead of being parsed: lists of kind `btdk`, and any list whose content does not parse cleanly as chunks. They are not altered.

Two other things that are not projects are recognized by their first bytes: a macOS AppleDouble sidecar file (`._name.aep`, starts with `00 05 16 07`) and an XML project (`.aepx`).

### Start of a file (measured)

In the six reference projects we compared, the first chunks are in this order:

```text
off 0   "RIFX"   off 4  u32 size      off 8  "Egg!"
off 12  "svap"   off 16 = 4           off 20..23  version word of the program that saved the file
off 24  "head"   off 28 = 20          off 32..51  head data
off 52  "nhed" ...
```

Version 1.x read the head at a fixed offset (32). Version 2 walks the chunks instead, and does not rely on these offsets.

## 2. The `head` chunk (measured, except where marked)

`head` data is 20 bytes in every file we have seen. Offsets below are inside the chunk.

| Bytes | Meaning |
|---|---|
| `[0:2]` u16 | **Format major.** After Effects major + 71, for After Effects 22 to 26 (93 to 97). |
| `[2:4]` u16 | **Format minor.** Changes between minor releases. Not monotonic across majors. |
| `[4:8]` u32 | **Version word** of the After Effects release the file is written for. |
| `[8:20]` | **Not understood.** Seen as `80 00 00 00 ...`. We never change it. |

All integers are big-endian.

### The version word

The same layout is used by the `svap` chunk. Bit 31 is the most significant bit.

```text
bit  31      reserved (0 in every known file)
bits 30..26  major, high part (5 bits)
bits 25..22  platform code (4 bits)
bits 21..19  major, low part (3 bits)        major = high * 8 + low
bits 18..15  minor (4 bits)
bits 14..11  patch (4 bits)
bit  10      reserved (1 in every known file)
bit   9      1 = release build, 0 = beta build
bit   8      reserved (0 in every known file)
bits  7..0   build number
```

Example: `0x0F0B0665` is 25.6.0 build 101, Windows, release.

Platform codes used by the code: 12 Windows, 13 macOS Intel, 14 macOS Apple silicon. All our native reference files are from Windows; the macOS codes are less checked.

The reserved bits are copied through unchanged when a word is rewritten.

### `head` and `svap` are different things

- `head[4:8]` is the **format** the file is written in.
- `svap` is the **program** that saved the file.

They differ when After Effects saves a copy for an older version: the copy carries the older format in `head`, while `svap` still names the After Effects that wrote it. In our files, projects saved as 22.x and 23.x from After Effects 24.5 have `svap` = 24.5. The converter leaves `svap` alone.

### Format numbers per version (measured)

Native values, from files saved by each version:

| Saved by After Effects | `head` format |
|---|---|
| 22.6.0 build 59 | 93.43 |
| 23.6.0 build 55 | 94.9 |
| 24.5.0 build 52 | 95.5 |
| 25.4.0 build 86 | 96.6 |
| 25.6.0 build 101 | 96.9 |
| 26.0.0 build 67 | 97.2 |

In these files the version word in `head` is the version that saved the file.

The format minor is not a function of the major: 24.5 writes 95.5, and 24.6 is stamped 95.6.

The format major follows the After Effects major in this range: 93 is 22, 94 is 23, 95 is 24, 96 is 25, 97 is 26. After Effects 27.0 writes 98.6 (measured on a file saved by After Effects 27.0 on macOS). The converter accepts source files up to format major 98.

Adobe numbered the releases 17 (2020), 18 (2021), then 22 (2022). There is no 19, 20 or 21. Whether the rule "major + 71" holds below 22 is **not known**: we have no files from After Effects 2020 or 2021. Version 1.x labelled two older header values "AE 21.x" and "AE 20.x", which cannot be real version numbers. Those targets were removed in 2.0.

### What the stamps are (measured)

When After Effects saves a copy for an older version, it writes a fixed pair, format and version word, into `head`. `version.rs` holds these:

| Target | Format | Word | Reads as |
|---|---|---|---|
| 26 | 97.11 | `0x0F128650` | 26.5.0 build 80 |
| 25 | 96.9 | `0x0F0B0661` | 25.6.0 build 97 |
| 24 | 95.6 | `0x0F030641` | 24.6.0 build 65 |
| 23 | 94.9 | `0x0B3B0637` | 23.6.0 build 55 |
| 22 | 93.43 | `0x0B33063B` | 22.6.0 build 59 |

They were read from copies saved by After Effects: 24, 23 and 22 by After Effects 24.5; 26 and 25 by After Effects 27.0. The stamp words carry the Windows platform code. The converter replaces it with the platform of the source file.

### Animation presets (measured)

An animation preset (`.ffx`) is a RIFX file of form `FaFX`. Its first chunk is a 16-byte `head`:

```text
off 0  u32 3      off 4  u32 format major      off 8  u32 format minor      off 12  u32 0
```

The format numbers are the same as a project's: a preset saved by After Effects 27.0 carries 98.6. There is no version word and no `svap`. After the head, a `LIST:besc` holds the effects and their properties with the same chunks as a project (`sspc`, `tdgp`, `tdmn`, `tdbs`, ...), including the chunks that the rules below remove (the 27.0 preset has `ofl2` and `elab`).

The converter applies the same steps as for a project and stamps only the format major and minor into `head[4..12]`.

## 3. What After Effects changes when it saves for an older version (measured)

Reproduced in `convert.rs`. In these rules "everywhere" means in the children of every `LIST`, at any depth, and at the top level. A match name is the bytes of a `tdmn` chunk up to the first NUL byte.

| Leaving format major | Rule |
|---|---|
| 98 (27) to 97 | **O1**, **R1**, **F1**. |
| 97 (26) to 96 | **A1**, **G1**, **P1**, **C1**. |
| 96 (25) to 95 | Nothing besides the stamp. |
| 95 (24) to 94 | **S1:** remove the pair `tdmn` `ADBE Shadow Color` + the `tdbs` list that follows it. |
| 94 (23) to 93 | **M1**, **L1**, **E1**, **B1** below. |

- **O1.** Remove every chunk with id `ofl2`.
- **R1.** Remove `tdmn` chunks named `ADBE3D Mat Rand Offset` or `ADBE3D Mat Rand Offset Seed`, with the list that follows each one.
- **F1.** Remove every chunk with id `mtsf`.
- **A1.** Remove every chunk with id `elab`.
- **G1.** Remove every list of kind `GdV2`.
- **P1.** Remove a `tdmn` chunk named `ADBE3D Material Parade`, with the list that follows it.
- **C1.** Remove every chunk with id `cety`.
- **S1.** Remove the pair `tdmn` named `ADBE Shadow Color` and the `tdbs` list that follows it. Shadow Color is a property that After Effects 2023 does not have.
- **M1.** A layer is a `LIST` with a direct `ldta` chunk of exactly 164 bytes. Its id is the big-endian u32 at `ldta[0..4]`, its track matte id the one at `ldta[160..164]`. Among sibling layers, a layer whose matte id is not 0 must come directly after the layer with that id. If not, the conversion is **blocked**: After Effects 22 cannot represent it.
- **L1.** Every `ldta` of exactly 164 bytes is shortened to its first 160 bytes. The last four bytes are the matte id that M1 has just checked.
- **E1.** Remove `tdmn` chunks named `ADBE Light Env Atom`, `ADBE Light Backgd Visible`, `ADBE Light Backgd Opacity` or `ADBE Light Backgd Blur`, together with the list that follows each one.
- **B1.** Remove a `tdbs` list whose previous sibling is a `blsi` chunk (a layer style option 2022 does not have).

Steps run from the source down to the target, highest first. If any rule blocks, nothing is written. Finally `head` is stamped.

### Opening in a real After Effects (measured, After Effects 24.5)

After Effects checks only the format major when it opens a file:

- A 26.0 project with the stamp of 24.5 (95.5), or of 24.6 (95.6, a minor higher than 24.5's own), or with only the major set to 95 (and the version word of 26.0): all opened.
- The 26.0 project without any change: refused, with "created with After Effects version 26.0 (Windows 64)".
- A real working project from 25.6 (25 compositions, 221 layers, 206 effects, 406 expressions): both the full stamp and the major-only change opened, with no expression errors.
- A project saved by After Effects 27.0 was converted and opened in After Effects 24.5.
- A template project (`.aet`) saved by After Effects 27.0, converted to 24: opened, with the same 105 items, 21 compositions and 94 layers. After Effects reported 17 effects it does not have (effects added after 24, such as Curl Noise and Unmult); they stay in the file as missing effects, as with a project.
- An animation preset saved by After Effects 27.0: refused as is ("this file was created with a newer version of After Effects"). Converted to 24, it applied, with the same two effects (Gaussian Blur, Box Blur) and the one animated property as in After Effects 27.0.

From this: to open in 24, the header is enough. The structural steps in the table are needed for targets 23 and 22, and have only been compared against After Effects's own files, not yet opened in those versions.

## 4. What counts as save noise (measured)

Two files saved by After Effects from the same project are never identical. `diff.rs` compares two files as trees and can hide differences that every save introduces. A change is noise when:

- it is the trailer (the XMP packet), or
- it is the `head` chunk and the first difference is at byte 8 or later, or
- the chunk has the same length in both files and its id is one of `sdat`, `aRbp`, `ftts`, `pard`, `mkif`, `ewin`, `fips`, `Rhed`, `nhed`, `nnhd`, or
- it is a `tdb4` property record of the same length that differs in exactly one byte, byte 119. That byte is session state: After Effects sets it while a project is open and clears it after reopening. It is not tied to the file version.

Removed or added chunks are never noise. `aep diff --no-noise` hides these differences.

## 5. How the rules were derived and tested

1. One project was built, and After Effects 24.5 saved it as 24.x, 23.x and 22.x with **Save a Copy As**. (The same was done with After Effects 27.0 for the 26 and 25 stamps.)
2. `aep diff --no-noise` compared each pair as trees. What remained after removing noise became the rules in section 3.
3. The converter's output for the 23 and 22 targets was compared with After Effects's own copy. For 22 no difference was found. For 23 the only difference is the panel outline state described below.
4. Fixtures are in `tests/fixtures` (small test projects, tracked in git). Unit tests in the crates cover the container, version words, file identification, the diff, and each rule. Real working projects are used locally and never committed.
5. The tester kit (`tools/ae`) lets volunteers produce the same set of files from other After Effects versions: a test project built by script and the copies saved by After Effects for each older version.

### Known gaps (not yet reproduced)

| Target | Gap |
|---|---|
| 23 | After Effects also shortens the `otln` chunk (panel outline state) when it removes properties. Not reproduced. |
| 26 | After Effects 27's own copy differs in the color settings of sources (chunks `dcui`, `apid` and an ICC profile in `Utf8`). Not reproduced. |
| 25 | After Effects 27's own copy differs in one migrated effect (Compound Blur). Not reproduced. |

### Not understood

- `head[8:20]`.
- How the format minor is assigned.
- Whether a mismatch between the format and the rest of a file's contents ever matters to an older After Effects. Opening works in all the cases we tried, but we do not know why After Effects accepts files that still contain data from a newer version.
- Anything below After Effects 22.
