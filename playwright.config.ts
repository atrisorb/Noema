import { defineConfig } from '@playwright/test';

export default defineConfig({
  use: {
    browserName: 'chromium',
    headless: false,
    viewport: { width: 1920, height: 1080 },
    launchOptions: {
      args: [
        '--use-gl=angle',
        '--use-angle=d3d11',
        '--enable-unsafe-webgpu',
        '--ignore-gpu-blocklist',
      ],
    },
  },
});
