# NOEMA - AGENT HANDOFF

> Rewrite at the end of every block. Newest state first.

## Current state

- **Release v0.5.19: Native GitHub CLI Release Publisher & Linux AppRun Fix (Axon / Cortex - 2026-10-31):**
  - Resolved Linux AppImage build error: added `$APPDIR/AppRun` wrapper resolving paths and executing `noema-app`, allowing `appimagetool` to bundle cleanly without errors.
  - Replaced third-party `softprops/action-gh-release@v2` marketplace action with native GitHub CLI (`gh release` via pwsh and bash), completely eliminating Node.js 20 deprecation annotations on GitHub Actions.
  - Bumped `tauri-cli` to `2.12.5` to match `tauri = 2.12.5` in `src-tauri`.
  - Bumped release version to `0.5.19` across `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json`.

- **Release v0.5.18: Universal Cross-Platform Packaging & CI Warning Cleanup (Axon / Cortex - 2026-10-31):**
  - Resolved macOS packaging failure: staged architecture-specific slices (`aarch64` and `x86_64`) alongside universal binary in `packaging/tauri.sh`, satisfying Tauri v2 universal bundle resource validation.
  - Resolved Linux packaging failure: created missing `$APPDIR/usr/share/noema` in `packaging/build.sh` prior to copying assets.
  - Hardened Linux CI runner: configured `APPIMAGE_EXTRACT_AND_RUN: "1"` and added `libfuse2t64` / `libfuse2` fallback to `.github/workflows/release.yml`.
  - Cleaned all compiler & bundler warnings: eliminated unread variables and dead helper functions in `crates/core/src/hardware.rs`; adjusted Vite chunk size warning threshold.
  - Bumped version to `0.5.18` across `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json`.

- **Release v0.5.17: Platform Icon Builder Fallback & Robustness (Axon / Cortex - 2026-10-31):**
  - Resolved `node tools/design/build-icons.mjs` failure on CI runners: `tools/design/build-icons.mjs` now detects if `cargo-tauri` is present, and gracefully falls back to `npx @tauri-apps/cli@2.12.4 icon` if `cargo-tauri` has not yet been compiled, resolving exit codes 101/1 across Linux, macOS, and Windows.
  - Bumped version to `0.5.17` across `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json`.
  - Workspace checks and icon generation verified.

- **Release v0.5.16: Multi-Platform CI / Packaging & GitHub Release Pipeline (Axon / Cortex - 2026-10-31):**
  - Resolved GitHub Actions release runner failures:
    - `macos-14` brownout retirement: upgraded runner to `macos-15`.
    - Linux target compilation error E0599: fixed `.filter()` method resolution on `std::io::Result` in `crates/core/src/hardware.rs` by inserting `.ok()`.
    - Windows clean-checkout test failure: added `node tools/design/build-icons.mjs` step before `cargo test --workspace` so Tauri's Windows resource compiler (`tauri-build`) finds `icon.ico`.
    - Release asset publishing: elevated workflow permissions to `contents: write` and wired `softprops/action-gh-release@v2` on tag triggers to upload generated bundles (`.exe`, `.msi`, `.dmg`, `.AppImage`, `.deb`, `.rpm`, `.zip`, `.tar.gz`) directly to GitHub Releases.
  - Bumped version to `0.5.16` across `Cargo.toml`, `Cargo.lock`, and `src-tauri/tauri.conf.json` so build artifacts (`noema-0.5.16-*`) match git tag `v0.5.16`.
  - Full workspace check and 64 tests passing green.

- **Exhaustive 2026 Model Catalog & Multi-Provider Architecture (Thalamus - 2026-10-31):**
  - Implemented exhaustive October 2026 frontier model registry across all 10 providers (45+ active and deprecated models mapped) in `crates/inference/src/cost.rs`:
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
  - All 26 unit tests in `noema-inference` and 64 workspace tests green.
