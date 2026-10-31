# NOEMA Brand Assets

Source of truth. All files here are SVG, currentColor or token-driven. No PNG, JPEG, GIF or BMP.

## Files (Block 10.3)

| File | Use |
|---|---|
| <code>noema-mark.svg</code> | The Lens mark, 32x32 grid |
| <code>noema-monogram.svg</code> | N filament monogram, 16px legible |
| <code>noema-wordmark.svg</code> | NOEMA wordmark, Newsreader 500, custom O |
| <code>noema-lockup.svg</code> | Horizontal mark + wordmark |
| <code>noema-lockup-stacked.svg</code> | Square/stacked lockup |
| <code>noema-app-icon.svg</code> | 1024 square app icon source |
| <code>favicon.svg</code> | Browser favicon |

## Build

- `node tools/design/sync-brand.mjs` copies `favicon.svg` to `crates/app/ui/public/favicon.svg`; `--check` fails on drift.
- `node tools/design/build-icons.mjs` renders platform icons from `noema-app-icon.svg` into `src-tauri/icons/`, which is git-ignored.
- `node tools/design/render-logo-suite.mjs` writes `docs/evidence/design-logo-suite.webp`.

## Rules

- Clearspace: height of the O.
- Minimum sizes: mark 16px, wordmark 96px wide, lockup 140px wide.
- One accent maximum. No gradient on the wordmark. No shadow, skew, rotation or outline.
- App icons: generate ICO/ICNS/PNG from <code>noema-app-icon.svg</code> into <code>dist/icons</code> or <code>target/icons</code>; never commit them.
- Icons live in <code>assets/icons/noema-icons.svg</code> as one sprite.
- Mascots live in <code>assets/mascots/&lt;agent&gt;.svg</code>, one per agent.
- Original assets are MIT. Third-party assets need a license and credit in <code>assets/LICENSES.md</code>.
- The asset audit fails on any forbidden extension or reference.
