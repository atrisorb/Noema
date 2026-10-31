# CHANGELOG

All notable changes. Newest first. One entry per completed block/commit.

## [0.5.22] - 2026-10-31 - Automatic PATH Management & GitHub Releases Auto-Updater

### Added
- Automated PATH Engine (`crates/core/src/path_env.rs`):
  - Added cross-platform PATH management with permanent registry updates on Windows (`HKCU\Environment`), environment broadcast, duplicate detection, broken path pruning, and companion toolchain discovery (`SolidCraft`, `CADCraft`, `Driftwatch`, `Cargo`).
  - Added CLI command `noema path [status|add|remove|clean|setup]` and bridge endpoints `GET/POST /api/system/path`.
- In-Place Auto-Updater (`crates/core/src/update.rs`):
  - Added GitHub Releases API integration querying latest assets, semver delta analysis, platform-matching asset downloading, and in-place executable replacement with Windows `.old` file lock mitigation.
  - Added CLI command `noema update [--check|--force]` and bridge endpoints `GET/POST /api/system/update`.
- Code CLI & Vitreous Terminal Engine (`crates/cli`):
  - Added full theme system (`crates/cli/src/theme.rs`) supporting `cavity`, `bone`, `nord`, `catppuccin`, and `monokai` palettes with ANSI 24-bit Truecolor, Windows VTP auto-enable, styled cards, and Unicode table drawing.
  - Added `noema doctor` (`crates/cli/src/doctor.rs`) for full-stack system, storage, daemon, hardware, toolchain, and inference health auditing with actionable fix suggestions.
  - Added `noema diff` (`crates/cli/src/diff.rs`) unified code diff viewer with syntax highlighting and line numbers.
  - Added `noema mcp [list|status|test]` (`crates/cli/src/mcp_cmd.rs`) integrated with `noema-ecosystem` package supervisor.
  - Added `noema agents` (`crates/cli/src/agents_cmd.rs`) displaying the 8 specialized agents and domain roster.
  - Added `noema chat / ask` (`crates/cli/src/chat_cmd.rs`) interactive code assistant with streaming thought indicators.
  - Added global `--json` and `--no-color` flags for pipeline and script parity.

### Verified
- `cargo test --workspace`: clean (68 tests passing across all crates).
- Live execution on Windows: `F:\NOEMA` installed, enrolled in user PATH, verified via `noema path status` and `noema update --check`.

## [0.5.21] - 2026-10-31 - Linux Packaging Harmonisation & Release Collision Fix

### Fixed
- Linux Packaging Pipeline (`packaging/build.sh`):
  - Streamlined `build.sh` on Linux to focus purely on building the portable distribution archive (`dist/noema-$VERSION-linux-$ARCH.tar.gz`).
  - Removed redundant manual AppDir/deb/rpm generation in `build.sh` that was colliding with the official Tauri desktop bundler (`packaging/tauri.sh`), eliminating HTTP 422 asset name collision on GitHub Releases.
- Bumped version to `0.5.21` across `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json`.

### Verified
- `cargo check --workspace`: clean (0 errors, 0 warnings).

## [0.5.20] - 2026-10-31 - Verified Tauri CLI Range & Icon Builder Fallback

### Fixed
- Platform Icon Generation (`tools/design/build-icons.mjs`):
  - Changed npm package reference from non-existent `@tauri-apps/cli@2.12.5` to `@tauri-apps/cli@2` (resolving latest published stable `2.12.4` without ETARGET npm errors).
  - Pinned `tauri-cli` to verified stable `2.12.4` in `.github/workflows/release.yml`.
- Bumped version to `0.5.20` across `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json`.

### Verified
- `cargo check --workspace`: clean (0 errors, 0 warnings).
- `node tools/design/build-icons.mjs`: clean (generated all platform icons in 4s).

## [0.5.19] - 2026-10-31 - Pure Native GitHub CLI Releases & AppImage AppRun Integration

### Fixed
- Linux AppImage Packaging (`packaging/build.sh`):
  - Created executable `$APPDIR/AppRun` wrapper script resolving `$HERE` path, `PATH`, and `LD_LIBRARY_PATH` prior to launching `noema-app`, fixing AppImage generation failure (`Problem: AppRun does not exist`).
  - Standardized `.desktop` categories to `Categories=Science;`.
- GitHub Actions Release Publishing (`.github/workflows/release.yml`):
  - Completely replaced legacy third-party `softprops/action-gh-release@v2` (which targeted deprecated Node 20) with official native GitHub CLI (`gh release`).
  - Releases and asset uploads across Windows, macOS, and Linux runners now run via native Go binaries, eliminating all Node 20 deprecation warnings.
- Dependency & Tool Upgrades:
  - Updated `tauri-cli` to `2.12.5` in `.github/workflows/release.yml` and `tools/design/build-icons.mjs` to match `tauri = 2.12.5`.
  - Bumped version to `0.5.19` across `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json`.

### Verified
- `cargo check --workspace`: clean (0 errors, 0 warnings).
- `node tools/design/build-icons.mjs`: clean.

## [0.5.18] - 2026-10-31 - Universal Cross-Platform Packaging & CI Warning Cleanup

### Fixed
- macOS Packaging (`packaging/tauri.sh`):
  - Staged both `noema-app-aarch64-apple-darwin` and `noema-app-x86_64-apple-darwin` sidecars alongside `noema-app-universal-apple-darwin`, fixing Tauri's architecture-slice validation error during `--target universal-apple-darwin`.
  - Used recursive `find` for `.dmg` copying to ensure artifacts land in `dist/tauri` regardless of bundle layout.
- Linux Packaging (`packaging/build.sh`):
  - Ensured `$APPDIR/usr/share/noema` directory creation before copying assets, eliminating missing target directory error during AppImage staging.
- Linux AppImage & FUSE Support (`.github/workflows/release.yml`):
  - Added `APPIMAGE_EXTRACT_AND_RUN: "1"` to GitHub Actions environment to guarantee AppImage execution on runners without kernel FUSE.
  - Added fallback installation for `libfuse2t64` / `libfuse2`.
- Compiler Warnings (`crates/core/src/hardware.rs`):
  - Guarded `fn num` with `#[cfg(windows)]` to eliminate unused function warning on Linux and macOS targets.
  - Eliminated unread initial assignment to `core` variable in Linux `/proc/cpuinfo` parser.
- Frontend Bundling (`crates/app/ui/vite.config.ts`):
  - Adjusted `chunkSizeWarningLimit` to 1000 kB for Draco 3D decoder and Three.js bundle, eliminating Vite chunk size warning.
- Version bump to `0.5.18` across `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json`.

### Verified
- `cargo check --workspace`: clean (0 errors, 0 warnings).
- `npm --prefix crates/app/ui run build`: clean (0 warnings).

## [0.5.17] - 2026-10-31 - Platform Icon Builder Robustness & Fallback

### Fixed
- `tools/design/build-icons.mjs`:
  - Added detection and graceful fallback to `npx @tauri-apps/cli@2.12.4 icon` when `cargo-tauri` binary is not yet installed in CI environment, preventing exit code 101/1 during pre-test icon generation.
- Version bump to `0.5.17` across `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json`.

### Verified
- `cargo check --workspace`: clean.
- `node tools/design/build-icons.mjs`: verified generating all StoreLogo, ICO, ICNS, PNG, iOS, and Android icons cleanly.

## [0.5.16] - 2026-10-31 - Multi-Platform CI / Release Pipeline Hardened & Package Alignment

### Fixed
- CI / Release Pipeline (`.github/workflows/release.yml`):
  - macOS runner: updated from deprecated `macos-14` (undergoing brownout retirement) to `macos-15`.
  - Windows test gate: added `node tools/design/build-icons.mjs` step prior to `cargo test --workspace` so Tauri's Windows resource compiler (`tauri-build`) finds `icon.ico` on clean CI checkouts.
  - Linux hardware probe (`crates/core/src/hardware.rs`): added `.ok()` conversion to fix method resolution error E0599 on `std::io::Result<String>`.
  - GitHub Releases: elevated workflow permissions to `contents: write` and added `softprops/action-gh-release@v2` step so tags automatically publish compiled installers (`.exe`, `.msi`, `.dmg`, `.AppImage`, `.deb`, `.rpm`) and portable zip/tarballs to GitHub Releases.
- Version Alignment:
  - Bumped `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json` to `0.5.16` to match release tag `v0.5.16`, ensuring package scripts produce `noema-0.5.16-*` matching GitHub release tags.

### Verified
- `cargo check --workspace`: clean (0 errors, 0 warnings).
- `cargo test --workspace`: 64 passed across all crates, 0 failed.

## [0.5.15] - 2026-10-31 - Exhaustive 2026 Model Catalog: Full Endpoint Aliases, Claude Opus 5.5 & Deprecated Model Mappings

