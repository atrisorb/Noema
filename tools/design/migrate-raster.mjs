#!/usr/bin/env node
// One-shot raster migration: tracked PNG -> WebP, then delete the PNG.
import { createRequire } from 'node:module';
import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, unlinkSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';

const require = createRequire(new URL('../../crates/app/ui/package.json', import.meta.url));
const { chromium } = require('playwright');
const root = process.cwd();
const files = execSync('git ls-files', { cwd: root, encoding: 'utf8' }).split('\n').filter((f) => /\.png$/i.test(f));
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const converted = [];
for (const file of files) {
  const abs = resolve(root, file);
  const b64 = readFileSync(abs).toString('base64');
  const webp = await page.evaluate(async (data) => {
    const img = new Image();
    img.src = 'data:image/png;base64,' + data;
    await img.decode();
    const canvas = document.createElement('canvas');
    canvas.width = img.naturalWidth; canvas.height = img.naturalHeight;
    canvas.getContext('2d').drawImage(img, 0, 0);
    return canvas.toDataURL('image/webp', 0.82).split(',')[1];
  }, b64);
  const out = abs.replace(/\.png$/i, '.webp');
  mkdirSync(dirname(out), { recursive: true });
  writeFileSync(out, Buffer.from(webp, 'base64'));
  unlinkSync(abs);
  converted.push({ from: file, to: file.replace(/\.png$/i, '.webp'), bytes: Buffer.from(webp, 'base64').length });
}
await browser.close();
writeFileSync('target/block10/raster-migration.json', JSON.stringify(converted, null, 2) + '\n');
console.log(`converted ${converted.length} PNG -> WebP`);
console.log(`bytes ${converted.reduce((n, f) => n + f.bytes, 0)}`);
