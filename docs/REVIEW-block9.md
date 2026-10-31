# Block 9 review — Tauri v2 desktop wrapper, bundles, perf baseline

- **Reviewer:** Cortex (Chief Architect & Governance Lead)
- **Date:** 2026-10-31
- **Range:** `41002b9..b77f17f` (HEAD `b77f17f`, tree clean)
- **Commits:** 91ad5c1, eb3325b, 8fb6845, f890aa4, d8bccca, b77f17f

## Verdict

**APPROVE.** No blocking code, evidence or contract defects. The shipped wrapper,
bundle config, installer payload and perf evidence check out. Four low-severity
documentation/traceability items plus one info note (D1-D4) should be corrected;
they do not change the artifacts or the command contract.

## Scope

- 51 files, +5202/-734. `Cargo.lock` (+3767, generated) and PNG evidence (~2.8 MB)
  dominate the churn; every path belongs to block 9: `src-tauri/`,
  `packaging/tauri*`, `tools/perf/`, `crates/app/ui` base plumbing, CI, docs.
- No unrelated rewrites. The Studio hash sync in `studio.ts`/`main.ts` goes beyond
  base resolution, but it is what the wrapper spec exercises (open/close without
  reload) and it stays inside the block's navigation surface.
- Handwritten logic ≈ 660 LOC (main.rs 145, perf tools 167, tests 167, packaging
  scripts 69, platform.ts 47, UI deltas ~37, fallback page 29). Over the 400-LOC
  default and no rationale is recorded (P1). Blocks 2 and 3 recorded theirs.
- Generated Tauri scaffold is minimal: `build.rs`, `tauri.conf.json`, icons,
  capability. No `src-tauri/gen` committed (git-ignored).

## Correctness

- `src-tauri/tauri.conf.json`: identifier `dev.noema.desktop`, `frontendDist ./frontend`,
  no static windows (built at runtime, label `main`), icons 32/128/128@2x/ico/icns/512,
  `bundle.targets = all`. Sidecar and `ui/`+`assets/` resources exist only in the
  bundle-time override, so a clean checkout compiles without a UI dist (verified).
- Sidecar: resolved next to the exe (or `NOEMA_APP_BIN`), started only when
  `/api/health` is down, receives `serve --port <u16>` plus `--ui`/`--assets` from the
  resource dir, `CREATE_NO_WINDOW` on Windows, killed on `RunEvent::Exit` when the
  shell spawned it. Existing bridges are reused and never killed.
- No shell injection: `Command` with argv, port parsed as `u16`, no interpolation into
  a shell; HTTP paths are constants.
- `platform.ts`: empty/absent base keeps the plain browser path at same origin; trailing
  slashes trimmed; `assetBase` defaults to `apiBase`; `isTauri()` gate only affects retry
  count. Wrapper spec asserts the injected base is used and that `/api/` + atlas assets
  never escape to the origin root (2/2 green).
- Mounted window is `WebviewUrl::External(http://127.0.0.1:<port>)`; the same page also
  boots as a plain browser page. No UI code calls `invoke`, coherent with same-origin HTTP.
- Low-risk gaps recorded as findings: the wrapper spec does not assert `/draco/` escapes
  (T1); the 60×100 ms health poll runs before window creation (C1); a crash/hard kill
  leaves the sidecar alive (C2); the fallback page does not reload when the sidecar
  becomes healthy (C3).

## Contract

- `crates/app/ui/WRAPPER.md` and `docs/AGENT_HANDOFF.md` publish the same three commands
  (`health`, `bridge_info`, `session_resume`); signatures and return shapes match
  `src-tauri/src/main.rs` (lines 95-115). No drift.
- UI consumes no command, and the window loads the loopback origin, so HTTP is the only
  data path. That is internally consistent; a future bundled-UI origin would need the
  documented injected base or CORS.
- Nit: WRAPPER.md says retries gate on `__TAURI_INTERNALS__`; the code's `isTauri()` also
  accepts `__TAURI__` (with `withGlobalTauri: true`, both exist). Retry budget is
  12×300 ms ≈ 3.3 s, not the documented "~3.5 s".

## Docs and evidence

- `docs/MASTER_PLAN.md` 355-360: all block-9 items ticked; claims match shipped files.
  Tauri 2.12.5 / tauri-build 2.7.4 are the latest stable on crates.io; tauri-cli 2.12.4
  is the latest stable CLI (2.12.5 does not exist).
- Installer sizes are exact: `dist/tauri/NOEMA_0.1.0_x64-setup.exe` = 5,431,825 B,
  `NOEMA_0.1.0_x64_en-US.msi` = 8,298,496 B. Read-only MSI tables show
  `noema-app.exe`, 3 GLBs incl. `body_full.glb`, the Draco decoder, and directories
  `ui/ assets/ draco/ anatomy/` — the layout the wrapper resolves. No installer was run.
