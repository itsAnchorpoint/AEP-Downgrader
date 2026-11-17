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
- **Highlight one word** of a headline on a lime plate: radius about 10 px, padding `0 10px 3px`, ink text. Example: "Open it in **any** older version.", "Drop it in **Applications**".
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
- **Fold**, the folded `.aep` ribbon with an arrowhead, is the illustrative motif for the site, the disk image and marketing. It reads as "this file goes down a version".
- UI icons are Lucide with stroke 2–3. Meaningful icons can be lime: for example, the lock in the privacy line and the shield in the first-launch hint.

## Motion

- `--ease-pop` for small affordances (press, toggle, badge), `--ease-out-quint` for entrances.
- 150–350 ms. Respect `prefers-reduced-motion`.

## Do and don't

| Do | Don't |
|---|---|
| One lime focal point per view | Several lime elements competing |
| Display type at 800 with tight tracking | Default tracking on large text |
| Depth: gradient, top highlight, soft shadow | Flat boxes on a flat background |
| Lime icon plus bold text for hints (as in the app footer) | Uppercase mono badges |
| Brand motifs (Fold ribbon, Trail mark) | Generic clip-art arrows and shapes |
| Real tokens and fonts in an HTML mock-up, rendered and reviewed | Hand-placed SVG text with fallback fonts |

## Making and reviewing assets

- Build static assets (disk image background, social images) as HTML with the real fonts (`apps/desktop/src/assets/fonts`) and render them with headless Chromium at 1x and 2x. See `apps/desktop/src-tauri/dmg/render.sh`.
- Review a rendered preview with the real surroundings (icons, Finder labels, window size) before shipping it.
