# NOEMA — ARCHITECTURE

Companion to `docs/MASTER_PLAN.md`. This file holds the concrete contracts: crate boundaries, DDL, RPC, config, manifests, rendering and pricing. When code and this file disagree, the code wins only after this file is updated in the same commit.

## 1. Crate boundaries

| Crate | Type | Public surface | May depend on |
|---|---|---|---|
| `noema-core` | lib + `noemad` bin (Block 2) | `ids`, `session`, `store`, `event`, `rpc`, `daemon`, `vector` | — |
| `noema-inference` | lib | `Standard`, `Request`, `StreamEvent`, `Usage`, `RateCard`, `Adapter`, `embed`, `cluster`, `cost` | core |
| `noema-ecosystem` | lib | `Manifest`, `Kind`, `Transport`, `McpSupervisor`, `Marketplace`, `Github` | core |
| `noema-cli` | bin `noema` | CLI surface, RPC client | core, inference, ecosystem |
| `noema-app` | bin | loopback HTTP bridge + staged UI | core, inference, hippocampus |
| `noema-desktop` | bin (`src-tauri`) | Tauri v2 window, `health`, `bridge_info`, `session_resume` | — (HTTP only) |

Rule: siblings never depend on each other. Cross-cutting value types live in `core`.

## 2. Data model (SQLite)

All tables use TEXT uuid PKs, integer unix-ms timestamps, `PRAGMA journal_mode=WAL`, `foreign_keys=ON`.

