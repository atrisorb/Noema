# NOEMA - Artifact Workspace Contract

Status: v1.0, 2026-10-31. Owner: Aesthesis. Behavior partner: Retina. Persistence partner: Axon.
Purpose: define the structure, editing model, export contract and quality gates for every model-generated artifact in NOEMA: dashboards, chat surfaces, templates, documents, decks, prototypes and SVG illustrations.

Claude Design is the structural reference. NOEMA must have the same surface anatomy - project, chat, canvas, inspector, comments, versions, design system, export, handoff - with an original implementation and clear improvements. It must not be a clone: no copied prompt text, code, assets, naming or visual identity.

## 1. What Claude Design actually does

Verified public facts, 2026-10-31:

- Anthropic Labs product powered by Claude Opus 4.7. Creates designs, prototypes, slides, one-pagers and marketing collateral from a prompt.
- Onboarding reads a codebase or design files and builds a design system. Multiple systems are supported; admins can lock one approved system.
- Import from prompt, images, DOCX/PPTX/XLSX, GitHub, local codebase or web capture.
- Refine with inline comments, direct text edits, layout controls, drag/resize/align, and Claude-made adjustment sliders for spacing, color and layout.
- Organization-scoped sharing: private, view link, edit access, group chat.
- Export to internal URL, folder, Canva, PDF, PPTX, standalone HTML. Handoff bundle to Claude Code.
- Works inside any Claude conversation, including Claude Code and Artifacts; standalone site until 2026-12-14.
- Claude Dashboards and Claude Motion are adjacent beta products; the artifact is the unit.

The leaked system prompt and the reverse-engineered public repo add implementation clues: HTML-based Design Components, a template plus logic class plus props metadata, direct-edit overrides, comment anchors, screen labels, a tweaks panel, a verifier step, and version copies. Treat those as research, not as text to copy.

## 2. Structural parity matrix

Same structure. Different soul. Every row is mandatory.

| Surface | Claude Design behavior | NOEMA parity | NOEMA improvement |
|---|---|---|---|
| Project / workspace | Design system + files + chats per project | Project with sessions, artifacts, assets, design system, members | Local-first, git-aware, no cloud dependency |
| Chat | Prompt, attachments, refinement, group chat | Transcript + composer + model/role picker + artifact cards | Token/cost meter, local models, deterministic welcome |
| Canvas | Live artifact, direct edit, drag/resize/align | Live canvas, direct edit, layout handles | 3D body context, split preview, no raster preview |
| Inspector / tweaks | Claude-made sliders, props, layout controls | Schema-driven props + token controls + code pane | Token-bound controls, every tweak maps to source |
| Design system | Import from code/design files, multiple systems, admin lock | <code>design-system.json</code> + tokens + components + rules | Versioned, diffable, CI-enforced |
| Components / assets | Import UI kits, copy assets into project | SVG component library + asset shelf + licenses | SVG-only, license/provenance on every asset |
| Comments | Inline element comments, mentions | Anchored comments on <code>data-noema-id</code> | Thread resolves to a patch, not a dead note |
| Versions | Copies, history, restore | Immutable versions + fork + diff + lineage | Graph, not a list; artifact health per version |
| Verification | <code>ready_for_verification</code>, load check, verifier | <code>artifact.verify</code> gate | A11y, contrast, perf, asset law, no-network |
| Export / share | PDF, PPTX, HTML, Canva, URL | HTML, SVG, PDF, PPTX, Markdown, JSON, <code>.noema.html</code> | No PNG/JPEG, reproducible export, offline |
| Code handoff | Bundle to Claude Code | Bundle to <code>noema</code> CLI / repo | Manifest + tests + design tokens, no screenshot |
| Collaboration | Org-scoped view/edit, group chat | Local roles + optional share link | No account required for local use |
| Templates | Starting points, brand-aware | Living template gallery with design DNA | Forkable, versioned, no raster thumbnails |
| Dashboards | Claude Dashboards beta | First-class artifact type | Data-bound, editable, exportable, accessible |
| Mobile / responsive | Web app | Responsive shell + Tauri desktop | Same artifact on desktop and webview |

