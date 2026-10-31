import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  timeout: 90_000,
  expect: { timeout: 10_000 },
  workers: 1,
  reporter: [['list']],
  use: {
    baseURL: 'http://127.0.0.1:4173',
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
    command: 'cargo run --quiet --manifest-path ../../../Cargo.toml -p noema-app -- serve --demo --port 4173 --ui dist --assets ../../../assets --db ../../../target/block5/e2e.db',
    url: 'http://127.0.0.1:4173/api/health',
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
