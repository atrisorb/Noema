# NOEMA — MASTER PLAN

> **Project name:** NOEMA (*νόημα*, "the content of thought")
> **Owner:** Adriano Sorbello · **Status:** Block 9 complete (Tauri v2 desktop wrapper, bundle pipeline); standalone packaging + desktop installers · **Date:** 2026-10-31
> **Repo:** `F:\Progetti\agentic-anatomy-workspace` · **License:** MIT
> **Rule:** this file is the single source of execution truth. Agents read it, execute one block, tick it, commit.

---

## 0. One-line pitch

A native desktop platform (`.exe` / `.dmg`) + parity CLI where AI agents live as luminous particles inside an explorable scientific 3D human body, think in a semantic neural graph inside the brain, dream at night, and report token/cost telemetry through a fully configurable Studio — no landing page, ever.

## 1. Pillars and acceptance shape

| Pillar | User-visible outcome | Crate |
|---|---|---|
| **The Body** | Orbit/pan/zoom from full body to cellular detail; beating Heart = *Anima* editor; agents travel nerves/vessels; click agent (3D or right rail) → instant deterministic greeting + 5 inspection tabs | `noema-app`, `noema-core` |
| **The Studio** | App boots straight into the last session (cwd, agent, repo, branch, diff, models, widget layout); every parameter editable without touching a file | `noema-app`, `crates/core` |
| **Ecosystem** | Install MCP (stdio/SSE/WS/OAuth2), Skills, Plugins from a built-in marketplace; proprietary GitHub plugin with `user/repo` autocomplete (public + private), branch switch, live diff | `noema-ecosystem` |
| **Inference/FinOps** | One adapter for every major API + local runtimes; per-model rate cards with peak/off-peak bands, cache and thinking pricing; widgetized BI with filter-to-widget | `noema-inference` |
| **Machine awareness** | Detects CPU/RAM/GPU/data-volume on Windows, macOS and Linux; derives render, inference and daemon defaults; every value overrideable from the Studio, RPC and CLI | `noema-core`, `noema-app`, `noema-cli` |
| **Parity CLI** | `noema` can do 100% of GUI actions headlessly | `noema-cli` |

**Non-goals for v1:** mobile clients, multi-user server, cloud sync, non-MIT marketplace hosting.

## 2. Fase 0 — audit results (2026-10-31, host `hp-atr`)

| Tool | Status | Note |
|---|---|---|
| rustc / cargo | ✅ 1.99.1 | stable-x86_64-pc-windows-msvc; ultima stable verificata con `rustup check` il 2026-10-31, pinnata da `rust-toolchain.toml` |
| rust targets | ✅ | `x86_64-pc-windows-msvc`, `x86_64-apple-darwin`, `aarch64-apple-darwin` installed |
| cargo-tauri | ✅ 2.12.4 | `.exe` + `.dmg` bundling path clear |
| trunk | ✅ 0.22.1 | wasm fallback available |
| git | ✅ 2.55.0 | system git reused by the GitHub plugin (no libgit2 dep) |
| node / npm | ✅ 26.13.0 / 12.0.2 | Vite + React + Playwright |
| Playwright | ✅ 1.64.1 via `npx` + MCP config present | visual QA possible (headful, WebGPU flags in `playwright-mcp-webgpu.json`) |
| python | ✅ 3.14.7 | optional sidecar only; not a runtime dependency |
| Blender CLI | ❌ missing | non-blocking: no offline bake, use procedural + pre-baked GLB |
| CADCraft CLI | ✅ 0.15.1 | `F:\Programmi\ArtCraft\CADCraft\cadcraft-cli.exe`; installed 2026-10-31 |
| SolidCraft CLI | ✅ 0.28.0 | `F:\Programmi\ArtCraft\SolidCraft\solidcraft-cli.exe`; parametric CAD, STEP, snapshots |
| MSVC `link.exe` on PATH | ⚠️ not on PATH | rustc resolves via VS Build Tools; verified by `cargo check` |
| Free disk | ⚠️ C: 9.1 GB, F: 466.8 GB | `target/` forced onto F: via `.cargo/config.toml`; keep dependency tree lean (no Bevy) |

**Verdict:** no blocking tool missing. Blender is still absent (procedural fallback). CADCraft 0.15.1 and SolidCraft 0.28.0 are installed with CLIs; Block 5 uses them as the primary CAD path.

### 2.1 Free asset and CAD toolchain (owner directive, 2026-10-31)

**Constraint: zero cost.** No paid software, no paid assets, no subscription API. Every tool below is free/open-source; every asset must be CC0, CC-BY, CC-BY-SA or public domain, with attribution recorded.

**SolidCraft + CADCraft are the primary CAD path (both installed 2026-10-31).** SolidCraft 0.28.0 (Tangency Labs) does parametric sketch/solid modeling with STEP input and replayable command scripts; CADCraft 0.15.1 does headless drafting and format conversion. Both ship CLIs that are not on `PATH` (absolute paths below). If a CLI regresses, the Tier 1-3 fallbacks below produce the same glTF deliverables. Installer state and the 1603 runbook: §2.2.