## 3. The NOEMA shell

~~~text
+--------------------------------------------------------------------------------+
| command bar: project / artifact / model / share / export / verify              |
+----------------+------------------------------------------+--------------------+
| chat           | canvas                                   | inspector          |
| 360px          | fluid                                    | 380px              |
|                |  +------------------------------------+  | tabs:              |
| transcript     |  | live artifact / responsive frame   |  | props / tokens     |
| composer       |  | direct edit / comments / handles   |  | layers / code      |
| artifact cards |  +------------------------------------+  | comments / history |
+----------------+------------------------------------------+--------------------+
| timeline: versions, branches, checks, exports                                  |
+--------------------------------------------------------------------------------+
~~~

- The chat is not a sidebar afterthought. It is one view of the artifact graph.
- The canvas is the artifact, not a screenshot. Preview is live SVG/iframe.
- The inspector has five tabs: Props, Tokens, Layers, Code, Comments. History is the timeline.
- The shell is collapsible. At <1100px the inspector becomes a bottom sheet; at <760px chat and canvas become tabs.
- No empty landing page. Open the last artifact or a template gallery with one action.

## 4. Artifact model

Every artifact is a single <code>.noema.html</code> file. It is valid HTML, opens in a browser, and contains everything needed to render offline. The host injects the runtime; the file does not depend on a CDN.

~~~html
<!doctype html>
<html lang="en" data-noema-artifact>
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Artifact title</title>
  <style id="noema-styles">/* tokens only */</style>
</head>
<body>
  <template id="noema-template">
    <!-- semantic markup with data-noema-id anchors -->
  </template>
  <script type="application/noema+json" id="noema-manifest">
    {
      "schema": "noema.artifact/1",
      "id": "art_...",
      "type": "dashboard",
      "title": "Model spend",
      "version": "1.0.0",
      "created_at": "2026-10-31T15:00:00Z",
      "author": "atr",
      "model": "deepseek/deepseek-flash",
      "prompt_hash": "sha256:...",
      "design_system": "noema-vitreous@1.0.0",
      "theme": "cavity",
      "props": {},
      "exports": ["html", "svg", "pdf", "json"],
      "lineage": [],
      "license": "MIT"
    }
  </script>
  <script type="module" id="noema-logic">
    /* optional, sandboxed, no network by default */
  </script>
</body>
</html>
~~~

Rules:
- <code>data-noema-id</code> is the stable edit anchor. Every editable element has one. Never reuse an id across elements.
- <code>data-noema-comment</code> pins a comment to an element. Keep it through refactors; drop it only when the element is deleted.
- <code>data-noema-screen-label</code> labels a slide/screen for comments. Use human numbering ("05"), not array index.
- The manifest is the source of truth for type, title, version, model, design system, lineage and exports.
- No external scripts, no remote fonts, no network by default. Inline SVG only. No raster assets in the artifact.
- The file must render a useful first paint before logic runs. Logic is progressive enhancement.
- The host sanitizes imports and exports. Never trust a <code>.noema.html</code> from an unknown source.

## 5. Editing model

Five editing paths, one source of truth:

1. **Direct edit.** Text, color, spacing, layout. Each change maps to a <code>data-noema-id</code> and produces a minimal patch.
2. **Layout handles.** Drag, resize, align, distribute. Snaps to the 8px grid; keyboard nudges 1/10px.
3. **Props / tweaks.** Schema-driven controls: text, color, int, float, range, boolean, enum. Curated swatches from design tokens. No control that does not map to source.
4. **Code.** Template, styles and logic panes. Syntax highlighting, lint, format. Direct code edits are first-class; the preview hot-reloads.
5. **Comments.** Inline comment anchored to an element. A resolved comment becomes a patch or a decision record. No orphan comments.

