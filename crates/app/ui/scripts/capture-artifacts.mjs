// Block 10.6/10.7 artifact evidence: gallery, dashboard, verify and exports. WebP/JSON.
import { chromium } from 'playwright';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const base = process.env.NOEMA_URL ?? 'http://127.0.0.1:4199';
const outDir = resolve('../../../docs/evidence');
mkdirSync(outDir, { recursive: true });
const meta = { capturedAt: new Date().toISOString(), base, shots: [], errors: [] };

const browser = await chromium.launch({ headless: false, args: ['--use-gl=angle', '--use-angle=d3d11', '--ignore-gpu-blocklist'] });
const context = await browser.newContext({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 1 });
const page = await context.newPage();
page.on('pageerror', (e) => meta.errors.push(String(e)));
page.on('console', (m) => { if (m.type() === 'error') meta.errors.push(m.text()); });
const cdp = await context.newCDPSession(page);
async function shot(name) {
  const { data } = await cdp.send('Page.captureScreenshot', { format: 'webp', quality: 85, captureBeyondViewport: false });
  writeFileSync(resolve(outDir, name), Buffer.from(data, 'base64'));
  meta.shots.push('docs/evidence/' + name);
  console.log(name);
}

await page.goto(base + '/');
await page.waitForFunction(() => window.__noema?.ready?.() === true, null, { timeout: 90_000 });
await page.waitForFunction(() => window.__noema.atlasReady() === true, null, { timeout: 90_000 });
await page.waitForTimeout(1200);

await page.locator('#artifacts-btn').click();
await page.waitForTimeout(500);
await shot('design-artifacts-gallery.webp');
meta.galleryCards = await page.locator('.aw-card').count();

await page.locator('.aw-card').first().click();
await page.waitForTimeout(600);
await shot('design-artifact-dashboard.webp');
meta.current = await page.evaluate(() => window.__noemaArtifacts.current());
meta.verify = await page.evaluate(() => window.__noemaArtifacts.verify());
meta.exportLengths = await page.evaluate(() => ({
  html: window.__noemaArtifacts.exportText('html').length,
  json: window.__noemaArtifacts.exportText('json').length,
  md: window.__noemaArtifacts.exportText('md').length,
}));

await page.locator('#aw-tabs button[data-pane="comments"]').click();
await page.locator('#aw-comment').fill('Check the chart label contrast');
await page.locator('#aw-comment-form button').click();
await page.waitForTimeout(200);
meta.comments = await page.locator('.aw-comment').count();
await page.locator('#aw-tabs button[data-pane="history"]').count();

writeFileSync(resolve(outDir, 'design-artifact-verify.json'), JSON.stringify(meta, null, 2) + '\n');
console.log(JSON.stringify({ verify: meta.verify?.ok, exports: meta.exportLengths, errors: meta.errors.length }));
await browser.close();
