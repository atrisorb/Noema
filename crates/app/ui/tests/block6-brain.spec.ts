import { expect, test, type Page } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const evidence = resolve('../../../docs/evidence');
mkdirSync(evidence, { recursive: true });

async function boot(page: Page) {
  await page.goto('/');
  await page.waitForFunction(() => window.__noema?.ready?.() === true, null, { timeout: 90_000 });
  await page.waitForTimeout(400);
}

test('brain graph stays crisp, tethered and switches wake/dream', async ({ page }) => {
  await boot(page);
  await page.evaluate(() => window.__noema.brainMode(true));
  await expect(page.locator('body')).toHaveClass(/brain-mode/);
  await page.waitForTimeout(1600);

  const near = await page.evaluate(() => window.__noema.brain());
  expect(near.available).toBe(true);
  expect(near.visible).toBe(true);
  expect(near.nodes).toBeGreaterThan(10);
  expect(near.edges).toBeGreaterThan(0);
  expect(near.labels).toBeGreaterThan(0);
  expect(near.maxRadius).toBeLessThanOrEqual(1.001);
  await page.screenshot({ path: resolve(evidence, 'block6-brain-zoom.png') });

  const first = (await page.evaluate(() => window.__noema.brain().nodeList))[0];
  const spot = await page.evaluate((id) => window.__noema.brainScreen(id), first.id);
  expect(spot).not.toBeNull();
  await page.mouse.click(spot.x, spot.y);
  await expect(page.locator('#brain-card')).toHaveClass(/open/);
  await expect(page.locator('#brain-card-title')).toHaveText(first.title);
  await page.waitForTimeout(400);
  await page.screenshot({ path: resolve(evidence, 'block6-brain-thought.png') });

  await page.mouse.move(800, 450);
  await page.mouse.wheel(0, 3000);
  await page.waitForTimeout(700);
  const far = await page.evaluate(() => window.__noema.brain());
  expect(far.cameraDistance).toBeGreaterThan(near.cameraDistance);
  expect(far.maxRadius).toBeLessThanOrEqual(1.001);
  await page.screenshot({ path: resolve(evidence, 'block6-brain-far.png') });

  await page.evaluate(() => window.__noema.dreamMode(true));
  await expect(page.locator('#dream-btn')).toHaveText('wake');
  await page.waitForTimeout(900);
  expect((await page.evaluate(() => window.__noema.brain())).mode).toBe('dream');
  await page.screenshot({ path: resolve(evidence, 'block6-brain-dream.png') });

  await page.evaluate(() => window.__noema.dreamMode(false));
  await page.waitForTimeout(400);
  const persisted = await page.evaluate(() => window.__noema.brain().dream);
  expect(persisted?.persisted).toBe(true);
  expect(persisted?.thought_count).toBe(near.nodes);

  await page.evaluate(() => window.__noema.brainMode(false));
  await page.waitForTimeout(400);
  const off = await page.evaluate(() => window.__noema.brain());
  expect(off.visible).toBe(false);
  expect(off.mode).toBe('wake');
});
