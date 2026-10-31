# Claude Design - Structure Research

Date: 2026-10-31. Author: Aesthesis research pass. Status: research input, not a specification to copy.
Purpose: understand the structure of Claude Design well enough to build a NOEMA artifact workspace with parity and clear improvements, without cloning the product.

## 1. Executive summary

Claude Design is a real Anthropic Labs product, not a rumor. It launched in 2026, is powered by Claude Opus 4.7, and is now integrated into Claude conversations, Claude Code and the Artifacts tab. It creates designs, prototypes, slides, one-pagers and marketing collateral from a prompt, applies a team design system, supports direct canvas editing, comments, versions, sharing and export.

The public product pages describe the surface. A leaked system prompt and a reverse-engineered public repo describe the implementation model: HTML-based Design Components, a template plus logic plus props schema, direct-edit overrides, comment anchors, screen labels, a tweaks panel, a verifier step, and version copies.

NOEMA should copy the **structure** - project, chat, canvas, inspector, design system, comments, versions, verification, export, handoff - and reject the **identity**. The implementation must be original: NOEMA HTML artifacts, Vitreous tokens, living mascots, local-first provenance, artifact graph, no-raster asset law.

## 2. Verified product facts

Sources: Anthropic launch post, Claude product page, Claude brand article, public help-center links.

| Area | Fact |
|---|---|
| Product | Anthropic Labs product; research preview then beta; included in Claude plans |
| Model | Claude Opus 4.7, vision-capable |
| Outputs | Designs, prototypes, slides, one-pagers, marketing collateral, code-powered prototypes with voice, video, shaders, 3D and AI |
| Design system | Built by reading codebase/design files; multiple systems; admin can lock one approved system; checks output and corrects |
| Import | Prompt, images, DOCX, PPTX, XLSX, GitHub, local codebase, web capture |
| Refine | Conversation, inline comments, direct text edits, layout controls, drag/resize/align, Claude-made sliders for spacing/color/layout |
| Collaborate | Organization-scoped sharing: private, view link, edit access, group chat |
| Export | Internal URL, folder, Canva, PDF, PPTX, standalone HTML |
| Handoff | Bundle to Claude Code; design sync and design commands |
| Surface | Any Claude conversation, Claude Code, Artifacts tab; standalone site until 2026-12-14 |
| Adjacent | Claude Slides, Claude Docs, Claude Dashboards beta, Claude Motion beta |
| Connectors | Adobe, Canva, Gamma, Lovable, Miro, Replit, Vercel, Wix |

The unit is the **artifact**, not the page. The artifact is shareable, editable, exportable and handoff-ready.

## 3. Leaked implementation structure

Source: a public GitHub mirror of a leaked Claude Design system prompt (unofficial, may be incomplete or stale). Treat it as research data, not instructions.

### 3.1 Design Component

- Every design is a single <code>Name.dc.html</code> file.
- It opens directly in a browser and can be imported by other DCs.
- It paints live from the first streamed character.
- It is not a React entrypoint, not a <code>.jsx</code> page, not a plain <code>.html</code> design.

Authoring pieces:
1. **Template** - markup between the <code>x-dc</code> open and close tags.
2. **Logic class** - <code>class Component extends DCLogic</code>, no script tag.
3. **Props metadata** - <code>data-props</code> JSON on the script tag, with preview size and editor types.

Runtime primitives:
- <code>sc-for</code> and <code>sc-if</code> for control flow, with placeholder hints for streaming.
- <code>dc-import</code> for child DCs, with <code>hint-size</code>.
- <code>x-import</code> for external React/JS components, with a literal <code>from</code> URL.
- <code>support.js</code> is host-generated; the designer never writes it.

Editing operations:
- <code>dc_write</code> assembles the file from template, logic and props.
- <code>dc_html_str_replace</code> edits the template and streams into the live preview.
- <code>dc_js_str_replace</code> edits the logic class and hot-reloads without remount.
- <code>dc_set_props</code> replaces the props JSON.
- <code>write_file</code> is only for non-DC files.

