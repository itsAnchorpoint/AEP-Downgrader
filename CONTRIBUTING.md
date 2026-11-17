# Contributing

Thanks for helping. This page says how to build and test the project, and the few rules that keep its promises true.

## The rules

1. **Conversion rules come from files After Effects itself saved.** A rule is added only after comparing a file that After Effects wrote with **Save a Copy As** to the same project saved natively, and the converter's output must match that copy. Do not add a rule from a guess, from another tool, or from how a file looks. Describe the comparison in the pull request.
2. **A target is Verified only if converted files were opened in a real After Effects of that version.** Otherwise it is Experimental. The status and its one-sentence basis live in `crates/aep-core/src/compat.rs`. Do not change a status without that evidence.
3. **Never commit a real client project.** Project files contain the paths of footage and may contain names and unreleased work. Only small synthetic projects belong in `tests/fixtures/`. Real projects stay on your machine (`tests/fixtures-local/` is ignored by git, as is every other `.aep`).
4. **Keep the privacy promise.** No telemetry, no uploads, no network access except the update check. A change that adds any of these will not be merged. See [docs/PRIVACY.md](docs/PRIVACY.md).
5. **Say what a conversion cannot do.** If a change alters what is removed or blocked, update [docs/COMPATIBILITY.md](docs/COMPATIBILITY.md) and the app's wording (`apps/desktop/src/i18n/en.json`, and the Russian file next to it).
6. **Follow the design guidelines.** New screens and assets use the existing tokens and match [docs/DESIGN.md](docs/DESIGN.md). Include a screenshot in the pull request.

## Layout

- `crates/aep-core`: the conversion. No dependencies, no file or network access. Module comments are the specification.
- `crates/aep-cli`: the `aep` command line, also used as a test bench.
- `apps/desktop`: the Tauri app (React and TypeScript front end, Rust back end in `src-tauri`). It is a separate Cargo workspace.
- `tools/ae`: the tester kit (ExtendScript for After Effects).
- `docs`: format, compatibility and privacy notes.

## Build and test

You need Rust (stable), Node.js 22 and pnpm. On Linux, install the WebKitGTK packages listed in `.github/workflows/ci.yml`.

Core and command line, from the repository root:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Desktop app, from `apps/desktop`:

```bash
pnpm install
pnpm exec tsc --noEmit
pnpm test
pnpm build
```

The app's Rust side, from `apps/desktop/src-tauri`:

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

Run the app with `pnpm tauri dev`, and build installers with `pnpm tauri build`. CI runs all of this on every push and pull request, and builds the installers on Windows (NSIS), macOS (dmg) and Linux (AppImage and deb).

The core also has to compile for WebAssembly: `cargo check -p aep-core --target wasm32-unknown-unknown`.

## Pull requests

- Branch from `v2` until 2.0 is released.
- Keep a change small and say why it is needed.
- Add a test for a new rule or a bug fix. Rules are tested against files in `tests/fixtures`.
- Update `CHANGELOG.md` for anything a user would notice, in plain words a motion designer understands.
- Code, documentation and commit messages are in English.

## Sending a file privately

If a file is not recognised or a conversion fails, and the project is confidential:

1. Do not attach it to an issue.
2. [Open an issue](https://github.com/itsAnchorpoint/AEP-Downgrader/issues) that says only which After Effects version and system saved the file, what the app showed, and that you can share the file privately. A private way to send it can then be arranged.

## Helping verify a target

See [tools/ae/README.md](tools/ae/README.md) for the tester kit. It builds a small test project with nothing personal in it. The results tell us which older versions your After Effects can save and whether converted files open.
