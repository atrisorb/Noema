#!/usr/bin/env node
// Render the eight mascots to one WebP sheet. Source stays SVG.
import { createRequire } from 'node:module';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const require = createRequire(new URL('../../crates/app/ui/package.json', import.meta.url));
const { chromium } = require('playwright');
const root = fileURLToPath(new URL('../../', import.meta.url));
const names = ['cortex', 'axon', 'thalamus', 'synapse', 'soma', 'hippocampus', 'retina', 'aesthesis'];
const svg = Object.fromEntries(names.map((n) => [n, readFileSync(`${root}assets/mascots/${n}.svg`, 'utf8')]));
const item = (name, size) => `<figure><div class="art" style="width:${size}px;height:${size}px">${svg[name].replace('<svg ', '<svg style="width:100%;height:100%" ')}</div><figcaption>${name} ${size}</figcaption></figure>`;
const html = `<!doctype html><html><head><meta charset="utf-8"><style>
  * { box-sizing: border-box; } body { margin: 0; font: 13px/1.4 system-ui, sans-serif; }
  .theme { padding: 24px 28px 30px; }
  .cavity { background: #05070B; color: #F4EFE6; }
  .bone { background: #F4EFE6; color: #0B0D12; }
  h2 { margin: 0 0 14px; font-size: 12px; letter-spacing: .14em; text-transform: uppercase; opacity: .6; }
  .grid { display: grid; grid-template-columns: repeat(8, 1fr); gap: 12px; }
  figure { margin: 0; display: flex; flex-direction: column; align-items: center; gap: 6px; }
  .art { display: flex; align-items: center; justify-content: center; }
  figcaption { font: 10px/1.2 ui-monospace, monospace; opacity: .55; }
  .small { margin-top: 18px; grid-template-columns: repeat(8, 1fr); }
</style></head><body>
  <section class="theme cavity"><h2>Cavity</h2><div class="grid">${names.map((n) => item(n, 96)).join('')}</div><div class="grid small">${names.map((n) => item(n, 24)).join('')}</div></section>
  <section class="theme bone"><h2>Bone</h2><div class="grid">${names.map((n) => item(n, 96)).join('')}</div><div class="grid small">${names.map((n) => item(n, 24)).join('')}</div></section>
</body></html>`;

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1280, height: 700 }, deviceScaleFactor: 1 });
await page.setContent(html, { waitUntil: 'load' });
const cdp = await page.context().newCDPSession(page);
const { data } = await cdp.send('Page.captureScreenshot', { format: 'webp', quality: 90, captureBeyondViewport: false });
mkdirSync(`${root}docs/evidence`, { recursive: true });
const out = `${root}docs/evidence/design-mascots.webp`;
writeFileSync(out, Buffer.from(data, 'base64'));
console.log(out);
await browser.close();
