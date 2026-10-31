# NOEMA - AGENT HANDOFF

> Rewrite this file at the end of every block. Keep it under ~120 lines. Newest state first.

## Current state

- **Block 10.5 complete (2026-10-31):** Vitreous Spine shell, Cavity/Bone toggle, Layers inspector, status filament. 10.6 artifact workspace is next. 28 rasters remain for 10.9.
- **Block 10.3 complete (2026-10-31):** SVG logo suite, favicon, generated platform icons.
- **Block 10.1 complete (2026-10-31):** design audit and freeze. `docs/design/AUDIT.md` is the baseline; UI source was read-only in that block.
- **Block 9 complete (2026-10-31):** Tauri v2 desktop wrapper + bundle pipeline. Owner push pending.
- **Design law:** `docs/design/NOEMA_DESIGN_SYSTEM.md` **v1.1** frozen. `ARTIFACT_WORKSPACE.md`, `ANATOMY_AND_ASSETS.md`, `DECISIONS.md`, `AUDIT.md` support it.
- **Agent:** id `noema-aesthesis`, name `NOEMA - Aesthesis`, workspace `C:\Users\Adriano Sorbello\.openclaw\workspaces\noema-aesthesis`, cwd repo, model `deepseek/deepseek-flash` (fallback FrogNano), thinking `high`, tools coding. Charter installed in the workspace and local `agents/noema-aesthesis.md` (git-ignored).
- **Block 10 scope:** tokens, SVG logo suite, eight mascots, glass shell, artifact workspace, dashboards/chat/templates, anatomy realism, asset migration, QA.

## Block 10.3 results

1. **Seven SVG sources:** `assets/brand/noema-mark.svg`, `noema-monogram.svg`, `noema-wordmark.svg`, `noema-lockup.svg`, `noema-lockup-stacked.svg`, `noema-app-icon.svg`, `favicon.svg`.
2. **Mark:** lens arc with a curved N filament and arterial dot. Rejected the earlier diagonal filament because it read as a prohibition sign. Tested at 16/24/32/48/96.
3. **Wordmark:** Newsreader 500 with a lens O and 1px filament; tight lockup spacing.
4. **Pipelines:** `sync-brand.mjs` (favicon, `--check`), `build-icons.mjs` (`cargo tauri icon` into `src-tauri/icons/`, git-ignored), `render-logo-suite.mjs` (WEBP sheet).
5. **Icons:** generated 32/64/128/128@2x PNG, ICO, ICNS, Windows Store and mobile sets. Removed 5 committed PNG + 1 ICO + `src-tauri/app-icon.png` from git. Tracked rasters: **28 PNG, 0 ICO**.
6. **Favicon:** `crates/app/ui/public/favicon.svg`, linked in `index.html`; generated from the canonical source.
7. **Evidence:** `docs/evidence/design-logo-suite.webp` (Cavity + Bone, all seven assets, mark sizes).
8. **Build:** `npm run build` green; favicon in `dist`.

## Block 10.4 results

1. **Eight SVGs:** `assets/mascots/{cortex,axon,thalamus,synapse,soma,hippocampus,retina,aesthesis}.svg`, 48x48, `body`/`trace`/`glow`, currentColor body, one accent.
2. **Loader:** `crates/app/ui/src/mascots.ts` imports raw SVGs through Vite; unknown ids return empty.
3. **Roster:** 24px mascot replaces the generic dot; accent stays per agent; hover raises the glow; reduced motion removes the transition.
4. **Evidence:** `docs/evidence/design-mascots.webp` at 96 and 24 in Cavity + Bone. Distinct silhouettes, no emoji.
5. **Aesthesis:** asset ships, roster stays at 7 until Cortex approves an eighth runtime agent.

## Block 10.5 results

1. **Shell:** command bar 56px, rail 72px, inspector 380px, status filament 28px; Layers moved out of the HUD into the inspector.
2. **Theme:** `data-theme="cavity"|"bone"` on `<html>`, persisted in `noema:theme`; scene background flips; CSS 30.16 KB raw / 6.14 KB gzip.
3. **A11y:** skip link, focus-visible ring, aria-expanded/hidden, Escape closes inspector and rail, reduced-motion override.
4. **Responsive:** inspector/rail become bottom sheets at 1024; rail becomes a bottom scroller at 768.
5. **Evidence:** `design-shell-cavity.webp`, `-bone.webp`, `-layers.webp`, `-mobile.webp`, `design-shell.json`; 7 agents, 60 fps.

## Next block: 10.6 artifact workspace

- Add the `.noema.html` model: manifest, template, token-only styles, optional logic, `data-noema-id` anchors.
- Build chat + canvas + inspector + timeline + assets + export in one overlay; parse, edit, props, comments, versions, verify.
- One real artifact must pass the verification gate and export HTML, JSON and Markdown; no raster preview.
- Evidence: artifact WEBP capture + verify JSON; Playwright smoke.

