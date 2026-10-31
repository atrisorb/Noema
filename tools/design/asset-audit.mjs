#!/usr/bin/env node
// Asset law gate. Fails on committed rasters and on product references.
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { execSync } from 'node:child_process';

const forbidden = /\.(png|jpe?g|gif|bmp|tif|tiff|ico)$/i;
const tracked = execSync('git ls-files', { encoding: 'utf8' }).split('\n').filter(Boolean);
const files = tracked.filter((f) => forbidden.test(f) && existsSync(f));
const product = tracked.filter((f) => /^(crates\/app\/ui\/(index\.html|src\/)|assets\/(?!anatomy\/LICENSES)|src-tauri\/tauri\.conf\.json|packaging\/build\.sh)/.test(f));
const refs = [];
for (const file of product) {
  if (!existsSync(file)) continue;
  const text = readFileSync(file, 'utf8');
  if (/\.(png|jpe?g|gif|bmp|tif|tiff|ico)/i.test(text)) refs.push(file);
}
const allowGenerated = refs.filter((f) => f === 'src-tauri/tauri.conf.json' || f === 'packaging/build.sh');
const violations = [...files, ...refs.filter((f) => !allowGenerated.includes(f))];
const report = { schema: 'noema.asset-audit/1', at: new Date().toISOString(), trackedFiles: tracked.length, forbiddenFiles: files, productReferenceViolations: violations, generatedIconReferences: allowGenerated, ok: violations.length === 0 };
writeFileSync('docs/evidence/design-asset-audit.json', JSON.stringify(report, null, 2) + '\n');
console.log(`asset audit: ${files.length} forbidden files, ${violations.length} violations`);
if (violations.length) process.exit(1);