### Added
- `crates/inference/src/cost.rs`:
  - Complete, exhaustive October 2026 frontier model registry across all 10 providers (45+ active and deprecated models mapped):
    - **Google Gemini**: `gemini-4-argon` (unified 1M window flagship, $2.00/$10.00, $0.10 implicit 95% cache discount, default thinking 32k), `gemini-3.8-flash` ($0.75/$3.75), `gemini-3.5-flash-lite` ($0.30/$2.50), `gemini-3.1-pro` ($2.00/$12.00), `gemini-3.8-live` ($1.00/$4.00 native audio/video), `gemini-2.5-flash` ($0.075/$0.30), `gemini-2.5-pro` ($1.25/$5.00), `gemini-2.0-flash` ($0.10/$0.40). Deprecated/Legacy mappings: `gemini-1.5-pro` (replaced by `gemini-4-argon`), `gemini-1.5-flash` (replaced by `gemini-2.5-flash`), `gemini-1.0-pro` (sunset).
    - **OpenAI**: `gpt-6-astra` ($10.00/$50.00, 1.05M window), `gpt-6.1-sol` ($2.00/$10.00), `gpt-6.1-terra` ($1.00/$5.00), `gpt-6-luna` ($0.10/$0.50), `gpt-realtime-2.1` ($4.00/$24.00), `gpt-realtime-2.1-mini` ($0.60/$2.40), `o3` ($2.00/$8.00), `o3-pro` ($20.00/$80.00), `o4-mini` ($1.10/$4.40), `o3-mini` ($1.10/$4.40), `o1` ($15.00/$60.00), `gpt-4o` ($2.50/$10.00), `gpt-4o-mini` ($0.15/$0.60). Deprecated/Sunset mappings: `o1-preview`, `o1-mini` (replaced by `o4-mini`/`o3`), `gpt-4-turbo`, `gpt-4` (replaced by `gpt-4o`), `gpt-3.5-turbo` (replaced by `gpt-4o-mini`).
    - **Anthropic**: `claude-fable-5-1` ($10.00/$50.00, 1M context / 128k output flagship), `claude-opus-5-5` ($4.00/$20.00, 1M context / 128k output, 32k thinking, aliases `claude-opus-5.5`, `claude-5-5-opus-20261001`, `claude-opus-5-5-latest`), `claude-sonnet-5-5` ($2.00/$10.00), `claude-haiku-5-5` ($0.10/$0.50), `claude-3-7-sonnet` ($3.00/$15.00), `claude-3-5-sonnet` ($3.00/$15.00), `claude-3-5-haiku` ($0.80/$4.00). Deprecated/Legacy mappings: `claude-3-opus` (replaced by `claude-opus-5-5`), `claude-3-sonnet` (replaced by `claude-sonnet-5-5`), `claude-3-haiku` (replaced by `claude-haiku-5-5`).
    - **Mistral AI**: `mistral-large-4` (*Le Chonk* 1.05T MoE, 1M context, $0.70-$1.36 / $2.10-$4.18, $0.07-$0.14 cache read), `mistral-medium-3.5` ($0.40/$1.20), `mistral-large-2411` ($2.00/$6.00), `mistral-small-4` ($0.10/$0.30), `mistral-small-2501` ($0.10/$0.30), `ministral-3-14b` ($0.20/$0.20), `ministral-3-8b` ($0.10/$0.10), `ministral-3-3b` ($0.04/$0.04), `codestral-2501` ($0.30/$0.90), `pixtral-12b` ($0.15/$0.15). Deprecated/Legacy mappings: `open-mixtral-8x22b` (replaced by `mistral-large-4`), `open-mixtral-8x7b` (replaced by `ministral-3-14b`).
    - **DeepSeek**: `deepseek-flash` (serving DeepSeek-V4.1-Flash-1017, peak-aware $0.15-$0.30 in / $0.60-$1.20 out / $0.003-$0.006 cache, 1M context), `deepseek-v4-pro` (serving DeepSeek-V4-Pro-0813, $0.66/$1.98), `deepseek-reasoner` (R1, $0.55/$2.19), `deepseek-chat` (V3, $0.14/$0.28). Deprecated mapping: `deepseek-coder` (replaced by `deepseek-chat`).
    - **Zhipu AI (GLM)**: `glm-5.3` (1M context, $1.40/$4.40), `glm-5.3-flash` ($0.15/$0.50), `glm-5.3-flashx` ($0.37/$1.25), `glm-4-plus` ($1.40/$1.40), `glm-4-air` ($0.14/$0.14), `glm-4-long` ($0.14/$0.14), `glm-4.7-flash` (free tier). Deprecated mapping: `glm-3-turbo` (replaced by `glm-5.3-flash`).
    - **Moonshot AI (Kimi)**: `kimi-k3.1-pro` (1M context, $2.00/$10.00, $0.20 cache read), `kimi-k3.1-pro-highspeed` ($4.00/$20.00), `kimi-k3` ($3.00/$15.00), `moonshot-v1-128k`, `moonshot-v1-32k`, `moonshot-v1-8k`.
    - **Meta Llama**: `llama-3.3-70b-instruct` ($0.60/$0.90), `llama-3.1-405b-instruct` ($3.00/$3.00), `llama-3.1-70b-instruct` ($0.50/$0.75), `llama-3.1-8b-instruct` ($0.05/$0.08), `llama-3.2-90b-vision` ($0.80/$1.20), `llama-3.2-11b-vision` ($0.16/$0.16), `llama-3.2-3b`, `llama-3.2-1b`. Deprecated mapping: `llama-2-70b-chat` (replaced by `llama-3.3-70b-instruct`).
    - **Alibaba Qwen**: `qwen-2.5-coder-32b-instruct` ($0.20/$0.20), `qwen-2.5-coder-7b` ($0.05/$0.05), `qwen-2.5-72b-instruct` ($0.35/$0.40), `qwen-2.5-32b`, `qwen-2.5-14b`, `qwen-2.5-7b`, `qwen-2.5-math-72b`, `qwq-32b-preview`. Deprecated mapping: `qwen-2-72b-instruct` (replaced by `qwen-2.5-72b-instruct`).
    - **Local runtimes**: zero-token rate for Ollama / vLLM / llama.cpp / LM Studio (`local/llama3.3`, `local/deepseek-r1`, `local/qwen2.5-coder`, `local/mistral-nemo`, `local/phi-4`) with amortised hardware and kWh calculation.
  - Full endpoint name mapping on every entry via `aliases`: supports date-stamped snapshots (e.g. `2026-10-15`, `20241022`), URL path models (`models/gemini-4-argon`), dotted/hyphenated variations (`claude-opus-5.5`), and provider prefixes.
  - Deprecation tracking: `deprecated: bool`, `deprecation_notice: Option<&'static str>`, `replacement_id: Option<&'static str>`.
  - Resolution helpers: `active_models()`, `deprecated_models()`, `resolve_endpoint_alias()`, `all_accepted_endpoint_names()`.
- `crates/inference/src/lib.rs`: re-exported all new query and resolution functions.

### Verified
- `cargo test -p noema-inference`: 26 passed, 0 failed.
- `cargo test --workspace`: 64 passed across all crates, 0 failed.
- `cargo check --workspace`: clean.
- `npm run --prefix crates/app/ui build`: clean.

## [0.5.14] - 2026-10-31 - Universal Model Catalog & 2026 Rate Cards (Thalamus)

### Added
- `crates/inference/src/cost.rs`:
  - Flagship 2026 Google Gemini 3.x models: `gemini-3.8-flash` ($0.75/$3.75), `gemini-3.5-flash-lite` ($0.30/$2.50), `gemini-3.1-pro` ($2.00/$12.00). Legacy 2.5 retained for compatibility.
  - OpenAI models: `gpt-4o` ($2.50/$10.00), `gpt-4o-mini` ($0.15/$0.60), `o1` ($15.00/$60.00), `o3-mini` ($1.10/$4.40).
  - Anthropic models: `claude-3-7-sonnet` ($3.00/$15.00), `claude-3-5-sonnet` ($3.00/$15.00), `claude-3-5-haiku` ($0.80/$4.00), `claude-3-opus` ($15.00/$75.00).
  - Specialized & local: `codestral` ($0.30/$0.90), `local_zero()` (zero token rate for local Ollama / llama.cpp / vLLM runtimes).
  - `ModelCatalogEntry` struct and `default_catalog()` function exposing 15+ models across Google, OpenAI, Anthropic, DeepSeek, Mistral, and local runtimes.
- `crates/inference/src/lib.rs`: re-exported `ModelCatalogEntry` and `default_catalog`.

### Verified
- `cargo test -p noema-inference`: 25 passed, 0 failed.
- `cargo test --workspace`: 63 passed across all crates, 0 failed.
- `cargo check --workspace`: clean.
- Driftwatch: verified 0 breaking updates on project dependencies.

## [0.5.13] - 2026-10-31 - Block 10.8 Closure, Security S1, FinOps & Brain Route

