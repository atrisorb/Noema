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

test('brain hash route #brain opens full viewport and Escape closes it', async ({ page }) => {
  await boot(page);

  // Navigate directly via hash
  await page.evaluate(() => { location.hash = 'brain'; });
  await page.waitForTimeout(600);

  // Verify brain mode is activated
  await expect(page.locator('body')).toHaveClass(/brain-mode/);
  await expect(page.locator('#brain-btn')).toHaveClass(/on/);

  // Status filament and command bar remain visible
  await expect(page.locator('#hud')).toBeVisible();
  await expect(page.locator('#status-filament')).toBeVisible();

  // Press Escape to dismiss
  await page.keyboard.press('Escape');
  await page.waitForTimeout(400);

  // Verify brain mode closed and hash cleared
  await expect(page.locator('body')).not.toHaveClass(/brain-mode/);
  const hashAfterEscape = await page.evaluate(() => location.hash);
  expect(hashAfterEscape === '' || hashAfterEscape === '#').toBe(true);

  // Re-open #brain then navigate to #studio without breakage
  await page.evaluate(() => { location.hash = 'brain'; });
  await page.waitForTimeout(400);
  await expect(page.locator('body')).toHaveClass(/brain-mode/);

  await page.evaluate(() => { location.hash = 'studio'; });
  await page.waitForTimeout(500);
  await expect(page.locator('body')).not.toHaveClass(/brain-mode/);
  await expect(page.locator('#studio-panel')).toHaveClass(/open/);

  // Return to clean state
  await page.evaluate(() => { location.hash = ''; });
  await page.waitForTimeout(300);
});
