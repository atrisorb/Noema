# NOEMA - AGENT HANDOFF

> Rewrite this file at the end of every block. Keep it under ~120 lines. Newest state first.

## Current state

- **Block 9 complete (2026-10-31):** Tauri v2 desktop wrapper + bundle pipeline shipped. Owner push pending. Details below.
- **Block 10 prepared, not executed (2026-10-31):** Vitreous design system v1.0, artifact workspace contract, anatomy/asset standard and Claude Design research are written. New agent **Aesthesis** (`noema-aesthesis`) is registered but **not started**. Owner runs it.
- **Design law:** `docs/design/NOEMA_DESIGN_SYSTEM.md` is the single source of truth. `docs/design/ARTIFACT_WORKSPACE.md`, `docs/design/ANATOMY_AND_ASSETS.md`, `docs/design/DECISIONS.md` and `docs/research/CLAUDE_DESIGN_2026.md` support it.
- **Agent:** id `noema-aesthesis`, name `NOEMA - Aesthesis`, workspace `C:\Users\Adriano Sorbello\.openclaw\workspaces\noema-aesthesis`, cwd repo, model `deepseek/deepseek-flash` (fallback FrogNano), thinking `high`, tools coding. Charter installed in the workspace and in the local `agents/noema-aesthesis.md` (git-ignored by repo policy). Not run.
- **Block 10 scope:** tokens, SVG logo suite, eight mascots, glass shell, artifact workspace, dashboards/chat/templates, anatomy realism, asset migration, QA. Checklist in `docs/MASTER_PLAN.md`.

## Design audit (start here when Aesthesis runs)

- `crates/app/ui/src/body.ts`: translucent capsule mannequin; 3 capsules per leg; foot is a 0.01-long capsule; one skin material. Lower legs/feet fail at **geometry + material**, not tint.
- `assets/anatomy/atlas/body_full.glb`: 1,872 nodes, 826 meshes, 2 materials (`Mat_Bone_Pro`, `Mat_Muscle_Pro`); bone/muscle only, no skin/fascia. Foot/leg bones exist. No node names match penis/genitalia beyond pelvic bones; the visible central pelvic protrusion still needs a mesh-level audit and removal if external. No toggle, neutral pelvis.
- **Raster debt:** 34 PNG + 1 ICO, zero JPEG/WEBP. Evidence PNGs, Tauri icons, one CAD render. Block 10.9 migrates to SVG/WEBP or deletes. Platform icons become generated, git-ignored artifacts.
- **Current UI:** generic dark cyan glass. Block 10 replaces it with Vitreous: Cavity/Bone, arterial coral, neural amber, venous cobalt, living glass, Newsreader/Instrument Sans/Commit Mono.

## Tauri command surface (Retina: bind to this, do not invent commands)

| Command | Signature (Rust) | Returns |
|---|---|---|
| `health` | `fn health() -> Result<Value, String>` | `{ "ok": true, "version": "<noema-core version>" }` from `GET /api/health` |
| `bridge_info` | `fn bridge_info() -> Value` | `{ "port": <u16>, "url": "http://127.0.0.1:<port>" }` |
| `session_resume` | `fn session_resume() -> Result<Value, String>` | last `session` object from `GET /api/studio` or `null` |

JS access (global enabled in `tauri.conf.json`):

~~~js
const invoke = window.__TAURI__?.core?.invoke;
if (invoke) {
  await invoke("health");
  await invoke("bridge_info");
  await invoke("session_resume");
}
~~~

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

1. **macOS/Linux installer formats are CI-defined.** Windows NSIS + MSI are built and inspected locally; dmg/AppImage/deb/rpm still need the first CI tag run. Signing deferred.
2. **macOS resource lookup.** UI/assets land in `Contents/Resources`; the wrapper passes them to the sidecar. If Tauri changes layout, check `docs/PLATFORMS.md` section 6.
3. **Universal macOS build** needs both Apple targets and a Developer ID; unsigned until then.
4. **Sidecar lifecycle (verified).** Cold start spawns `noema-app serve`; closing the window kills only the child it spawned. An already-running bridge is left alive.
5. **Security surface (S1).** `src-tauri/capabilities/main.json` accepts remote IPC from any loopback port, `withGlobalTauri` is true and CSP is null. Narrow the patterns and set a CSP before shipping.
6. **Session diff / live usage writer** from Block 8 remain open (Thalamus + git probe).
7. **Full-viewport Brain route** remains Retina's navigation follow-up.
8. **App icon is a generated placeholder.** Block 10.3 replaces it with the SVG source and generation script; no raster committed.
9. **Raster migration** is Block 10.9. Until then the repo still has 34 PNG + 1 ICO; the design law forbids new ones.
10. **Design docs vs local charter.** `docs/design/**` is tracked. `AGENTS.md` and `agents/**` are intentionally git-ignored ("Local agent instructions - never push"), so the Aesthesis charter is host-local.

## Evidence

- `cargo check --workspace`: green (1.19s, cached) after the design-doc commit.
- `agents.entries.noema-aesthesis`: present with model, workspace, cwd, thinking level, tool profile; workspace charter installed; agent not run.
- Design docs: `docs/design/README.md`, `NOEMA_DESIGN_SYSTEM.md`, `ARTIFACT_WORKSPACE.md`, `ANATOMY_AND_ASSETS.md`, `DECISIONS.md`; research `docs/research/CLAUDE_DESIGN_2026.md`.
- Block 9 evidence remains `docs/evidence/block9-desktop.png`, `block9-desktop-autostart.png`, `block9-webview2.json`; those PNGs migrate to WEBP in Block 10.9.
- No `git push` performed.