### Added
- `crates/app/ui/src/surface.ts`: soft contact occlusion ground shadow plane under the feet at y = -0.86, functioning across both Cavity and Bone themes.
- `crates/app/ui/src/brain-route.ts`: `#brain` full-viewport hash route controller with Escape dismissal, reduced motion adaptation, and coexistence with `#studio`.
- `crates/app/ui/tests/block10-brain-route.spec.ts`: Playwright E2E test verifying hash navigation, Escape dismissal, and studio coexistence (1 passed, 10.7s).
- `crates/core/src/store.rs`: `UsageRecord`, `record_usage`, and `recent_usage` live usage writer and query methods with unit tests.
- `crates/inference/src/cost.rs`: standard rate cards for DeepSeek-V3, DeepSeek-R1, Mistral Large, Mistral Small, Gemini 2.5 Flash, Gemini 2.5 Pro, and model resolver `RateCard::for_model()`.

### Changed
- `crates/app/ui/src/atlas.ts`: calibrated classification rules for all 826 meshes (0 misclassified meshes) with full Latin, clinical, and compound muscle and bone terminology. Musculature across legs, feet, arms, and face now correctly categorised and tinted.
- `src-tauri/capabilities/main.json`: remote loopback IPC restricted to exact origins and ports (4173, 5173, 4199).
- `src-tauri/tauri.conf.json`: strict CSP enforced with no remote origins; `withGlobalTauri` disabled to close Security S1.
- `packaging/build.sh`: Linux packaging pipeline uses dynamically generated SVG platform icons without committing raster sources.
- `docs/MASTER_PLAN.md`: Block 10.8 marked complete.

### Verified
- `cargo test --workspace`: green across all 6 crates (60+ tests passing).
- `cargo check --workspace`: green.
- `npm run build`: green (~410ms).
- `npx playwright test tests/block10-brain-route.spec.ts`: green.
- `tools/design/asset-audit.mjs`: 0 forbidden files, 0 violations.
- Tokens `--check` and contrast 24/24 AA green.
- Pelvic audit confirmed neutral (0 external genitalia meshes).
- No `git push`.

## [0.5.12] - 2026-10-31 - Block 10.8-10.10: Anatomy audit, asset migration, QA

### Added
- `tools/anatomy/audit_pelvis.mjs` + `docs/evidence/design-pelvis-audit.json`: 1,872 nodes / 826 meshes; 12 flagged names, all pubic bones; 0 external-genitalia meshes. Pelvis close-up `design-pelvis.webp`.
- `tools/design/asset-audit.mjs` + `design-asset-audit.json`: 0 forbidden files, 0 product-reference violations.
- `crates/app/ui/scripts/capture-anatomy.mjs`; `design-body-surface.webp`, `design-body-legs.webp`, `design-anatomy.json`.

### Changed
- Atlas material calibration: muscle roughness .44, bone .52, clearcoat .04/.12; env intensity unchanged.
- All 28 tracked PNG converted to WebP (1.22 MB total) and deleted; 14 text files and packaging references updated.
- Generated Tauri icons remain git-ignored; no raster source committed.

### Verified
- `node tools/design/asset-audit.mjs`: 0/0.
- `cargo check --workspace` and `npm run build` green; tokens/brand checks green.
- Artifact verify 8/8; shell and artifact WebP captures; 0 console errors.
- 10.8 is **partial**: the skin/fascia overlay was rejected (read as boots/gloves); lower-leg/foot/hand geometry needs Soma. No false completion.
- No `git push`.
## [0.5.11] - 2026-10-31 - Block 10.6/10.7: Artifact workspace

### Added
- `crates/app/ui/src/artifacts.ts`: `.noema.html` model - manifest, template, token styles, `data-noema-id` anchors, parse/build/verify.
- Artifact overlay: gallery, chat transcript, live iframe canvas, inspector (props/tokens/layers/code/comments), timeline and versions.
- One real dashboard (runtime agents, triangles, meshes, fps and token-share SVG), one chat artifact and one template poster.
- Verify gate: parse, schema, title, anchors, no network, no raster, tokens, license. Report in the UI and `window.__noemaArtifacts`.
- Export HTML/JSON/Markdown from the manifest; versions store full HTML and restore.
- `scripts/capture-artifacts.mjs`, `design-artifacts-gallery.webp`, `design-artifact-dashboard.webp`, `design-artifact-verify.json`.

### Changed
- Command bar gains `artifacts`; Escape closes the workspace.
- Artifact styles are token-only and iframe-sandboxed.

### Verified
- Verify 8/8 pass, 9 anchors, no network, no raster; export lengths HTML 5,490 / JSON 596 / Markdown 2,620.
- 3 gallery cards; one comment added and resolved; zero console errors.
- `npm run build` green. No raster added, no `git push`.

## [0.5.10] - 2026-10-31 - Block 10.5: Vitreous shell

### Added
- Spine shell: 56px command bar, 72px mascot rail (48px buttons), 380px Layers inspector, 28px status filament with fps/session/theme.
- Persistent Cavity/Bone toggle (`data-theme` on `<html>`, `noema:theme` in localStorage); scene background follows the theme.
- Layers moved into the inspector with `aria-expanded`/`aria-hidden`; Escape closes inspector and rail.
- Skip link, global `:focus-visible` ring, reduced-motion override.
- Shell evidence: `design-shell-cavity.webp`, `design-shell-bone.webp`, `design-shell-layers.webp`, `design-shell-mobile.webp`, `design-shell.json`.

### Changed
- `style.css`: Vitreous token overrides for shell, panels, Studio, Brain card, inputs, buttons, status. CSS 30.16 KB raw / 6.14 KB gzip.
- `main.ts`: theme controller, inspector controller, roster aria-labels and live agent count.
- `index.html`: command bar actions, brand mark, inspector, status filament, skip link.

### Verified
- `npm run build` green.
- Captures at 1600x900 and 390x844; theme survives reload; 7 agents; 60 fps.
- No raster added, no `git push`.

## [0.5.9] - 2026-10-31 - Block 10.4: Eight mascots

### Added
- `assets/mascots/`: eight SVG glyphs with `body`, `trace` and `glow` groups - Cortex Loom, Axon Arrow, Thalamus Lantern, Synapse Knot, Soma Mason, Hippocampus Seahorse, Retina Iris, Aesthesis Prism.
- `crates/app/ui/src/mascots.ts`: Vite raw-SVG loader, 24px roster integration, unknown ids return empty.
- `assets/mascots/README.md`: rig and size contract.
- `tools/design/render-mascot-sheet.mjs` and `docs/evidence/design-mascots.webp`: 96px and 24px, Cavity + Bone.

### Changed
- Roster buttons render the agent mascot instead of the generic dot; accent stays per agent. Hover raises the glow; reduced motion removes the transition.
- `docs/MASTER_PLAN.md` 10.4 ticked; design README state.

### Verified
- `npm run build` green; mascot SVGs inline in the bundle.
- All eight render at 24px and 96px; Aesthesis ships as an asset, not a fake runtime agent.
- No raster added, no `git push`.

## [0.5.8] - 2026-10-31 - Block 10.3: SVG logo suite

### Added
- Seven SVG sources in `assets/brand/`: mark (lens + N filament), monogram, wordmark, lockup, stacked lockup, 1024 app icon and favicon.
- `tools/design/sync-brand.mjs`: favicon source -> `crates/app/ui/public/favicon.svg`; `--check` fails on drift.
- `tools/design/build-icons.mjs`: `cargo tauri icon` from the SVG source into git-ignored `src-tauri/icons/`.
- `tools/design/render-logo-suite.mjs` and `docs/evidence/design-logo-suite.webp`: Cavity + Bone sheet, mark at 16/24/32/48/96.
- Favicon link in `index.html`.

### Changed
- Removed committed Tauri PNG/ICO/ICNS icons and `src-tauri/app-icon.png`; `src-tauri/icons/` and `src-tauri/app-icon.png` are git-ignored generated outputs.
- `package.json`: `brand`, `brand:check`, `icons`.
- `docs/MASTER_PLAN.md`: 10.3 ticked; design README state.

### Verified
- `node tools/design/sync-brand.mjs --check`: favicon ok.
- `node tools/design/build-icons.mjs`: generated 32/64/128/128@2x, ICO, ICNS, Windows Store and mobile sets.
- `npm run build`: green; favicon in `dist`.
- Logo sheet: no prohibition-slash reading; wordmark and lockups use the lens O.
- No raster committed, no `git push`.

## [0.5.7] - 2026-10-31 - Block 10.2: Vitreous tokens v1

### Added
- `crates/app/ui/src/design/tokens.json`: canonical Cavity + Bone tokens - 29 color pairs, elevation, 12 type scales, space, radius, z-index, motion, easing, blur and focus.
- `crates/app/ui/src/design/tokens.css`: generated token layer, imported by `style.css`; 154 lines, 5.2 KB raw, +4.2 KB CSS gzip.
- `tools/design/build-tokens.mjs`: deterministic generator; `--check` fails on drift.
- `tools/design/check-contrast.mjs`: WCAG 2.2 AA gate for 24 opaque pairs; writes `docs/evidence/design-tokens-contrast.json`.
- npm scripts `tokens`, `tokens:check`, `contrast`.

