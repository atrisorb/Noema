// Host bridge: one bundle runs from the loopback server and inside the Tauri webview.
// The wrapper injects window.__NOEMA_API_BASE__ (or window.__NOEMA__.apiBase) before the module runs.

interface HostHooks { apiBase?: string; assetBase?: string }

const trim = (value: string) => value.replace(/\/+$/, '');

function hooks(): HostHooks {
  return (window as unknown as { __NOEMA__?: HostHooks }).__NOEMA__ ?? {};
}

export function isTauri(): boolean {
  return typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
}

export function apiBase(): string {
  const injected = (window as unknown as { __NOEMA_API_BASE__?: unknown }).__NOEMA_API_BASE__;
  const base = hooks().apiBase ?? (typeof injected === 'string' ? injected : '');
  return typeof base === 'string' ? trim(base) : '';
}

export function assetBase(): string {
  return trim(hooks().assetBase ?? apiBase());
}

export const apiUrl = (path: string) => apiBase() + path;

export function assetUrl(path: string): string {
  const suffix = path.startsWith('/') ? path : `/${path}`;
  return assetBase() + suffix;
}

// GET only: retries absorb the wrapper/server start race. Never retry a write.
export async function getJson<T>(path: string, attempts = 1): Promise<T> {
  let last: unknown = null;
  for (let i = 0; i < attempts; i += 1) {
    try {
      const res = await fetch(apiUrl(path));
      if (!res.ok) throw new Error(await res.text());
      return (await res.json()) as T;
    } catch (e) {
      last = e;
      if (i + 1 < attempts) await new Promise((resolve) => setTimeout(resolve, 300));
    }
  }
  throw last;
}
