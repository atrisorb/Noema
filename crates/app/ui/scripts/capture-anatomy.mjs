// Block 10.8 anatomy evidence: surface overlay, legs and pelvis. WebP/JSON.
import { chromium } from 'playwright';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const base = process.env.NOEMA_URL ?? 'http://127.0.0.1:4199';
const outDir = resolve('../../../docs/evidence');
mkdirSync(outDir, { recursive: true });
const meta = { capturedAt: new Date().toISOString(), base, shots: [] };
const browser = await chromium.launch({ headless: false, args: ['--use-gl=angle', '--use-angle=d3d11', '--ignore-gpu-blocklist'] });
const context = await browser.newContext({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 1 });
const page = await context.newPage();
const cdp = await context.newCDPSession(page);
async function shot(name, clip) {
  const { data } = await cdp.send('Page.captureScreenshot', { format: 'webp', quality: 85, captureBeyondViewport: false, ...(clip ? { clip } : {}) });
  writeFileSync(resolve(outDir, name), Buffer.from(data, 'base64'));
  meta.shots.push('docs/evidence/' + name);
  console.log(name);
}
await page.goto(base + '/');
await page.waitForFunction(() => window.__noema?.ready?.() === true, null, { timeout: 90_000 });
await page.waitForFunction(() => window.__noema.atlasReady() === true, null, { timeout: 90_000 });
await page.waitForTimeout(1500);
meta.atlas = await page.evaluate(() => window.__noema.atlasStats());
meta.debug = await page.evaluate(() => window.__noema.debug());
meta.sample = await page.evaluate(() => window.__noema.sample());
await shot('design-body-surface.webp');
await shot('design-body-legs.webp', { x: 600, y: 430, width: 400, height: 440, scale: 2 });
await shot('design-pelvis.webp', { x: 700, y: 410, width: 200, height: 210, scale: 3 });
writeFileSync(resolve(outDir, 'design-anatomy.json'), JSON.stringify(meta, null, 2) + '\n');
await browser.close();