| Tier | Tool | License | Role | Install (free) | Verify |
|---|---|---|---|---|---|
| 0 | **SolidCraft CLI 0.28.0** (Tangency Labs) | proprietary app, free in suite | Parametric sketches/solids, STEP, script replay, snapshot/render, MCP server | installed: `F:\Programmi\ArtCraft\SolidCraft\solidcraft-cli.exe` | `& '<path>' --version` → `solidcraft-cli 0.28.0` |
| 0 | **CADCraft CLI 0.15.1** (owner) | proprietary app, free in suite | Headless drafting: `info`, `convert` (dxf/dwg/dwt/svg/png/pdf), `run`, `mcp` | installed: `F:\Programmi\ArtCraft\CADCraft\cadcraft-cli.exe` | `& '<path>' --version` → `cadcraft-cli 0.15.1` |
| 1 | **FreeCAD 1.x** | LGPL-2.1+ | Parametric B-Rep, STEP/IGES, Python API, headless `freecadcmd` | official installer, freecad.org (winget index lacks it) | `freecadcmd --version`; export a cube to STEP |
| 1 | **CadQuery 2.x** | Apache-2.0 | Python parametric CAD, STEP/STL/3MF export | `python -m pip install cadquery` | `python -c "import cadquery; print(cadquery.__version__)"` |
| 1 | **OpenSCAD** | GPL-2.0 | Scripted CSG, STL/3MF/DXF/SVG | openscad.org installer | `openscad --version` |
| 2 | **Blender LTS** | GPL-3.0 | Procedural anatomy, geometry nodes, materials, glTF/FBX export, Cycles/EEVEE renders | blender.org installer (or Steam) | `blender --version`; headless `blender -b -P script.py` |
| 2 | **Meshy CLI** (free tier) | proprietary free tier | Text/image-to-3D, textures, rigging, format conversion | skill `meshy-3d-agent`; browser sign-in, no API key | `meshy --version`; one free generation |
| 3 | **trimesh + manifold3d** | MIT / Apache-2.0 | Mesh cleanup, booleans, decimation, glTF export | `python -m pip install trimesh manifold3d` | `python -c "import trimesh, manifold3d"` |
| 3 | **pymeshlab** | GPL-3.0 | Headless mesh filters, repair, simplification | `python -m pip install pymeshlab` | `python -c "import pymeshlab"` |
| 3 | **glTF-Transform CLI** | MIT | Optimize GLB: prune, dedupe, Draco/Meshopt, texture resize | `npm i -g @gltf-transform/cli` | `gltf-transform --version` |
| 3 | **glTF-Validator** | Apache-2.0 | Hard gate: every shipped GLB must validate clean | `npm i -g gltf-validator` (Khronos) | `gltf-validator --version` |
| 3 | **F3D** | BSD-3 | Headless turntable/screenshot renders for visual QA | `winget install f3d-app.f3d` | `f3d --version` |
| 3 | **ImageMagick** | Apache-2.0 | Texture resize/compose, contact sheets | `winget install ImageMagick.ImageMagick` | `magick -version` |
| 3 | **KTX-Software** | Apache-2.0 | KTX2/Basis textures for WebGL | `winget install KhronosGroup.KTX-Software` | `ktx --version` |
| 4 | **Gmsh** | GPL-2.0+ | FEM meshing, only if simulation is added | gmsh.info installer | `gmsh --version` |

**Asset pipeline (Block 5, ordered):**

1. `driftwatch digest` + version check of every tool against latest stable.
2. SolidCraft CLI for parametric organs (`run`/`eval`/`script`, STEP in/out, `snapshot` for QA); CADCraft CLI for drafting and conversion (`info`, `convert`); if both regress, CadQuery/FreeCAD; OpenSCAD for simple procedural parts.
3. Blender headless (`blender -b -P tools/blender/*.py`): procedural detail, materials, LODs, GLB export.
4. trimesh/pymeshlab: weld, repair, decimate to budget.
5. glTF-Transform: prune/dedupe/compress; glTF-Validator must pass with zero errors.
6. F3D renders a turntable PNG set; `vision-helper` + Retina review the frames.
7. Write `assets/LICENSES.md` (file, source URL, author, license); CC-BY/CC-BY-SA attribution in the About screen.

**Asset sources (free):** BodyParts3D (CC-BY-SA 2.1 JP), Z-Anatomy (CC-BY-SA 4.0), Open Anatomy Project (CC-BY 4.0), Poly Haven and ambientCG (CC0 HDRIs/PBR), Wikimedia Commons (per-file license check).

**Gates:** no paid tool or asset; total GLB budget per scene set; 60 fps in Block 5; every GLB validated; license manifest complete before any asset is committed.

### 2.2 ArtCraft suite — verified state and installer runbook (2026-10-31, `hp-atr`)

Suite updated 2026-10-31; all apps live in `F:\Programmi\ArtCraft\<App>`. No CLI is on `PATH`: call the absolute path and check the version at block start.

| App | Version | CLI | Commands used by NOEMA |
|---|---|---|---|
| SolidCraft (Tangency Labs) | 0.28.0 | `SolidCraft\solidcraft-cli.exe` | `run` / `eval` / `script` (`design.sc`, `part.step`), `snapshot`, `render`, `exec`, `commands`, `mcp` (stdio) |
| CADCraft | 0.15.1 | `CADCraft\cadcraft-cli.exe` | `info`, `convert` (dxf/dwg/dwt/svg/png/pdf), `run`, `commands`, `mcp` |

Verified on `hp-atr`: `solidcraft-cli 0.28.0`; `cadcraft-cli 0.15.1 (660e15c9...)`; `cadcraft.exe` and `cadcraft-cli.exe` present at the path above.

**Install incident (CADCraft 0.13.0 → 0.15.1, error 1603).** The MSI carries `LaunchCondition: Installed OR ALLUSERS <> 1 OR NOT CRAFT_USER_DIR`. The launcher passes `ALLUSERS=1` while the package defaults to `ALLUSERS=2` / `MSIINSTALLPERUSER=1` / `CRAFT_SCOPE=user`; the legacy scan finds `F:\Programmi\ArtCraft\CADCraft\cadcraft.exe` and sets `CRAFT_USER_DIR`, so the per-user → per-machine guard aborts before the install sequence. Fix applied: remove the stale 0.13.0 registration (`{FF850FBE-CE32-4849-8199-530B166A78EE}`), delete the leftover app folder, then install `cadcraft-0.15.1-windows-x64.msi` with `ALLUSERS=1` and `INSTALLFOLDER=F:\Programmi\ArtCraft\CADCraft`. Verify `DisplayVersion=0.15.1` on `{A7F703E8-5986-48D5-B127-737D92ED94DD}` and `cadcraft-cli --version`. The Launcher's **Retry install** works only after the legacy copy is gone.

