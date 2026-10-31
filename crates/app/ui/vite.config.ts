import { defineConfig } from 'vite';

export default defineConfig({
  base: './',
  server: { port: 5173, proxy: { '/api': 'http://127.0.0.1:4173' } },
  build: { outDir: 'dist', emptyOutDir: true, target: 'es2022', chunkSizeWarningLimit: 1000 },
});