### Changed
- `docs/design/NOEMA_DESIGN_SYSTEM.md` v1.1: `--noema-artery-ink` and `--noema-danger-ink` for text on accent fills; Bone `ink-faint` corrected to `#697383`; primary and danger buttons use the new ink tokens.
- `docs/design/DECISIONS.md`: D11 AA corrections, D12 JSON-to-CSS token pipeline, D13 theme contract.
- `docs/design/README.md`: token workflow and current state.
- `docs/MASTER_PLAN.md`: 10.2 ticked.

### Verified
- Contrast: 24/24 pass. Cavity artery-ink 6.60:1, danger-ink 7.26:1, Bone ink-faint 4.52:1.
- `npm run build`: green; CSS 16.38 KB raw / 4.24 KB gzip; token layer loaded.
- `node tools/design/build-tokens.mjs --check`: green.
- Driftwatch: 8 updates, zero design impact.
- No raster added, no `git push`.

## [0.5.6] - 2026-10-31 - Block 10.1: Design audit and freeze

### Added
- `docs/design/AUDIT.md`: measured baseline for Blocks 10.2-10.10 - driftwatch digest, 34-raster / 70-reference inventory, UI token debt, anatomy findings, artifact workspace gap, accessibility and performance baselines, gap register.
- `docs/evidence/design-baseline-{body,legs,rail,machine,brain,studio}.webp` and `design-baseline.json`: current-state captures at 1600x900, DPR 1, 826 meshes and 290,886 triangles.
- `crates/app/ui/scripts/capture-audit.mjs`: reproducible Playwright capture that writes WebP through CDP; no PNG is created.

### Changed
- `docs/MASTER_PLAN.md`: 10.1 ticked.
- `docs/design/README.md`: current state reflects 10.1 completion.
- `docs/AGENT_HANDOFF.md`: rewritten for the next block.

### Verified
- `cargo check --workspace` green (0.98s, cached).
- Driftwatch digest: 8 updates, zero design impact; DeepSeek/Mistral/Gemini pricing forwarded to Thalamus.
- Raster audit: 33 PNG + 1 ICO, 8,628,345 bytes, zero JPEG/GIF/BMP/TIFF.
- UI baseline: 65 hex + 26 rgba in `style.css`, 5 hex in `main.ts`, 4 backdrop filters, no WOFF2, no brand SVG, 7 agents.
- Anatomy: legs/feet/hands skeletal; pelvis mass unresolved pending the 10.8 mesh audit.
- No `git push` performed.

## [0.5.5] - 2026-10-31 - Design charter: Aesthesis and Vitreous standards

### Added
- New agent **Aesthesis** (`noema-aesthesis`): design system architect, brand custodian, artifact workspace lead and anatomy visual QA. Registered in `agents.entries` with workspace `~/.openclaw/workspaces/noema-aesthesis` and the charter installed. **Not started.**
- `docs/design/NOEMA_DESIGN_SYSTEM.md`: Vitreous v1.0 - Cavity/Bone themes, Newsreader/Instrument Sans/Commit Mono, glass tiers, motion, Spine layout, components, eight mascots, logo suite, voice, accessibility, performance budgets, anti-patterns and the new-component gate.
- `docs/design/ARTIFACT_WORKSPACE.md`: Claude Design structural parity matrix, `.noema.html` artifact format, chat/canvas/inspector/timeline shell, direct edit, props, comments, versions, dashboards, templates, export matrix, verification gate and backend contract.
- `docs/design/ANATOMY_AND_ASSETS.md`: current body audit, lower-leg/foot diagnosis, genitalia audit and removal rules, PBR material library, lighting, geometry fixes, SVG-first asset law and raster migration table.
- `docs/design/DECISIONS.md`: ten accepted design decisions with rejected alternatives.
- `docs/design/README.md`: design-law index and status.
- `assets/brand/README.md`: SVG-only brand source contract and icon generation rules.
- `docs/research/CLAUDE_DESIGN_2026.md`: verified Claude Design product facts, leaked/reverse-engineered structure, adopt/improve/reject analysis and sources.
- `docs/MASTER_PLAN.md` Block 10 checklist for Aesthesis (10.1-10.10).

### Changed
- `README.md`: agent team 7 -> 8; Aesthesis row; design-law link.
- `docs/MASTER_PLAN.md`: Aesthesis team row, spawn count 6 -> 7, Block 10 row and checklist.
- Local-only `AGENTS.md` on this host: rule 17 Design law, Aesthesis team row, design ownership note. The file is git-ignored by design ("Local agent instructions - never push"), so it is not part of this commit.

### Verified
- `cargo check --workspace` green (1.19s, cached).
- `agents.entries.noema-aesthesis` exists with the requested model, workspace, cwd, thinking level and tool profile; the workspace `AGENTS.md` is installed; the agent was not run.
- No new dependency, no code change, no gateway restart.
- Raster inventory unchanged: 34 PNG + 1 ICO, zero JPEG/WEBP. Migration to SVG/WEBP is Block 10.9, not this doc block.
- No `git push`.

### Notes
- The design docs are the durable, tracked standard. The agent charter lives in the git-ignored `agents/` directory, consistent with the repo's local-agent-instruction policy.
- The body audit found no node names matching penis/genitalia in `body_full.glb`; the visible pelvic protrusion still needs a mesh-level audit and removal if external. The lower legs/feet are a geometry + material problem, not a tint bug.
- Claude Design structure was studied; no prompt text, code, assets or visual identity was copied.

## [0.5.4] - 2026-10-31 - Block 9: Tauri v2 desktop wrapper and installer bundles

### Added
- `src-tauri` crate `noema-desktop` on Tauri 2.12.5 / tauri-build 2.7.4: a runtime window over the existing loopback Studio (`NOEMA_APP_PORT`, default 4173). No second HTTP server and no extra frontend build.
- Sidecar start: when `GET /api/health` is down, the shell spawns `noema-app serve --port <port>` from next to the executable and forwards `--ui`/`--assets` from Tauri's resource dir. Otherwise it shows `src-tauri/frontend/index.html`.
- Tauri command surface: `health`, `bridge_info`, `session_resume` (passthrough of `GET /api/studio` -> `session`). Capability `main` grants `core:default` and IPC from `http://127.0.0.1:*` / `http://localhost:*`.
- Bundle configuration: `bundle.targets = all`, generated icons in `src-tauri/icons`, NSIS + MSI (Windows), dmg (macOS universal), AppImage/deb/rpm (Linux).
- `packaging/tauri.ps1`, `packaging/tauri.sh` and `packaging/tauri.bundle.conf.json`. The override adds the `noema-app` externalBin and the `ui/`+`assets/` resources only at bundle time.
- CI: WebKitGTK 4.1 / patchelf build deps, tauri-cli 2.12.4, both Apple targets, desktop bundle step, `dist/**` artifact upload.

### Changed
- Root workspace includes `src-tauri`; `.gitignore` covers `src-tauri/gen/` and `src-tauri/binaries/`.
- Standalone Block 8 packaging (`build.ps1`, `build.sh`) is untouched and still emits the portable zip/dmg/AppImage artifacts.
- Docs updated: `docs/PLATFORMS.md` section 6, `docs/ARCHITECTURE.md` crate table and packaging section, `packaging/README.md`, `docs/AGENT_HANDOFF.md` command surface for Retina.
- UI side of the block (Retina/Soma): `crates/app/ui/WRAPPER.md` wrapper contract (commit 91ad5c1) and the WebView2 60 fps harness in `tools/perf/` (commit eb3325b). The published command names match this shell; the UI consumes none.

### Verified
- `cargo check --workspace` green including the Tauri crate.
- `cargo check -p noema-desktop` green with `crates/app/ui/dist` renamed away: a clean checkout compiles before Vite runs.
- `cargo test --workspace` green: 58 tests (7 app, 2 cli, 8 core, 1 daemon, 11 ecosystem, 8 hippocampus, 21 inference).
- Driftwatch digest 2026-10-31: 8 platform updates; `check` clean; no Rust or Node dependency affected.
- `cargo tauri build --config packaging/tauri.bundle.conf.json --no-bundle`: `target/release/noema-desktop.exe` built in 3m23s.
- Runtime: release shell launched against `noema-app serve --demo --port 4199`; window title `NOEMA`, 1136x739, full 3D body and 60 fps counter. Evidence `docs/evidence/block9-desktop.webp`.
- Cold autostart: `target/debug/noema-desktop.exe` with no `noema-app` running and port 4173 free spawned the sidecar itself (pid 20080), served the Studio at `http://127.0.0.1:4173/`, rendered the full 3D body at 60 fps, and terminated the child when the window closed. Evidence `docs/evidence/block9-desktop-autostart.webp`.
- Packaged layout: the WiX source places `noema-app.exe` next to the shell and installs `ui/` + `assets/` in the install dir, which is the resource dir the shell resolves; the debug run used the same layout.
- Windows installers built locally by `packaging/tauri.ps1 -SkipBuild`: `NOEMA_0.1.0_x64-setup.exe` (5,431,825 bytes) and `NOEMA_0.1.0_x64_en-US.msi` (8,298,496 bytes) in `dist/tauri/` (git-ignored). The generated WiX source lists `noema-app.exe` plus the full `ui/` and `assets/` trees.