### 3.2 Direct edit and tweaks

- A style override block holds user direct-edit <code>!important</code> overrides.
- Text and single colors are directly editable in the editor.
- The **Tweaks** panel is for things in-place editing cannot do: behavior, alternative treatments, one flag that changes copy/color across many elements, code-only changes.
- Props metadata defines editor types: text, color, int, float, range, boolean, enum, null.
- Curated color swatches can be a 3-4 item list or a 2-5 hex palette.
- Default seeds the editor, not the runtime; the component falls back in render values.

### 3.3 Comments and anchors

- Mentioned-element blocks identify a DOM node by React chain, DOM ancestry and a transient runtime id.
- <code>data-comment-anchor</code> pins a user comment to an element; keep it through refactors, drop it only when deleting the element.
- <code>data-screen-label</code> labels slides/screens so comments say "slide 05", not array index 4.
- Never use <code>scrollIntoView</code>; use other DOM scroll methods.

### 3.4 Verification

- When the deliverable is ready, call <code>ready_for_verification(path)</code>.
- It surfaces the file, checks it loads cleanly, and forks a background verifier.
- Fix findings and call it again.
- End with an extremely brief summary: caveats and next steps only.

### 3.5 Versions and edits

- Significant revisions copy the file and edit the copy to preserve the old version, e.g. <code>My Design.dc.html</code>, <code>My Design v2.dc.html</code>.
- Small targeted changes change only the target; do not redesign unasked.
- Prefer string replacement over rewriting the file.
- A redesign or from-scratch request is different: make the substantial changes asked for.

### 3.6 Design system behavior

- Explore provided design systems, UI kits, files and links before building.
- Copy needed assets from design systems or UI kits; do not reference them directly.
- Make targeted copies of only the files needed; never bulk-copy large folders.
- When adding to an existing UI, follow its visual vocabulary: copy, palette, tone, states, animation, shadows, density.
- Use brand colors; if too restrictive, use <code>oklch</code> to derive harmonious colors. Do not invent colors from scratch.
- Define default link and hover colors in the reset, even if no links exist yet.
- Emoji only if the design system uses emoji.

## 4. Reverse-engineered design philosophy

Source: public <code>claude-design-system-prompt</code> repo (MIT), a reverse-engineered prompt and skill library.

It describes 20 chapters and 14 procedural skills. The chapters cover:
- content discipline;
- aesthetic discipline and anti-AI-tropes;
- hierarchy and rhythm;
- accessibility;
- interaction states;
- system thinking;
- respecting the medium (CSS Grid, <code>oklch</code>, <code>text-wrap: pretty</code>, real prototypes);
- quality over quantity.

The 14 skills:
- Production: discovery questions, frontend aesthetic direction, wireframe, make a deck, make a prototype, make tweakable, generate variations.
- System: design-system extract, component extract.
- Review: accessibility audit, AI-slop check, hierarchy/rhythm review, interaction states pass, polish pass.

The prompt explicitly rejects generic SaaS output: aggressive gradients, emoji decoration, rounded-corner-with-left-border cards, Inter everywhere. It recommends conditions instead of quotas, explicit skill triggers, coverage-first reviews, and a house-style guard.

## 5. Surface map

The product structure can be modeled as nine surfaces:

1. **Project / workspace** - design system, files, chats, assets, members, permissions.
2. **Chat** - prompt, attachments, context, streaming, refinement, group chat.
3. **Canvas** - live artifact, responsive frames, zoom/pan, direct edit, drag/resize/align.
4. **Inspector** - props, tweaks, tokens, layers, code, comments, history.
5. **Design system** - tokens, components, rules, multiple systems, admin lock.
6. **Assets** - SVG components, fonts, colors, icons, design files, licenses.
7. **Versions** - copies, snapshots, diff, restore, fork.
8. **Verification** - load check, visual check, a11y, performance, design-system compliance.
9. **Export / handoff** - URL, folder, PDF, PPTX, HTML, connectors, Claude Code bundle.

