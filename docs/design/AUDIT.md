# NOEMA - Block 10.1 Design Audit

Status: complete, 2026-10-31. Owner: Aesthesis.
Scope: audit and freeze only. No visual code changed, no token introduced, no design decision reopened.
Purpose: establish the measured baseline that Blocks 10.2-10.10 must close, with reproducible evidence.

## 1. Method and inputs

Read in full: `docs/design/NOEMA_DESIGN_SYSTEM.md`, `ARTIFACT_WORKSPACE.md`, `ANATOMY_AND_ASSETS.md`, `DECISIONS.md`, `README.md`, `agents/noema-aesthesis.md`, `docs/MASTER_PLAN.md` Block 10, `docs/AGENT_HANDOFF.md`, last 20 lines of `CHANGELOG.md`.

Commands and sources:

- Driftwatch: `node F:\tools\driftwatch\bin\driftwatch digest --project F:\Progetti\agentic-anatomy-workspace`.
- Git: `git status --porcelain=v1 -b`, `git ls-files`, `git grep`.
- Runtime: `target/debug/noema-app.exe serve --demo --port 4199 --ui crates/app/ui/dist --assets assets --db target/block10/audit.db`.
- Captures: `crates/app/ui/scripts/capture-audit.mjs` (Playwright 1.64.1, Chromium headed, ANGLE D3D11, 1600x900, DPR 1). WebP is written by CDP `Page.captureScreenshot`; no PNG is created or committed.
- Measurements: file bytes, `git grep` counts, CSS/TS regex counts, `window.__noema` runtime values.

Repo state at audit: branch `main`, ahead 1, working tree clean, HEAD `e34734b`, 171 tracked files.

## 2. Driftwatch digest

8 updates since the last digest, all dated 2026-09-30. Design impact: **none**.

| Platform | Change | Impact |
|---|---|---|
| Railway | Webhooks page changed | None (no Railway dependency in design) |
| Render | Changelog page changed | None |
| Netlify | API get-started page changed | None |
| Brevo API | Changelog page changed | None |
| Scaleway Generative APIs | Model catalog changed | None for design; model list owned by Thalamus |
| DeepSeek models + pricing | Pricing page changed | Not design. Thalamus rate cards may need a refresh |
| Mistral models + pricing | Pricing page changed | Not design. Thalamus rate cards may need a refresh |
| Google Gemini pricing | Pricing page changed | Not design. Thalamus rate cards may need a refresh |

No dependency was added or bumped in 10.1, so no `driftwatch check` was required. The three pricing changes are forwarded as a non-blocking note for Thalamus; they do not alter tokens, glass, type or anatomy.

## 3. Raster and reference inventory

Tracked rasters: **34 files, 8,628,345 bytes (8.23 MiB)**.

| Group | Files | Bytes | Content | 10.9 disposition |
|---|---|---|---|---|
| `docs/evidence/*.png` | 26 | 8,193,139 | Block 5-9 screenshots and baselines | Convert to WebP q82, update links and JSON shot paths, delete PNG |
| `src-tauri/icons/{32x32,128x128,128x128@2x,icon}.png` + `icon.ico` | 5 | 133,745 | Tauri placeholder icon set | Generate from `assets/brand/noema-app-icon.svg` into a git-ignored output dir; remove committed copies |
| `src-tauri/app-icon.png` | 1 | 40,853 | Source used by `cargo tauri icon` | Replace with SVG source; generation only |
| `crates/app/ui/evidence/block9-wrapper-studio.webp` | 1 | 170,181 | Wrapper evidence | Convert to WebP |
| `assets/anatomy/cad/vertebra.webp` | 1 | 90,427 | Raster duplicate of `vertebra-section.svg` / `.pdf` / `.dxf` | Delete; SVG is the source |

Counts by extension: 33 PNG, 1 ICO, **0 JPEG, 0 GIF, 0 BMP, 0 TIFF**. No new raster was added in this block.

Largest rasters: `block9-baseline-cellular.webp` 614,701 B, `block5-heart-close.webp` 611,923 B, `block9-baseline-dpr2.webp` 590,866 B. Twenty-six evidence PNGs alone are 95% of raster bytes; migrating them is the single largest asset win.

Reference inventory: **70 references in 27 tracked text files** (excluding `package-lock.json`).

| Reference owner | Count | Note |
|---|---|---|
| Playwright specs (`tests/*.spec.ts`, `*.perf.ts`) | 22 in 7 files | Tests write PNG evidence directly; 10.9/10.10 must switch them to WebP |
| `src-tauri/tauri.conf.json` | 5 | Bundled icon paths |
| `packaging/build.sh` | 2 | Reuses `docs/evidence/block8-render.webp` as a Linux icon; must generate from SVG |
| Docs (`CHANGELOG`, `MASTER_PLAN`, `BRAIN`, `HARDWARE`, `PLATFORMS`, `STUDIO`, `REVIEW-block9`, `AGENT_HANDOFF`, `design/ANATOMY`, `LICENSES`) | 29 | Link-only; update during migration |
| Evidence JSON (`block9-*.json`) | 6 | `shot` fields point at PNG |
| `marketplace/system-visual-qa` | 4 | Skill and script default to PNG output; update contract to WebP |
| `tools/perf` | 3 | README and `webview2-fps.mjs` default shot path |
| `.gitignore` | 1 | `crates/app/icons/*.png` allowlist; keep generated outputs ignored |

