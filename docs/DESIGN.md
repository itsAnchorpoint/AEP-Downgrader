# Design guidelines

How AEP Downgrader looks, so that every new screen, page or asset matches without starting over. The tokens live in `apps/desktop/src/styles/tokens.css` (colors, type, radii, shadows, easing) and `accents.css` (the five accents). This page says how to use them.

**Reference artifact:** the disk image window, `apps/desktop/src-tauri/dmg/background.html`. When in doubt, match its level of finish.

## Character

Bold and kinetic: black, warm paper and one loud lime. Confident type, tactile surfaces, light and depth. Minimal, but never bare: a flat screen with a few boxes on it is not finished.

## Color

- **Dark is the brand surface** (`#0b0b0c`). Use it for anything that represents the product outside the app: disk image, site, social images. The app also has light, dark and After Effects themes; the same rules apply to each.
- **Lime (`--color-accent`, `#d6ff3d`) marks the one thing that matters** on a view: the main action, the drop target, the highlighted word. One loud lime element per view, plus small lime details (icons, separators). Text on lime is always ink (`--color-accent-fg`).
- Text levels: `--color-fg` for content, `--color-muted` for secondary text. Lines: `--color-line`, or white at 6–8 % on dark brand surfaces.
- Never introduce a color outside the tokens. Status colors (`ok`, `warn`, `danger`) are only for status.

## Type

- **Onest** for everything a person reads. Display: weight 800, tracking `-0.035em` to `-0.04em`, line height about 1.05. Body: weight 600–700 at 13–15 px.
- **Highlight one word** of a headline on a lime plate: radius about 10 px, padding `0 10px 3px`, ink text. Example: "Drop it in **Applications**", "Install in **one click**". Use it on brand surfaces (disk image, installer, site), not on the app's working screens.
- **JetBrains Mono only for data**: version numbers, file extensions, technical values, log details. Not for labels, badges or decoration. No uppercase, letter-spaced "label" badges.
- Sentence case everywhere. Short, plain words a motion designer uses.

## Shape

- Radii from the tokens: `sm` 8, `md` 14, `lg` 22, `xl` 26 px; large cards and tiles 22–28 px.
- Buttons are pills. Primary: filled `--color-strong`. Secondary: 2 px outline.
- Outlines are 1.5–2 px. Dashed outlines mean "drop here".

## Depth, light and texture

What turns minimal into finished. Use all of them lightly, so that each is felt more than seen.

- **Light:** a soft white glow from the top left (about 7 % opacity), an accent glow behind the focal element (about 15 %), and a vignette that darkens the edges.
- **Texture:** film grain at 6–9 % (overlay blend), plus optionally a faint dot grid (16 px pitch, about 7 %) masked so that it fades out towards the edges.
- **Surfaces:** a vertical gradient (slightly lighter at the top), a 1 px inner highlight on the top edge (white at 6–8 %), and a soft, deep shadow below (`0 24px 48px -16px` black at about 80 %).
- **Accent elements:** a subtle gradient (`#e6ff78` to `#d2fb36`), an inner highlight, and a colored glow under them.
- **In the app:** WebKitGTK on Linux is slow with stacked `backdrop-filter` blurs and live SVG filters. Use static gradients and shadows, and a pre-rendered grain image instead of `feTurbulence`. Animate only `transform` and `opacity`.

## Brand marks and icons

- **Trail** is the app icon and the small mark in the header.
- **Fold**, the folded `.aep` ribbon with an arrowhead, is the illustrative motif for the site, the disk image, the installer and marketing. It reads as "this file goes down a version".
- **The lockup** on the app's start screen (`components/Lockup.tsx`): the Fold ribbon above the word "Downgrader", its arrow running down past the end of the word to the depth of the "g". The word's left edge sits on the content edge; the ribbon floats above it and may reach into the title bar. Proportions to the font size: ribbon height 1.96, ribbon left edge 3.27 from the D, ribbon top 1.54 above the capitals, a visible gap between the "r" and the arrowhead. In the app the ribbon takes the chosen accent (lime keeps the drawn colors; other accents and the After Effects theme mix the faces from the accent, `.fold-ribbon` in `index.css`), so it is inlined rather than an `<img>`. The app icon and the brand surfaces stay lime. Its shadow sits on a wrapper, so the reveal animation's clip never cuts it.
- **The version staircase** is the marketing motif for wide formats (donation covers, the site): year pills in JetBrains Mono from `.aep 2027` down to 2022 on the Trail zigzag, the line fading in from the left and turning lime, one pill (2024, the verified target) filled lime, an arrowhead at the end. Keep it centered so narrow crops still show the middle years.
- UI icons never use AI clichés (sparkles, magic wands, stars) for ordinary features.
- UI icons are Lucide with stroke 2–3. Meaningful icons can be lime: for example, the lock in the privacy line and the shield in the first-launch hint.

