#!/usr/bin/env node
// Render the brand suite to one WebP evidence sheet. No raster source enters the repo.
import { createRequire } from 'node:module';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const require = createRequire(new URL('../../crates/app/ui/package.json', import.meta.url));
const { chromium } = require('playwright');
const root = fileURLToPath(new URL('../../', import.meta.url));
const files = ['noema-mark.svg', 'noema-monogram.svg', 'noema-wordmark.svg', 'noema-lockup.svg', 'noema-lockup-stacked.svg', 'noema-app-icon.svg', 'favicon.svg'];
const svg = Object.fromEntries(files.map((f) => [f, readFileSync(`${root}assets/brand/${f}`, 'utf8')]));

const item = (name, size, label) => `<figure><div class="art" style="width:${size}px;height:${size}px">${svg[name].replace('<svg ', `<svg style="width:100%;height:100%"`)}</div><figcaption>${label}</figcaption></figure>`;
const sizes = [16, 24, 32, 48, 96].map((s) => item('noema-mark.svg', s, `mark ${s}`)).join('');
const html = `<!doctype html><html><head><meta charset="utf-8"><style>
  * { box-sizing: border-box; } body { margin: 0; font: 13px/1.4 system-ui, sans-serif; }
  .theme { padding: 24px 28px; display: grid; grid-template-columns: repeat(4, 1fr); gap: 18px 22px; }
  .cavity { background: #05070B; color: #F4EFE6; }
  .bone { background: #F4EFE6; color: #0B0D12; }
  h2 { grid-column: 1 / -1; margin: 0 0 4px; font-size: 12px; letter-spacing: .14em; text-transform: uppercase; opacity: .6; }
  figure { margin: 0; display: flex; flex-direction: column; gap: 6px; align-items: flex-start; }
  .art { display: flex; align-items: center; justify-content: center; }
  figcaption { font: 11px/1.2 ui-monospace, monospace; opacity: .55; }
  .sizes { display: flex; gap: 18px; align-items: end; grid-column: 1 / -1; }
  .sizes figure { align-items: center; }
</style></head><body>
  <section class="theme cavity"><h2>Cavity</h2>
    ${item('noema-mark.svg', 96, 'mark')}${item('noema-monogram.svg', 96, 'monogram')}${item('noema-wordmark.svg', 240, 'wordmark')}${item('noema-lockup.svg', 260, 'lockup')}
    ${item('noema-lockup-stacked.svg', 150, 'stacked')}${item('noema-app-icon.svg', 160, 'app icon')}${item('favicon.svg', 64, 'favicon')}
    <div class="sizes">${sizes}</div>
  </section>
  <section class="theme bone"><h2>Bone</h2>
    ${item('noema-mark.svg', 96, 'mark')}${item('noema-monogram.svg', 96, 'monogram')}${item('noema-wordmark.svg', 240, 'wordmark')}${item('noema-lockup.svg', 260, 'lockup')}
    ${item('noema-lockup-stacked.svg', 150, 'stacked')}${item('noema-app-icon.svg', 160, 'app icon')}${item('favicon.svg', 64, 'favicon')}
  </section>
</body></html>`;

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1280, height: 1240 }, deviceScaleFactor: 1 });
await page.setContent(html, { waitUntil: 'load' });
const cdp = await page.context().newCDPSession(page);
const { data } = await cdp.send('Page.captureScreenshot', { format: 'webp', quality: 90, captureBeyondViewport: false });
mkdirSync(`${root}docs/evidence`, { recursive: true });
const out = `${root}docs/evidence/design-logo-suite.webp`;
writeFileSync(out, Buffer.from(data, 'base64'));
console.log(out);
await browser.close();