The UI itself has **zero raster references**: `index.html` uses a canvas and inline SVG only; `style.css` contains no `url(...)`; no favicon is declared. This is the right starting point for the SVG-first law.

Asset sources and licenses are intact: atlas CC BY-SA 4.0, procedural meshes and CAD CC0-1.0, no third-party texture or scan. `assets/brand/` contains a README only; the seven SVG brand files do not exist yet.

## 4. UI and visual baseline

Measured on the current build (`crates/app/ui`, Vite 8.3.5, three 0.187.0):

| Fact | Measured | Design target | Gap |
|---|---|---|---|
| Stylesheet | 14,103 B raw, 12,141 B built, 3,097 B gzip, 127 lines | Tokens + primitives | No token layer |
| Hard-coded hex | 65 in `style.css`, 5 in `main.ts` | 0 in components | 70 hex sites |
| Raw rgba | 26 in `style.css` | Tokens | 26 sites |
| Backdrop filters | 4 declarations, blur 8/14px, no fallback, no veil | Max two visible, veil for text | Up to 3 surfaces can stack |
| Type | Segoe UI + Consolas | Newsreader / Instrument Sans / Commit Mono | 0 WOFF2 in repo |
| Brand assets | No logo, monogram, favicon, mascot or sprite | 7 brand SVGs + 8 mascots + icon sprite | All missing |
| Agent roster | 7 entries: Axon, Cortex, Hippocampus, Retina, Soma, Synapse, Thalamus | 8 mascots, Aesthesis included | 1 missing |
| HUD | 8 layer controls in two rows at 1600px; brand wraps to 2 lines | 56px command bar | Layout overflow |
| Shell | Floating HUD, left roster, right drawers; no Spine | 72px rail, 380px inspector, 28px status filament | Structural rebuild |
| JS | 723,332 B raw, 183,591 B gzip (three.js incl.) | Body HUD 0 KB design system; Studio <40 KB | Bundle not split |
| Studio | Functional session, role-model, project and BI widget panels | Artifact workspace with chat/canvas/inspector/timeline | No artifact model |

Visual read: generic dark cyan glass. Gradients, cyan accents, green fps, monospace form fields. No Vitreous tokens, no bone/artery/nerve/vein palette, no editorial type, no lens material. Studio is dense but readable; it does not use the design system at all.

## 5. Anatomy baseline

Runtime values at 1600x900:

- Atlas: 1 system loaded, **826 meshes**, **290,886 triangles**, 2 materials (`Mat_Bone_Pro`, `Mat_Muscle_Pro`), 826 visible meshes, 0 transparent meshes, 0 load errors.
- Default layers: skin **off**, muscles on, skeleton on, nerves on, vessels on, organs on, brain on.
- Procedural skin: one `MeshPhysicalMaterial`, opacity 0.12, clearcoat 0.4, double-sided. Torso, pelvis and head are spheres; limbs are capsules.
- Procedural leg: thigh capsule to y -0.60, calf capsule to y -0.90, foot capsule from y -0.91 to -0.92 with z +0.09. Foot length 0.01 in a 1.72 body.

Findings from `design-baseline-legs.webp` (pelvis to feet, 2x):

1. Lower legs are bone-dominant: tibia and fibula are white sticks with muscle straps at the sides. No skin, fat or fascia surface fills the silhouette.
2. Feet are skeletal: metatarsals and phalanges are exposed, no heel volume, no arch, no nails, no contact shadow. The feet read as floating bones.
3. Hands are skeletal as well; fingers are bones with no skin surface.
4. The pelvis shows a central mass between the thighs. Node names do not identify it; it cannot be classified from names or this render. **No claim is made in this audit.** Block 10.8 must run the mesh-level pelvic audit defined in `ANATOMY_AND_ASSETS.md` section 4 and prove the result with front, side and close-up WebP.
5. The body reads as a muscle/bone study, not a living body. This confirms the diagnosis in `ANATOMY_AND_ASSETS.md`: geometry plus material, not tint.

These findings match the pre-existing design doc. Block 10.1 adds fresh evidence, not a new diagnosis.

## 6. Artifact workspace baseline

Nothing exists yet:

- 0 `.noema.html` files.
- No artifact parser, manifest, template, props schema, comment anchor, version graph, export pipeline or verification gate.
- No `design-system.json`, no `tokens.json`, no `crates/app/ui/src/design/`.
- Studio is a session/role-model/BI widget surface, not an artifact workspace.

Blocks 10.6-10.7 start from zero. Tokens from 10.2 are a hard prerequisite; nothing in 10.6 should hard-code color, type or spacing.

## 7. Accessibility baseline

Measured in source, not yet audited with axe:

