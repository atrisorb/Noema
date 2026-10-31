# NOEMA Design

Status: v1.1 frozen 2026-10-31. Owner: **Aesthesis** (`noema-aesthesis`).
This directory is the design law of NOEMA. Code follows it.

## Documents

| Document | What it is |
|---|---|
| [NOEMA_DESIGN_SYSTEM.md](NOEMA_DESIGN_SYSTEM.md) | Brand, tokens, type, glass, motion, layout, components, mascots, logo, voice, accessibility, performance, anti-patterns |
| [ARTIFACT_WORKSPACE.md](ARTIFACT_WORKSPACE.md) | Claude Design structure parity, artifact format, chat, dashboards, templates, export, versions, verification |
| [ANATOMY_AND_ASSETS.md](ANATOMY_AND_ASSETS.md) | Photoreal body, lower legs/feet, neutral pelvis, genitalia removal, material library, asset law, migration |
| [DECISIONS.md](DECISIONS.md) | Accepted design decisions with rationale and rejected alternatives |
| [AUDIT.md](AUDIT.md) | Block 10.1 measured baseline, asset/reference inventory, gap register |
| [../research/CLAUDE_DESIGN_2026.md](../research/CLAUDE_DESIGN_2026.md) | Claude Design product research and structural findings |

## Tokens

- `crates/app/ui/src/design/tokens.json` is canonical. Both themes live in one file.
- `crates/app/ui/src/design/tokens.css` is generated. Never edit it by hand.
- `node tools/design/build-tokens.mjs` regenerates; add `--check` to fail on drift.
- `node tools/design/check-contrast.mjs` gates 24 opaque pairs and writes `docs/evidence/design-tokens-contrast.json`.
- Theme contract: `data-theme="cavity"` or `"bone"` on `<html>`; Cavity is the default; `color-scheme` follows.
- `style.css` imports the token layer. Components read `--noema-*` only.

## Reading order

1. Design system.
2. Artifact workspace.
3. Anatomy and assets.
4. Decisions.
5. Research, for context only.

## Non-negotiables

- SVG for logos, icons, diagrams and mascots. No PNG, JPEG, GIF or BMP in the repo.
- WEBP only for real photography, never for UI chrome or logos.
- Platform icons are generated build artifacts from SVG.
- Tokens only. No hard-coded hex in components.
- Glass is a material, not a decoration. Maximum two blur layers.
- Every artifact is editable, downloadable, exportable, forkable and versioned.
- Photoreal anatomy, neutral pelvis, no external genitalia, fixed lower legs/feet.
- WCAG 2.2 AA, 60fps, performance budgets.
- No Claude Design prompt text, code, assets or visual identity. Structure only.

## How the agent works

Aesthesis is registered but not auto-started. The owner dispatches it from OpenClaw. It reads `agents/noema-aesthesis.md`, executes Block 10 in sub-blocks 10.1-10.10, one commit each, and reports evidence in WEBP/JSON.

## Current state

- Design law: written and frozen (v1.1, 2026-10-31).
- Agent: Aesthesis running; Block 10.2 complete.
- Block 10: 10.1 audit and 10.2 tokens complete; 10.3 logo suite next.
- Tokens: 29 color pairs, 12 type scales, 9 space steps, radius/z/motion/motion curves; Cavity + Bone; 24/24 contrast pairs pass AA.
- Raster debt: 34 files / 8.23 MiB (33 PNG + 1 ICO), 70 references. See [AUDIT.md](AUDIT.md).
- Body: muscle/bone atlas + capsule skin. Baseline confirms skeletal lower legs, feet and hands; the pelvic mass is unresolved pending the 10.8 mesh audit.