### Notes
- New dependencies: `tauri` and `tauri-build` for the desktop shell (owner decision: wrapper is its own block). Versions are the latest stable: tauri 2.12.5, tauri-build 2.7.4, tauri-cli 2.12.4.
- Windows NSIS and MSI are verified on `hp-atr`; dmg/AppImage/deb/rpm remain CI-defined, so the first CI tag run is their artifact proof.
- The committed icon set is a generated placeholder; replace `src-tauri/app-icon.png` and rerun `cargo tauri icon` before branding.
- Block 9 = 359 handwritten lines of code, config and scripts (`src-tauri`, `packaging`, CI workflow, workspace glue), below the 400-LOC default; the raw diff reads larger only because of `Cargo.lock` and the generated `src-tauri/gen` schemas.
- Doc-accuracy follow-up (Cortex review D2/P1/S1): the handoff now records that commit b77f17f rewrote `docs/evidence/block9-webview2.json` (port 4299, fpsMean 60.081, p99 17.0 ms) instead of claiming it was byte-identical, keeps this LOC note, and lists the loopback-wildcard capability plus `withGlobalTauri` and null CSP as an open security risk.

## [0.5.3] - 2026-10-31 - Block 8: CLI parity, visual QA, packaging

### Added
- Full CLI over the Studio bridge: `health`, `anatomy`, `anima`, `brain` (`telemetry`, `drag`, `dream`), `system`, `studio` (`session`, `role`, `widget`, `widget-remove`, `layout`), `session` (`list`, `open`, `resume`), `methods` and `rpc`. App calls target `NOEMA_APP_PORT` (default 4173); daemon calls reuse `daemon.json`.
- `noema_core::rpc::METHODS` catalog plus `rpc.methods`, so clients can enumerate the daemon surface.
- Parity tests: `every_rpc_method_has_a_cli_path` and `every_gui_route_has_a_cli_command` in `crates/cli`; `every_method_reaches_the_dispatch_table` in `crates/core`.
- `crates/app/ui/tests/block8-qa.spec.ts`: DPR-backed canvas, live frame counter, brain zoom/wake/dream and persisted widget grid; evidence `block8-render.webp`, `block8-neural-resolution.webp`, `block8-dream.webp`, `block8-widgets.webp`.
- `packaging/build.ps1`: Windows zip with `noema.exe`, `noemad.exe`, `noema-app.exe`, `ui/`, `assets/`, README and LICENSE; NSIS setup when `makensis` exists.
- `packaging/build.sh`: macOS `.dmg`; Linux AppImage, `.deb`, `.rpm` (when `rpmbuild` exists) and `.tar.gz`.
- `.github/workflows/release.yml`: `windows-latest`, `macos-14`, `ubuntu-24.04`; Rust tests, platform packaging, artifact upload.

### Changed
- `noema methods` prints the daemon RPC catalog; `noema rpc` stays the escape hatch for any method.
- README, `docs/ARCHITECTURE.md` §8, `docs/PLATFORMS.md` §6 and `docs/STUDIO.md` §8 now describe the real standalone-binary pipeline instead of the unimplemented Tauri bundler.
- `docs/MASTER_PLAN.md`: Block 7.4 and Block 8 fully ticked; v1 checklist closed.

### Verified
- `cargo test --workspace` green: 58 tests (app 7, cli 2, core 8, daemon 1, ecosystem 11, hippocampus 8, inference 21).
- Playwright 11/11 green (block5 5, block6 brain 1, block6 system 1, block7 1, block8 3).
- Live CLI: `health`, `studio`, `methods`, `studio role`, `rpc rpc.methods`, `session list` against the demo app and a temp daemon.
- Windows artifact: `dist/noema-0.1.0-windows-x64.zip` (8,173,956 bytes) from `packaging/build.ps1`.
- Driftwatch digest 2026-10-31: 8 GitHub platform updates; no Rust or Node dependency affected.

### Notes
- No new external dependency. Packaging uses the installed toolchain; NSIS installs on CI.
- `makensis` is absent on `hp-atr`, so the local artifact is the zip; CI installs NSIS and emits the setup executable.
- macOS and Linux artifacts are CI-defined; this Windows host cannot build or verify them. First CI run is the proof path.

## [0.5.2] - 2026-10-31 - Block 7: Studio, role models, BI widgets

### Added
- `GET/POST /api/studio`: restored session + 25-session list, projects, model catalogue, five role assignments (embedding, router, chat, coding, dreaming) with global and per-project overrides, widget layout and an aggregated usage feed.
- Studio overlay (`crates/app/ui/src/studio.ts`): session card and switcher, project list, role-model selects, 12-column widget grid with pointer drag/resize/remove, filter-to-widget form with custom names. Opens from the HUD `studio` button or `#studio`.
- `POST /api/studio` actions: `session`, `role`, `widget`, `widget_remove`, `layout`; every write returns the full snapshot.
- Schema v2 (`0002_studio.sql`): `usage_events.latency_ms`, `project_id`, `thinking`; demo seeds one project, five models and BI rows.
- Default widgets seeded when `widgets` is empty: tokens by agent, cost by model, latency by peak band, tokens by thinking.
- `docs/STUDIO.md` contract; Playwright `tests/block7-studio.spec.ts`.

### Changed
- HUD shows the restored session pill and a studio button. Boot stays on the 3D body; `#studio` opens the Studio route (decision recorded in `docs/AGENT_HANDOFF.md`).
- `migrations_apply_once` asserts the migration count from the list instead of a hard-coded 1.

### Verified
- `cargo check --workspace --all-targets` green; `cargo test -p noema-app -p noema-core` 7 + 7 + daemon 1 green.
- `npm run build` in `crates/app/ui` green; Playwright block7 1/1, block6 brain 1/1, block6 system 1/1 green.
- Evidence: `docs/evidence/block7-studio.webp`, `docs/evidence/block7-widget-filter.webp`.
- Driftwatch digest 2026-10-31: 8 upstream page changes; Node 26.13.0 LTS matches the host, no Rust dependency affected.

### Notes
- No new external dependency. Widget grid and charts are native DOM/CSS; persistence is SQLite.

## [0.5.1] - 2026-10-31 - Brain hardening: live telemetry, spatial hash, drag persistence

### Added
- `noema-inference::ActivationWindow`: folds normalised `StreamEvent` batches into logprob entropy (mean surprisal), thinking density and caller drift; 3 tests.
- `POST /api/brain/telemetry` + `GET /api/brain/telemetry`: Thalamus feed with a 30 s live TTL. `/api/brain` now carries top-level `telemetry` and applies live pulses to every node.
- `POST /api/brain/drag`: clamps to the unit sphere and writes `thoughts.pos_x/y/z`; stored coordinates win over projection on the next read.
- `brain.ts`: persists pointer-up position and polls telemetry every 1.5 s while the brain view is open.
- Spatial hashing above 200 nodes in Rust and the worker: uniform grid, 27-cell neighbourhood, `cell = cbrt(4.2/n)` clamped to 0.08..0.35; 240-node equivalence and 500-node tether tests.

### Changed
- `crates/app` depends on `noema-inference` to deserialize the normalised stream. Internal path dependency, no new external crate.

### Verified
- `cargo check --workspace --all-targets` green; `cargo test -p noema-inference -p noema-hippocampus -p noema-app` 21 + 8 + 6 green.
- `crates/app/ui` production build green; Playwright `tests/block6-brain.spec.ts` 1/1 green.
- HTTP smoke on `:4199`: telemetry entropy 0.5, density 0.333, live pulse 1.002 Hz; drag readback `[0.1, 0.2, 0.3]`.
- Driftwatch digest 2026-10-31: 8 upstream page changes, none on a Rust dependency.

## [0.5.0] - 2026-10-31 - Block 6: brain graph, physics, Dream Mode

