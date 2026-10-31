#!/usr/bin/env node
// Update text references for the raster migration map and write the audit.
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { basename } from 'node:path';

const map = JSON.parse(readFileSync('target/block10/raster-migration.json', 'utf8'));
const files = execSync('git ls-files', { encoding: 'utf8' }).split('\n').filter(Boolean)
  .filter((f) => /\.(md|json|html|css|ts|tsx|js|mjs|sh|toml|rs)$/.test(f))
  .filter((f) => !f.startsWith('crates/app/ui/tests/') && !f.startsWith('tools/perf/') && !f.endsWith('package-lock.json'));
let changed = 0;
for (const file of files) {
  let text = readFileSync(file, 'utf8');
  const before = text;
  for (const { from, to } of map) {
    text = text.split(from).join(to);
    text = text.split(basename(from)).join(basename(to));
  }
  if (text !== before) { writeFileSync(file, text); changed += 1; }
}
// Packaging uses a generated PNG icon, never an evidence screenshot.
let pkg = readFileSync('packaging/build.sh', 'utf8');
pkg = pkg.split('docs/evidence/block8-render.webp').join('src-tauri/icons/128x128.png');
pkg = pkg.split('docs/evidence/block8-render.png').join('src-tauri/icons/128x128.png');
writeFileSync('packaging/build.sh', pkg);
console.log(`references updated in ${changed} files`);