**Rule:** never drive the Launcher/MSI unattended (UAC credential prompts); the owner triggers installs. No install-scope changes (`ALLUSERS`, `CRAFT_SCOPE`) outside this runbook. `solidcraft-cli mcp setup` may register the MCP server for AI clients, but OpenClaw config edits need owner approval (AGENTS rule 10).

## 3. Architecture

### 3.1 Process topology

```
┌───────────────┐     JSON-RPC 2.0 / NDJSON      ┌──────────────────────────┐
│ noema (CLI)   │ ─────────────────────────────► │ noemad                   │
├───────────────┤                                │  tokio runtime           │
│ noema-app     │ ◄──── events (notifications) ── │  state machine + store   │
│ (Tauri/WebView)│                                │  inference + ecosystem   │
└───────────────┘                                └───────────┬──────────────┘
                                                             │
                                    SQLite WAL + sqlite-vec ──┘
```

- **Single daemon per user**, one client connection per CLI invocation, long-lived WS-like connection for the UI.
- **Transport:** loopback TCP on an ephemeral port written to `%APPDATA%\noema\daemon.json` (Windows) / `~/Library/Application Support/noema/daemon.json` (macOS) / `$XDG_DATA_HOME/noema/daemon.json` or `~/.local/share/noema/daemon.json` (Linux); optional OS named pipe (`\\.\pipe\noema`, unix socket) when present.
- **Auth:** 32-byte random token in the port file, required in `params._auth`; file ACL owner-only.
- **Boot:** UI asks `session.resume` first; never renders an empty landing page. CLI defaults to `status`, which returns the same payload.

### 3.2 Crate graph

```
core        (no deps on siblings)  → ids, sessions, storage, event bus, daemon bin
inference   → depends on core (usage/telemetry ids)
ecosystem   → depends on core (workspace/session binding)
cli         → core + inference + ecosystem (RPC client only)
app         → core (Tauri commands; UI talks RPC like the CLI)
```

Dependency direction is one-way; `core` never imports a sibling. Cross-cutting types live in `core`.

### 3.3 IPC contract (JSON-RPC 2.0, NDJSON frames)

| Method | Direction | Purpose |
|---|---|---|
| `session.resume` | C→S | last session snapshot (project, agent, cwd, repo, branch, diff stat, models, widget layout) |
| `session.open` / `session.close` | C→S | create/restore session |
| `agent.list` / `agent.inspect` | C→S | runtime activity, tokens, cost, assigned MCP/skills/plugins, workdirs |
| `chat.send` | C→S | streaming; responses arrive as `chat.delta`, `chat.usage`, `chat.done` |
| `models.*`, `rates.*` | C→S | provider/model/rate-card CRUD |
| `mcp.*`, `skill.*`, `plugin.*` | C→S | lifecycle: install, enable, start, stop, logs |
| `repo.*` | C→S | GitHub auth, search `user/repo`, branches, checkout, diff |
| `brain.*` | C→S | thoughts, edges, clusters, dream runs, activation telemetry |
| `widget.*` | C→S | save/update layout and filter-to-widget specs |
| `event.*` | S→C | notifications: `usage`, `thought`, `agent.move`, `health` |

Errors: JSON-RPC codes + `data.kind` (`auth`, `not_found`, `validation`, `transport`, `provider`). Versioned by `noema/v1`.

### 3.4 Database (SQLite, WAL, ms timestamps, uuid TEXT PKs)

Core tables: `projects`, `sessions`, `messages`, `agents`, `providers`, `models`, `rate_cards`, `rate_windows`, `usage_events`, `thoughts`, `thought_edges`, `dreams`, `widgets`, `mcp_servers`, `skills`, `plugins`, `repos`, `settings`, `schema_migrations`.
Vectors: `thoughts_vec` as `vec0(thought_id, embedding float[dim])` when `sqlite-vec` loads; otherwise brute-force cosine over `thoughts.embedding` BLOB.
Full DDL: `docs/ARCHITECTURE.md` §4.

### 3.5 Config and secrets

- Layered TOML: `%APPDATA%\noema\config.toml` (global) → `<project>\.noema\project.toml` (project) → env `NOEMA_*` → CLI flags.
- **Zero file-touching:** every key is declared in a schema registry (`settings` table + static descriptors) that generates the Studio configurator and `noema config …`; JSON/TOML is an implementation detail.
- Secrets only in the OS keychain (`keyring` crate): provider keys, GitHub token, MCP OAuth tokens.

### 3.6 Frontend

- **React 19 + TypeScript + Vite + Tailwind**, state in `zustand`, data via the RPC client (same protocol as CLI).
- Routes: `Studio` (default, session-restored), `Body`, `Brain`, `Marketplace`, `Settings`. No blank home.
- Widget grid: `react-grid-layout`; chart renderer: `uPlot` (small, fast) with a thin spec adapter; every widget persists `{filters, chart, name, position}`.

### 3.7 Rendering strategy