## Working screens

- **One loud element per view**: the main action (Convert) or the drop circle. A chosen option among several (target tiles, segmented settings) is an outline in the text color on a raised surface, not an accent fill. Unchosen options are quieter: a thin outline in the line color and muted text.
- **Status colors mean status**: red only for "can't convert", amber for something that changes or is lost, green for verified or done. Files that are simply not projects (macOS sidecars) are muted, not red. An experimental target keeps its amber badge where it is chosen, and the report says so too.
- **Say it in the row**: every file row states its report in a word (nothing lost, N changes, can't convert), so nobody has to open it to find out.
- **The title bar** is the app's own on macOS (overlay, traffic lights in the bar) and Windows (no frame, own window buttons with a red close); Linux keeps the system frame and the bar becomes a toolbar. It carries only the tools; the product name is the start screen's lockup.
- **Fit the space, not the window**: the drop circle and its rings size themselves to the drop zone's height (container queries), so the content stays centered under the update banner, with the log open and in a small window.

## Motion

- `--ease-pop` for small affordances (press, toggle, badge), `--ease-out-quint` for entrances.
- 150–350 ms. Respect `prefers-reduced-motion`.
- No idle loops: the drop rings pulse only on hover or while files are dragged over the window. One-time intros (the lockup) play once per session.

## Do and don't

| Do | Don't |
|---|---|
| One lime focal point per view | Several lime elements competing |
| Display type at 800 with tight tracking | Default tracking on large text |
| Depth: gradient, top highlight, soft shadow | Flat boxes on a flat background |
| Lime icon plus bold text for hints (as in the app footer) | Uppercase mono badges |
| Brand motifs (Fold ribbon, Trail mark) | Generic clip-art arrows and shapes |
| Real tokens and fonts in an HTML mock-up, rendered and reviewed | Hand-placed SVG text with fallback fonts |

## Brand surfaces

Everything outside the app follows this page too, so the product looks like one thing from the first download:

| Surface | State |
|---|---|
| App window (all themes) | Done: second pass, checked on real Windows and macOS |
| macOS disk image window | Done: the reference artifact |
| Windows installer (the whole window, every page, also updates and uninstalling) | Done: drawn by our NSIS plugin, `apps/desktop/src-tauri/windows` |
| App icon (Trail) | Done |
| README header | Done: icon, one line, the app window with a report (`assets/screenshot.png`, a mock-backend shot framed as a macOS window) |
| Donation pages (Boosty, Tribute) | Done: Trail avatar, covers with the version staircase |
| Website, download and support pages | Done: `site/`, the hero is a live copy of the app window (a demo, then the visitor's own file checked in the browser) |
| Link previews (Open Graph, GitHub social preview) | Done: `site/og/art.html`, rendered by `site/og/render.sh` into `site/public/og/` and `assets/social-preview.jpg` |

## Making and reviewing assets

- Build static assets (disk image background, social images) as HTML with the real fonts (`apps/desktop/src/assets/fonts`) and render them with headless Chromium at 1x and 2x. See `apps/desktop/src-tauri/dmg/render.sh`.
- Review a rendered preview with the real surroundings (icons, Finder labels, window size) before shipping it.
- **Windows installer window:** the plugin in `apps/desktop/src-tauri/windows/ui` draws every page in software (tiny-skia, rustybuzz, the app's fonts), so it looks the same on every Windows version and under Wine. Preview all pages as PNGs without Windows: `AEPD_PREVIEW=<dir> cargo test --release preview -- --ignored` in that folder. Its art (Fold, Trail) is rendered by `windows/assets/render.sh`; the texts are in `windows/hooks.nsh`.
- **Disk image window:** Finder draws file names in black whenever the window has a background picture, even in Dark Mode. Put a light plate behind each label (the app's plate is paper, the Applications plate is lime). Finder's labels use text size 16 and sit about 84 pt below the icon center.
