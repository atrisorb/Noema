#!/usr/bin/env node
// Generate platform icons from the SVG source. Output is git-ignored.
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const source = fileURLToPath(new URL('../../assets/brand/noema-app-icon.svg', import.meta.url));
const output = fileURLToPath(new URL('../../src-tauri/icons', import.meta.url));
const result = spawnSync('cargo', ['tauri', 'icon', source, '-o', output], { cwd: root, stdio: 'inherit' });
process.exit(result.status ?? 1);
