# NOEMA UI — wrapper contract

One bundle (`crates/app/ui/dist`) serves browser mode and the Tauri v2 window.
No build variant, no wrapper-specific code path.

## Hosts

| Host | Document origin | Base used |
|---|---|---|
| `noema-app serve` (default port 4173) | `http://127.0.0.1:4173` | `''` (same origin) |
| Tauri window (`WebviewUrl::External`) | same loopback URL | `''` (same origin) |
| Tauri window over a bundled UI (future) | `http://tauri.localhost` | injected base |

## Injected globals (optional, read per request)

| Global | Type | Meaning |
|---|---|---|
| `window.__NOEMA_API_BASE__` | string | origin + optional prefix for `/api/*` |
| `window.__NOEMA__.apiBase` | string | same; wins over the flat global |
| `window.__NOEMA__.assetBase` | string | optional, defaults to `apiBase` |

Empty or absent base means "relative to the document origin". Trailing slashes
are trimmed. The current wrapper opens the loopback URL, so it injects nothing.

## UI guarantees

- No absolute `file://` paths; Vite `base` is `./`.
- Anatomy GLB, DRACO decoder and every `/api` call resolve through `platform.ts`.
- Hash route `#studio` opens the Studio on boot and on `hashchange`; closing clears it.
- Body scene renders before the first API call; never a blank landing page.
- GET retries (~3.5 s) only when `window.__TAURI_INTERNALS__` exists, to absorb the
  server start race. Writes are never retried.
- `window.__noema.platform()` reports `{tauri, apiBase, assetBase, hash}` for QA.

## Wrapper side

- Commands published by the shell: `health`, `bridge_info`, `session_resume`
  (`src-tauri/src/main.rs`). The UI consumes none: the window is same-origin with
  `noema-app`, so HTTP already carries everything. Do not add commands for the UI.
- Serve the UI dist at `/` (UI files first) and shared assets at `/assets/*`,
  exactly like `noema-app`.
- If the window ever loads a bundled UI origin instead of the loopback URL, inject
  `__NOEMA_API_BASE__` before the bundle runs, or add CORS plus `OPTIONS` preflight.
  Same-origin needs neither.