- No `:focus-visible` rule. One `textarea:focus` outline only.
- No skip link.
- No `prefers-reduced-motion` path anywhere in CSS or TS.
- 13 `aria-label` attributes; 34 landmark/role/button matches in `index.html`.
- The body is canvas-only. No text alternative, no table or list of anatomical structures.
- Small controls: roster pills use 5px 10px padding and 11px text; several toolbar buttons are below the 24px desktop target.
- No automated contrast check exists; the design doc requires WCAG 2.2 AA.

This is a baseline, not a failure verdict. 10.5 and 10.10 must produce axe, keyboard and contrast evidence.

## 8. Performance baseline

From `docs/evidence/block9-baseline.json` (Playwright Chromium, 1600x900, 5s window):

| Scenario | FPS mean | FPS p50 | p99 frame |
|---|---|---|---|
| Full atlas, DPR 1 | 60.1 | 59.9 | 17.0 ms |
| Brain wake, DPR 1 | 60.1 | 59.9 | 17.1 ms |
| Cellular zoom, DPR 1 | 60.1 | 59.9 | 16.8 ms |
| Full atlas, DPR 2 | **35.4** | **30.0** | 50.0 ms |

DPR 2 fails the 60fps budget. The body is 826 meshes; the cost is draw-call and shading bound, not triangle bound. 10.8 must add LODs and verify on WebView2, not only Chromium.

## 9. Gap register

| # | Gap | Evidence | Block | Acceptance |
|---|---|---|---|---|
| 1 | No token layer; 70 hard-coded hex, 26 rgba | Section 4 | 10.2 | `tokens.css` + `tokens.json`, Cavity and Bone, zero hex in touched components |
| 2 | No brand SVG, no favicon, placeholder app icon | Section 3-4 | 10.3 | 7 SVG sources, generated icons, no committed raster |
| 3 | 7 agents, no mascots, no sprite | Section 4 | 10.4 | 8 SVGs at 16/24/96/512, reduced-motion, sprite loader |
| 4 | No Spine shell; 4 blur declarations, up to 3 visible | Section 4 | 10.5 | Lens/Membrane/Cavity primitives, max two blurs, responsive at 1280/1024/768/480 |
| 5 | No artifact workspace | Section 6 | 10.6-10.7 | One real artifact in every claimed format, edit/fork/version/verify |
| 6 | Lower legs, feet, hands skeletal; skin/fascia absent; pelvis unresolved | Section 5 | 10.8 | Photoreal materials, foot/hand geometry, neutral pelvis, close-up WebP, glTF-Validator clean |
| 7 | 34 rasters, 70 references | Section 3 | 10.9 | Asset audit green; zero forbidden extensions; references updated |
| 8 | No focus-visible, reduced-motion, axe or contrast evidence | Section 7 | 10.5/10.10 | WCAG 2.2 AA evidence, keyboard pass, axe report |
| 9 | DPR 2 at 35.4 fps | Section 8 | 10.8/10.10 | LODs, 60fps WebView2 trace |
| 10 | Aesthesis missing from roster | Section 4 | 10.4/10.5 | 8 agents, mascot and rail entry; plan change proposed to Cortex, not edited here |

## 10. Freeze decisions for this block

1. `docs/design/**` v1.0 remains frozen. 10.1 introduces no token, color, type or decision change.
2. Evidence is WebP only. The capture script writes through CDP and never creates a PNG; this is the required pattern for 10.9/10.10.
3. The UI source is untouched. 10.1 is read-only on `crates/app/ui/src`; only the capture script and evidence are added.
4. Anatomy statements are limited to what the render and runtime show. The pelvic mass is unresolved until the 10.8 mesh audit; no removal claim is made here.
5. The 7-vs-8 agent roster is recorded as a gap. `docs/MASTER_PLAN.md` is owned by Cortex; Aesthesis proposes, does not rewrite.
6. Driftwatch pricing changes are forwarded to Thalamus as a note; no design action.

## 11. Evidence

| File | Content |
|---|---|
| `docs/evidence/design-baseline-body.webp` | Full body, 1600x900, current HUD and roster |
| `docs/evidence/design-baseline-legs.webp` | Pelvis-to-feet close-up, 2x |
| `docs/evidence/design-baseline-rail.webp` | Agent rail open |
| `docs/evidence/design-baseline-machine.webp` | Machine panel |
| `docs/evidence/design-baseline-brain.webp` | Brain view |
| `docs/evidence/design-baseline-studio.webp` | Studio overlay |
| `docs/evidence/design-baseline.json` | Capture metadata, atlas stats, debug box, layers, agent count |
| `crates/app/ui/scripts/capture-audit.mjs` | Reproducible WebP capture |

Runtime: 826 meshes, 290,886 triangles, 7 agents, 1600x900, DPR 1. Capture timestamp in the JSON.

## 12. Exit criteria

- [x] All design docs and the agent charter read.
- [x] Driftwatch digest run and classified.
- [x] Full asset and reference inventory with counts and disposition.
- [x] Current-state WebP captures, no PNG committed.
- [x] `docs/design/AUDIT.md` written.
- [x] No out-of-scope edit; design law unchanged.
- [x] `cargo check --workspace` green.
- [x] Changelog, handoff, plan tick and atomic commit; no push.