```sql
CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);

CREATE TABLE projects (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, root_path TEXT NOT NULL,
  created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);

CREATE TABLE sessions (
  id TEXT PRIMARY KEY, project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
  agent_id TEXT NOT NULL, model_id TEXT NOT NULL, cwd TEXT NOT NULL,
  repo TEXT, branch TEXT, widget_layout TEXT, created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL);

CREATE TABLE messages (
  id TEXT PRIMARY KEY, session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
  role TEXT NOT NULL, content TEXT NOT NULL, reasoning TEXT,
  input_tokens INTEGER NOT NULL DEFAULT 0, output_tokens INTEGER NOT NULL DEFAULT 0,
  reasoning_tokens INTEGER NOT NULL DEFAULT 0, cache_read INTEGER NOT NULL DEFAULT 0,
  cache_write INTEGER NOT NULL DEFAULT 0, cost_usd REAL NOT NULL DEFAULT 0,
  model_id TEXT, created_at INTEGER NOT NULL);
CREATE INDEX idx_messages_session ON messages(session_id, created_at);

CREATE TABLE providers (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, standard TEXT NOT NULL,
  base_url TEXT NOT NULL, key_ref TEXT, headers TEXT, created_at INTEGER NOT NULL);

CREATE TABLE models (
  id TEXT PRIMARY KEY, provider_id TEXT REFERENCES providers(id) ON DELETE CASCADE,
  name TEXT NOT NULL, capability TEXT NOT NULL,
  context_window INTEGER, params TEXT, created_at INTEGER NOT NULL);

CREATE TABLE rate_cards (
  id TEXT PRIMARY KEY, model_id TEXT NOT NULL REFERENCES models(id) ON DELETE CASCADE,
  currency TEXT NOT NULL DEFAULT 'USD',
  input REAL NOT NULL DEFAULT 0, output REAL NOT NULL DEFAULT 0,
  reasoning REAL NOT NULL DEFAULT 0, cache_read REAL NOT NULL DEFAULT 0,
  cache_write REAL NOT NULL DEFAULT 0, per_call REAL NOT NULL DEFAULT 0,
  local_hardware_usd REAL NOT NULL DEFAULT 0, local_lifespan_hours REAL NOT NULL DEFAULT 0,
  local_watts REAL NOT NULL DEFAULT 0, kwh_price REAL NOT NULL DEFAULT 0);

CREATE TABLE rate_windows (
  id TEXT PRIMARY KEY, rate_card_id TEXT NOT NULL REFERENCES rate_cards(id) ON DELETE CASCADE,
  timezone TEXT NOT NULL, weekday_mask INTEGER NOT NULL,  -- bit 0 = Monday
  start_minute INTEGER NOT NULL, end_minute INTEGER NOT NULL, multiplier REAL NOT NULL);

CREATE TABLE usage_events (
  id TEXT PRIMARY KEY, session_id TEXT, agent_id TEXT, model_id TEXT NOT NULL,
  kind TEXT NOT NULL,  -- input|output|reasoning|cache_read|cache_write
  tokens INTEGER NOT NULL, cost_usd REAL NOT NULL,
  peak INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL);
CREATE INDEX idx_usage_time ON usage_events(created_at);

CREATE TABLE thoughts (
  id TEXT PRIMARY KEY, session_id TEXT, agent_id TEXT, title TEXT NOT NULL,
  body TEXT NOT NULL, concepts TEXT, cluster_id TEXT,
  embedding BLOB, importance REAL NOT NULL DEFAULT 0.5,
  pos_x REAL, pos_y REAL, pos_z REAL, created_at INTEGER NOT NULL);

CREATE TABLE thought_edges (
  id TEXT PRIMARY KEY, from_id TEXT NOT NULL REFERENCES thoughts(id) ON DELETE CASCADE,
  to_id TEXT NOT NULL REFERENCES thoughts(id) ON DELETE CASCADE,
  weight REAL NOT NULL, relation TEXT, UNIQUE(from_id, to_id));

CREATE TABLE dreams (
  id TEXT PRIMARY KEY, session_id TEXT, started_at INTEGER NOT NULL,
  finished_at INTEGER, summary TEXT, insights TEXT,
  input_thoughts TEXT, output_thoughts TEXT);

CREATE TABLE agents (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, role TEXT, model_id TEXT,
  workdirs TEXT, skills TEXT, plugins TEXT, mcp_servers TEXT,
  greeting TEXT NOT NULL, created_at INTEGER NOT NULL);

CREATE TABLE widgets (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, chart TEXT NOT NULL,
  spec TEXT NOT NULL, x INTEGER NOT NULL, y INTEGER NOT NULL,
  w INTEGER NOT NULL, h INTEGER NOT NULL, pinned INTEGER NOT NULL DEFAULT 0);

CREATE TABLE mcp_servers (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, transport TEXT NOT NULL,
  command TEXT, args TEXT, env TEXT, cwd TEXT, url TEXT, headers TEXT,
  auth TEXT, timeout_ms INTEGER, retry TEXT, enabled INTEGER NOT NULL DEFAULT 1);

CREATE TABLE skills (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL,
  path TEXT NOT NULL, manifest TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1);

CREATE TABLE plugins (
  id TEXT PRIMARY KEY, name TEXT NOT NULL, version TEXT NOT NULL,
  path TEXT NOT NULL, manifest TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1);

CREATE TABLE repos (
  id TEXT PRIMARY KEY, full_name TEXT NOT NULL, branch TEXT,
  last_diff_stat TEXT, last_synced_at INTEGER, pinned INTEGER NOT NULL DEFAULT 0);

CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL, scope TEXT NOT NULL);

-- Optional, when sqlite-vec loads:
CREATE VIRTUAL TABLE thoughts_vec USING vec0(thought_id TEXT PRIMARY KEY, embedding FLOAT[1536]);
```

Vector fallback: brute-force cosine in Rust over `thoughts.embedding` (dimension read from the model profile). No ANN dependency required for v1. `sqlite-vec` is loaded only when `NOEMA_SQLITE_VEC` points at the shared library and `vec_version()` answers; otherwise the store reports `BruteForce`. `thoughts_vec` is created lazily once embeddings exist (Block 3).

Migrations are append-only: `crates/core/migrations/0001_init.sql` is embedded at compile time, applied inside a transaction and recorded in `schema_migrations`.

