import { expect, test } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

test.use({
  headless: true,
  launchOptions: {
    args: [
      '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader',
      '--ignore-gpu-blocklist', '--enable-unsafe-webgpu',
    ],
  },
});

const evidence = resolve('evidence');
mkdirSync(evidence, { recursive: true });

test('wrapper base drives api and anatomy assets, no escape to origin root', async ({ page }) => {
  const seen: string[] = [];
  const escaped: string[] = [];
  page.on('request', (request) => {
    const path = new URL(request.url()).pathname;
    if (path.startsWith('/api/') || path.startsWith('/assets/anatomy/')) escaped.push(path);
  });
  await page.route('**/__wrapper__/**', async (route) => {
    const url = new URL(route.request().url());
    const target = url.origin + url.pathname.replace('/__wrapper__', '') + url.search;
    seen.push(new URL(target).pathname);
    await route.continue({ url: target });
  });
  await page.addInitScript(() => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = { mock: true };
    (window as unknown as Record<string, unknown>).__NOEMA_API_BASE__ = '/__wrapper__';
  });

  await page.goto('/');
  await expect(page.locator('#roster button')).toHaveCount(7);
  await expect.poll(() => seen.some((p) => p === '/api/anatomy')).toBe(true);
  await expect.poll(() => seen.some((p) => p.includes('/assets/anatomy/atlas/'))).toBe(true);

  const platform = await page.evaluate(() => window.__noema.platform());
  expect(platform.tauri).toBe(true);
  expect(platform.apiBase).toBe('/__wrapper__');
  expect(platform.assetBase).toBe('/__wrapper__');
  expect(escaped).toEqual([]);
});

test('hash route restores the last session and clears without reload', async ({ page }) => {
  await page.addInitScript(() => {
    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  });
  await page.goto('/');
  await expect(page.locator('#scene')).toBeVisible();
  await expect(page.locator('#roster button')).toHaveCount(7);
  await expect(page.locator('#studio-panel')).not.toHaveClass(/open/);

  await page.evaluate(() => { location.hash = 'studio'; });
  await expect(page.locator('#studio-panel')).toHaveClass(/open/);
  await expect(page.locator('#role-grid select')).toHaveCount(5);
  const restored = await page.evaluate(() => window.__noema.studio());
  expect(restored.session?.repo).toBe('agentic-anatomy-workspace');
  expect(restored.session?.branch).toBe('main');
  expect(restored.widgets.length).toBeGreaterThan(0);
  await page.screenshot({ path: resolve(evidence, 'block9-wrapper-studio.png'), fullPage: false });

  await page.evaluate(() => { location.hash = ''; });
  await expect(page.locator('#studio-panel')).not.toHaveClass(/open/);

  await page.goto('/#studio');
  await expect(page.locator('#studio-panel')).toHaveClass(/open/);
  expect(await page.evaluate(() => location.hash)).toBe('#studio');
});