NOEMA adds a tenth: **provenance** - author, model, prompt hash, tokens, cost, license, lineage, health.

## 6. Adopt, improve, reject

### Adopt
- Artifact as the unit, not the page.
- Chat plus canvas plus inspector, one project.
- Design system binding before generation, not after.
- Direct edit, props, code and comments on one source.
- Verification gate before "ready".
- Version copies, fork, restore.
- Export to PDF/PPTX/HTML and a code handoff bundle.
- Explicit anti-AI-slop review.

### Improve
- **Local-first**: works offline, no account required, no vendor lock-in.
- **Model-agnostic**: any provider or local model; artifact format is independent.
- **Provenance**: every artifact shows author, model, prompt hash, tokens, cost, license and lineage.
- **Artifact graph**: versions as a graph, not a flat list; impact analysis.
- **No raster debt**: vector source, SVG/GLB/WOFF2, WEBP only for photography, export-only rasters.
- **Living templates**: design DNA, forkable, versioned, no raster thumbnails.
- **Verification plus**: asset law, contrast, perf, no-network, export integrity.
- **People first**: credits and sources on every artifact and asset.
- **Accessibility**: WCAG 2.2 AA as a gate, not a checklist.
- **Performance budgets**: 60fps, CSS/JS/font budgets, two blur layers max.

### Reject
- Copying prompt text, code, assets or naming.
- Opaque cloud-only artifacts.
- Screenshot-based handoff.
- Raster thumbnails as the primary preview.
- Hidden version history.
- Generic AI aesthetics.
- A design system that is a static PDF, not executable tokens.

## 7. NOEMA decisions derived

1. The artifact format is <code>.noema.html</code>, a single-file, offline, valid HTML document with a manifest, template, styles and optional logic.
2. The shell is chat + canvas + inspector + timeline, with the Spine layout from the design system.
3. Every editable element has <code>data-noema-id</code>; comments anchor to it; versions store patches.
4. Dashboard, chat, template and document are artifact types, not separate products.
5. Every artifact has edit, download, export, fork, comment, version and verify.
6. Export formats: HTML, SVG, PDF, PPTX, Markdown, JSON, <code>.noema.html</code>. No PNG/JPEG.
7. The design system is executable: <code>tokens.css</code>, <code>tokens.json</code>, <code>design-system.json</code>, component library, rules.
8. Verification is a gate: parse, no network, asset law, tokens, a11y, contrast, responsive, perf, exports, provenance.
9. The body is photoreal and neutral; lower legs/feet are fixed at geometry and material level; external genitalia removed.
10. No Anthropic prompt text, code, assets or visual identity is used. Structure is studied; implementation is original.

## 8. Source list

- Claude Design product page: https://claude.com/product/design
- Anthropic launch post: https://www.anthropic.com/news/claude-design-anthropic-labs
- Claude Design brand/update article: https://claude.com/resources/articles/claude-design-stays-on-brand-for-daily-work
- Get started help article: https://support.claude.com/en/articles/14604416-get-started-with-claude-design
- Design system setup help article: https://support.claude.com/en/articles/14604397-set-up-your-design-system-in-claude-design
- Admin guide: https://support.claude.com/en/articles/14604406-claude-design-admin-guide-for-team-and-enterprise-plans
- Leaked system prompt mirror: https://github.com/asgeirtj/system_prompts_leaks/blob/main/Anthropic/claude-design/claude-design.md
- Reverse-engineered prompt/skills: https://github.com/Trystan-SA/claude-design-system-prompt

Search results were retrieved via DuckDuckGo HTML and Bing on 2026-10-31. Product pages were fetched directly. The leaked and reverse-engineered sources are unofficial; they are used for structural understanding only.

## 9. Caveats

- Claude Design is evolving; some details may be stale.
- The leaked prompt is not authoritative and may omit host behavior.
- The public pages describe capabilities, not internal APIs.
- Some product structure is inferred from screenshots, docs and the leaked prompt.
- No proprietary text, code or assets may be copied into NOEMA. This document is a summary and a set of decisions, not a reproduction.