## 3. IPC — JSON-RPC 2.0

Frame: one JSON object per line (NDJSON), UTF-8, max 8 MiB. The cap is enforced while reading: an oversize frame gets `-32700` and the daemon closes the connection immediately.

```json
{"jsonrpc":"2.0","id":7,"method":"session.resume","params":{"_auth":"<token>"}}
{"jsonrpc":"2.0","id":7,"result":{"session":{...},"diff_stat":"3 files changed, 41 insertions(+), 7 deletions(-)","resume":{"last_session":"...","widget_layout":"[...]","role_models":{"chat":"..."},...}}}
{"jsonrpc":"2.0","method":"event.usage","params":{"agent_id":"...","tokens":1234,"cost_usd":0.0021}}
```

- Port discovery: the platform data root (`%APPDATA%\noema` on Windows, `~/Library/Application Support/noema` on macOS, `$XDG_DATA_HOME/noema` or `~/.local/share/noema` on Linux) contains `daemon.json` → `{ "port": 51234, "token": "...", "pid": 123, "version": "0.1.0" }`, mode 0600 on unix; on Windows `icacls /inheritance:r /grant:r <user>:F` (best-effort, warning logged on failure). `NOEMA_DATA_DIR` overrides the root everywhere.
- Auth: constant-time compare; wrong/missing token → `-32001 auth`.
- On connect the daemon writes one `event.health` notification (`pid`, `uptime_s`) before the first reply, so a client can prove the stream without waiting for the 15 s heartbeat. Clients must skip notification frames when matching replies.
- `session.open` validation: `session.id` must be a uuid, `agent`/`model` 1-128 chars of `[A-Za-z0-9._:/@-]`, `cwd` absolute, `project` must exist. Violations → `-32602`.
- Streaming: `chat.send` replies with `chat.delta` notifications carrying `{session_id, seq, text}` and terminates with `chat.done` including `usage`.
- Cancellation: `chat.cancel {session_id}`; adapter must drop the HTTP stream.
- Backpressure: notifications are buffered per connection (cap 1024); overflow drops `event.agent.move` frames before `event.usage`.

### Methods (v1)

```
session.resume | open | close | list
chat.send | cancel
agent.list | inspect
models.list | upsert | delete      providers.list | upsert | delete
rates.list | upsert                usage.summary | series
mcp.list | upsert | delete | start | stop | logs
skill.list | install | enable      plugin.list | install | enable
repo.auth | search | branches | checkout | diff
brain.thoughts | edges | clusters | drag | telemetry | dream.start | dream.status
widget.list | upsert | delete | layout
config.get | set | schema
system.profile | override | reset
```

`system.profile {refresh?}` returns `{profile, tuning, overrides, cached, generated_at_ms}`. `system.override {overrides}` deep-merges a JSON patch into `settings('hardware.overrides')` after validating the effective tuning; `system.reset` clears the patch and the cached profile, then re-probes. See §11.

### Inference contract (Block 3)

- `Adapter::encode` returns `HttpCall { method, url, headers, body }`. The transport is a thin reqwest stream added with `chat.send`; no provider SDK.
- Streaming normalises to `StreamEvent::{Delta, Reasoning, Logprob, Usage, Done}`: one SSE payload or NDJSON line in, events out. `ActivationWindow` folds a batch into `{logprob_entropy, thinking_density, cosine_drift}`; `POST /api/brain/telemetry` exposes it to the graph without a new schema.
- Ten standards: OpenAI Chat, OpenAI Responses, Anthropic, Gemini, DeepSeek, Ollama, llama.cpp, vLLM, LM Studio, custom HTTP.
- Usage normalisation: input excludes cache reads and output excludes reasoning tokens where the provider folds them together (OpenAI, Gemini). Anthropic reports them separately.
- Pricing: `RateCard` per million tokens + per-call fee + optional IANA rate windows (first match wins) + local hardware amortisation; the caller supplies hours for local share.
- Clustering: `Extraction` (concepts/relations/causal_links) + embeddings -> `ThoughtGraph`. Edge weight = relation weight x (0.5 + 0.5 x cosine); clusters are single-link unions at weight >= 0.6.
- Persistence: `ThoughtGraph` is the `brain.*` payload; `crates/core` owns the INSERTs. Drag positions are stored in the existing `thoughts.pos_x/y/z` columns and win over the projection on read. No schema change needed.