- Both desktop screenshots are 1136x739 and show the real 3D body; the WebView2 capture is
  1120x700 (the shell's default inner size) with UA `Edg/156` — consistent with a wrapper
  window, not a plain Chromium launch. The DPR 2 baseline screenshot is 3200x1800, i.e. the
  documented 1.75 cap on a 1600x900 viewport.
- Perf numbers cross-check against `docs/evidence/block9-webview2.json`:
  60.081 fps mean, p99 17.0 ms, 0 hitches; baseline full-atlas 60.110 fps, p99 17.0 ms.
  The documented "60.1 / 17.0 / 0" is correct.
- D1: `tools/perf/README.md:30` and `:38` publish p99 16.9 / 16.8 ms; the committed JSON
  says 17.0 ms in both cells (`block9-baseline.json`, `block9-webview2.json`). Stale from
  the pre-b77f17f run. Line 34's "25-33 fps" also differs from the recorded DPR2 mean
  35.4 fps (p50 30.0) — label it as a range of p50 frames or refresh the numbers.
- D2: `docs/AGENT_HANDOFF.md:18` says the re-run left `block9-webview2.json`
  "byte-identical to the commit". Git shows b77f17f changed it (url 4199→4299, fps
  60.121→60.081, p99 16.8→17.0). The conclusion (numbers stand) is right; the provenance
  sentence is not.
- D3: `docs/evidence/block9-harness-selftest.json:4` labels the run "webview2 (CDP)", but
  its UA is plain `Chrome/156` at 1280x720 — it was produced with `--launch`
  (`tools/perf/webview2-fps.mjs:73` hardcodes the engine string). Relabel or derive the
  engine from the page UA.
- D4: "WebView2 Runtime 156.0.4314.50" is true on this host (installed under
  `C:\Program Files (x86)\Microsoft\EdgeWebView\Application`) but is not recorded in any
  evidence JSON (UA reports only 156.0.0.0). Add it to the harness output or cite the probe.
- CHANGELOG 0.5.4 is accurate on code, versions, sizes and the desktop/standalone split.

## Hygiene

- No tracked secrets, `.env*`, `target/`, `dist/`, `node_modules/`, `src-tauri/binaries/`
  or installers; largest tracked blob is the pre-existing 8.2 MB anatomy GLB. Committed
  evidence PNGs are ≤ 615 KB. `.gitignore` adds `src-tauri/gen/` and `src-tauri/binaries/`.
- New dependencies tauri 2.12.5 + tauri-build 2.7.4 are latest stable; rationale is in
  CHANGELOG 0.5.4. Driftwatch digest: 8 platform page updates, none touching Rust/Node or
  Tauri; `check` returned no new entries.
- No remote is configured and `main` is the only branch: no push exists in history.

## Commands run

| Command | Result |
|---|---|
| `git status --porcelain` before/after | clean (one test-regenerated evidence PNG restored) |
| `cargo check --workspace` | exit 0, `Finished` |
| `cargo test --workspace` | exit 0, 58 passed (7 app, 2 cli, 8 core, 1 daemon, 11 ecosystem, 8 hippocampus, 21 inference) |
| `cargo check -p noema-desktop` with `crates/app/ui/dist` renamed away | exit 0, dist restored |
| `npx playwright test tests/block9-wrapper.spec.ts` | 2 passed (21.9 s) |
| `driftwatch digest` / `check` | 8 page updates, none dependency-relevant; no new entries |
| MSI `File`/`Directory` tables (WindowsInstaller COM, read-only) | sidecar + ui/assets/draco/anatomy confirmed |
| WebView2 runtime directory listing | 156.0.4314.50 |

Not run (out of brief): installer execution, a fresh `cargo tauri build`, the full
Playwright suite, macOS/Linux bundles.

## Findings

| ID | Severity | Location | Issue |
|---|---|---|---|
| D1 | low | `tools/perf/README.md:30,34,38` | Stale p99 (16.9/16.8 vs 17.0) and DPR2 range vs JSON |
| D2 | low | `docs/AGENT_HANDOFF.md:18` | "byte-identical" claim contradicted by b77f17f |
| D3 | low | `docs/evidence/block9-harness-selftest.json:4`, `tools/perf/webview2-fps.mjs:73` | Chromium `--launch` run mislabeled as WebView2 |
| D4 | info | `tools/perf/README.md:39` | WebView2 runtime build not captured in evidence JSON |
| P1 | low | block scope | ~660 handwritten LOC > 400 default, rationale not recorded |
| S1 | low | `src-tauri/capabilities/main.json:8-13`, `tauri.conf.json:14` | IPC open to any loopback port; `withGlobalTauri`; `csp: null` |
| C1 | low | `src-tauri/src/main.rs:84-89` | Up to ~6 s (pathological ~54 s) health poll blocks window creation |
| C2 | low | `src-tauri/src/main.rs:139-142` | Kill only on `RunEvent::Exit`; crash/hard kill orphans the sidecar |
| C3 | low | `src-tauri/src/main.rs:122-127` | Fallback page never retries when the sidecar becomes healthy |
| T1 | low | `crates/app/ui/tests/block9-wrapper.spec.ts:20-24` | Escape guard covers `/api/` and atlas only, not `/draco/` |

## Residual risks

1. **Placeholder icon** (concentric rings). Replace `src-tauri/app-icon.png`, rerun
   `cargo tauri icon`, before branding.
2. **macOS/Linux bundles unproven** until the first tag run; macOS signing deferred.
3. **DPR 2 fill-rate lock**: the 1.75 cap renders 2800x1575 (4.4 MP); the committed row is
   35.4 fps mean with 121 hitches on the GTX 1650, and the spec only asserts > 20 fps.
   Shipped window is 1120x700 at DPR 1, so this is headroom data, not a regression.
4. **Baseline conditions**: the DPR 2 row was measured with another live renderer on the
   same GPU (documented in `tools/perf/README.md`); comparable numbers need the same
   condition. The DPR 1 baseline and wrapper rows are consistent within 0.03 fps.
5. **Loopback IPC surface**: capability allows any `127.0.0.1:*`/`localhost:*` origin with
   `core:default`; only three read-only commands exist, and the UI invokes none.
6. **Sidecar lifecycle**: reused bridges survive window close by design; a crashed shell can
   leave `noema-app` listening on the port until the next launch reuses or the user stops it.

## Push note

Code, config, tests and evidence are ready for the owner to push. The D1-D4 doc edits are
non-blocking; fix them in a small doc commit (Axon's file, not this review).