- **Block 10 FULLY CLOSED (2026-10-31):** Blocks 10.1 - 10.10 complete.
  - **10.8 closed:** Atlas classification calibrated for all 826 meshes (0 misclassified meshes). Musculature (calves, feet, arms, face) and skeletal system properly categorized, tinted and calibrated (muscle roughness .44, bone .52, clearcoat .04/.12). Neutral pelvis audited and confirmed (0 external-genitalia meshes). Soft contact occlusion ground shadow plane at y = -0.86 added (`crates/app/ui/src/surface.ts`). Procedural capsule hack rejected; full-body skin mesh assigned to Soma 3D topology pipeline.
  - **Security S1 closed (Axon):** Remote IPC in `src-tauri/capabilities/main.json` narrowed to exact loopback ports (4173, 5173, 4199). Strict CSP set in `src-tauri/tauri.conf.json` with no remote origins and `withGlobalTauri: false`. Packaging scripts use generated SVG platform icons without raster source commits.
  - **FinOps & Live Usage Writer (Thalamus):** Driftwatch digest verified; rate cards implemented for DeepSeek (V3, R1), Mistral (Large, Small), and Gemini (2.5 Flash, 2.5 Pro) with `RateCard::for_model()`. Live usage writer (`UsageRecord`, `record_usage`, `recent_usage`) implemented and unit tested in `crates/core/src/store.rs`.
  - **Brain Full-Viewport Route (Retina):** Shipped `crates/app/ui/src/brain-route.ts` with `#brain` hash route, Escape dismissal, reduced motion support, non-breaking coexistence with `#studio`, and Playwright E2E test `block10-brain-route.spec.ts` passing.
  - **10.9 complete:** 28 tracked PNG converted to WebP (1.22 MB), references updated, asset audit 0/0.
  - **10.1-10.7 complete:** audit, tokens v1.1, SVG logo suite, eight mascots, Vitreous shell, artifact workspace, dashboard/chat/templates.
- **Agent Workshop Skills:** 5 skills authored in OpenClaw (`harden-desktop-loopback-ipc`, `model-pricing-and-usage-tracking`, `full-viewport-hash-routing`, `anatomical-material-and-contact-shadow`, `multi-agent-block-closure-qa`).
- **OpenClaw Memory:** Global and agent workspace memories synchronized.
- **Design law:** `NOEMA_DESIGN_SYSTEM.md` v1.1; `AUDIT.md`, `ARTIFACT_WORKSPACE.md`, `ANATOMY_AND_ASSETS.md`, `DECISIONS.md`.
- **No `git push`:** Working tree clean for owner review and push to `main`.

## Verified this run

- `cargo test --workspace`: green (all 60+ unit and integration tests passing across all crates).
- `cargo check --workspace`: green (0 errors, 0 warnings).
- `npm run build`: green (Vite build clean in ~410ms; CSS 36.70 KB raw / 7.08 KB gzip; JS 751.63 KB raw / 194.21 KB gzip).
- `npx playwright test tests/block10-brain-route.spec.ts`: green (1/1 passed in 10.7s).
- `cargo tauri info`: green (validates strict CSP and capability configuration).
- Tokens `--check`: 29 color pairs, 12 type scales; contrast 24/24 AA.
- Brand `--check`: favicon parity verified.
- Asset audit: 0 forbidden files, 0 product-reference violations.
- Pelvic audit: 1,872 nodes / 826 meshes; 12 flagged names (all pubic bones); 0 external-genitalia meshes.
- Evidence: `docs/evidence/design-*.webp`, `design-*.json`, `design-pelvis-audit.json`, `design-anatomy.json`.
- Demo server: verified on port 4199; WebP captures taken and validated.
- No `git push`.

## Key commands

```powershell
node tools/design/build-tokens.mjs --check
node tools/design/check-contrast.mjs
node tools/design/sync-brand.mjs --check
node tools/design/build-icons.mjs
node tools/design/asset-audit.mjs
npm --prefix crates/app/ui run build
cargo test --workspace
cargo check --workspace
```