## 4. Config layering

1. Built-in defaults (compiled).
2. `config.toml` in the platform data root (`%APPDATA%\noema`, `~/Library/Application Support/noema`, `$XDG_DATA_HOME/noema` or `~/.local/share/noema`).
3. `<project>\.noema\project.toml`.
4. `NOEMA_*` environment variables.
5. Request/CLI flags.

Schema registry drives: Studio form, CLI completion, validation errors, docs. Secret-valued keys reference the keychain (`key_ref`), never store values; the backend is DPAPI on Windows, Keychain on macOS and Secret Service on Linux (`crates/core/src/platform.rs`).

Hardware-derived defaults are not config-file keys: they are the `settings` rows `hardware.profile` and `hardware.overrides` (scope `hardware`), exposed by `system.*` and §11. Overrides win over the derivation and survive profile refresh.

## 5. Ecosystem manifests (`noema.json`) and MCP supervisor

`noema-ecosystem` validates three kinds: `mcp`, `skill`, `plugin`. Unknown fields are rejected; errors name the field (`mcp.command: stdio transport requires a command`); `json_schema(kind)` publishes draft 2020-12 for the Studio and the CLI.

MCP example:

```json
{
  "name": "memory-mcp",
  "version": "1.0.0",
  "kind": "mcp",
  "description": "Semantic introspection over NOEMA thoughts, edges and dreams.",
  "entry": "server.mjs",
  "transport": "stdio",
  "permissions": ["db:read", "fs:read"],
  "env": ["NOEMA_DB"],
  "mcp": { "command": "node", "args": ["server.mjs"], "env": {} }
}
```

Plugin example:

```json
{
  "name": "github-plugin",
  "version": "1.0.0",
  "kind": "plugin",
  "description": "Proprietary GitHub plugin: user/repo search, branches, checkout, live diff, session pin.",
  "entry": "noema",
  "permissions": ["git", "network:github.com", "keychain"],
  "env": ["NOEMA_GITHUB_TOKEN"],
  "plugin": { "homepage": "https://github.com" }
}
```

Transports: `stdio` (command/args/env/cwd), `sse` (url + endpoint event), `websocket` (url). Auth: Bearer, OAuth2 access token (PKCE verifier/challenge helpers), mTLS identity for HTTP, custom headers. Secrets use `Secret`; its `Debug` is `Secret(***)`.

Supervisor: one entry per name, JSON-RPC ids, per-server log ring (512 lines), `timeout` and `retry { max_attempts, backoff }`. Retries cover transport failures only, never RPC errors. `initialize` performs the MCP handshake, followed by `tools/list` / `tools/call`.

Skills are Claude Code compatible: `SKILL.md` frontmatter (`name`, `description`, `version`, `license`, `allowed-tools`) plus optional `noema.json`.

Marketplace bundles `memory-mcp` (Node + `node:sqlite`, read-only), `system-visual-qa` (Playwright CLI) and `github-plugin`. Installs validate before writing, copy the package tree and record `installed.json`.

GitHub plugin: device flow or PAT (`NOEMA_GITHUB_TOKEN`, then `GITHUB_TOKEN`), `user/repo` search, branch list, checkout/diff through the system `git` binary (argv only), and `.noema/pin.json` pins repo + branch + session.

## 6. Rendering contracts

**Block 5 implementation (three.js r187, WebGL2, `crates/app/ui`).** The UI is a Vite bundle served by `noema-app serve`; the Tauri v2 shell in `src-tauri` opens that same loopback URL (Block 9). No CDN, no external request at runtime.

