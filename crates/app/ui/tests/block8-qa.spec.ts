import { expect, test, type Page } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const evidence = resolve('../../../docs/evidence');
mkdirSync(evidence, { recursive: true });

async function boot(page: Page) {
  await page.goto('/');
  await page.waitForFunction(() => window.__noema?.ready?.() === true, null, { timeout: 90_000 });
  await page.waitForTimeout(600);
}

test('renders at the device pixel ratio and keeps the frame loop alive', async ({ page }) => {
  await boot(page);

  const draw = await page.evaluate(() => {
    const canvas = document.querySelector<HTMLCanvasElement>('#scene')!;
    return {
      width: canvas.width,
      height: canvas.height,
      cssWidth: canvas.clientWidth,
      cssHeight: canvas.clientHeight,
      dpr: devicePixelRatio,
      cap: 1.5,
    };
  });
  expect(draw.width).toBeGreaterThanOrEqual(Math.floor(draw.cssWidth * Math.min(draw.dpr, draw.cap)) - 2);
  expect(draw.height).toBeGreaterThanOrEqual(Math.floor(draw.cssHeight * Math.min(draw.dpr, draw.cap)) - 2);

  const first = await page.evaluate(() => window.__noema.renderInfo().frame);
  await page.waitForTimeout(700);
  const second = await page.evaluate(() => window.__noema.renderInfo().frame);
  expect(second).toBeGreaterThan(first);
  await page.screenshot({ path: resolve(evidence, 'block8-render.png') });
});

test('brain stays crisp through zoom, wake and dream', async ({ page }) => {
  await boot(page);
  await page.evaluate(() => window.__noema.brainMode(true));
  await page.waitForTimeout(1600);

  const near = await page.evaluate(() => window.__noema.brain());
  expect(near.visible).toBe(true);
  expect(near.labels).toBeGreaterThan(0);
  expect(near.maxRadius).toBeLessThanOrEqual(1.001);
  await page.screenshot({ path: resolve(evidence, 'block8-neural-resolution.png') });

  await page.mouse.move(800, 450);
  await page.mouse.wheel(0, -2400);
  await page.waitForTimeout(500);
  const close = await page.evaluate(() => window.__noema.brain());
  expect(close.maxRadius).toBeLessThanOrEqual(1.001);
  expect(close.labels).toBeGreaterThan(0);

  await page.evaluate(() => window.__noema.dreamMode(true));
  await expect(page.locator('#dream-btn')).toHaveText('wake');
  await page.waitForTimeout(700);
  const frameA = await page.evaluate(() => window.__noema.renderInfo().frame);
  await page.waitForTimeout(500);
  const frameB = await page.evaluate(() => window.__noema.renderInfo().frame);
  expect(frameB).toBeGreaterThan(frameA);
  expect((await page.evaluate(() => window.__noema.brain())).mode).toBe('dream');
  await page.screenshot({ path: resolve(evidence, 'block8-dream.png') });
  await page.evaluate(() => window.__noema.dreamMode(false));
});

test('studio widgets render from the bridge and keep their grid', async ({ page }) => {
  await page.goto('/#studio');
  await expect(page.locator('#studio-panel')).toHaveClass(/open/);
  await expect(page.locator('.widget-card').first()).toBeVisible();

  const before = await page.evaluate(async () => (await fetch('/api/studio')).json());
  expect(before.widgets.length).toBeGreaterThanOrEqual(4);
  expect(before.usage.length).toBeGreaterThan(0);

  const card = page.locator('.widget-card').first();
  const id = await card.getAttribute('data-id');
  const box = await card.locator('.widget-head').boundingBox();
  await page.mouse.move(box!.x + 20, box!.y + 10);
  await page.mouse.down();
  await page.mouse.move(box!.x + 120, box!.y + 10, { steps: 8 });
  await page.mouse.up();

  await expect
    .poll(async () => {
      const snapshot = await page.evaluate(async () => (await fetch('/api/studio')).json());
      return snapshot.widgets.find((w: { id: string }) => w.id === id)?.x;
    })
    .not.toBe(before.widgets.find((w: { id: string }) => w.id === id)?.x);

  await page.screenshot({ path: resolve(evidence, 'block8-widgets.png') });
});