- **Body:** Three.js scene, glTF anatomical layers (nervous, vascular, skeletal, muscular) with per-layer opacity; camera tween/focus; picking via GPU-id buffer for agents and organs.
- **Infinite resolution:** geometry = WebGL2 meshes; all labels and 2D graph primitives = SVG overlay; node glyphs = analytic SDF in shader (crisp at any zoom). LOD switches at fixed zoom thresholds, no texture text.
- **Brain:** instanced SDF nodes + SDF edges; physics in a Web Worker at 60 fps; positions returned to Rust for persistence.
- **Agents:** emissive particle trails along spline paths sampled from anatomical curves; click = raycast on capsule proxy; greeting is static from DB — **no LLM call**.

### 3.8 Inference and FinOps

- One `Request` normaliser → per-standard adapter → one `StreamEvent` stream.
- Standard coverage: OpenAI Chat/Responses, Anthropic Messages, Gemini, DeepSeek (native `reasoning_content`), Ollama, llama.cpp server, vLLM, LM Studio, custom HTTP.
- Full sampling surface incl. `top_k`, `min_p`, `typical_p`, `repeat_penalty`, `seed`, `stop`, `reasoning_effort`/`thinking_budget`, `logprobs`, template vars, extra headers/payload.
- Pricing: `RateCard` per model with `rate_windows` (weekday/time band, IANA timezone), cache read/write, reasoning, per-call, local amortisation (`hardware_cost_usd`, `lifespan_hours`, `watts`, `kwh_price`).
- Clustering: embeddings → kNN graph → LLM structured extraction (`concepts`, `relations`, `causal_links`) → weights → clusters persisted, never keyword-only.

### 3.9 Ecosystem standards

- `noema.json` manifest (JSON Schema validated) for `mcp | skill | plugin`, plus `SKILL.md` frontmatter compatible with Claude Code.
- MCP transports: stdio (command, args, env, cwd), SSE, WebSocket; auth Bearer/OAuth2 PKCE/mTLS/custom headers; timeout + retry policy.
- Marketplace index is a signed static JSON catalog; bundled packages live in `marketplace/bundled/`.
- **GitHub plugin:** system `git` binary only. OAuth device flow + PAT; `GET /user/repos` + `GET /search/repositories` autocomplete; branch list; `git diff` rendered in the Studio; session pinned to repo+branch.

### 3.10 Packaging

- Tauri bundler targets: Windows NSIS + MSI; macOS `.dmg` (universal); Linux AppImage + `.deb` + `.rpm`. CI matrix builds on `windows-latest`, `macos-14` and `ubuntu-24.04`; artifacts signed later.
- Linux runtime floor: WebKitGTK 4.1, GTK 3, libsecret (Secret Service) and `lspci` for GPU names. Full matrix, prerequisites and gaps: `docs/PLATFORMS.md`.
- CLI + daemon shipped inside the app bundle and as a standalone `noema` archive.
- Hardware awareness is compiled into every target: paths, keychain backend and probes are selected with `cfg` in `crates/core/src/platform.rs` and `crates/core/src/hardware.rs`; see `docs/HARDWARE.md`.

## 4. Agent team

| # | Name | id | Owns | Cannot touch |
|---|---|---|---|---|
| 1 | **Cortex** | `noema-cortex` | `docs/MASTER_PLAN.md`, `docs/AGENT_HANDOFF.md`, repo hygiene, cross-block review, final QA | feature crates (review only) |
| 2 | **Axon** | `noema-axon` | `crates/core`, `crates/cli`, daemon, IPC, DB migrations, packaging | `app/ui` |
| 3 | **Thalamus** | `noema-thalamus` | `crates/inference`, embeddings, clustering, pricing | `app/ui` |
| 4 | **Synapse** | `noema-synapse` | `crates/ecosystem`, manifests, MCP supervisor, marketplace, GitHub plugin | `app/ui` 3D |
| 5 | **Soma** | `noema-soma` | 3D body/heart/agents, shaders, camera, `app/ui` 3D layer | `crates/core` |
| 6 | **Hippocampus** | `noema-hippocampus` | brain graph, SDF/SVG rendering, physics, UMAP, Dream Mode | `crates/cli` |
| 7 | **Retina** | `noema-retina` | Studio UI, session UX, configurator, widgets, Playwright QA | `crates/inference` |
| 8 | **Aesthesis** | `noema-aesthesis` | Design system, brand, SVG/mascots, artifact workspace UX, anatomy visual QA | `crates/core`, `crates/inference`, `crates/ecosystem` |

Cortex may spawn the other seven (`subagents.allowAgents`). Everyone else works alone, no self-expansion.

## 5. Governance protocol (iron)

1. Read `docs/AGENT_HANDOFF.md`, `git status`, last 20 lines of `CHANGELOG.md`.
2. Run the driftwatch digest for this repo (`node F:\tools\driftwatch\bin\driftwatch digest --project F:\Progetti\agentic-anatomy-workspace`) and address or log its findings.
3. Execute **one** block from §6.
4. `cargo check --workspace` green (+ tests where they exist).
5. Tick `[x]` in §7.
6. Append `CHANGELOG.md` (date, block, files, verification).
7. Rewrite `docs/AGENT_HANDOFF.md` (state, decisions, next, risks).
8. `git add -A && git commit -m "<type>(<scope>): <detail>"`.
9. **Never `git push`.** Owner reviews and pushes to `main`.
10. No destructive git. No secrets in the repo. No dependency without a rationale line.
11. Hard block only → ask owner. Otherwise decide + record.
12. **Latest stable only.** Every package, language, library, module, runtime and tool tracks the latest stable release; no old pins, no prerelease unless the owner asks in writing. New dependency = driftwatch `check` first, then the green build.

## 6. Execution blocks

