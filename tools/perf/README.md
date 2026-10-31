# Perf harness (NOEMA 3D scene)

Frame-time measurement for the same scene in two runtimes: the Playwright browser
baseline and the Tauri v2 / WebView2 wrapper. One sampler, one comparison.

## Files

| File | Role |
|---|---|
| `frame-sampler.js` | Injected into the page. `__noemaPerf.info()` (GPU, DPR, canvas, draw calls) and `__noemaPerf.sample(ms)` (rAF deltas → fps/p50/p95/p99/hitches). |
| `webview2-fps.mjs` | Attaches over CDP, samples the scene, compares against the browser baseline, writes JSON + screenshot. |
| `crates/app/ui/tests/block9-perf.perf.ts` | Browser baseline: full atlas, brain, cellular zoom, DPR 2. |
| `crates/app/ui/playwright.perf.config.ts` | Isolated port `:4199` and DB so perf runs never collide with functional suites on `:4173`. |

Metrics: `fpsMean = 1000/mean(delta)`; a hitch is a frame over 25 ms (1.5x the
16.67 ms budget); `droppedFrames` counts frames over 1000/55 ms.

## Browser baseline

```powershell
cd crates/app/ui
npx playwright test -c playwright.perf.config.ts
```

Writes `docs/evidence/block9-baseline.json` plus `block9-baseline-*.png`.
The config runs `target/debug/noema-app.exe`, so `cargo build -p noema-app` first
when the app changed, and `npm run build` when `src/` changed.

Measured 2026-10-31 on HP-ATR at 8fb6845 (GTX 1650, WebGL2 ANGLE D3D11,
Chromium 156): full atlas 60.1 fps / p99 17.0 ms / 0 hitches / 841 draw calls /
292k triangles; brain and cellular zoom also 60.1 fps.

DPR 2 clamps to the machine cap 1.75 (2800x1575, 4.4 MP). That size is fill-rate
bound on this GPU: 60.1 fps with the desktop idle; 35.4 fps mean (p99 50 ms, 121
hitches, 24.2 hitches/s) while a second live renderer drew the same scene at
60 fps. Treat the DPR 2 row as a headroom measurement, not the shipped window.

Wrapper: default window 1120x700 (src-tauri/src/main.rs), 60.1 fps / p99 17.0 ms /
0 hitches (port 4299, WebView2 Runtime 156.0.4314.50).

## WebView2 wrapper

CDP is the only attach path that needs no change to the wrapper crate. Launch the
wrapper with the extra browser argument and let it attach:

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
$env:NOEMA_APP_PORT = "4199"        # optional, must match the server the window shows
node tools/perf/webview2-fps.mjs    # defaults: --cdp http://127.0.0.1:9222
```

The harness waits for `__noema.ready()` and `atlasReady()`, samples 5 s, screenshots
to `docs/evidence/block9-webview2.png`, and fails when fps is below 90% of the
baseline, p99 above 1.5x baseline + 2 ms, or hitches/s above baseline + 1.
Flags: `--ms`, `--out`, `--shot`, `--baseline`, `--no-compare`, `--url-pattern`.

Smoke-test without a wrapper (same CDP surface, same metrics):

```powershell
node tools/perf/webview2-fps.mjs --launch --url http://127.0.0.1:4199/ --cdp http://127.0.0.1:9333
```

`--launch` drives Chromium, not WebView2. The harness derives `engine` from the CDP
user agent (only an `Edg/` UA counts as WebView2) and, on the launch path, writes
`note: "self-test only, not a WebView2 measurement"`. Use it to validate the
harness, never as wrapper evidence.

Host: WebView2 Runtime 156.0.4314.50 (EdgeWebView/Application), Tauri 2.12.5.

## Linux: WebKitGTK caveats

- **No CDP.** WebKitGTK does not implement the Chrome DevTools Protocol, so this
  harness cannot attach on Linux. Use `tauri-driver` + WebKitWebDriver and inject
  `frame-sampler.js` with `execute_script`, or keep a Chromium build of the same
  scene as the comparison arm.
- **Compositor variance.** Wayland + DMA-BUF vs X11 changes frame pacing; record
  the session type with every number. `WEBKIT_DISABLE_DMABUF_RENDERER=1` fixes a
  black canvas on some Mesa drivers but usually lowers fps — treat it as a
  fallback, not a quality mode.
- **vsync is not 60 Hz everywhere.** Compare against the measured refresh period of
  the same host, not a hard-coded 16.67 ms.
- **GPU process flags differ.** `WEBKIT_DISABLE_COMPOSITING_MODE=1` forces software
  compositing and cannot validate the 60 fps target.

## macOS

WKWebView has no CDP either; `safaridriver` is not supported by `tauri-driver`.
Validated macOS wrapper fps stays manual (Instruments) until a WebDriver-capable
path is available.
