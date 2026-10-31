#!/usr/bin/env node
// WCAG 2.2 AA gate for opaque token pairs. Writes docs/evidence/design-tokens-contrast.json.
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const tokensPath = fileURLToPath(new URL('../../crates/app/ui/src/design/tokens.json', import.meta.url));
const outPath = fileURLToPath(new URL('../../docs/evidence/design-tokens-contrast.json', import.meta.url));
const tokens = JSON.parse(readFileSync(tokensPath, 'utf8'));

const hex = (value, label) => {
  if (!/^#[0-9a-fA-F]{6}$/.test(value)) throw new Error(`${label} is not an opaque hex: ${value}`);
  return value;
};
const luminance = (value) => {
  const n = hex(value, 'color').slice(1);
  const v = [0, 2, 4].map((i) => parseInt(n.slice(i, i + 2), 16) / 255)
    .map((c) => (c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4)));
  return 0.2126 * v[0] + 0.7152 * v[1] + 0.0722 * v[2];
};
const ratio = (a, b) => {
  const x = luminance(a);
  const y = luminance(b);
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
};

const pairs = [
  ['cavity', 'ink', 'bg', 4.5, 'body text'],
  ['cavity', 'ink-soft', 'bg', 4.5, 'body copy'],
  ['cavity', 'ink-muted', 'surface', 4.5, 'supporting copy'],
  ['cavity', 'ink-faint', 'surface', 4.5, 'metadata'],
  ['cavity', 'artery', 'bg', 4.5, 'brand text'],
  ['cavity', 'artery-ink', 'artery', 4.5, 'primary button'],
  ['cavity', 'nerve', 'bg', 3, 'focus ring and live'],
  ['cavity', 'vein', 'surface', 4.5, 'link and secondary data'],
  ['cavity', 'lymph', 'surface', 4.5, 'success'],
  ['cavity', 'danger', 'surface', 4.5, 'error'],
  ['cavity', 'danger-ink', 'danger', 4.5, 'danger button'],
  ['cavity', 'warning', 'surface', 4.5, 'warning'],
  ['bone', 'ink', 'bg', 4.5, 'body text'],
  ['bone', 'ink-soft', 'bg', 4.5, 'body copy'],
  ['bone', 'ink-muted', 'surface', 4.5, 'supporting copy'],
  ['bone', 'ink-faint', 'surface', 4.5, 'metadata'],
  ['bone', 'artery', 'bg', 4.5, 'brand text'],
  ['bone', 'artery-ink', 'artery', 4.5, 'primary button'],
  ['bone', 'nerve', 'bg', 3, 'focus ring and live'],
  ['bone', 'vein', 'surface', 4.5, 'link and secondary data'],
  ['bone', 'lymph', 'surface', 4.5, 'success'],
  ['bone', 'danger', 'surface', 4.5, 'error'],
  ['bone', 'danger-ink', 'danger', 4.5, 'danger button'],
  ['bone', 'warning', 'surface', 4.5, 'warning'],
];

const results = pairs.map(([theme, fgKey, bgKey, min, use]) => {
  const themeTokens = tokens.themes[theme];
  if (!themeTokens) throw new Error(`missing theme ${theme}`);
  const fg = themeTokens.color[fgKey];
  const bg = themeTokens.color[bgKey];
  if (!fg || !bg) throw new Error(`missing pair ${theme}.${fgKey}/${bgKey}`);
  const value = Number(ratio(fg, bg).toFixed(2));
  return { theme, fg: fgKey, bg: bgKey, fgValue: fg, bgValue: bg, use, min, ratio: value, pass: value >= min };
});

const failed = results.filter((r) => !r.pass);
const report = {
  schema: 'noema.contrast-report/1',
  generatedAt: new Date().toISOString(),
  standard: 'WCAG 2.2 AA',
  tokens: { name: tokens.name, designSystem: tokens.designSystem },
  summary: { total: results.length, passed: results.length - failed.length, failed: failed.length },
  pairs: results,
};
writeFileSync(outPath, `${JSON.stringify(report, null, 2)}\n`);

for (const r of results) {
  console.log(`${r.pass ? 'pass' : 'FAIL'} ${r.ratio.toFixed(2)} ${r.theme} ${r.fg}/${r.bg} (min ${r.min}, ${r.use})`);
}
console.log(`${report.summary.passed}/${report.summary.total} pass -> ${outPath}`);
if (failed.length) process.exit(1);