| Block | Title | Owner | Exit criteria |
|---|---|---|---|
| 1 | Foundations, repo, Rust workspace, agent registration | **Cortex** | `cargo check --workspace` green, docs + checklist exist, 7 agents registered, first commit |
| 2 | Local DB, session continuity, core state | **Axon** | migrations run, `session.resume` returns last state, event bus streams, CLI `status` reads daemon |
| 2.1 ✅ | Core hardening from Block 2 review | **Axon** | `session.open` validates ids/models, 8 MiB frame cap enforced, daemon integration test green, Windows ACL documented, clippy clean |
| 3 | Universal inference, embeddings, FinOps | **Thalamus** | 8+ adapters, streaming normalised, cost matches hand-calculated fixtures, clustering produces thoughts+edges |
| 4 | Ecosystem, manifests, marketplace, GitHub plugin | **Synapse** | manifest validation, stdio/SSE/WS MCP lifecycle, bundled packages install, `user/repo` search + diff works |
| 5 | 3D world: body, heart/anima, agents | **Soma** | 60 fps orbit/zoom, heart editor persists, agent click → deterministic greeting + 5 tabs, every GLB glTF-Validator clean, asset license manifest complete |
| 6 | Brain: graph, physics, Dream Mode | **Hippocampus** | crisp at all zoom levels, tether limit holds, wake/dream switch, UMAP projection + logprob telemetry |
| 7 | Studio, configurator, BI widgets | **Retina** | boot into last session, all settings editable, drag/resize widgets, filter-to-widget persists |
| 5.5 ✅ | Hardware awareness + Windows/macOS/Linux target | **Cortex** | profile+tuning persisted, overrideable via CLI/RPC/HTTP/Studio, 7 core tests + Playwright green, docs `HARDWARE.md` + `PLATFORMS.md` |
| 8 ✅ | CLI parity, visual QA, packaging | **Axon** (QA: **Retina**, review: **Cortex**) | `noema` covers every UI action, Playwright suite green, `.exe` + `.dmg` + AppImage/deb/rpm produced; Windows artifact verified, macOS/Linux via CI |
| 9 ✅ | Tauri v2 desktop wrapper, bundles, installer pipeline | **Axon** | window loads the loopback Studio, three commands, NSIS/MSI/dmg/AppImage/deb/rpm configured, standalone zip path intact |
| 10 | Vitreous design system, brand, artifact workspace, anatomy realism | **Aesthesis** | tokens + SVG brand + 8 mascots, artifact workspace edit/export, lower legs/feet fixed, no external genitalia, zero PNG/JPEG, 60fps + a11y evidence |

## 7. Risks and recorded decisions

| Decision | Rationale |
|---|---|
| Tauri v2 + WebView, not Bevy | ship size, compile time, native bundling already installed; WebGPU/WebGL covers the rendering brief |
| System `git` for the GitHub plugin | already present, zero heavy deps, exact diff semantics |
| SQLite + `sqlite-vec`, not DuckDB | embedded, single file, WAL concurrency; DuckDB adds 100+ MB and no vector index |
| Physics in a TS worker | 60 fps drag without IPC jitter; Rust stays authoritative for topology |
| Procedural glTF anatomy | Blender/CADCraft absent; keeps Block 5 unblocked |
| `noema`/`noemad` names | CLI parity requires a stable binary name |
| Free-only asset/CAD toolchain | owner directive 2026-10-31: zero paid software, assets or APIs. CADCraft primary; FreeCAD/CadQuery/OpenSCAD/Blender/trimesh/glTF-Transform free fallbacks. See §2.1 |
| Hardware probe without a new crate | std + one OS command per backend covers CPU/RAM/GPU/disk on all three targets; `sysinfo` has no GPU and adds a large Windows dependency. Rationale recorded in `docs/HARDWARE.md` |
| Cache hardware profile 12 h + explicit refresh | probing costs one PowerShell/CIM call on Windows; docking stations/eGPUs need a manual refresh. Overrides persist across refreshes |
| Linux is v1, not a future port | owner directive 2026-10-31: same data paths, secret backend, probes and packaging targets. Windows remains the validated host; Linux CI is the proof path |
| Standalone binary packaging, not the Tauri bundler (Block 8) | the shell serves the UI over loopback and has no `src-tauri` crate; the scripts stage `noema`, `noemad`, `noema-app`, `ui/` and `assets/` into zip/dmg/AppImage/deb/rpm. Adding Tauri is a separate, dependency-heavy block; docs state the actual pipeline |

Open risks: C: disk pressure (mitigated by target dir), high-fidelity anatomy asset licensing, macOS signing not yet available on this host, no physical Linux host has run the suite yet (CI defined, first run pending), Windows CIM caps reported VRAM at 4 GiB.

---

## MASTER EXECUTION CHECKLIST (STATO LAVORI)

### Blocco 1: Fondamenta, Repo, Workspace Rust & Registrazione Agenti
- [x] **1.1** Scelta del nome progetto (**NOEMA**), `git init`, `.gitignore`, `LICENSE` (MIT), `README.md`.
- [x] **1.1a** `.gitignore` copre Rust, Node, asset 3D, OS, segreti (`.env*`).
- [x] **1.2** Creazione e registrazione dei 7 Sub-Agenti specifici nell'ambiente OpenClaw (Cortex, Axon, Thalamus, Synapse, Soma, Hippocampus, Retina) con charter ferree.
- [x] **1.2a** Verifica: tutti e 7 gli id risolvono in `agents.entries.*`, workspace + charter installati, `agents.ownership` invariato.
- [x] **1.3** Setup del Cargo Workspace (`core`, `inference`, `ecosystem`, `cli`, `app`) e compilazione base.
- [x] **1.3a** `cargo check --workspace` verde.
- [x] **1.4** Inizializzazione `/docs` (`MASTER_PLAN.md`, `ARCHITECTURE.md`, `AGENT_HANDOFF.md`) e `CHANGELOG.md`.
- [x] **1.5** Verifica Piano + Primo Git Commit (NO Push).

