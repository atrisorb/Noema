# NOEMA - AGENT HANDOFF

> Rewrite at the end of every block. Newest state first.

## Current state

- **Exhaustive 2026 Model Catalog & Multi-Provider Architecture (Thalamus - 2026-10-31):**
  - Implemented exhaustive October 2026 frontier model registry across 10 providers in `crates/inference/src/cost.rs`:
    - **Google Gemini**: `gemini-4-argon` (unified 1M window, $2.00/$10.00, $0.10 implicit 95% cache discount, default thinking 32k), `gemini-3.8-flash` ($0.75/$3.75), `gemini-3.5-flash-lite` ($0.30/$2.50), `gemini-3.1-pro` ($2.00/$12.00), `gemini-3.8-live` ($1.00/$4.00 native audio/video), `gemini-2.5-flash`, `gemini-2.5-pro`.
    - **OpenAI**: `gpt-6-astra` ($10.00/$50.00), `gpt-5.6-sol` ($5.00/$30.00), `gpt-5.6-terra` ($2.50/$15.00), `gpt-5-mini` ($0.20/$0.80), `o3` ($2.00/$8.00), `o3-pro` ($20.00/$80.00), `o3-mini` ($1.10/$4.40), `o4-mini` ($1.10/$4.40), `o1` ($15.00/$60.00), `gpt-4o`, `gpt-4o-mini`.
    - **Anthropic**: `claude-sonnet-5-5` ($2.00/$10.00), `claude-opus-5-5` ($4.00/$20.00), `claude-haiku-5-5` ($0.10/$0.50), `claude-3-7-sonnet`, `claude-3-5-sonnet`, `claude-3-5-haiku`, `claude-3-opus`.
    - **Mistral AI**: `mistral-large-4` (Le Chonk 1.05T MoE, 1M context, $1.36/$4.18, $0.14 cache read), `mistral-large-2411`, `mistral-small-2501`, `codestral-2501`.
    - **DeepSeek**: `deepseek-flash` (V4.1-Flash, peak-aware $0.15-$0.30 in / $0.60-$1.20 out / $0.003-$0.006 cache), `deepseek-reasoner` (R1), `deepseek-chat` (V3).
    - **Zhipu AI (GLM)**: `glm-5.3` (1M context, $1.40/$4.40), `glm-5.3-flash` ($0.15/$0.50), `glm-5.3-flashx` ($0.37/$1.25), `glm-4.7-flash` (free tier).
    - **Moonshot AI (Kimi)**: `kimi-k3.1-pro` (1M context, $2.00/$10.00, $0.20 cache read), `kimi-k3.1-pro-highspeed` ($4.00/$20.00), `kimi-k3` ($3.00/$15.00).
    - **Meta Llama**: `llama-3.3-70b-instruct`, `llama-3.1-405b-instruct`.
    - **Alibaba Qwen**: `qwen-2.5-coder-32b`.
    - **Local runtimes**: zero-token cost for Ollama / vLLM / llama.cpp / LM Studio with amortised hardware and kWh calculation.
  - Comprehensive parameters on `ModelCatalogEntry`: context window, max output tokens, thinking support, default thinking budget, vision, audio, tools, prompt caching, and wire protocol.
  - Query helpers: `default_catalog()`, `catalog_entry()`, `all_providers()`, `models_by_provider()`, and `RateCard::for_model()`.
  - Added `Standard::Moonshot`, `Standard::Zhipu`, `Standard::Mistral` to wire codec protocol in `crates/inference/src/lib.rs` and `crates/inference/src/adapter.rs`.
  - All 24 unit tests in `noema-inference` and 63 workspace tests green.
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
