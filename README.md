# NOEMA

**The anatomical mind.** A desktop-native agent platform where agents, thoughts and dreams live inside an explorable 3D human body — with a first-class CLI that has full parity with the GUI.

> `noema` — Greek *νόημα*: the content of thought. Name chosen 2026-10-31.

## Pillars

| # | Pillar | Binary / crate |
|---|--------|----------------|
| 1 | **The Body** — scientific 3D anatomy, beating Heart holding the *Anima* (global identity/system prompts), agents travelling along nerves and vessels | `noema-app` (local shell + Three.js) |
| 2 | **The Studio** — zero-landing-page workspace, session continuity, role-based model assignment, drag-and-drop BI widgets | `noema-app` + `crates/core` |
| 3 | **Ecosystem** — MCP (stdio/SSE/WS/OAuth2), Skills, Plugins, integrated Marketplace, proprietary GitHub plugin | `noema-ecosystem` |
| 4 | **Parity CLI** — every GUI action available from the terminal | `noema` (`crates/cli`) + `noemad` (`crates/core`) |

## Stack

- Rust 2024, Cargo workspace (7 crates), tokio, SQLite + `sqlite-vec`, JSON-RPC 2.0 over loopback IPC. Toolchain: latest stable, pinned by `rust-toolchain.toml` (MSRV floor 1.88, verified on 1.99.1).
- **Version policy:** every package, language, library and tool stays on the latest stable release; driftwatch (`F:\tools\driftwatch`) is the mandatory drift/changelog check. See `docs/ARCHITECTURE.md` §9.
- Local loopback shell (`noema-app`) + TypeScript + Three.js/WebGL2-SDF + Vite; the UI is plain DOM/CSS, no framework.
- CLI shipped as `noema`; daemon `noemad`; standalone packaging: Windows zip (NSIS setup when `makensis` exists), macOS `.dmg`, Linux AppImage/`.deb`/`.rpm`. Desktop bundles (Tauri v2, `src-tauri`): NSIS + MSI, universal `.dmg`, AppImage/`.deb`/`.rpm`. Scripts: `packaging/`; CI matrix: `.github/workflows/release.yml`.

## Quickstart

```powershell
cargo check --workspace                       # workspace health
cargo test --workspace                        # Rust suite
npm --prefix crates/app/ui ci                 # UI deps once
npm --prefix crates/app/ui run build          # UI bundle
cargo run -p noema-app -- serve --demo        # http://127.0.0.1:4173
cargo run -p noema-cli -- studio              # CLI parity: Studio snapshot
cargo run -p noema-cli -- system profile      # machine profile + tuning
```

## Platforms & hardware

| Target | Installer | Runtime | Notes |
|---|---|---|---|
| Windows 10 22H2 / 11 x64 | zip + NSIS setup; desktop NSIS + MSI | WebView2-capable | Validated host; CIM hardware probe; zip built locally |
| macOS 13+ Intel/Apple Silicon | DMG; desktop universal DMG | WKWebView-capable | CI target; artifact produced on `macos-14` |
| Linux x86_64 | AppImage, deb, rpm; desktop bundles | WebKitGTK 4.1-capable | CI target; first CI run pending |

NOEMA detects CPU, RAM, GPU/VRAM and the data volume, derives render/inference/daemon
defaults and caches them for 12 h. Every value can be overridden from the Studio Machine
panel, `noema system override '<json>'`, RPC `system.override` or `POST /api/system`.
Details: [`docs/HARDWARE.md`](docs/HARDWARE.md) and [`docs/PLATFORMS.md`](docs/PLATFORMS.md).

The Studio lives at `#studio`: restored session, projects, five role-model slots and a
drag/resize BI grid with filter-to-widget. Contract: [`docs/STUDIO.md`](docs/STUDIO.md).

## Repo map

```
crates/core         state machine, sessions, SQLite/vector store, event bus, noemad
crates/inference    universal provider adapters, streaming, embeddings, pricing/FinOps
crates/ecosystem    manifest validation, MCP supervisor, marketplace, GitHub plugin
crates/cli          `noema` CLI — 1:1 parity with the Studio
crates/app          loopback Studio server + UI (app/ui)
src-tauri           Tauri v2 desktop wrapper (noema-desktop)
packaging/          standalone + desktop staging scripts; CI contract
docs/               MASTER_PLAN.md, ARCHITECTURE.md, STUDIO.md, HARDWARE.md, PLATFORMS.md, AGENT_HANDOFF.md
agents/             per-agent charters (also installed to OpenClaw workspaces)
```

## Agent team (7)

| Agent | id | Role |
|-------|----|------|
| **Cortex** | `noema-cortex` | Chief Architect & Governance Lead |
| **Axon** | `noema-axon` | Core Rust, Daemon & CLI Engineer |
| **Thalamus** | `noema-thalamus` | Universal Inference & FinOps Engine |
| **Synapse** | `noema-synapse` | Ecosystem, Marketplace & GitHub Plugin |
| **Soma** | `noema-soma` | 3D Bio-Anatomical & Shader Engine |
| **Hippocampus** | `noema-hippocampus` | Neuro-Semantic Graph & Physics |
| **Retina** | `noema-retina` | Studio UI, Session Continuity & BI Widgets |

## Governance

Read `docs/MASTER_PLAN.md` → work one block → tick checklist → update `CHANGELOG.md` + `docs/AGENT_HANDOFF.md` → atomic commit. **`git push` is forbidden** — the owner pushes after review.

## License

MIT © 2026 Adriano Sorbello
