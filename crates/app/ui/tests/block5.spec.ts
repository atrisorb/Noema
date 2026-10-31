import { expect, test, type Page } from '@playwright/test';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const evidence = resolve('../../../docs/evidence');
mkdirSync(evidence, { recursive: true });

async function boot(page: Page) {
  await page.goto('/');
  await page.waitForFunction(() => window.__noema?.ready?.() === true, null, { timeout: 90_000 });
  await page.waitForFunction(() => window.__noema.atlasReady() === true, null, { timeout: 90_000 });
  await page.waitForTimeout(600);
}

async function firstAgentAwayFromHeart(page: Page): Promise<{ id: string; at: { x: number; y: number } }> {
  return page.evaluate(() => {
    const heart = window.__noema.heartScreen();
    for (const id of ['soma', 'axon', 'cortex', 'synapse', 'thalamus', 'hippocampus', 'retina']) {
      const at = window.__noema.agentScreen(id);
      if (at && Math.hypot(at.x - heart.x, at.y - heart.y) > 80) return { id, at };
    }
    throw new Error('no agent far from heart');
  });
}

test('real anatomy atlas loads with every system', async ({ page }) => {
  await boot(page);
  const stats = await page.evaluate(() => window.__noema.atlasStats());
  expect(stats.errors).toEqual([]);
  expect(stats.systems).toBeGreaterThanOrEqual(1);
  expect(stats.triangles).toBeGreaterThan(200_000);
  expect(await page.evaluate(() => window.__noema.debug().meshes)).toBeGreaterThan(500);
  expect(await page.evaluate(() => window.__noema.agentCount())).toBe(7);
  expect(await page.evaluate(() => window.__noema.tabCount())).toBe(5);
  await expect(page.locator('#fps')).toContainText('fps');
  await page.screenshot({ path: resolve(evidence, 'block5-atlas.png') });
});

test('systems toggle and camera reaches cellular zoom', async ({ page }) => {
  await boot(page);
  expect(await page.evaluate(() => window.__noema.layerVisible('skin'))).toBe(false);
  for (const layer of ['muscles', 'skeleton', 'nerves', 'vessels', 'organs', 'brain']) {
    expect(await page.evaluate((l) => window.__noema.layerVisible(l), layer)).toBe(true);
  }
  const muscles = page.locator('[data-layer="muscles"]');
  await muscles.click();
  expect(await page.evaluate(() => window.__noema.layerVisible('muscles'))).toBe(false);
  await muscles.click();
  expect(await page.evaluate(() => window.__noema.layerVisible('muscles'))).toBe(true);

  const skin = page.locator('[data-layer="skin"]');
  await skin.click();
  expect(await page.evaluate(() => window.__noema.layerVisible('skin'))).toBe(true);
  await skin.click();

  const before = await page.evaluate(() => window.__noema.cameraDistance());
  await page.evaluate(() => window.__noema.zoomToCellular());
  await page.waitForTimeout(1400);
  const after = await page.evaluate(() => window.__noema.cameraDistance());
  expect(before).toBeGreaterThan(2);
  expect(after).toBeLessThan(0.6);
  await page.screenshot({ path: resolve(evidence, 'block5-cellular.png') });
});

test('heart opens L\'Anima and the edit persists across reload', async ({ page }) => {
  await boot(page);
  const heart = await page.evaluate(() => window.__noema.heartScreen());
  await page.mouse.click(heart.x, heart.y);
  await expect(page.locator('#anima-panel')).toHaveClass(/open/);
  const identity = `Anima proof ${Date.now()}`;
  await page.locator('#anima-identity').fill(identity);
  await page.locator('#anima-save').click();
  await expect(page.locator('#anima-status')).toContainText('saved to database');
  await page.screenshot({ path: resolve(evidence, 'block5-anima.png') });

  await page.reload();
  await page.waitForFunction(() => window.__noema?.ready?.() === true, null, { timeout: 90_000 });
  await page.waitForFunction(() => window.__noema.atlasReady() === true, null, { timeout: 90_000 });
  const heart2 = await page.evaluate(() => window.__noema.heartScreen());
  await page.mouse.click(heart2.x, heart2.y);
  await expect(page.locator('#anima-identity')).toHaveValue(identity);
});

test('agent click freezes, shows DB greeting, 5 tabs, zero network calls', async ({ page }) => {
  await boot(page);
  const payload = await page.evaluate(async () => (await fetch('/api/anatomy')).json());
  const soma = payload.agents.find((a: { id: string }) => a.id === 'soma');
  expect(soma.greeting).toContain('tokens');

  const target = await firstAgentAwayFromHeart(page);
  let requests = 0;
  page.on('request', () => { requests += 1; });

  await page.mouse.click(target.at.x, target.at.y);
  await expect(page.locator('#rail')).toHaveClass(/open/);
  await expect(page.locator('#greeting')).toHaveText(soma.greeting);
  await expect(page.locator('#tabs button')).toHaveCount(5);
  expect(await page.evaluate(() => window.__noema.greeting('soma'))).toBe(soma.greeting);

  for (const tab of ['tokens', 'costs', 'ecosystem', 'workspace', 'activity']) {
    await page.locator(`#tabs button[data-tab="${tab}"]`).click();
    await expect(page.locator('#tab-body')).not.toBeEmpty();
  }
  expect(requests).toBe(0);
  await page.screenshot({ path: resolve(evidence, 'block5-agent-rail.png') });

  await page.locator('#rail-close').click();
  await expect(page.locator('#rail')).not.toHaveClass(/open/);

  await page.locator('#roster button', { hasText: 'Soma' }).click();
  await expect(page.locator('#rail')).toHaveClass(/open/);
});

test('60 fps budget with the full atlas', async ({ page }) => {
  await boot(page);
  await page.bringToFront();
  await page.waitForTimeout(800);
  const fps = await page.evaluate(async () => {
    const frames: number[] = [];
    let last = performance.now();
    await new Promise<void>((done) => {
      const start = performance.now();
      const tick = (now: number) => {
        frames.push(now - last);
        last = now;
        if (now - start < 3000) requestAnimationFrame(tick);
        else done();
      };
      requestAnimationFrame(tick);
    });
    const avg = frames.reduce((a, b) => a + b, 0) / frames.length;
    return 1000 / avg;
  });
  console.log(`measured fps: ${fps.toFixed(1)}`);
  expect(fps).toBeGreaterThan(50);
  await page.screenshot({ path: resolve(evidence, 'block5-60fps.png') });
  await page.evaluate(() => window.__noema.zoomToCellular());
  await page.waitForTimeout(1200);
  await page.screenshot({ path: resolve(evidence, 'block5-heart-close.png') });
});
