// Shared frame sampler: injected into the page by Playwright (browser baseline)
// and by the WebView2 CDP harness, so both paths report identical metrics.
globalThis.__noemaPerf = {
  info() {
    const canvas = document.querySelector('canvas#scene') || document.querySelector('canvas');
    const gl = canvas && (canvas.getContext('webgl2') || canvas.getContext('webgl'));
    let renderer = 'no-webgl';
    let vendor = 'no-webgl';
    if (gl) {
      const ext = gl.getExtension('WEBGL_debug_renderer_info');
      if (ext) {
        renderer = gl.getParameter(ext.UNMASKED_RENDERER_WEBGL);
        vendor = gl.getParameter(ext.UNMASKED_VENDOR_WEBGL);
      }
    }
    const app = globalThis.__noema;
    return {
      renderer,
      vendor,
      dpr: devicePixelRatio,
      canvas: canvas ? { width: canvas.width, height: canvas.height, cssWidth: canvas.clientWidth, cssHeight: canvas.clientHeight } : null,
      appFps: app && app.fps ? app.fps() : null,
      renderInfo: app && app.renderInfo ? app.renderInfo() : null,
      ua: navigator.userAgent,
    };
  },
  stats(samples, windowMs) {
    const sorted = [...samples].sort((a, b) => a - b);
    const at = (p) => (sorted.length ? sorted[Math.min(sorted.length - 1, Math.floor(p * sorted.length))] : 0);
    const mean = samples.length ? samples.reduce((a, b) => a + b, 0) / samples.length : 0;
    const hitches = samples.filter((v) => v > 25).length;
    return {
      frames: samples.length,
      windowMs,
      fpsMean: mean ? 1000 / mean : 0,
      fpsP50: at(0.5) ? 1000 / at(0.5) : 0,
      mean,
      p50: at(0.5),
      p95: at(0.95),
      p99: at(0.99),
      max: sorted.length ? sorted[sorted.length - 1] : 0,
      hitches,
      hitchesPerSec: hitches / (windowMs / 1000),
      droppedFrames: samples.filter((v) => v > 1000 / 55).length,
    };
  },
  sample(windowMs = 5000) {
    return new Promise((done) => {
      const samples = [];
      let last = performance.now();
      const start = last;
      const tick = (now) => {
        samples.push(now - last);
        last = now;
        if (now - start < windowMs) requestAnimationFrame(tick);
        else done(globalThis.__noemaPerf.stats(samples, now - start));
      };
      requestAnimationFrame(tick);
    });
  },
};
