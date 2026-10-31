# NOEMA - AGENT HANDOFF

> Rewrite at the end of every block. Newest state first.

## Current state

- **Universal Model Catalog & 2026 Rate Cards (Thalamus - 2026-10-31):**
  - Updated catalog in `crates/inference/src/cost.rs` to track October 2026 stable flagship models:
    - Google Gemini: Gemini 3.8 Flash, 3.5 Flash-Lite, 3.1 Pro (legacy 2.5 Flash / Pro retained).
    - OpenAI: GPT-4o, GPT-4o mini, o1, o3-mini.
    - Anthropic: Claude 3.7 Sonnet, Claude 3.5 Sonnet, Claude 3.5 Haiku, Claude 3 Opus.
    - DeepSeek: DeepSeek-V3, DeepSeek-R1.
    - Mistral: Mistral Large, Mistral Small, Codestral.
    - Local: Llama 3.3 70B, Qwen 2.5 Coder 32B via zero-token card with local hardware amortisation.
  - Exported `ModelCatalogEntry` and `default_catalog()` from `crates/inference`.
  - All 25 unit tests in `noema-inference` and 63 workspace tests green.
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