Undo/redo across all paths. Autosave to a draft; explicit "Save version" creates an immutable snapshot. A small targeted change changes only that target; do not redesign unasked. A redesign request is explicit and creates a new version.

## 6. Chat contract

- Transcript: author, mascot, role, timestamp, model, tokens, cost. No chat bubbles; hairline separators.
- Composer: multiline, attachments, model/role picker, token/cost estimate, send. Ctrl/Cmd+Enter sends.
- Artifact cards: live preview, type, title, version, actions (open, fork, export, more).
- Streaming: deterministic placeholders, no layout shift, cancel. Reasoning hidden by default; "show reasoning" is a user action.
- Context chips: attached files, design system, selected element, current artifact. Every chip is removable.
- Commands: <code>/design</code>, <code>/dashboard</code>, <code>/template</code>, <code>/export</code>, <code>/verify</code>, <code>/fork</code>. Discoverable, not required.
- Empty chat: three starter prompts based on the current project. No generic "How can I help?".
- Group chat: participants, mentions, edit locks, presence. Local-first; no account required.

## 7. Dashboard contract

A dashboard is an artifact with <code>type: "dashboard"</code> and a data contract. It is not a screenshot of a chart.

- Widget spec: <code>{id, title, metric, dimension, filters, chart, layout, source}</code>.
- Metrics: tokens, cost, latency, events, success rate, cache hit, model mix. Dimensions: agent, model, project, peak, thinking, kind, time.
- Layout: 12-column grid, 8px gutter, drag/resize, keyboard alternative. Persisted in the artifact manifest.
- Data: local API or static JSON embedded in the artifact. No arbitrary network fetch. If offline, show the last snapshot with a stale badge.
- Charts: SVG, 1.5px lines, direct labels, accessible table fallback. No 3D pies, no dual axes, no rainbow.
- Export: HTML, SVG, PDF, JSON. A dashboard must remain editable after export (source included in HTML).
- Accessibility: table view, keyboard navigation, colorblind-safe series, text alternatives.

## 8. Template contract

Templates are living seeds, not static screenshots.

- Gallery categories: Dashboard, Chat, Deck, One-pager, Report, Prototype, Anatomy poster, Blank.
- Each template is a <code>.noema.html</code> with <code>type: "template"</code>, a design DNA block (theme, density, motion, accent), and a <code>fork</code> action.
- Preview is a live iframe or SVG poster. No PNG/JPEG thumbnails.
- "Use template" creates a new artifact with lineage to the template. The template is never mutated.
- Templates are versioned. A template update does not silently change existing artifacts.
- Every template must pass the same verify gate as a generated artifact.

## 9. Export, download and share

| Format | Use | Rules |
|---|---|---|
| <code>.noema.html</code> | Source, editable | Single file, offline, manifest included |
| HTML | Standalone artifact | Inline CSS/SVG, no network, sanitized |
| SVG | Vector artifact, logo, diagram | No raster embed, text as paths only when needed |
| PDF | Print, review | Vector where possible, embedded fonts, no screenshots |
| PPTX | Decks | Editable text and shapes, no flattened images |
| Markdown | Docs, handoff | Frontmatter with manifest, relative links |
| JSON | Data, manifest, dashboard | Stable schema, versioned |
| WEBP | Preview only, photography only | Never a deliverable for UI/logo; no PNG/JPEG |

Every artifact has a visible Export button. Export is reproducible: same artifact + same design system = same output. Show a checksum in the manifest. No "download screenshot" as the primary action.

Share: local link, file export, optional org link. Private by default. No public link without an explicit action. Share sheet shows what is included (source, data, comments, history).

## 10. Versions and provenance

