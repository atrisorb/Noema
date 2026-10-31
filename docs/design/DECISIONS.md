# NOEMA Design Decisions

Accepted 2026-10-31. Owner: Aesthesis. Each decision names the rejected alternative so future agents do not reopen it without evidence.

| ID | Decision | Rationale | Rejected alternative |
|---|---|---|---|
| D1 | **Vitreous** visual language: living glass, anatomy, arterial coral, bone, ink. | Unique, memorable, tied to the body; avoids generic SaaS glass. | Frosted-card glassmorphism; neon cyberpunk; warm Italian cafe (Flow Relay's identity). |
| D2 | Two themes: **Cavity** (dark default) and **Bone** (light). | The body is dark; reading and print need a light theme. | Single dark theme; theme-less tokens. |
| D3 | Type: **Newsreader** display, **Instrument Sans** UI, **Commit Mono** data. | Editorial authority, sharp UI, technical data. | Inter; Space Grotesk; system-only. |
| D4 | **SVG-first asset law**: no PNG/JPEG/GIF/BMP; WEBP only for real photography. | Crisp at every DPI, tiny, editable, no raster debt. | PNG icon set; JPEG hero; WebP for UI. |
| D5 | Artifact format: single-file **.noema.html** with manifest, template, styles and optional logic. | Portable, offline, editable, exportable, no lock-in. | Proprietary JSON canvas; React-only; screenshot handoff. |
| D6 | **Structural parity** with Claude Design, original identity and implementation. | The structure is the best current answer; the identity must be NOEMA. | Clone; ignore; build a generic canvas. |
| D7 | **Local-first, model-agnostic, provenance on every artifact.** | Reliable, private, no vendor lock-in, people and sources visible. | Cloud-only; model-specific format; anonymous output. |
| D8 | Every artifact is **editable, downloadable, exportable, forkable, versioned, verifiable**. | No dead ends; the artifact is the product. | Preview-only; export as screenshot; hidden history. |
| D9 | Body: **photoreal, neutral pelvis, no external genitalia, fixed lower legs/feet at geometry + material level.** | Scientific dignity, realism, the owner's explicit requirement. | Tint over a geometry defect; keep genitalia; cartoon body. |
| D10 | **Aesthesis owns design law**; visual-only override across app/ui; behavior stays with the owning agent. | Fast visual progress without breaking IPC or data. | Design agent edits core; every visual change needs a committee. |