- **Atlas:** `assets/anatomy/atlas/body_full.glb` — real Z-Anatomy écorché (via `dohunjeon51-design/web-anatomy-3d`, **CC BY-SA 4.0**), 826 named structures (bones, muscles, cartilages, teeth), 291k triangles, 1.72 m frame. Attribution and share-alike are recorded in `assets/anatomy/LICENSES.md`.
- **Layers:** `skin` (reserved), `muscles`, `skeleton`, `nerves`, `vessels`, `organs`, `brain`, plus `agents`. The atlas is loaded once, normalised (feet at −0.86 m), then reparented in two passes (collect, then attach) into layer groups; per-mesh materials are cloned and tinted by structure name (bone ivory, muscle red, nerve yellow, vessel red/blue, organ-specific colours).
- **Camera:** OrbitControls; auto-fit frames the full body with a 1.35 margin from the vertical FOV; distance 0.03-7 units; 1.1 s eased fly-to. `zoomToCellular` hides muscles, fades the skeleton to 40% and centres the cardiac anchor, revealing the procedural heart and circulation.
- **Brain graph (Block 6):** `crates/hippocampus` projects `thoughts.embedding` (UMAP-style cosine kNN + SGD, five functional anchors), `GET /api/brain` returns nodes/edges/pulse, and `brain-worker.ts` integrates springs + repulsion with a hard 1.0 tether clamp. Rendering is instanced SDF node glyphs (analytic disc, `fwidth`) + vector edges + SVG labels; no raster text. `POST /api/dream` persists Dream Mode runs. Full contract in `docs/BRAIN.md`.
- **Heart ("L'Anima"):** a pulsing procedural heart inside the thorax (atlas has no cardiac mesh); click or the 30 px heart zone opens the identity/system-prompt editor. Save = `POST /api/anima` → `settings` table (`scope='anima'`); reload proves persistence.
- **Circulation:** six Catmull-Rom flows (aorta, carotids, descending aorta, femorals, pulmonary/venous return) drive additive particle systems (220 + 220 + 120 + 120 arterial, 160 + 140 venous), depth-test off for the x-ray flow read.
- **Agent path:** seven deterministic splines inside the body; luminous additive sprites; the frozen agent enlarges. GPU picking uses ID proxies in a WebGLRenderTarget with bit-encoded channels (1/2/4) so colour-space transfer cannot corrupt ids.
- **Click agent:** GPU pick → freeze → right rail (380 px, 380 ms); greeting already in the boot payload (from `agents.greeting` + `usage_events` + `sessions`), five tabs. Zero LLM and zero network call at click — asserted by Playwright.
- **Deterministic greeting:** pure Rust `compose_greeting(&AgentFacts)`, unit-tested; the app crate has no HTTP client dependency.
- **Fallback:** the procedural body from Block 5 remains when the atlas is absent; it is hidden after a successful load.
- **Budget:** 60.2 fps at 1600×900 on ANGLE/D3D11 (GTX 1650), 0.3M triangles, pixel ratio 1.5, no shadows, no post-processing (the composer was removed after it produced an empty render).
- **Known limit:** the free Z-Anatomy processed files carry only metallic/roughness palette maps; albedo textures and a full internal-organ atlas are not available under a free licence in this environment. Organs are the next asset step.

## 7. Pricing math

```
token_cost = (in·p_in + out·p_out + reason·p_reason + cr·p_cr + cw·p_cw) / 1_000_000
window     = multiplier(timezone, weekday_mask, start_minute..end_minute)
local      = hardware_usd / lifespan_hours × (watts / 1000 × kwh_price) × hours
total      = token_cost × window + per_call + local_share
```

Rounding: store full `f64`, display 4 decimals. Currency conversion is out of scope for v1.

## 8. Packaging

