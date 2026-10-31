#!/usr/bin/env node
// Generate platform icons from the SVG source. Output is git-ignored.
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const source = fileURLToPath(new URL('../../assets/brand/noema-app-icon.svg', import.meta.url));
const output = fileURLToPath(new URL('../../src-tauri/icons', import.meta.url));

// Check if `cargo tauri` is installed; fallback to `npx @tauri-apps/cli`
const hasCargoTauri = spawnSync('cargo', ['tauri', '--version'], { stdio: 'ignore' }).status === 0;

let result;
if (hasCargoTauri) {
  result = spawnSync('cargo', ['tauri', 'icon', source, '-o', output], { cwd: root, stdio: 'inherit' });
} else {
  const isWin = process.platform === 'win32';
  const npxCmd = isWin ? 'npx.cmd' : 'npx';
  result = spawnSync(npxCmd, ['@tauri-apps/cli@2.12.5', 'icon', source, '-o', output], {
    cwd: root,
    stdio: 'inherit',
    shell: isWin
  });
}

process.exit(result.status ?? 1);

