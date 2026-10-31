#!/usr/bin/env node
// Usage: node visual-qa.mjs <url> <out.png> [--width 1440] [--height 900] [--wait 800]
// Uses the Playwright CLI that already ships with the repo (npx). No extra dependency.

import { spawnSync } from 'node:child_process';
import { existsSync, statSync } from 'node:fs';

const args = process.argv.slice(2);
const url = args[0];
const out = args[1];
const flag = (name, fallback) => {
  const index = args.indexOf('--' + name);
  return index >= 0 && args[index + 1] ? args[index + 1] : fallback;
};

if (!url || !out) {
  console.error('usage: visual-qa.mjs <url> <out.png>');
  process.exit(2);
}
if (!/^https?:\/\//i.test(url)) {
  console.error('url must be http(s)');
  process.exit(2);
}

const width = flag('width', '1440');
const height = flag('height', '900');
const wait = flag('wait', '800');
const npx = process.platform === 'win32' ? 'npx.cmd' : 'npx';

const run = spawnSync(
  npx,
  [
    '--yes',
    'playwright',
    'screenshot',
    '--viewport-size=' + width + ',' + height,
    '--wait-for-timeout=' + wait,
    url,
    out,
  ],
  { stdio: 'inherit', shell: false },
);

if (run.status !== 0 || !existsSync(out)) {
  console.error(JSON.stringify({ ok: false, status: run.status }));
  process.exit(1);
}
console.log(JSON.stringify({ ok: true, out, bytes: statSync(out).size }));