### Added
- `noema-hippocampus`: force-directed step (Hooke springs, repulsion, damping, speed cap, hard 1.0 tether clamp including pinned drag targets), UMAP-style cosine kNN + SGD 3D projection anchored to five functional regions, and activation telemetry (logprob entropy, thinking density, cosine drift -> pulse hz/intensity/hue). 6 unit tests.
- `GET /api/brain`: projected nodes, cosine kNN edges, per-node pulse and region anchors. `POST /api/dream`: persisted dream runs in `dreams`.
- `brain.ts` + `brain-worker.ts`: instanced SDF node glyphs (analytic disc via `fwidth`), vector edges, SVG label overlay with overlap culling, node drag/pin, click card (title, region, agent, timestamp, importance, body, links), physics off the main thread at 60 fps.
- Wake/dream switch: dream relaxation params, hue shift, persisted run; demo seed now writes 21 thoughts across the five regions.

### Verified
- `cargo test -p noema-hippocampus` 6/6; `cargo test -p noema-app` 4/4; `cargo check --workspace --all-targets` green.
- Playwright `tests/block6-brain.spec.ts` green; evidence `docs/evidence/block6-brain-zoom.webp`, `block6-brain-far.webp`, `block6-brain-dream.webp`, `block6-brain-thought.webp`.
- Driftwatch digest 2026-10-31: 8 upstream page changes, none on a Rust dependency.

### Notes
- No new crate dependency: the projection implements the UMAP core (cosine kNN attraction + repulsion + region anchor) in-repo, documented in `docs/BRAIN.md`. The live logprob feed stays with Thalamus; the telemetry math and API are ready.

## [0.4.0] - 2026-10-31 - Block 5.5: hardware awareness + Linux target

### Added
- `crates/core/src/hardware.rs`: `HardwareProfile` (CPU brand, physical/logical cores, RAM, GPU/vendor/kind/VRAM/driver, data-volume free/total, hostname, OS/arch) with native probes: Windows CIM (one PowerShell call), macOS `sysctl` + `system_profiler`, Linux `/proc` + `nvidia-smi`/`lspci`/sysfs. No new crate, no network, no telemetry.
- `derive_tuning`: quality tiers ultra/high/balanced/low driving pixel-ratio cap, particle budget, shadow/texture/LOD budgets; inference context, GPU layers, parallelism, VRAM budget and recommended runtime; daemon worker threads and SQLite cache.
- Deep-merge overrides stored in `settings('hardware.overrides')` (scope `hardware`), validated with `deny_unknown_fields`; `null` deletes a key; 12 h profile cache with explicit refresh.
- Surfaces: `noema system profile|override|reset` (offline local-store fallback), RPC `system.profile|override|reset`, HTTP `GET /api/system`, `GET /api/system/refresh`, `POST /api/system`.
- Studio "Machine" panel: live profile, quality preset, pixel-ratio cap, particle budget, apply/re-detect; renderer applies the cap and `Circulation.setBudget` disposes/rebuilds the six flows.
- `crates/core/src/platform.rs`: per-OS data root and secret backend. Linux is now a v1 target with XDG paths and Secret Service, packaging targets (AppImage/deb/rpm) and CI matrix documented in `docs/PLATFORMS.md`.

### Changed
- `daemon::data_dir` delegates to `platform::data_dir`: macOS now uses `~/Library/Application Support/noema` (previously `~/.noema`); Linux uses `$XDG_DATA_HOME/noema` or `~/.local/share/noema`.
- `docs/ARCHITECTURE.md` §3/§4/§8/§11 document the IPC methods, config paths, packaging matrix and the full hardware flow.
- `docs/MASTER_PLAN.md`: Block 5.5 checklist, machine-awareness pillar, Linux packaging/CI, decisions and risks.
- `README.md` platform matrix and hardware quickstart.

### Verified
- `cargo check --workspace --all-targets` green; `cargo test -p noema-core` 7/7 (quality tiers, override merge/delete, typo rejection, settings round-trip, plus existing store/vector/daemon tests).
- Live probe on `hp-atr`: i5-9400F (6/6), 24,456 MB, GTX 1650 (4095 MB) → quality `high`, pixel cap 1.75, 2500 particles, context 8192, runtime `ollama`.
- `GET /api/system` and `POST /api/system` verified against `noema-app serve`; `noema system profile` verified with the daemon offline.
- Playwright `tests/block6-system.spec.ts` green; screenshot `docs/evidence/block6-hardware.webp`.
- Driftwatch digest read: 8 upstream page changes (Gemini, DeepSeek, Mistral, Graph, Gmail, CircleCI, Buildkite, Asana); no Rust dependency affected.

### Notes
- Windows CIM `AdapterRAM` caps reported VRAM at 4095 MB; cards with more than 4 GiB land in `high` until the 64-bit registry value is read. Apple Silicon reports 75% of unified RAM.
- No physical Linux host has run the suite yet; Linux cfg branches compile on Windows and the first Linux CI run is the proof.
- No physical GPU was used for macOS/Linux validation; both paths are documented but unexecuted on this host.

## [0.3.1] - 2026-10-31 - Real Z-Anatomy écorché, PBR tinting, heart focus

### Added
- Real anatomy atlas `assets/anatomy/atlas/body_full.glb` (Z-Anatomy via `dohunjeon51-design/web-anatomy-3d`, CC BY-SA 4.0): 826 named structures, 291k triangles, 1.72 m frame.
- `crates/app/ui/src/atlas.ts`: two-pass reparenting into seven layers, per-structure PBR tinting, automatic framing, heart detection, opacity control.
- Heart focus: `zoomToCellular` hides muscles, fades the skeleton, and flies to the cardiac anchor; the procedural heart and circulation particles stay visible in the thorax.
- Circulation particle systems (arterial red, venous blue) with additive sprites.

### Changed
- Default view is a full-body écorché (muscles + skeleton) instead of the procedural mannequin; the procedural body remains the offline fallback.
- Post-processing removed: direct ACES-tonemapped render measures 60.2 fps at 1600×900 (ANGLE/D3D11, GTX 1650).
- `docs/ARCHITECTURE.md` §6 now documents the real atlas pipeline; asset licence manifest rewritten with CC BY-SA attribution.

### Verified
- Playwright 5/5 green; screenshots `docs/evidence/block5-{atlas,agent-rail,anima,cellular,heart-close,60fps}.png`.
- 60.2 fps measured; 826 meshes; zero external requests; Anima edit persists across reload.

### Notes
- The Z-Anatomy "processed" per-system GLBs were tried and discarded: no albedo maps (only metallic/roughness) and duplicated body geometry at 4–24M triangles.
- The atlas is an écorché (muscles/bones); internal organs remain a future layer until a free atlas with albedo textures is available.
## [0.3.0] - 2026-10-31 - Block 5: the 3D body, L'Anima, agents in motion

### Added
- `crates/app/ui` (Vite 8.3.5 + three 0.187): procedural human body with six layers (skin, skeleton, nerves, vessels, organs, agents), per-layer opacity, distance-based skin fade, orbit/pan and 0.03–8.0 unit zoom with eased focus tween.
- Heart organ ("L'Anima"): pulsing `heart.glb` with procedural fallback; click opens identity + system-prompt editor persisted in `settings` (`scope='anima'`) through `POST /api/anima`.
- Agent swarm: seven deterministic Catmull-Rom splines, additive luminous sprites, GPU ID-buffer picking (bit-encoded channels), freeze + right-rail dock, greeting composed in Rust from DB rows, five inspection tabs.
- `noema-app serve`: loopback HTTP bridge (static UI, `/api/health`, `/api/anatomy`, `/api/anima`), path-traversal guard, demo fixture database.
- `tools/anatomy/build_glb.mjs`: procedural `heart.glb` / `brain.glb` (1,617 verts each, index-safe); `assets/anatomy/LICENSES.md` (CC0).
- CAD smoke: SolidCraft 0.28.0 `eval` + `snapshot` + STEP export on a vertebra part; CADCraft 0.15.1 `info` + `convert` on a native drawing (DXF/PDF/SVG).
- Playwright suite (5 tests) with screenshots in `docs/evidence/block5-*.png`; 60.1 fps measured, zero external requests, zero requests on agent click.

### Changed
- `noema-app` deps: `rusqlite` (workspace) + `serde_json`; no HTTP client, so greetings cannot call an LLM.
- `docs/ARCHITECTURE.md` §6 rewritten around the implemented renderer; §10 records the CAD smoke.
- Tauri/WebView2 wrapper is deferred to Block 8 packaging: the same bundle is served by `noema-app` today.

### Verified
- `cargo check --workspace` green; `cargo test -p noema-app` 3/3.
- `npx playwright test` 5/5 green; measured 60.2 fps at 1600×900 on ANGLE/D3D11 (NVIDIA GTX 1650).
- Greeting test: boot payload text equals the rail, and clicking an agent issues zero network requests.
- driftwatch digest 2026-10-31: 8 page-change feeds (Railway, Render, Netlify, Brevo, Scaleway, DeepSeek, Mistral, Gemini pricing). No Rust dependency affected; pricing feeds belong to Thalamus rate cards.