| Target | Command | Output |
|---|---|---|
| Windows | `packaging/build.ps1` | `noema-<ver>-windows-x64.zip`, NSIS `-setup.exe` when `makensis` exists |
| macOS | `packaging/build.sh` | `noema-<ver>-macos-<arch>.dmg` |
| Linux | `packaging/build.sh` | AppImage, `.deb`, `.rpm` (when `rpmbuild` exists), `.tar.gz` |
| CLI | `cargo build --release -p noema-cli -p noema-core` | `noema[.exe]`, `noemad[.exe]` |

The standalone scripts build release binaries, build the Vite UI, stage binaries + `ui/` + `assets/` +
README + LICENSE, and emit portable artifacts under `dist/` (git-ignored).

Block 9 adds the `src-tauri` crate (`noema-desktop`, Tauri 2.12.5): a native window over the same
loopback Studio, with `noema-app` as a sidecar and `ui/`/`assets/` resources. `packaging/tauri.ps1` and
`packaging/tauri.sh` emit NSIS + MSI, a universal dmg and AppImage/deb/rpm into `dist/tauri/`. The
sidecar and resources are passed by `packaging/tauri.bundle.conf.json` only at bundle time, so the
workspace compiles without a built UI.

CI: `.github/workflows/release.yml` runs `cargo test --workspace` on `windows-latest`, `macos-14` and
`ubuntu-24.04`, calls the standalone script, installs tauri-cli 2.12.4, calls the desktop bundle script
and uploads `dist/**` with `actions/upload-artifact@v7`. Windows installs NSIS through Chocolatey; Linux
installs `rpm`, `appimagetool`, WebKitGTK 4.1 and `patchelf`. Linux floor and prerequisites:
`docs/PLATFORMS.md`.

## 9. Version and dependency policy

