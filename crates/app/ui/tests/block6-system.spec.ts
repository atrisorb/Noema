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

test('machine profile drives the renderer and accepts an override', async ({ page }) => {
  await boot(page);
  const detected = await page.evaluate(() => window.__noema.machineProfile());
  expect(detected?.profile.platform).toBeTruthy();
  expect(detected?.profile.logical_cores).toBeGreaterThan(0);
  expect(detected?.tuning.render.pixel_ratio_cap).toBeGreaterThan(0);
  expect(detected?.tuning.render.particle_budget).toBeGreaterThan(0);
  expect(Array.isArray(detected?.tuning.reasons)).toBe(true);

  await page.locator('#machine-btn').click();
  await expect(page.locator('#machine-panel')).toHaveClass(/open/);
  await expect(page.locator('#machine-specs')).toContainText('ram');
  await expect(page.locator('#machine-specs')).toContainText('quality');

  await page.locator('#machine-particles').fill('1200');
  await page.locator('#machine-save').click();
  await expect(page.locator('#machine-status')).toContainText('saved');
  const patched = await page.evaluate(() => window.__noema.machineProfile());
  expect(patched?.tuning.render.particle_budget).toBe(1200);
  expect(patched?.overrides).toMatchObject({ render: { particle_budget: 1200 } });

  await page.screenshot({ path: resolve(evidence, 'block6-hardware.png') });
});
