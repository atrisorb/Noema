// Block 10.5 shell evidence: Cavity, Bone, inspector, mobile. WebP only.
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

async function shot(name) {
  const { data } = await cdp.send('Page.captureScreenshot', { format: 'webp', quality: 84, fromSurface: true, captureBeyondViewport: false });
  writeFileSync(resolve(outDir, name), Buffer.from(data, 'base64'));
  meta.shots.push({ file: `docs/evidence/${name}`, viewport: page.viewportSize(), theme: await page.evaluate(() => document.documentElement.dataset.theme) });
  console.log(name);
}
async function boot() {
  await page.goto(`${base}/`);
  await page.waitForFunction(() => window.__noema?.ready?.() === true, null, { timeout: 90_000 });
  await page.waitForFunction(() => window.__noema.atlasReady() === true, null, { timeout: 90_000 });
  await page.waitForTimeout(1500);
}

await boot();
await shot('design-shell-cavity.webp');

await page.locator('#layers-btn').click();
await page.waitForTimeout(500);
await shot('design-shell-layers.webp');
await page.locator('#layers-close').click();
await page.waitForTimeout(300);

await page.locator('#theme-btn').click();
await page.waitForTimeout(600);
await shot('design-shell-bone.webp');

await page.reload();
await page.waitForFunction(() => window.__noema?.ready?.() === true, null, { timeout: 90_000 });
meta.themePersisted = await page.evaluate(() => document.documentElement.dataset.theme);
meta.agentCount = await page.evaluate(() => window.__noema.agentCount());
meta.fps = await page.locator('#fps').innerText();
await page.locator('#theme-btn').click();
await page.waitForTimeout(300);

await page.setViewportSize({ width: 390, height: 844 });
await page.waitForTimeout(700);
await shot('design-shell-mobile.webp');

writeFileSync(resolve(outDir, 'design-shell.json'), `${JSON.stringify(meta, null, 2)}\n`);
await browser.close();