- **Latest stable, always.** Every language, package, library, module, runtime, action and tool tracks the latest stable upstream release. No old-but-working pins. No alpha/beta/RC/nightly unless the owner asks in writing.
- Adding or bumping a dependency requires: driftwatch evidence, a one-line rationale in the commit body, and a green cargo check --workspace (+ tests where they exist). Docs record the verified version.
- **Driftwatch** (F:\tools\driftwatch, owner's tool) is the drift/changelog authority. CLI:
  - 
ode F:\tools\driftwatch\bin\driftwatch digest --project F:\Progetti\agentic-anatomy-workspace - block start.
  - 
ode F:\tools\driftwatch\bin\driftwatch check - before a dependency change.
  - 
ode F:\tools\driftwatch\bin\driftwatch assess <entryId> --project <repo> --context "<task>" - before a major bump.
  - MCP tools get_digest / check_updates / ssess_update are preferred when the server is registered (.mcp.json); always pass context.
- A reaking|deprecation|security hit on a dependency in use blocks the block until fixed or logged in AGENT_HANDOFF.md risks with the entry id.

## 10. Asset and CAD toolchain (free only)

Zero-cost rule: no paid software, assets or subscription APIs. CADCraft 0.15.1 and SolidCraft 0.28.0 (both installed 2026-10-31; CLIs under `F:\Programmi\ArtCraft\...`) are the primary CAD path. Free fallbacks, all documented in docs/MASTER_PLAN.md §2.1:

| Stage | Tool | License | Output |
|---|---|---|---|
| Parametric CAD | SolidCraft (parametric solids), CADCraft (drafting), FreeCAD (reecadcmd), CadQuery, OpenSCAD | owner / LGPL-2.1+ / Apache-2.0 / GPL-2.0 | STEP, STL, 3MF, source |
| Procedural modeling | Blender LTS headless (lender -b -P) | GPL-3.0 | GLB, textures, turntable renders |
| Generative 3D | Meshy CLI (free tier, browser sign-in) | proprietary free tier | GLB, textures, rigs |
| Mesh hygiene | trimesh + manifold3d, pymeshlab | MIT / Apache-2.0 / GPL-3.0 | repaired, decimated meshes |
| glTF optimization | glTF-Transform, glTF-Validator | MIT / Apache-2.0 | optimized, validated GLB |
| Visual QA | F3D headless renders + vision-helper | BSD-3 / skill | PNG turntables |
| Textures | ImageMagick, KTX-Software | Apache-2.0 | PNG/WebP/KTX2 |

Pipeline order: SolidCraft/CADCraft → FreeCAD/CadQuery/OpenSCAD → Blender → trimesh/pymeshlab → glTF-Transform → glTF-Validator → F3D → vision QA. Licenses: CC0/CC-BY/CC-BY-SA/PD only; ssets/LICENSES.md records file, source, author and license; CC-BY attribution ships in About.

**Block 5 CAD smoke (2026-10-31).** `solidcraft-cli 0.28.0`: `eval` + `run --out vertebra.step` + `snapshot --view iso` on the parametric vertebra (`assets/anatomy/cad/vertebra.json`, 9 closed bodies, 20,187 mm3, GPU snapshot). `cadcraft-cli 0.15.1`: `info` (13 entities) + `convert` to PDF and SVG on a native drafting drawing (`vertebra-section.dxf`), because CADCraft opens DXF/DWG/SVG only — not STEP or PNG. No paid tool or asset was used.

## 11. Hardware awareness and platform layer

Owner directive 2026-10-31: NOEMA must adapt to the machine it is installed on, expose the
result, and let every derived value be overridden. Windows, macOS and Linux are all v1
targets. Implementation: §crates/core/src/hardware.rs§ and §crates/core/src/platform.rs§.

### 11.1 Flow

§§§
OS probe (one command per backend)
  -> HardwareProfile  (cpu, cores, ram, gpus+vram, data volume, host, os/arch)
  -> derive_tuning()  (pure function, no IO)
  -> deep merge hardware.overrides
  -> Tuning           (quality, render, inference, daemon, reasons)
  -> settings rows + RPC/HTTP/CLI + Studio Machine panel
§§§

The profile is cached in §settings§ (scope §hardware§) for 12 h; §refresh§ re-probes.
Overrides are a sparse JSON patch and are never discarded by a refresh. Invalid patches
(unknown key, wrong type) are rejected before the row is written.

### 11.2 Probe backends

| OS | CPU/RAM | GPU | Disk |
|---|---|---|---|
| Windows | one §powershell -NoProfile -NonInteractive§ CIM call (§Win32_Processor§, §Win32_ComputerSystem§) | §Win32_VideoController§ (name, AdapterRAM, driver) | §Win32_LogicalDisk§ for the data-volume drive |
| macOS | §sysctl§ (§machdep.cpu.brand_string§, §hw.physicalcpu§, §hw.logicalcpu§, §hw.memsize§) | §system_profiler SPDisplaysDataType -json§; Apple Silicon VRAM = 75% of RAM | §df -kP§ |
| Linux | §/proc/cpuinfo§, §/proc/meminfo§ | §nvidia-smi§ CSV, then §lspci§, then §/sys/class/drm/card0/device/uevent§; AMD VRAM from §mem_info_vram_total§ | §df -kP§ |

No new crate: §sysinfo§ covers CPU/RAM/disk but not GPU and adds a large Windows
dependency. The fixed OS commands keep the core dependency-free and the probe auditable.
Software adapters (Microsoft Basic, Remote Display, Parsec, llvmpipe, SwiftShader, virtual)
are classified §software§ and excluded from the best-GPU choice.

### 11.3 Derived tuning

| Group | Fields | Source |
|---|---|---|
| §quality§ | §ultra§ / §high§ / §balanced§ / §low§ | best GPU kind+VRAM and total RAM |
| §render§ | §pixel_ratio_cap§, §particle_budget§, §shadows§, §max_texture_mb§, §lod_bias§ | quality tier |
| §inference§ | §max_context_tokens§, §parallel_requests§, §gpu_layers§, §vram_budget_mb§, §recommended_runtime§ | VRAM, RAM, logical cores |
| §daemon§ | §worker_threads§, §sqlite_cache_mb§ | logical cores, RAM |
| §reasons§ | short evidence strings shown in the Studio | profile |

Exact thresholds and tier tables: §docs/HARDWARE.md§ §4. The function is pure and unit
tested, so a profile change is reviewable without running a GPU.

### 11.4 Override contract

- §null§ deletes a key; objects merge recursively; scalars/arrays replace.
- §serde(deny_unknown_fields)§ rejects typos at every level.
- Storage: §settings('hardware.overrides')§, scope §hardware§.
- Surfaces: §noema system profile|override|reset§; RPC §system.profile|override|reset§;
  HTTP §GET /api/system§, §GET /api/system/refresh§, §POST /api/system {overrides}§;
  Studio header "machine" panel (quality preset, pixel cap, particle budget).
- The CLI falls back to the local SQLite store when the daemon is offline, so reading the
  profile never depends on the daemon.

### 11.5 Runtime application

- Boot: §renderer.setPixelRatio(min(devicePixelRatio, render.pixel_ratio_cap))§ and
  §new Circulation(render.particle_budget)§.
- Save: pixel ratio is re-applied and §circulation.setBudget(n)§ disposes/rebuilds the six
  particle flows.
- §max_texture_mb§, §shadows§ and §lod_bias§ are persisted for the brain/graph and packaging
  blocks; they are visible/editable now. Inference and daemon values are the documented
  defaults the adapters must consume.

### 11.6 Platform layer

| Concern | Windows | macOS | Linux |
|---|---|---|---|
| Data root | §%APPDATA%/noema§ | §~/Library/Application Support/noema§ | §$XDG_DATA_HOME/noema§ or §~/.local/share/noema§ |
| Port file | mode + icacls | 0600 | 0600 |
| Secrets | DPAPI | Keychain | Secret Service |
| Installer | NSIS/MSI | DMG | AppImage/deb/rpm |
| WebView | WebView2 | WKWebView | WebKitGTK 4.1 |

§NOEMA_DATA_DIR§ overrides the root on every OS. §docs/PLATFORMS.md§ owns the support
matrix, Linux prerequisites, CI matrix and known gaps.

### 11.7 Privacy and limits

- Local OS commands only; no telemetry or network call in the probe.
- Stored profile includes hostname, CPU/GPU names, RAM and free disk; keep the database
  as private as a log.
- Windows CIM caps reported VRAM at 4 GiB; Apple unified memory is an estimate; Linux
  names need §lspci§; WSL/containers report VM/host values. Refreshing after a dock or
  driver change is the supported remedy.


## 12. Studio surface (Block 7)

The Studio is a hash route (`#studio`) over the same loopback HTTP server, not a second app.

| Endpoint | Contract |
|---|---|
| `GET /api/studio` | Restored session, 25-session list, projects, models, five role assignments, widgets and aggregated usage. |
| `POST /api/studio` | Actions `session`, `role`, `widget`, `widget_remove`, `layout`; returns the full snapshot. |

Role assignments live in `settings` with keys `role.<capability>@global` and
`role.<capability>@project:<id>` (scope `studio`). Capabilities: `embedding`, `router`,
`chat`, `coding`, `dreaming`.

Schema v2 (`crates/core/migrations/0002_studio.sql`) extends `usage_events`:

```sql
ALTER TABLE usage_events ADD COLUMN latency_ms INTEGER;
ALTER TABLE usage_events ADD COLUMN project_id TEXT;
ALTER TABLE usage_events ADD COLUMN thinking TEXT;
```

Widgets persist in `widgets` as `{id,name,chart,spec,x,y,w,h,pinned}`; `spec` is
`{metric,dimension,filters:[{dimension,value}]}`, and the server clamps the 12-column grid.
The full contract and verified flows are in `docs/STUDIO.md`.
