// Measures the NOEMA 3D scene inside a WebView2-wrapped window over CDP,
// using the same sampler and metrics as the Playwright browser baseline.
//
//   $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
//   node tools/perf/webview2-fps.mjs
//
// Exits 1 when the wrapper regresses against docs/evidence/block9-baseline.json.
// --launch <url> opens a Chromium with the same CDP surface first: use it to smoke-test
// this harness (and to measure the same scene) before the WebView2 wrapper exists.
import { createRequire } from 'node:module';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const require = createRequire(new URL('../../crates/app/ui/package.json', import.meta.url));
const { chromium } = require('playwright');

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, '../..');

const argv = process.argv.slice(2);
const arg = (name, fallback) => {
  const i = argv.indexOf(`--${name}`);
  return i >= 0 ? argv[i + 1] : fallback;
};
const cdp = arg('cdp', process.env.NOEMA_CDP_URL ?? 'http://127.0.0.1:9222');
const pattern = new RegExp(arg('url-pattern', process.env.NOEMA_WRAPPER_URL ?? '127\\.0\\.0\\.1|localhost|tauri://|https?://tauri'));
const windowMs = Number(arg('ms', '5000'));
const out = resolve(root, arg('out', 'docs/evidence/block9-webview2.json'));
const shot = resolve(root, arg('shot', 'docs/evidence/block9-webview2.png'));
const baselinePath = resolve(root, arg('baseline', 'docs/evidence/block9-baseline.json'));
const compare = !argv.includes('--no-compare');

const GPU_ARGS = [
  '--use-gl=angle', '--use-angle=d3d11', '--ignore-gpu-blocklist',
  '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding',
  '--disable-features=CalculateNativeWinOcclusion',
];
let launched = null;
if (argv.includes('--launch')) {
  const url = arg('url', 'http://127.0.0.1:4199/');
  const port = new URL(cdp).port || '9222';
  launched = await chromium.launch({ headless: false, args: [...GPU_ARGS, `--remote-debugging-port=${port}`] });
  const page = await launched.newPage();
  await page.goto(url);
  await page.waitForTimeout(1500);
}

const browser = await chromium.connectOverCDP(cdp);
const pages = browser.contexts().flatMap((c) => c.pages()).filter((p) => pattern.test(p.url()));
if (!pages.length) {
  console.error(`no page matching ${pattern} on ${cdp}; set WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 in the wrapper process`);
  await browser.close();
  process.exit(2);
}
const page = pages[0];
console.log(`attached: ${page.url()}`);

try { await page.bringToFront(); } catch { /* WebView2 window has no page focus API */ }
await page.waitForFunction(() => globalThis.__noema?.ready?.() === true, null, { timeout: 90_000 });
await page.waitForFunction(() => globalThis.__noema.atlasReady() === true, null, { timeout: 90_000 });
await page.addScriptTag({ path: resolve(root, 'tools/perf/frame-sampler.js') });
await page.waitForTimeout(800);

const info = await page.evaluate(() => globalThis.__noemaPerf.info());
const stats = await page.evaluate((ms) => globalThis.__noemaPerf.sample(ms), windowMs);
mkdirSync(dirname(out), { recursive: true });
await page.screenshot({ path: shot });

// Engine is derived from the CDP user agent: only an Edg/ UA is a real WebView2.
const ua = info.ua ?? '';
const chromeVersion = (ua.match(/Chrome\/(\d+(?:\.\d+)*)/) ?? [])[1] ?? 'unknown';
const isWebView2 = /Edg\//.test(ua);
const engine = isWebView2 ? `webview2 (CDP, Edge ${chromeVersion})` : `chromium (CDP, Chrome ${chromeVersion})`;

const doc = {
  capturedAt: new Date().toISOString(),
  host: process.env.COMPUTERNAME ?? 'unknown',
  engine,
  cdp,
  url: page.url(),
  windowMs,
  info,
  stats,
  shot: relative(root, shot).replaceAll('\\', '/'),
};
if (launched) doc.note = 'self-test only, not a WebView2 measurement: --launch drives Chromium over CDP';
if (!isWebView2) console.warn(`engine from UA is "${engine}"; not a WebView2 measurement`);

const checks = [];
let ok = true;
if (compare && existsSync(baselinePath)) {
  const baseline = JSON.parse(readFileSync(baselinePath, 'utf8'));
  const base = baseline.runs.find((r) => r.scenario === 'full-atlas' && r.info.dpr === 1) ?? baseline.runs[0];
  const fpsFloor = base.stats.fpsMean * 0.9;
  checks.push({ check: 'fps >= 90% of browser baseline', base: base.stats.fpsMean, got: stats.fpsMean, pass: stats.fpsMean >= fpsFloor });
  const p99Ceil = base.stats.p99 * 1.5 + 2;
  checks.push({ check: 'p99 <= 1.5x baseline + 2 ms', base: base.stats.p99, got: stats.p99, pass: stats.p99 <= p99Ceil });
  const hitchCeil = base.stats.hitchesPerSec + 1;
  checks.push({ check: 'hitches/s <= baseline + 1', base: base.stats.hitchesPerSec, got: stats.hitchesPerSec, pass: stats.hitchesPerSec <= hitchCeil });
  doc.baseline = { path: 'docs/evidence/block9-baseline.json', scenario: base.scenario, fpsMean: base.stats.fpsMean, p99: base.stats.p99 };
  ok = checks.every((c) => c.pass);
} else if (compare) {
  console.warn(`baseline missing at ${baselinePath}; recording only`);
}
doc.checks = checks;

writeFileSync(out, `${JSON.stringify(doc, null, 2)}\n`);
console.log(`${stats.fpsMean.toFixed(1)} fps mean | p50 ${stats.p50.toFixed(1)} ms | p99 ${stats.p99.toFixed(1)} ms | hitches ${stats.hitches} | ${info.renderer}`);
for (const c of checks) console.log(`${c.pass ? 'ok' : 'FAIL'} ${c.check}: ${Number(c.got).toFixed(2)} (base ${Number(c.base).toFixed(2)})`);
console.log(`json: ${out}`);
await browser.close();
if (launched) await launched.close();
process.exit(ok ? 0 : 1);
