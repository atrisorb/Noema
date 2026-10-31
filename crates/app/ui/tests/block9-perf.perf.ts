import { expect, test, type Page } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve } from 'node:path';

const evidence = resolve('../../../docs/evidence');
const sampler = fileURLToPath(new URL('../../../../tools/perf/frame-sampler.js', import.meta.url));
mkdirSync(evidence, { recursive: true });

type Stats = { fpsMean: number; fpsP50: number; p50: number; p95: number; p99: number; hitches: number; frames: number };
type Record_ = { scenario: string; stats: Stats; info: Record<string, unknown>; shot: string };

const runs: Record_[] = [];
const WINDOW = 5000;
let machine: unknown = null;

async function boot(page: Page) {
  await page.goto('/');
  await page.waitForFunction(() => (window as any).__noema?.ready?.() === true, null, { timeout: 90_000 });
  await page.waitForFunction(() => (window as any).__noema.atlasReady() === true, null, { timeout: 90_000 });
  await page.addScriptTag({ path: sampler });
  machine = await page.evaluate(() => (window as any).__noema.machineProfile());
  await page.bringToFront();
  await page.waitForTimeout(800);
  await page.evaluate(() => new Promise<void>((r) => requestAnimationFrame(() => requestAnimationFrame(() => r()))));
  await page.waitForTimeout(1500);
}

async function measure(page: Page, scenario: string, shot: string): Promise<Stats> {
  const info = await page.evaluate(() => (window as any).__noemaPerf.info());
  const stats = await page.evaluate((ms) => (window as any).__noemaPerf.sample(ms), WINDOW);
  const path = resolve(evidence, shot);
  await page.screenshot({ path });
  runs.push({ scenario, stats, info, shot: `docs/evidence/${shot}` });
  console.log(`${scenario}: ${stats.fpsMean.toFixed(1)} fps mean, p50 ${stats.p50.toFixed(1)} ms, p99 ${stats.p99.toFixed(1)} ms, hitches ${stats.hitches}, renderer ${info.renderer}`);
  return stats;
}

test.afterAll(() => {
  const doc = {
    capturedAt: new Date().toISOString(),
    host: process.env.COMPUTERNAME ?? 'unknown',
    engine: 'playwright-chromium (browser baseline)',
    viewport: { width: 1600, height: 900 },
    windowMs: WINDOW,
    machine,
    runs,
  };
  writeFileSync(resolve(evidence, 'block9-baseline.json'), `${JSON.stringify(doc, null, 2)}\n`);
});

test.describe('browser baseline dpr 1', () => {
  test.use({ deviceScaleFactor: 1 });

  test('full atlas holds the 60 fps budget', async ({ page }) => {
    await boot(page);
    const stats = await measure(page, 'full-atlas', 'block9-baseline-full.png');
    const info = runs.at(-1)!.info as { renderer: string; canvas: { width: number; cssWidth: number } };
    expect(info.renderer).not.toContain('SwiftShader');
    expect(info.canvas.width).toBe(Math.floor(info.canvas.cssWidth * 1));
    // Steady-state budget: p50, not mean, so a single startup stall cannot mask a real drop.
    expect(stats.fpsP50).toBeGreaterThan(55);
  });

  test('brain view and cellular zoom stay smooth', async ({ page }) => {
    await boot(page);
    await page.evaluate(() => (window as any).__noema.brainMode(true));
    await page.waitForTimeout(1200);
    const brain = await measure(page, 'brain-wake', 'block9-baseline-brain.png');
    expect(brain.fpsMean).toBeGreaterThan(50);

    await page.evaluate(() => (window as any).__noema.brainMode(false));
    await page.evaluate(() => (window as any).__noema.zoomToCellular());
    await page.waitForTimeout(1500);
    const cellular = await measure(page, 'cellular-zoom', 'block9-baseline-cellular.png');
    expect((await page.evaluate(() => (window as any).__noema.cameraDistance()))).toBeLessThan(0.6);
    expect(cellular.fpsMean).toBeGreaterThan(45);
  });
});

test.describe('browser baseline dpr 2', () => {
  test.use({ deviceScaleFactor: 2 });

  test('hidpi clamps to the pixel ratio cap and records the cost', async ({ page }) => {
    await boot(page);
    const stats = await measure(page, 'full-atlas', 'block9-baseline-dpr2.png');
    const info = runs.at(-1)!.info as { dpr: number; canvas: { width: number; cssWidth: number } };
    const cap = (machine as { tuning: { render: { pixel_ratio_cap: number } } }).tuning.render.pixel_ratio_cap;
    expect(info.dpr).toBe(2);
    expect(info.canvas.width).toBe(Math.floor(info.canvas.cssWidth * Math.min(info.dpr, cap)));
    // 4.4 MP is fill-rate bound on this host: another live WebView2 renderer halves it to 30 fps.
    // Record the cost; only guard against a collapse.
    expect(stats.fpsMean).toBeGreaterThan(20);
  });
});