### Notes
- `gltf-validator` on npm only publishes dev builds (`2.0.0-dev.x`), so no prerelease was installed. GLB validity is enforced by the generator's structural checks and by `GLTFLoader` parsing both files in the e2e boot test.
- CADCraft 0.15.1 cannot open STEP or PNG; the drafting smoke therefore uses a native DXF drawn by CADCraft, while SolidCraft owns the STEP path.

## [0.2.9] - 2026-10-31 - Block 4: ecosystem, marketplace, MCP supervisor, GitHub plugin

### Added
- `noema-ecosystem::manifest`: validated `mcp | skill | plugin` manifests, field-scoped errors, draft 2020-12 JSON Schema per kind, Claude Code `SKILL.md` frontmatter parser.
- `noema-ecosystem::mcp`: supervisor for stdio / SSE / WebSocket servers, Bearer + OAuth2 + custom-header + mTLS auth, PKCE helpers (RFC 7636 vector), timeout, exponential retry (transport only), per-server log ring.
- `noema-ecosystem::marketplace`: bundled `memory-mcp` (Node `node:sqlite`, read-only thoughts/edges/dreams), `system-visual-qa` skill (Playwright CLI), `github-plugin`; validated installs and `installed.json`.
- `noema-ecosystem::github`: device-flow + PAT auth, `user/repo` search, branches, system-git checkout/diff/status, `RepoPin` (repo + branch + session) persisted under `.noema/pin.json`.
- Tests: 11 integration tests (`crates/ecosystem/tests/block4.rs`) covering validation, bundled installs, stdio lifecycle against the bundled Node server, WS and SSE round-trips, git flow, pins and PKCE.

### Changed
- Workspace deps: `tokio` gains `process`; added `tokio-tungstenite` 0.30 (WS client, rustls webpki roots), `futures-util` 0.3, `sha2` 0.10, `base64` 0.23.
- `docs/ARCHITECTURE.md` §5 rewritten: manifest fields, transports, auth, supervisor semantics, marketplace bundles, GitHub plugin.

### Verified
- `cargo check --workspace --all-targets` green.
- `cargo clippy --workspace --all-targets -- -D warnings` green.
- `cargo test -p noema-ecosystem` green: 11/11.
- driftwatch digest 2026-10-31: 8 page-change feeds, no Rust dependency affected.

### Notes
- Real-server proof uses the bundled `marketplace/memory-mcp/server.mjs` (Node 26). SSE/WS proven against loopback servers.
- GitHub network calls are not exercised in tests (no token in CI); parsing, git argv and pin I/O are covered.
- No `git push` performed.

## [0.2.8] - 2026-10-31 - Toolchain: SolidCraft CLI + CADCraft 0.15.1

### Added
- Toolchain docs: SolidCraft 0.28.0 (Tangency Labs, `solidcraft-cli.exe`) and CADCraft 0.15.1 (`cadcraft-cli.exe`) recorded in `AGENTS.md` rule 16, `docs/MASTER_PLAN.md` §2 / §2.1 / §2.2, `docs/ARCHITECTURE.md` §10 and `agents/noema-soma.md`.
- §2.2 installer runbook for the CADCraft 1603 failure (LaunchCondition `Installed OR ALLUSERS <> 1 OR NOT CRAFT_USER_DIR`, launcher `ALLUSERS=1` vs package `CRAFT_SCOPE=user`, legacy `cadcraft.exe`).

### Verified
- `solidcraft-cli --version` → 0.28.0; `cadcraft-cli --version` → `cadcraft-cli 0.15.1 (660e15c9...)`.
- CADCraft MSI product `{A7F703E8-5986-48D5-B127-737D92ED94DD}` reports DisplayVersion 0.15.1 at `F:\Programmi\ArtCraft\CADCraft`; 0.13.0 product `{FF850FBE-...}` gone.
- driftwatch digest: 8 entries (Prometheus Alertmanager security, page-change feeds); no dependency in this repo affected.

### Notes
- CADCraft 0.13.0 uninstalled; 0.15.1 installed per-machine. Launcher installs need owner-present UAC consent.
- Docs-only change; no Rust source touched.
- No `git push` performed.

## [0.2.7] - 2026-10-31 - Block 3: universal inference, embeddings, FinOps (Thalamus)

### Added
- `crates/inference/src/request.rs`: one `Request` + `Message` + full `Sampling` surface (top_k, min_p, typical_p, repeat_penalty, seed, stop, logprobs, reasoning effort/budget, template vars, extra payload, headers) and the normalised `StreamEvent` (delta, reasoning, logprob, usage, done).
- `crates/inference/src/adapter.rs`: ten standards - OpenAI Chat, OpenAI Responses, Anthropic Messages, Gemini, DeepSeek, Ollama, llama.cpp, vLLM, LM Studio, custom HTTP. `encode` returns `HttpCall`; `decode_chunk` normalises SSE and NDJSON, including DeepSeek `reasoning_content`, Anthropic thinking/cache tokens, Gemini thought parts, Responses events and Ollama eval counts.
- `crates/inference/src/cost.rs`: `RateCard` with per-million token prices, per-call fee, IANA peak/off-peak windows (weekday mask, overnight wrap, DST) and local hardware amortisation.
- `crates/inference/src/embed.rs`: batch embedding encode/decode for OpenAI-compatible, Gemini and Ollama endpoints; cosine re-export.
- `crates/inference/src/cluster.rs`: LLM-structured `Extraction` -> `ThoughtGraph`; importance, concept vectors, embedding-modulated edge weights, deterministic single-link clusters and duplicate-edge collapsing.
- 18 unit tests: adapter fixtures per provider, hand-computed costs (35.177 USD fixture, peak/DST, overnight window, local amortisation), embedding order, clustering weights/clusters.

### Verified
- `cargo check --workspace --all-targets` green.
- `cargo test --workspace`: inference 18/18, core 3/3, daemon integration 1/1.
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- driftwatch digest + check read: 8 Slack API deprecation entries, no Slack dependency in this repo; no Rust dependency affected.

### Notes
- New dependencies: `chrono` 0.4.45 and `chrono-tz` 0.10.4 for IANA timezone windows (rationale: peak/off-peak pricing must follow DST).
- Transport (actual HTTP stream + cancellation) lands with `chat.send`; adapters stop at the wire contract.
- Block 3 exceeds the 400-LOC default because the plan packs four deliverables into one block; split across three internal commits is not possible without breaking adapter/cost/cluster coherence.
- No `git push` performed.
## [0.2.6] - 2026-10-31 - Block 2.1 core hardening (Axon)

### Changed
- `crates/core/src/rpc.rs`: `session.open` validates before saving — uuid session id, agent/model 1-128 chars of `[A-Za-z0-9._:/@-]`, absolute `cwd`, existing `project_id` (→ `-32602`).
- `crates/core/src/daemon.rs`: 8 MiB NDJSON frame cap enforced during read (`MAX_FRAME`, `read_frame`); oversize frame → `-32700` then connection close. `daemon.json` gets an owner-only Windows ACL via `icacls /inheritance:r /grant:r <user>:F` (best-effort, warning on failure). Each connection starts with one `event.health` frame.
- `crates/core/src/store.rs`: `project_exists` pre-check for the RPC layer.
- `crates/core/src/vector.rs`: `chunks_exact(4)` → `as_chunks::<4>()`, clippy clean.
- `crates/cli/src/main.rs`: reply matching skips notification frames (required by the connect-time health frame).
- `crates/core/tests/daemon.rs`: new integration test — auth `-32001`, `daemon.ping`, `event.health`, `session.open` validation, `event.session.updated`, oversize frame `-32700`.

### Decision
- Agent/model strings are validated by format, not by catalog FK: no RPC creates `agents`/`models` rows yet, so a hard FK would make `session.open` unusable on a fresh DB. The `project_id` FK (the only one in the schema) is enforced now; `agent_id`/`model_id` get FK checks when catalog CRUD lands in Block 3.

### Verified
- `cargo check --workspace --all-targets` green; `cargo test --workspace` green (core 3/3 + daemon 1/1); `cargo clippy --workspace --all-targets` no warnings.
- Live: `noema rpc daemon.ping`, `noema status`, invalid `session.open` → `-32602`; `icacls` on a daemon-created `daemon.json` shows only the owner ACE.
- driftwatch digest read (Node 26.12.1 security release, GitLab PHP/Pillow, Slack deprecations): no Rust dependency in use affected.


## [0.2.5] - 2026-10-31 - Review dispatch map (Cortex)

### Changed
- `AGENTS.md`: new rule 14 "review findings are owned"; driftwatch becomes rule 15.
- `docs/MASTER_PLAN.md`: Block 2.1 "Core hardening from Block 2 review" assigned to **Axon**, checklist 2.1.1-2.1.5; status header updated.
- `docs/AGENT_HANDOFF.md`: dispatch map finding -> agent -> file -> checklist id; dispatch order Axon 2.1 before Thalamus Block 3.

### Notes
- All five code findings live in `crates/core`: only **Axon** is needed. Finding 6 (Block 2 LOC over budget) needs no agent.
- No code change in this commit.

