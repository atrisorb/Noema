# NOEMA - AGENT HANDOFF

> Rewrite this file at the end of every block. Keep it under ~120 lines. Newest state first.

## Current state

- **Block 10.1 complete (2026-10-31):** design audit and freeze. `docs/design/AUDIT.md` is the measured baseline; design law unchanged; UI source untouched. Next: 10.2 tokens.
- **Block 9 complete (2026-10-31):** Tauri v2 desktop wrapper + bundle pipeline. Owner push pending.
- **Design law:** `docs/design/NOEMA_DESIGN_SYSTEM.md` v1.0 frozen. `ARTIFACT_WORKSPACE.md`, `ANATOMY_AND_ASSETS.md`, `DECISIONS.md`, `AUDIT.md` support it.
- **Agent:** id `noema-aesthesis`, name `NOEMA - Aesthesis`, workspace `C:\Users\Adriano Sorbello\.openclaw\workspaces\noema-aesthesis`, cwd repo, model `deepseek/deepseek-flash` (fallback FrogNano), thinking `high`, tools coding. Charter installed in the workspace and local `agents/noema-aesthesis.md` (git-ignored).
- **Block 10 scope:** tokens, SVG logo suite, eight mascots, glass shell, artifact workspace, dashboards/chat/templates, anatomy realism, asset migration, QA. Checklist in `docs/MASTER_PLAN.md`.

## Block 10.1 results (read `docs/design/AUDIT.md`)

1. **Driftwatch:** 8 updates, zero design impact. DeepSeek/Mistral/Gemini pricing changes go to Thalamus rate cards, not design.
2. **Rasters:** 33 PNG + 1 ICO = 34 files, 8,628,345 bytes, 0 JPEG/GIF/BMP/TIFF. 26 evidence PNGs are 95% of raster bytes. 70 references in 27 tracked text files.
3. **UI baseline:** 65 hard-coded hex + 26 rgba in `style.css`, 5 hex in `main.ts`, 4 backdrop filters, 0 WOFF2, 0 brand SVG, 7 roster agents. No token layer.
4. **Perf:** 60.1 fps at DPR 1; 35.4 fps at DPR 2 (p50 30). DPR 2 fails the budget. Browser baseline, not WebView2.
5. **Anatomy:** 826 meshes, 290,886 triangles, 2 materials. Lower legs, feet and hands are skeletal; skin opacity 0.12; foot capsule 0.01. Pelvic central mass unresolved; mesh-level audit is 10.8, no claim made.
6. **Artifact workspace:** zero `.noema.html`, no parser, manifest, versions, comments or export. 10.6-10.7 start from zero.
7. **A11y:** no `:focus-visible`, no skip link, no reduced-motion, canvas-only body, no axe or contrast evidence.
8. **Roster gap:** 7 agents vs 8 design mascots. Aesthesis missing from the rail; mascot + entry belong to 10.4/10.5.

## Next block: 10.2 tokens

- Create `crates/app/ui/src/design/tokens.css` and `tokens.json` from the design system sections 2-6.
- Cavity and Bone values; token names stable; no hard-coded hex in touched surfaces.
- Update `DECISIONS.md` only if a decision changes; 10.1 froze it.
- Evidence: token diff, contrast checks for text on glass, no PNG.

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

## Open items / risks

1. **macOS/Linux installer formats are CI-defined.** Windows NSIS + MSI built locally; dmg/AppImage/deb/rpm await the first CI tag. Signing deferred.
2. **macOS resource lookup.** UI/assets land in `Contents/Resources`; check `docs/PLATFORMS.md` section 6 if Tauri changes layout.
3. **Universal macOS build** needs both Apple targets and a Developer ID; unsigned until then.
4. **Sidecar lifecycle (verified).** Cold start spawns `noema-app serve`; closing the window kills only its child. An already-running bridge is left alive.
5. **Security surface (S1).** `src-tauri/capabilities/main.json` accepts remote IPC from any loopback port, `withGlobalTauri` is true and CSP is null. Narrow patterns and set a CSP before shipping.
6. **Session diff / live usage writer** from Block 8 remain open (Thalamus + git probe).
7. **Full-viewport Brain route** remains Retina's navigation follow-up.
8. **App icon is a generated placeholder.** 10.3 replaces it with SVG source + generation script; no raster committed.
9. **Raster migration** is 10.9. Until then the repo has 34 rasters; the design law forbids new ones. The 10.1 capture script writes WebP only.
10. **Design docs vs local charter.** `docs/design/**` is tracked. `AGENTS.md` and `agents/**` are git-ignored ("Local agent instructions - never push").
11. **Pelvis claim is open.** The 10.1 render shows a central mass; do not describe it as genitalia or as removed until the 10.8 mesh-level audit and close-up WebP.
12. **Agent roster 7 vs 8.** Needs an Aesthesis mascot and rail entry; propose plan change to Cortex, do not edit the plan.

## Evidence

- Block 10.1: `docs/design/AUDIT.md`; `docs/evidence/design-baseline-body.webp`, `-legs.webp`, `-rail.webp`, `-machine.webp`, `-brain.webp`, `-studio.webp`, `design-baseline.json`; `crates/app/ui/scripts/capture-audit.mjs`.
- `cargo check --workspace`: green (0.98s, cached) after 10.1.
- Block 9 evidence remains `docs/evidence/block9-desktop.png`, `block9-desktop-autostart.png`, `block9-webview2.json`; those PNGs migrate to WebP in 10.9.
- No `git push` performed.
