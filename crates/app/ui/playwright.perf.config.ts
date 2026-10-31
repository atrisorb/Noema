import { defineConfig } from '@playwright/test';

// Isolated port + database so perf runs never collide with the functional suites
// that share :4173, and the server is the prebuilt binary (no cargo lock contention).
export default defineConfig({
  testDir: './tests',
  testMatch: '*.perf.ts',
  timeout: 120_000,
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: 'http://127.0.0.1:4199',
    browserName: 'chromium',
    headless: false,
    viewport: { width: 1600, height: 900 },
    launchOptions: {
      args: [
        '--use-gl=angle', '--use-angle=d3d11', '--ignore-gpu-blocklist', '--enable-unsafe-webgpu',
        '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding',
        '--disable-features=CalculateNativeWinOcclusion',
      ],
    },
  },
  webServer: {
    command: '"../../../target/debug/noema-app.exe" serve --demo --port 4199 --ui dist --assets ../../../assets --db ../../../target/block5/perf.db',
    url: 'http://127.0.0.1:4199/api/health',
    reuseExistingServer: false,
    timeout: 60_000,
  },
});
