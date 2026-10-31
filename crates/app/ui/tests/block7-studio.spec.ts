import { expect, test, type Page } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const evidence = resolve('../../../docs/evidence');
mkdirSync(evidence, { recursive: true });

async function openStudio(page: Page) {
  await page.goto('/#studio');
  await expect(page.locator('#studio-panel')).toHaveClass(/open/);
  await page.waitForFunction(() => window.__noema?.studio?.() != null);
  await expect(page.locator('#role-grid select')).toHaveCount(5);
  await expect(page.locator('.widget-card').first()).toBeVisible();
}

test('studio restores the last session and persists roles, widgets and layout', async ({ page }) => {
  await openStudio(page);

  const boot = await page.evaluate(() => window.__noema.studio());
  expect(boot.session?.repo).toBe('agentic-anatomy-workspace');
  expect(boot.session?.branch).toBe('main');
  expect(boot.projects).toBeGreaterThanOrEqual(1);
  expect(boot.sessions).toBeGreaterThanOrEqual(7);
  expect(boot.widgets.length).toBeGreaterThanOrEqual(4);
  expect(boot.usage).toBeGreaterThan(0);
  await expect(page.locator('#studio-sub')).toContainText('main');

  await page.locator('#role-scope').selectOption('');
  await page.locator('select[data-role="chat"]').selectOption('claudeapikey/claude-opus-5-5');
  await expect
    .poll(async () => (await page.evaluate(() => window.__noema.studio())).roles.chat.global)
    .toBe('claudeapikey/claude-opus-5-5');

  await page.locator('#widget-name').fill('cost by soma');
  await page.locator('#widget-metric').selectOption('cost');
  await page.locator('#widget-dim').selectOption('agent');
  await page.locator('#widget-filter-dim').selectOption('agent');
  await page.locator('#widget-filter-val').selectOption('soma');
  await page.locator('#widget-add').click();
  const card = page.locator('.widget-card', { hasText: 'cost by soma' });
  await expect(card).toBeVisible();
  await page.screenshot({ path: resolve(evidence, 'block7-widget-filter.png') });

  const placed = (await page.evaluate(() => window.__noema.studio().widgets)).find((w) => w.name === 'cost by soma');
  expect(placed?.spec.filters).toEqual([{ dimension: 'agent', value: 'soma' }]);

  await card.scrollIntoViewIfNeeded();
  const head = await card.locator('.widget-head').boundingBox();
  await page.mouse.move(head!.x + 24, head!.y + 12);
  await page.mouse.down();
  await page.mouse.move(head!.x + 264, head!.y + 12, { steps: 10 });
  await page.mouse.up();
  await expect
    .poll(async () => (await page.evaluate(() => window.__noema.studio().widgets)).find((w) => w.name === 'cost by soma')?.x)
    .toBeGreaterThan(placed!.x);

  await page.screenshot({ path: resolve(evidence, 'block7-studio.png') });

  await page.reload();
  await expect(page.locator('#studio-panel')).toHaveClass(/open/);
  await expect(page.locator('.widget-card', { hasText: 'cost by soma' })).toBeVisible();
  const persisted = await page.evaluate(() => window.__noema.studio());
  const widget = persisted.widgets.find((w) => w.name === 'cost by soma');
  expect(widget?.x).toBeGreaterThan(placed!.x);
  expect(persisted.roles.chat.global).toBe('claudeapikey/claude-opus-5-5');

  const sessions = page.locator('#studio-sessions .session-row');
  await sessions.nth(1).click();
  await expect
    .poll(async () => (await page.evaluate(() => window.__noema.studio())).session?.id)
    .toBe(await sessions.nth(1).getAttribute('data-session'));

  await page.locator('.widget-card', { hasText: 'cost by soma' }).locator('.widget-remove').click();
  await expect(page.locator('.widget-card', { hasText: 'cost by soma' })).toHaveCount(0);
});