### Blocco 2: Database Locale, Continuità di Sessione & Core State
- [x] **2.1** Schema DB (SQLite + Vector Store) per sessioni, pensieri, sogni, telemetria token/costi e configurazioni.
- [x] **2.1a** Migrazioni versionate (`schema_migrations`) e apertura WAL.
- [x] **2.1b** Caricamento opzionale `sqlite-vec` + fallback cosine brute-force.
- [x] **2.2** Motore di persistenza "Zero-Landing-Page": caricamento automatico di ultimo workspace, agente, repo, branch e diff all'avvio.
- [x] **2.2a** `session.resume` restituisce anche layout widget e modelli per ruolo.
- [x] **2.3** Event Bus asincrono per sincronizzazione real-time tra CLI, Studio e Mondo 3D.
- [x] **2.3a** `noemad` binario: token di auth, port file, lifecycle.
- [x] **2.4** Aggiornamento Docs/Changelog + Verifica Piano + Git Commit (NO Push).

### Blocco 2.1: Hardening post-review (follow-up Block 2)
- [x] **2.1.1** `session.open` valida id sessione, agente e modello (formato + FK) prima del salvataggio.
- [x] **2.1.2** Cap frame NDJSON 8 MiB applicato in `noemad`; doc allineata.
- [x] **2.1.3** Test integrazione daemon: auth `-32001`, `daemon.ping`, stream `event.health`.
- [x] **2.1.4** `daemon.json` ACL owner-only su Windows (best-effort) o rischio esplicito in HANDOFF.
- [x] **2.1.5** `cargo clippy --workspace --all-targets` senza warning (`chunks_exact_to_as_chunks` in `vector.rs`).

### Blocco 3: Motore di Inferenza Universale, Embedding & FinOps Pricing
- [x] **3.1** Adattatori multi-standard (OpenAI, Anthropic, Gemini, DeepSeek, Ollama, llama.cpp, custom endpoints) con tutti i parametri di sampling e thinking.
- [x] **3.1a** Streaming normalizzato (`delta`, `reasoning`, `usage`, `done`).
- [x] **3.1b** `logprobs` e `reasoning_content` esposti alla telemetria cervello.
- [x] **3.2** Motore di calcolo costi (Input/Output/Reasoning/Cache, Peak/Off-Peak hours, modelli locali e remoti).
- [x] **3.2a** Fixture di test con costi calcolati a mano.
- [x] **3.3** Pipeline di Embedding e Clustering Semantico guidato da LLM per la costruzione dei nodi pensiero.
- [x] **3.3a** Estrazione strutturata concetti/relazioni/pesi, persistenza nodi+archi.
- [x] **3.4** Aggiornamento Docs/Changelog + Verifica Piano + Git Commit (NO Push).

### Blocco 4: Ecosistema, Standardizzazione MCP/Skill/Plugin & Plugin GitHub Proprietario
- [x] **4.1** Parser e validatore degli schemi standardizzati per MCP, Skill e Plugin (formato Claude Code compatibile).
- [x] **4.1a** JSON Schema + `SKILL.md` frontmatter + errori di validazione parlanti.
- [x] **4.2** Supervisor MCP per server locali (`stdio`) e remoti (`SSE`/`WS` con Auth/OAuth2/Headers).
- [x] **4.2a** Retry policy, timeout, log per server.
- [x] **4.3** Sviluppo del Marketplace integrato e dei pacchetti proprietari preinstallati.
- [x] **4.3a** Memory/Semantic introspection MCP + System/Visual QA skill.
- [x] **4.4** Sviluppo del Plugin GitHub Proprietario: autenticazione, ricerca `utente/repo` (pubbliche e private), selettore branch e vista diff.
- [x] **4.4a** Autocomplete live, checkout, diff, pin sessione su repo+branch.
- [x] **4.5** Aggiornamento Docs/Changelog + Verifica Piano + Git Commit (NO Push).

### Blocco 5: Il Mondo 3D (Corpo Umano, Cuore/Anima & Agenti in Movimento)
- [x] **5.1** Pipeline 3D (WebGPU/Three.js + integrazione SolidCraft CLI 0.28.0 + CADCraft CLI 0.15.1) e camera 3D (orbita, zoom microscopico, customizzazione livelli).
- [x] **5.1a** Layer anatomici con opacità e focus progressivo.
- [x] **5.1b** CLI smoke: `solidcraft-cli eval` + `snapshot` on one anatomy part, `cadcraft-cli info` / `convert` on the exported STEP/PDF; artifacts under `assets/`.
- [x] **5.2** Rendering del Cuore ("L'Anima") e interfaccia di configurazione dell'identità centrale.
- [x] **5.3** Sistema di movimento fluido ad alta velocità degli agenti nel corpo, intercettazione al clic (3D + scheda laterale destra) e saluto deterministico istantaneo con menu di ispezione.
- [x] **5.3a** Cinque tab: attività, token, costi, skill/plugin/MCP, workspace.
- [x] **5.3b** Zero chiamate LLM per il saluto.
- [x] **5.4** Aggiornamento Docs/Changelog + Verifica Piano + Git Commit (NO Push). Evidence: `docs/evidence/block5-world.png`, `block5-anima.png`, `block5-agent-rail.png`, `block5-cellular.png`, `block5-60fps.png`; 5/5 Playwright green; 60.1 fps.