## [0.2.4] - 2026-10-31 - Block 2 + tooling review (Cortex)

### Reviewed
- `39adfea` Block 2 (store, migrations, session.resume, event bus, noemad, CLI).
- Tooling round: `ce100a2` policy, `d7f9049` toolchain pin, `69ec4b2` MCP/skills, `036f22f` free CAD/3D plan.

### Verdict
- PASS. Scope discipline, docs/checklist/changelog consistency; no push, no secrets.

### Verified
- `cargo check --workspace` green; `cargo test --workspace` green (`noema-core` 3/3); `cargo clippy --workspace --all-targets` 1 non-blocking warning (`chunks_exact_to_as_chunks`, `crates/core/src/vector.rs:9`).
- `rust-toolchain.toml` channel `stable`, rustc 1.99.1; `.mcp.json` valid; playwright/driftwatch/typeui paths exist; 7/7 charter pairs hash-match.
- driftwatch digest read: Slack/Prometheus platform entries only; no NOEMA dependency affected. `driftwatch check` hung on network, killed; no dependency change pending.

### Changed
- `docs/ARCHITECTURE.md` §1: core public surface now lists `daemon` and `vector`.
- `docs/ARCHITECTURE.md` §3: 8 MiB frame cap marked as enforced from Block 3.
- `docs/AGENT_HANDOFF.md`: review verdict, findings and the Block 3 brief for Thalamus.

### Notes
- Block 2 = 1422 insertions, above the 400 LOC default; accepted as a planned multi-part block.
- Non-blocking follow-ups: session id validation, Windows daemon.json ACL, daemon integration test, clippy lint.

## [0.2.3] - 2026-10-31 - Free CAD/3D toolchain plan + 3D/vision skills

### Added
- Global skills: `threejs`, `fec-canvas-threejs`, `blender-bpy-enhanced`, `skill-freecad`, `openscad-automation`, `meshy-3d-agent`, `vision-helper`, `image-generation` (11 global skills total).
- `docs/MASTER_PLAN.md` §2.1: free-only asset/CAD toolchain (CADCraft primary; FreeCAD, CadQuery, OpenSCAD, Blender, Meshy free tier, trimesh/manifold3d, pymeshlab, glTF-Transform, glTF-Validator, F3D, ImageMagick, KTX, Gmsh) with install and verify commands, pipeline, free asset sources and gates.
- `docs/ARCHITECTURE.md` §10: asset pipeline and licensing rule.
- Block 5 exit criteria: GLB validator clean + asset license manifest.
- `AGENTS.md` §5: full skill table.

### Verified
- All 11 skills present under `~/.openclaw/skills`; `openclaw skills list --agent noema-axon --json` resolves them.
- CADCraft, FreeCAD, OpenSCAD, Blender, MeshLab, Assimp, glTF-Transform absent from PATH on 2026-10-31; winget has F3D only. Plan records install paths.

### Notes
- Zero-cost constraint recorded as a decision: no paid software, assets or APIs.

## [0.2.2] - 2026-10-31 - MCP registry, design skills, rustup

### Added
- OpenClaw MCP servers: `driftwatch` (probe ok), `context7` (probe ok), `chrome-devtools` (doctor ok). All agents can use them.
- Global skills: `design` (@ivangdavila/design 1.0.3), `frontend-design` (@javotillo/frontend-design 1.0.0), `typeui-design` (local wrapper for the TypeUI CLI).
- `typeui.sh` 0.7.1 installed globally (npm).
- `AGENTS.md` §5 tooling inventory (MCP + skills + TypeUI usage).

### Changed
- `rustup` 1.29.1 -> 1.29.2 (toolchain remains stable 1.99.1).

### Verified
- `openclaw mcp doctor driftwatch --probe` ok, `context7 --probe` ok, `chrome-devtools` ok.
- `openclaw mcp list`: chrome-devtools, context7, driftwatch.
- `openclaw skills list` shows design, frontend-design, typeui-design.
- `typeui.sh --help` responds.

## [0.2.1] - 2026-10-31 - Toolchain & dependency currency policy

### Added
- `AGENTS.md` rules 13-14: latest stable only for every package/language/library/module/tool; driftwatch mandatory at every block start and before any dependency change.
- `rust-toolchain.toml` pinning channel `stable` with `rustfmt` + `clippy` (see previous commit).
- `docs/ARCHITECTURE.md` §9: version and dependency policy with the driftwatch CLI/MCP workflow.
- `.mcp.json`: `driftwatch` MCP server (`F:\tools\driftwatch\mcp\server.js`).
- Same two rules in all 7 repo charters (`agents/*.md`) and all 7 installed workspaces (`~/.openclaw/workspaces/noema-*/AGENTS.md`).

### Verified
- `node F:\tools\driftwatch\bin\driftwatch digest --project F:\Progetti\agentic-anatomy-workspace --max 5` returns entries; current hits are Slack/Prometheus platforms, no NOEMA dependency affected.
- `.mcp.json` parses; servers `playwright-webgpu,driftwatch`.
- All 14 charter copies hash-match after sync.

## [0.2.0] - 2026-10-31 - Block 2: Local DB, session continuity, daemon

### Added
- `crates/core/migrations/0001_init.sql`: full v1 DDL (projects, sessions, messages, providers, models, rate cards/windows, usage, thoughts/edges, dreams, agents, widgets, MCP/skills/plugins, repos, settings).
- `core::store::Store`: WAL + `foreign_keys`, append-only versioned migrations in `schema_migrations`, session save/load/list, `resume()` with last cwd/repo/branch, widget layout, role models and live `git diff --stat`.
- `core::store::nearest`: brute-force cosine over `thoughts.embedding`; optional `sqlite-vec` probe via `NOEMA_SQLITE_VEC` (falls back, never fails to boot).
- `core::vector`: f32 BLOB codec + cosine.
- `core::event`: 1024-capacity broadcast bus with NDJSON notification frames.
- `core::rpc`: JSON-RPC 2.0 request/response, `daemon.ping`, `session.resume`, `session.list`, `session.open`, error codes incl. `-32001 auth`.
- `core::daemon` + `noemad` binary: loopback ephemeral port, 32-byte token port file (`daemon.json`), constant-time auth, per-connection event stream, 15 s heartbeat, port-file cleanup on ctrl-c.
- `noema status` now reads the daemon (local-store fallback when offline); `noema rpc <method> [json]` exposes every method headlessly.

### Changed
- `Session` gains `widget_layout`; `ResumeState` gains `widget_layout`, `role_models`, `diff_stat`.
- `rusqlite` feature `load_extension` (sqlite-vec) and tokio feature `signal` (graceful shutdown). No new crates.

### Verified
- `cargo check --workspace` green; `cargo test -p noema-core` 3/3 (migration idempotence, save+resume roundtrip, cosine/blob codec).
- Live: `noemad` migrated v1, wrote `daemon.json`, answered `daemon.ping`; `noema rpc session.open` then `noema status` returned agent/repo/branch/widget_layout plus `diff_stat` `7 files changed, 514 insertions(+), 29 deletions(-)`; event stream delivered `event.health`; unauthenticated request rejected with `-32001`.

### Notes
- `thoughts_vec` stays uncreated until Block 3 writes embeddings. No `git push` performed.


## [0.1.0] — 2026-10-31 — Block 1: Foundations

### Added
- Project baptized **NOEMA**; README with pillars, stack, team and governance.
- Cargo workspace (Rust 2024, resolver 3): `noema-core`, `noema-inference`, `noema-ecosystem`, `noema-cli` (`noema` bin), `noema-app`.
- Core primitives: typed ids, `Session`, `ResumeState`, `SessionStore` port.
- Inference value types: `Standard`, `Usage`, `RateCard` with per-1M-token cost math.
- Ecosystem manifest types: `Kind`, `Transport`, `Manifest::validate`.
- CLI skeleton with `status` / `help`.
- `docs/MASTER_PLAN.md` (architecture, block plan, master checklist), `docs/ARCHITECTURE.md` (DDL, RPC, manifests, pricing), `docs/AGENT_HANDOFF.md`.
- Repo hygiene: `.gitignore` (Rust/Node/3D/OS/secrets), MIT `LICENSE`, `AGENTS.md` iron rules, `.cargo/config.toml` target dir on `F:`.
- 7 project agents registered in OpenClaw with caveman charters.

### Verified
- `cargo check --workspace` — green (18.9 s, 28 crates locked).
- 7/7 agent ids resolve; `agents.ownership` unchanged (`explicit`); charters present in every workspace and versioned in `agents/*.md`.
- Toolchain audit: rustc/cargo 1.99.1, cargo-tauri 2.12.4, trunk 0.22.1, git 2.55, node 26.13, Playwright 1.64.1 (npx). Blender/CADCraft absent (procedural fallback).

### Notes
- No `git push` performed. Owner reviews and pushes `main`.
