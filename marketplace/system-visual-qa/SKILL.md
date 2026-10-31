---
name: system-visual-qa
description: Headless Playwright screenshots and WebGL/WebGPU sanity checks for NOEMA UI states.
version: 1.0.0
license: MIT
allowed-tools: [Bash, Read, Write]
---

# System / Visual QA

Use this skill after any UI change. Never claim a visual fix without a screenshot path.

## Procedure

1. Build the UI (`npm run build` in the app folder) and start the dev server.
2. Capture the state: `node scripts/visual-qa.mjs <url> <out.png>`.
3. Read the PNG with the vision tool. Check layout, contrast, clipping and WebGPU fallbacks.
4. Record the screenshot path in the report and in `docs/AGENT_HANDOFF.md`.

## Rules

- Headful rendering is forbidden in CI; use `--headless` defaults.
- Redact tokens before passing any URL to the script.
- One screenshot per acceptance criterion, named `<criterion>.png`.