### Blocco 5.5: Consapevolezza Hardware & Supporto Windows/macOS/Linux
- [x] **5.5.1** `crates/core/src/hardware.rs`: profilo macchina (CPU, core, RAM, GPU/VRAM, disco dati, host, OS/arch) con probe nativo Windows CIM, macOS sysctl+system_profiler, Linux /proc+nvidia-smi/lspci/sysfs; nessuna dipendenza nuova.
- [x] **5.5.2** `crates/core/src/platform.rs`: data root per OS (`%APPDATA%`, `~/Library/Application Support`, XDG) e backend keychain (DPAPI/Keychain/Secret Service); `daemon.json` continua 0600/icacls.
- [x] **5.5.3** Tuning derivato e persistito (`settings` scope `hardware`): qualità ultra/high/balanced/low, pixel ratio, particle budget, texture budget, LOD, contesto/offload/parallelismo inferenza, thread daemon e cache SQLite; TTL 12 h + refresh.
- [x] **5.5.4** Override profondi con validazione (`null` cancella, chiavi ignote rifiutate) esposti da CLI (`noema system profile|override|reset`), RPC (`system.profile|override|reset`) e HTTP (`GET/POST /api/system`).
- [x] **5.5.5** Studio “Machine” panel: profilo live, preset qualità, pixel cap, particle budget, apply/re-detect; il renderer applica pixel ratio e circulation ricostruisce i flussi al salvataggio.
- [x] **5.5.6** Supporto Linux allineato: percorsi XDG, permessi 0600, Secret Service, probe Linux, target AppImage/deb/rpm e CI matrix aggiornata; documentato in `docs/PLATFORMS.md`.
- [x] **5.5.7** Docs/Changelog + test: 7 unit test core verdi, Playwright `block6-system.spec.ts` verde, evidence `docs/evidence/block6-hardware.png`; `docs/HARDWARE.md` + `docs/PLATFORMS.md` completi.

### Blocco 6: Il Cervello (Rete Neurale Semantica, Fisica Vincolata & Dream Mode)
- [x] **6.1** Motore di rendering ibrido 3D/SVG/SDF a risoluzione infinita (zero degrado visivo a qualsiasi zoom).
- [x] **6.2** Fisica Force-Directed con vincolo elastico rigido (`max_tether_radius`) per il trascinamento dei singoli pensieri e apertura al clic.
- [x] **6.2a** Test: drag oltre il raggio non sposta la cascata.
- [x] **6.3** Mappatura scientifica delle attivazioni LLM (UMAP 3D + entropia logprobs/thinking) e switch Vista Veglia / Vista Sogno (Dream Mode).
- [x] **6.3a** Proiezione ancorata a regioni funzionali (prefrontale, ippocampo, Broca/Wernicke, motoria).
- [x] **6.3b** Feed live da Thalamus: `ActivationWindow` su `StreamEvent` normalizzati, `POST/GET /api/brain/telemetry`, campione live 30 s applicato ai pulse; UI poll 1.5 s.
- [x] **6.3c** Spatial hashing oltre 200 nodi (griglia uniforme, 27 celle, lato `cbrt(4.2/n)`), Rust + worker; test 240 e 500 nodi.
- [x] **6.3d** Persistenza drag: `POST /api/brain/drag` scrive `thoughts.pos_x/y/z`; le coordinate salvate vincono sulla proiezione.
- [x] **6.4** Aggiornamento Docs/Changelog + Verifica Piano + Git Commit (NO Push). Evidence: `docs/evidence/block6-brain-zoom.png`, `block6-brain-far.png`, `block6-brain-dream.png`, `block6-brain-thought.png`; Playwright `tests/block6-brain.spec.ts` green; 8/8 hippocampus + 6/6 app + 21/21 inference tests; HTTP smoke on `:4199` for live telemetry and drag readback.

### Blocco 7: Lo Studio, Configuratore Totale & Motore Widget BI
- [x] **7.1** Interfaccia completa dello Studio: gestione progetti, sessioni, assegnazione modelli (Embedding, Decisionale, Chat, Coding, Dreaming) e parametri predefiniti. — `crates/app/ui/src/studio.ts`, `GET/POST /api/studio`; global + per-project role keys in `settings` (schema v2).
- [x] **7.2** Griglia di Widget BI interattivi (spostabili, ridimensionabili, rimovibili) per tutte le viste di utilizzo e spesa. — 12-column grid, drag/resize/remove persisted in `widgets`; metrics tokens/cost/latency over agent/model/project/peak/thinking.
- [x] **7.3** Generatore dinamico "Filter-to-Widget" con salvataggio e naming custom/default. — spec `{metric,dimension,filters[]}`; custom names; defaults seeded when the table is empty.
- [x] **7.4** Aggiornamento Docs/Changelog + Verifica Piano + Git Commit (NO Push). — `docs/STUDIO.md`, `CHANGELOG.md` 0.5.2, commit `ea86668`.

