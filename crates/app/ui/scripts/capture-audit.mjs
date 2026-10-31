// Block 10.1 baseline capture. PNG never touches the repo: CDP writes WebP.
import { chromium } from 'playwright';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const base = process.env.NOEMA_URL ?? 'http://127.0.0.1:4199';
const outDir = resolve('../../../docs/evidence');
mkdirSync(outDir, { recursive: true });

const meta = { capturedAt: new Date().toISOString(), base, shots: [] };
const browser = await chromium.launch({
  headless: false,
  args: ['--use-gl=angle', '--use-angle=d3d11', '--ignore-gpu-blocklist', '--disable-backgrounding-occluded-windows'],
});
const context = await browser.newContext({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: 1 });
const page = await context.newPage();
const cdp = await context.newCDPSession(page);

async function shot(name, clip) {
  const { data } = await cdp.send('Page.captureScreenshot', {
    format: 'webp', quality: 82, fromSurface: true, captureBeyondViewport: false, ...(clip ? { clip } : {}),
  });
  writeFileSync(resolve(outDir, name), Buffer.from(data, 'base64'));
  meta.shots.push({ file: `docs/evidence/${name}`, clip: clip ?? null });
  console.log(name);
}

await page.goto(`${base}/`);
await page.waitForFunction(() => window.__noema?.ready?.() === true, null, { timeout: 90_000 });
await page.waitForFunction(() => window.__noema.atlasReady() === true, null, { timeout: 90_000 });
await page.waitForTimeout(1500);
meta.atlas = await page.evaluate(() => window.__noema.atlasStats());
meta.debug = await page.evaluate(() => window.__noema.debug());
meta.agents = await page.evaluate(() => window.__noema.agentCount());
await shot('design-baseline-body.webp');
await shot('design-baseline-legs.webp', { x: 600, y: 430, width: 400, height: 440, scale: 2 });

await page.locator('#roster button').first().click();
await page.waitForTimeout(700);
await shot('design-baseline-rail.webp');
await page.locator('#rail-close').click();
await page.waitForTimeout(400);

await page.locator('#machine-btn').click();
await page.waitForTimeout(700);
await shot('design-baseline-machine.webp');
await page.locator('#machine-close').click();
await page.waitForTimeout(400);

await page.locator('#brain-btn').click();
await page.waitForTimeout(1600);
await shot('design-baseline-brain.webp');

await page.locator('#studio-btn').click();
await page.waitForFunction(() => window.__noema?.studio?.() != null, null, { timeout: 30_000 });
await page.waitForTimeout(1000);
await shot('design-baseline-studio.webp');

writeFileSync(resolve(outDir, 'design-baseline.json'), `${JSON.stringify(meta, null, 2)}\n`);
await browser.close();
