#!/usr/bin/env node
// Canonical brand SVG -> UI public favicon. --check fails on drift.
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const source = fileURLToPath(new URL('../../assets/brand/favicon.svg', import.meta.url));
const target = fileURLToPath(new URL('../../crates/app/ui/public/favicon.svg', import.meta.url));
const svg = readFileSync(source, 'utf8');
const check = process.argv.includes('--check');

if (check) {
  let current = '';
  try { current = readFileSync(target, 'utf8'); } catch { /* missing is a failure */ }
  if (current !== svg) {
    console.error('sync-brand: crates/app/ui/public/favicon.svg is stale; run node tools/design/sync-brand.mjs');
    process.exit(1);
  }
  console.log('favicon ok');
  process.exit(0);
}
mkdirSync(dirname(target), { recursive: true });
writeFileSync(target, svg);
console.log(`wrote ${target}`);