### Blocco 8: CLI Proprietaria, Visual QA Autonomo & Packaging (.exe / .dmg / AppImage)
- [x] **8.1** Completamento della CLI proprietaria con parità 100% rispetto a tutte le funzioni della GUI. — `health`, `anatomy`, `anima`, `brain` (+telemetry/drag/dream), `system`, `studio` (session/role/widget/widget-remove/layout), `session` (list/open/resume), `methods`, `rpc`; app bridge via `NOEMA_APP_PORT`, daemon via `daemon.json`.
- [x] **8.1a** Test di parità: ogni metodo RPC raggiungibile da `noema`. — `rpc::METHODS` catalog + `rpc.methods`; unit tests `every_rpc_method_has_a_cli_path` and `every_gui_route_has_a_cli_command`; live calls `noema rpc rpc.methods`, `session list`, `studio role`.
- [x] **8.2** Test visivi e funzionali autonomi (Playwright / Vision tool) su risoluzione neurale, animazioni 3D e widget. — `tests/block8-qa.spec.ts`: DPR-backed canvas, live frame counter, brain zoom/wake/dream, persisted widget grid; 11/11 suite green.
- [x] **8.3** Configurazione e test della pipeline di build multipiattaforma per generare gli installer `.exe` (Windows), `.dmg` (macOS) e AppImage/`.deb`/`.rpm` (Linux). — `packaging/build.ps1` (zip + NSIS when `makensis`), `packaging/build.sh` (dmg/AppImage/deb/rpm/tar.gz); Windows artifact built locally (`dist/noema-0.1.0-windows-x64.zip`, 8.17 MB); macOS/Linux artifacts are produced by their runners.
- [x] **8.3a** CI matrix Windows + macOS + Linux (ubuntu-24.04), artefatti allegati. — `.github/workflows/release.yml` on `windows-latest`, `macos-14`, `ubuntu-24.04`; Rust tests, platform package script, `upload-artifact@v7`.
- [x] **8.4** Revisione finale di tutta la documentazione (`/docs`, `README.md`, `CHANGELOG.md`), spunta finale di tutti i task `[x]` e Git Commit conclusivo (NO Push). — README, ARCHITECTURE §8, PLATFORMS §6, STUDIO §8, packaging README, CHANGELOG 0.5.3, final commit.

### Blocco 9: Tauri v2 Desktop Wrapper, Bundle Configuration & Installer Pipeline
- [x] **9.1** Crates wrapper `src-tauri` (`noema-desktop`) con Tauri 2.12.5 / tauri-build 2.7.4. Finestra creata a runtime su `http://127.0.0.1:NOEMA_APP_PORT` (default 4173); nessun server HTTP duplicato, nessuna build frontend aggiuntiva. Se il bridge non risponde, avvia il sidecar `noema-app` accanto all'eseguibile e ripiega su `src-tauri/frontend/index.html`. Autostart provato a freddo (nessun `noema-app`, 4173 libera): il sidecar parte su 4173, la finestra carica lo Studio a 60 fps ed esce alla chiusura; evidence `docs/evidence/block9-desktop-autostart.png`.
- [x] **9.2** Command surface minimale: `health`, `bridge_info`, `session_resume` (passthrough di `GET /api/studio` → `session`). Capability `main` con `core:default` e `remote.urls` su `http://127.0.0.1:*` / `http://localhost:*`.
- [x] **9.3** Bundle: `bundle.targets = all` (NSIS + MSI su Windows, DMG su macOS, AppImage/deb/rpm su Linux), icone generate in `src-tauri/icons`, risorse `ui/` + `assets/` e sidecar `noema-app` passati da `packaging/tauri.bundle.conf.json` al momento del bundle. Installer Windows verificati localmente: `NOEMA_0.1.0_x64-setup.exe` 5.18 MiB, `NOEMA_0.1.0_x64_en-US.msi` 7.91 MiB.
- [x] **9.4** Pipeline: `packaging/tauri.ps1` (Windows), `packaging/tauri.sh` (macOS universal, Linux), `.github/workflows/release.yml` con dipendenze WebKitGTK, Tauri CLI 2.12.4 e upload `dist/**`; zip/`.dmg`/tar standalone di Block 8 invariati.
- [x] **9.5** Docs/Changelog + Verifica Piano + Git Commit (NO Push). — `docs/AGENT_HANDOFF.md` (command surface per Retina + autostart proof), `packaging/README.md`, `docs/PLATFORMS.md` §6, CHANGELOG 0.5.4; `cargo check --workspace` verde, `cargo test --workspace` verde, configurazione bundle validata da cargo-tauri; layout installer verificato (sidecar accanto all'exe, `ui/`+`assets/` nella resource dir).


### Blocco 10: Vitreous Design System, Brand, Artifact Workspace & Anatomy Realism
- [x] **10.1** Audit + freeze: read `docs/design/*`, driftwatch digest, full asset/reference inventory, current-state WEBP captures, `docs/design/AUDIT.md`.
- [x] **10.2** Decisions + tokens v1: `docs/design/DECISIONS.md`, `crates/app/ui/src/design/tokens.css`, `tokens.json`; Cavity + Bone; no hard-coded hex in touched surfaces.
- [x] **10.3** Logo suite: mark, monogram, wordmark, lockup, stacked lockup, favicon and app-icon source as SVG; icon generation script; no raster committed.
- [ ] **10.4** Mascots: eight agent SVG mascots with shared rig, 16/24/96/512 sizes, trace/breathe motion, reduced-motion fallback, sprite loader.
- [ ] **10.5** Glass + shell: Lens/Membrane/Cavity primitives, Spine layout, HUD, agent rail, inspector, Studio, Brain, Anima, Machine; dark/light; responsive.
- [ ] **10.6** Artifact workspace: chat + canvas + inspector + timeline + assets + export; `.noema.html` parse/edit/props/comments/versions/verify.
- [ ] **10.7** Dashboards, chat and templates: one real dashboard, one chat surface, one template gallery; every artifact editable, downloadable, exportable, forkable, versioned.
- [ ] **10.8** Anatomy realism: material library, skin/fascia surface, lower-leg/foot geometry, neutral pelvis, external genitalia removed, lighting, LOD, glTF-Validator clean. Coordinate with Soma; Aesthesis verifies.
- [ ] **10.9** Asset migration: all PNG/JPEG/ICO sources -> SVG/WEBP/generated; update references; `tools/design/asset-audit.mjs` green; CI gate.
- [ ] **10.10** QA + docs: Playwright, axe, contrast, 60fps trace, WEBP evidence, `cargo check --workspace` green, CHANGELOG + AGENT_HANDOFF updated, atomic commit (no push).
