# NOEMA - AGENT HANDOFF

> Rewrite this file at the end of every block. Keep it under ~120 lines. Newest state first.

## Current state

- **Block:** 9 complete (2026-10-31), autostart path proven. **Tauri v2 desktop wrapper + bundle pipeline shipped.** Cortex review APPROVE (28941ab, findings low/info); doc-accuracy follow-up committed. Next: owner push.
- **Autostart:** cold launch of `target/debug/noema-desktop.exe` with no `noema-app` and port 4173 free spawned `noema-app serve --port 4173` (sidecar pid 20080), loaded the Studio with the full 3D body at 60 fps, and killed the sidecar when the window closed. Evidence `docs/evidence/block9-desktop-autostart.png`.
- **Wrapper:** `src-tauri` crate `noema-desktop`, Tauri 2.12.5 / tauri-build 2.7.4. The window loads the existing Studio over loopback; no HTTP server and no frontend build live in this crate.
- **Bridge target:** `http://127.0.0.1:NOEMA_APP_PORT` (default 4173). If `GET /api/health` fails, the shell starts `noema-app serve --port <port>` from next to the executable, passing `--ui`/`--assets` from Tauri's resource dir. If that also fails it shows `src-tauri/frontend/index.html`.
- **Bundles:** `bundle.targets = all` -> NSIS + MSI (Windows), dmg (macOS universal), AppImage/deb/rpm (Linux). Icons in `src-tauri/icons`.
- **Installer layout:** `noema-app.exe` sits next to `NOEMA.exe`; `ui/` and `assets/` sit in the install dir, which is Tauri's resource dir. This is exactly what `app_bin()` (next to exe) and `spawn_app()` (resource dir) resolve; confirmed in the generated WiX and in the debug resource staging.
- **Windows artifacts:** NSIS `NOEMA_0.1.0_x64-setup.exe` (5.18 MiB) and MSI `NOEMA_0.1.0_x64_en-US.msi` (7.91 MiB) built locally in `dist/tauri/` (git-ignored); WiX source confirms the `noema-app` sidecar plus `ui/` and `assets/`.
- **Packaging:** `packaging/tauri.ps1`, `packaging/tauri.sh`, `packaging/tauri.bundle.conf.json`. Standalone zip/dmg/AppImage scripts from Block 8 are unchanged.
- **Tests:** `cargo check --workspace` green; `cargo test --workspace` 58 green (desktop crate has no tests). `cargo check -p noema-desktop` green with `crates/app/ui/dist` absent, so a clean checkout compiles before Vite runs.
- **CLI/RPC unchanged:** app bridge via `NOEMA_APP_PORT`, daemon via `daemon.json`, `rpc::METHODS`.
- **UI contract:** `crates/app/ui/WRAPPER.md` (Retina, commit 91ad5c1) publishes the same three commands and states the UI consumes none: the window is same-origin with `noema-app`. No command drift.
- **Perf:** Soma's WebView2 harness (`tools/perf/webview2-fps.mjs`, commit eb3325b) measured 60 fps inside the wrapper; evidence `docs/evidence/block9-webview2.json`. Commit b77f17f **rewrote** that file against the f890aa4 baseline: url port 4199 -> 4299 (`http://127.0.0.1:4299/`), fpsMean 60.121 -> 60.081, p99 16.8 -> 17.0 ms. Current numbers are the b77f17f ones and are not byte-identical to the earlier commit.
- **Driftwatch:** digest 2026-10-31, 8 platform updates; `check` clean; no Rust/Node dependency affected. New deps: `tauri` 2.12.5 and `tauri-build` 2.7.4 (owner decision: the wrapper is its own block).

## Tauri command surface (Retina: bind to this, do not invent commands)

| Command | Signature (Rust) | Returns |
|---|---|---|
| `health` | `fn health() -> Result<Value, String>` | `{ "ok": true, "version": "<noema-core version>" }` from `GET /api/health` |
| `bridge_info` | `fn bridge_info() -> Value` | `{ "port": <u16>, "url": "http://127.0.0.1:<port>" }` |
| `session_resume` | `fn session_resume() -> Result<Value, String>` | last `session` object from `GET /api/studio` (`id, project_id, agent_id, model_id, cwd, repo, branch, updated_at`) or `null` |

JS access (global enabled in `tauri.conf.json`):

```js
const invoke = window.__TAURI__?.core?.invoke;
if (invoke) {
  await invoke("health");         // { ok, version }
  await invoke("bridge_info");    // { port, url }
  await invoke("session_resume"); // session | null
}
```

- Window label: `main`. Capability `src-tauri/capabilities/main.json` grants `core:default` and IPC from `http://127.0.0.1:*` and `http://localhost:*` only.
- The same page still runs in a plain browser (Playwright). Always guard with `window.__TAURI__` and fall back to `fetch` on `bridge_info().url`.
- New command? Propose it in this file first; Axon owns `src-tauri`.