- Every save creates an immutable version: id, parent, author, timestamp, model, prompt hash, design system, diff summary.
- Version graph, not a flat list. Fork creates a branch; merge is explicit.
- Provenance panel: author, model, prompt hash, tokens, cost, tools used, design system, license, sources.
- "Why this changed" note is required for a version created by an agent. One sentence.
- Restore never deletes history. Export includes the provenance manifest.

## 11. Verification gate

<code>artifact.verify</code> runs before "ready":

1. HTML parse and console errors.
2. No network requests; no remote assets.
3. Asset law: no PNG/JPEG/GIF/BMP; SVG/GLB/WOFF2 only.
4. Design system: tokens exist; no hard-coded hex; theme parity.
5. Accessibility: axe, contrast, focus order, labels, reduced motion.
6. Responsive: 1440, 1024, 768, 390.
7. Performance: first paint, interaction, layout shift, blur layers.
8. Links and exports: every claimed format exists and opens.
9. Provenance: manifest complete; license present.
10. Human review: one sentence on what to look at.

The verifier returns a report with severity, element, and fix. "Ready" is blocked on critical findings.

## 12. Security and local-first

- Render artifacts in a sandboxed iframe with a strict CSP. No network by default; allowlist only explicit connectors.
- Sanitize HTML/SVG on import and export. No script from untrusted artifacts.
- No localStorage keys without a <code>noema:</code> prefix. Never clear keys you did not write.
- No credentials in artifacts. Secrets stay in the OS keychain.
- Local-first: every artifact works offline. Cloud sync is optional, never required.
- Model-agnostic: any provider or local model can generate; the artifact format does not depend on a model.

## 13. Backend contract (coordinate with Axon)

Proposed tables, not implemented by Aesthesis:

- <code>artifacts(id, project_id, type, title, current_version_id, created_at, updated_at, license, design_system)</code>
- <code>artifact_versions(id, artifact_id, parent_id, author, model, prompt_hash, manifest_json, source_blob, created_at, note)</code>
- <code>artifact_comments(id, artifact_id, version_id, anchor, author, body, resolved_at)</code>
- <code>artifact_exports(id, artifact_id, version_id, format, path, checksum, created_at)</code>
- <code>artifact_assets(id, artifact_id, kind, path, license, credit)</code>

Endpoints: <code>GET/POST /api/artifacts</code>, <code>GET /api/artifacts/:id</code>, <code>POST /api/artifacts/:id/versions</code>, <code>POST /api/artifacts/:id/verify</code>, <code>POST /api/artifacts/:id/export</code>, <code>POST /api/artifacts/:id/fork</code>, <code>POST /api/artifacts/:id/comment</code>.

Every endpoint returns the artifact summary and the current version. CLI parity is mandatory: <code>noema artifact list|open|create|fork|export|verify</code>.

## 14. Acceptance tests

Playwright, one spec per surface:

- Create a dashboard from a template, edit a title, change a token color, add a widget, resize it, reload, assert persistence.
- Add a comment on an element, resolve it, assert the anchor survives a refactor.
- Create a version, fork it, assert the graph and the diff.
- Export HTML, SVG, PDF, PPTX, Markdown and JSON; assert each opens and contains the manifest.
- Verify a deliberate hard-coded hex and a PNG reference; assert the gate fails.
- Keyboard-only pass: open chat, send, open artifact, edit, export.
- Axe pass on chat, canvas, inspector and export modal.
- Perf: first paint <1.2s local, 60fps interaction, two blur layers max.
- Asset audit: zero forbidden extensions in the repo.

## 15. Non-clone statement

Claude Design's structure is the reference because it is the current best answer to "chat plus canvas plus design system plus export". NOEMA copies the structure, not the product. The implementation is original: <code>.noema.html</code>, Vitreous tokens, living mascots, local-first provenance, artifact graph, no-raster asset law, and a verification gate that is stricter than "it loads". No Anthropic prompt text, code, assets or visual identity is used.