## Token workflow

```
node tools/design/build-tokens.mjs           # regenerate tokens.css
node tools/design/build-tokens.mjs --check   # fail on drift
node tools/design/check-contrast.mjs         # 24 AA pairs -> design-tokens-contrast.json
```

Rules: edit only `tokens.json`; never hand-edit `tokens.css`; a token change bumps `designSystem` and needs a `DECISIONS.md` row if it changes a visual rule.

## Tauri command surface (bind to this, do not invent commands)

| Command | Signature (Rust) | Returns |
|---|---|---|
| `health` | `fn health() -> Result<Value, String>` | `{ "ok": true, "version": "<noema-core version>" }` from `GET /api/health` |
| `bridge_info` | `fn bridge_info() -> Value` | `{ "port": <u16>, "url": "http://127.0.0.1:<port>" }` |
| `session_resume` | `fn session_resume() -> Result<Value, String>` | last `session` object from `GET /api/studio` or `null` |

```js
const invoke = window.__TAURI__?.core?.invoke;
if (invoke) {
  await invoke("health");
  await invoke("bridge_info");
  await invoke("session_resume");
}
```

- Window label `main`. Capability `src-tauri/capabilities/main.json` grants `core:default` and IPC from loopback only.
- The same page runs in a plain browser; guard with `window.__TAURI__` and fall back to `fetch` on `bridge_info().url`.
- New command? Propose it here first; Axon owns `src-tauri`.

## Decisions still valid

1. **Studio is a hash route.** `#studio` opens the overlay; `/` stays on the body.
2. **No duplicated server.** The wrapper never serves HTTP; it reads the `noema-app` loopback bridge.
3. **Bundle-only overrides.** Sidecar + UI/assets resources live in `packaging/tauri.bundle.conf.json`; plain `cargo check --workspace` needs no built UI.
4. **Sidecar naming.** `bundle.externalBin: binaries/noema-app`; packaging stages `noema-app-<target-triple>[.exe]`.
5. **Remote IPC is narrow.** Only `127.0.0.1` and `localhost`; do not widen.
6. **No `git push`.** Owner reviews and pushes.
7. **Tokens are generated.** JSON canonical, CSS generated, contrast gated (D12).
8. **AA ink on fills.** Use `artery-ink` / `danger-ink`, never `ink`, on those fills (D11).

## Open items / risks

1. **Font binaries are missing.** Tokens name Newsreader, Instrument Sans and Commit Mono, but the repo has 0 WOFF2. Fonts must land, be subset and licensed (<160 KB) before 10.5 ships the shell. Fallbacks are system fonts for now.
2. **Token adoption is zero in components.** 65 hex sites in `style.css` and 5 in `main.ts` remain. 10.5 migrates surfaces; new code must read `--noema-*` only.
3. **macOS/Linux installer formats are CI-defined.** Windows NSIS + MSI built locally; dmg/AppImage/deb/rpm await the first CI tag. Signing deferred.
4. **macOS resource lookup.** UI/assets land in `Contents/Resources`; check `docs/PLATFORMS.md` section 6 if Tauri changes layout.
5. **Security surface (S1).** `src-tauri/capabilities/main.json` accepts remote IPC from any loopback port, `withGlobalTauri` is true and CSP is null. Narrow patterns and set a CSP before shipping.
6. **Session diff / live usage writer** from Block 8 remain open (Thalamus + git probe).
7. **Full-viewport Brain route** remains Retina's navigation follow-up.
8. **Placeholder app icon.** 10.3 replaces it with SVG source + generation script; no raster committed.
9. **Raster migration** is 10.9. The repo still has 34 rasters; the 10.1 capture script writes WebP only.
10. **Pelvis claim is open.** Do not describe the central mass as genitalia or as removed until the 10.8 mesh audit and close-up WebP.
11. **Agent roster 7 vs 8.** Needs an Aesthesis mascot and rail entry; propose plan change to Cortex, do not edit the plan.
12. **Design docs vs local charter.** `docs/design/**` is tracked. `AGENTS.md` and `agents/**` are git-ignored.

## Evidence

- Block 10.2: `crates/app/ui/src/design/tokens.json`, `tokens.css`; `tools/design/build-tokens.mjs`, `check-contrast.mjs`; `docs/evidence/design-tokens-contrast.json` (24/24).
- Block 10.1: `docs/design/AUDIT.md`; `docs/evidence/design-baseline-*.webp`; `design-baseline.json`; `crates/app/ui/scripts/capture-audit.mjs`.
- `cargo check --workspace` and `npm run build`: green after 10.2.
- Block 9 evidence remains `docs/evidence/block9-desktop.png`, `block9-desktop-autostart.png`, `block9-webview2.json`; those PNGs migrate to WebP in 10.9.
- No `git push` performed.