## Block 9 deliverables

| Item | Deliverable | File |
|---|---|---|
| Wrapper crate | runtime window over loopback, sidecar start, fallback page | `src-tauri/src/main.rs`, `src-tauri/frontend/index.html` |
| Command glue | health, bridge_info, session_resume | `src-tauri/src/main.rs` |
| Capability | label `main`, loopback remote IPC, `core:default` | `src-tauri/capabilities/main.json` |
| Bundle config | targets/icon/resources, bundle-only `externalBin` | `src-tauri/tauri.conf.json`, `packaging/tauri.bundle.conf.json` |
| Icons | 32/128/128@2x PNG, ico, icns, 512 PNG | `src-tauri/icons/` |
| Windows pipeline | NSIS + MSI into `dist/tauri/` | `packaging/tauri.ps1` |
| macOS/Linux pipeline | universal dmg; AppImage/deb/rpm | `packaging/tauri.sh` |
| CI | WebKitGTK deps, tauri-cli 2.12.4, macOS targets, `dist/**` upload | `.github/workflows/release.yml` |

## Decisions (still valid)

1. **Studio is a hash route.** `#studio` opens the overlay; `/` stays on the body.
2. **No duplicated server.** The wrapper never serves HTTP; it reads the noema-app loopback bridge.
3. **Bundle-only overrides.** Sidecar + UI/assets resources live in `packaging/tauri.bundle.conf.json`, so plain `cargo check --workspace` does not require a built UI or staged binary.
4. **Sidecar naming.** `bundle.externalBin: binaries/noema-app`; packaging stages `noema-app-<target-triple>[.exe]` (macOS universal uses `noema-app-universal-apple-darwin`).
5. **Remote IPC is narrow.** Only `127.0.0.1` and `localhost` patterns; do not widen the capability.
6. **No `git push`.** Owner reviews and pushes.

## Open items / risks

1. **macOS/Linux installer formats are CI-defined.** Windows NSIS + MSI are built and inspected locally; dmg/AppImage/deb/rpm still need the first CI tag run. Signing stays deferred.
2. **macOS resource lookup.** UI/assets land in `Contents/Resources`; the wrapper passes them to the sidecar explicitly. If Tauri changes resource layout, check `docs/PLATFORMS.md` section 6.
3. **Universal macOS build** needs both Apple targets (CI installs them) and stays unsigned until the owner supplies a Developer ID.
4. **Sidecar lifecycle (verified).** Cold start spawns `noema-app serve`; closing the window kills only the child it spawned (desktop 30384 / sidecar 20080, 2026-10-31). An already-running bridge is still left alive.
5. **Session diff / live usage writer** from Block 8 remain open (Thalamus + git probe).
6. **Full-viewport Brain route** remains Retina's navigation follow-up.
7. **App icon is a generated placeholder** (concentric rings). Replace `src-tauri/app-icon.png` and rerun `cargo tauri icon src-tauri/app-icon.png` before branding.
8. **Security surface (S1).** `src-tauri/capabilities/main.json` accepts remote IPC from any loopback port (`http://127.0.0.1:*`, `http://localhost:*`), `app.withGlobalTauri` is `true` and `app.security.csp` is `null`. Any page served on any loopback port inside this window can call `health`, `bridge_info` and `session_resume`. Narrow the patterns to the Studio port and set a CSP before shipping.

## Evidence

- `cargo check --workspace`: green with `noema-desktop` in the workspace.
- `cargo test --workspace`: 58 tests green.
- `cargo check -p noema-desktop` with `crates/app/ui/dist` renamed away: green (bundle-only resources/sidecar).
- `cargo tauri build --config packaging/tauri.bundle.conf.json --no-bundle`: `target/release/noema-desktop.exe` built in 3m23s.
- `docs/evidence/block9-desktop.png`: Tauri window (1136x739) with the full 3D body, layer sliders, agent rail and the 60 fps counter, captured via `PrintWindow` from the release binary.
- `docs/evidence/block9-desktop-autostart.png`: same window from the cold-start run: URL `http://127.0.0.1:4173/`, 60 fps, full 3D body.
- Autostart sequence: 4173 free and no `noema-app` before launch; wrapper pid 30384, sidecar 20080; `GET /api/health` -> `{"ok":true,"version":"0.1.0"}`; `WM_CLOSE` left neither process and no listener on 4173.
- Evidence hygiene: the 17 block5-8 PNGs regenerated during Soma's re-run were restored by the owner before this commit; `git status` was clean at start.
- `packaging/tauri.ps1 -SkipBuild`: NSIS 5,431,825 bytes + MSI 8,298,496 bytes; WiX `main.wxs` includes `noema-app.exe`, `ui/` and `assets/`.
- Bundle config: `cargo tauri build --help` / tauri-build accepted `targets`, icons, capability and the bundle override.
