# NOEMA - Vitreous Design System

Status: v1.0, frozen 2026-10-31. Owner: **Aesthesis** (`noema-aesthesis`).
This document is the single source of truth for brand, color, type, glass, motion, layout, iconography, mascots, accessibility, performance and asset law. Code follows it. When code and this document diverge, this document wins for intent; code wins for current implementation only until the gap is closed.

Flow Relay's design system was used as a governance model: hard rules at the top, tokens before components, a component map, an accessibility contract, anti-patterns and a new-component checklist. The visual language below is original to NOEMA.

## Hard rules

1. **One source of truth.** This document. No visual change without a design decision record.
2. **SVG-only identity.** Logos, icons, diagrams and mascots are SVG. No PNG, JPEG, GIF or BMP in the repo. WEBP only for real photography. Platform icons are generated build artifacts from SVG.
3. **Tokens only.** No hard-coded hex in a component. Use `var(--noema-*)`.
4. **Glass is a material, not a decoration.** Maximum two blur layers. Cavity surfaces are opaque. Readability wins.
5. **Photoreal where physical, vector where symbolic.** Anatomy, skin and materials are physically based 3D. Logos and icons stay vector. Never rasterize a logo.
6. **Motion with cause.** 60fps or cut the effect. Every animation has a reduced-motion path.
7. **Accessibility is a gate.** WCAG 2.2 AA, keyboard complete, focus visible, contrast checked, semantics correct.
8. **Every artifact is editable, downloadable and exportable.** No dead-end preview. No raster-only deliverable.
9. **People and provenance are visible.** Author, model, date, license and source on every artifact and asset.
10. **No generic AI aesthetic.** No purple gradient, no emoji icon, no Inter-everywhere, no floating blobs, no glass on every surface, no SaaS card with a left accent border.
11. **Performance is a contract.** Budgets below are gates, not aspirations.
12. **Authoritative, not authoritarian.** Clear defaults, reversible actions, no dark patterns, no blame copy.

## Table of contents

1. Brand foundations
2. Color tokens
3. Typography
4. Space, radius, elevation, z-index
5. Glass system
6. Motion
7. Layout shell
8. Components
9. Iconography
10. Mascots
11. Logo
12. Voice and tone
13. Accessibility
14. Performance
15. Artifact surfaces
16. Anatomy and 3D
17. Asset law
18. Anti-patterns
19. New component checklist
20. Governance

## 1. Brand foundations

NOEMA is an anatomical mind. The interface is a lens over living tissue: bone, blood, nerve and glass. It should feel like an editorial anatomy lab at night, not a SaaS dashboard.

| Axis | NOEMA |
|---|---|
| Mood | Calm, precise, alive, editorial, slightly uncanny |
| Material | Living glass, bone, arterial coral, neural amber, venous cobalt |
| Light | Dark cavity by default; warm bone for reading and print |
| Shape | Lenses, filaments, hairlines, soft rectangles, no blobs |
| Type | Newsreader display, Instrument Sans UI, Commit Mono data |
| Motion | Breath, trace, settle. No bounce, no spin, no confetti |
| Voice | Short, human, exact. No hype, no emoji, no exclamation |

**Not NOEMA:** cyberpunk neon, purple AI gradient, sterile hospital blue, cartoon anatomy, glassmorphism on every card, rounded-square icon set, emoji mascots.

**Themes.** Two named themes, one token set:
- **Cavity** - dark default. Near-black, bone text, arterial coral, glass over the body.
- **Bone** - light. Warm ivory, ink text, deeper accents, frosted bone glass.

## 2. Color tokens

Every token has a Cavity and a Bone value. Token names never change between themes. Components read variables; the theme flips values.

### 2.1 Core surfaces and ink

| Token | Cavity | Bone | Use |
|---|---|---|---|
| `--noema-bg` | `#05070B` | `#F4EFE6` | Page / canvas background |
| `--noema-bg-soft` | `#080C13` | `#EDE6DA` | Recessed areas, hover wash |
| `--noema-surface` | `#0B111B` | `#FBF8F2` | Opaque cards, inputs, tables |
| `--noema-surface-2` | `#101827` | `#E8DFD1` | Nested surface, alternate row |
| `--noema-ink` | `#F4EFE6` | `#0B0D12` | Primary text, headings |
| `--noema-ink-soft` | `#D8D2C6` | `#2A2F3A` | Body copy, secondary |
| `--noema-ink-muted` | `#98A2B3` | `#5C6472` | Supporting copy, labels |
| `--noema-ink-faint` | `#7C8798` | `#7A8494` | Metadata only, never body |
| `--noema-border` | `rgba(244,239,230,.12)` | `rgba(11,13,18,.12)` | Card, input, divider |
| `--noema-border-soft` | `rgba(244,239,230,.06)` | `rgba(11,13,18,.06)` | Internal hairline |

### 2.2 Accents and status

| Token | Cavity | Bone | Use |
|---|---|---|---|
| `--noema-artery` | `#FF5A5F` | `#C7353F` | Primary action, brand, active nav |
| `--noema-artery-deep` | `#D93A44` | `#9E2732` | Hover, pressed |
| `--noema-artery-soft` | `rgba(255,90,95,.14)` | `rgba(199,53,63,.10)` | Tint, selected row |
| `--noema-nerve` | `#FFC857` | `#9A6A00` | Focus ring, live, warning-of-good-kind |
| `--noema-nerve-soft` | `rgba(255,200,87,.14)` | `rgba(154,106,0,.10)` | Focus halo, live tint |
| `--noema-vein` | `#6C8CFF` | `#3F5BD6` | Information, links, secondary data |
| `--noema-vein-soft` | `rgba(108,140,255,.14)` | `rgba(63,91,214,.10)` | Info tint |
| `--noema-lymph` | `#7DFFC4` | `#1F7A55` | Success, healthy |
| `--noema-danger` | `#FF6B6B` | `#B3261E` | Destructive, error |
| `--noema-warning` | `#FFB86B` | `#8A5A00` | Warning |
| `--noema-info` | `#6C8CFF` | `#3F5BD6` | Info |

Accents are never used as large background fills except the primary button and selected row. Status is never color-only: add icon, label or shape.

### 2.3 Glass tokens

| Token | Cavity | Bone | Use |
|---|---|---|---|
| `--noema-glass-lens` | `rgba(244,239,230,.06)` | `rgba(11,13,18,.04)` | Lens panel over the body |
| `--noema-glass-membrane` | `rgba(244,239,230,.09)` | `rgba(11,13,18,.06)` | HUD, rail, toolbar |
| `--noema-glass-cavity` | `#0B111B` | `#FBF8F2` | Opaque reading surface |
| `--noema-glass-line` | `rgba(244,239,230,.14)` | `rgba(11,13,18,.12)` | 1px glass border |
| `--noema-glass-highlight` | `rgba(255,255,255,.18)` | `rgba(255,255,255,.65)` | Inner top highlight |
| `--noema-glass-veil` | `rgba(5,7,11,.55)` | `rgba(244,239,230,.65)` | Text safety layer |
| `--noema-shadow-lens` | `0 1px 2px rgba(0,0,0,.5), 0 24px 64px rgba(0,0,0,.35)` | `0 1px 2px rgba(60,40,20,.08), 0 24px 64px rgba(60,40,20,.12)` | Lens elevation |

### 2.4 Chart tokens

`--noema-chart-1` artery, 2 nerve, 3 vein, 4 lymph, 5 bone, 6 muted. Use 1.5px lines, 12% area fill, no 3D pies, no rainbow. Labels are always direct; legends only when unavoidable.

## 3. Typography

| Role | Family | Fallback | Notes |
|---|---|---|---|
| Display | **Newsreader** variable | Georgia, serif | Editorial, human, optical sizing |
| UI | **Instrument Sans** variable | system-ui, sans-serif | Sharp at 11-17px |
| Data | **Commit Mono** | ui-monospace, monospace | Tabular, timestamps, code |

All fonts are self-hosted WOFF2, subset Latin + Italian diacritics, OFL licensed. Total font payload <160KB. Preload only the display face. `font-display: swap`.

### 3.1 Type scale

| Token | Size / line | Weight | Use |
|---|---|---|---|
| `--noema-display-xl` | `clamp(48px, 7vw, 80px)` / 1.02 | 300 | Hero, one per screen |
| `--noema-display-l` | `clamp(38px, 5vw, 56px)` / 1.06 | 300 | Section opener |
| `--noema-display-m` | `clamp(30px, 3.5vw, 40px)` / 1.1 | 400 | Artifact title |
| `--noema-title` | 30px / 1.18 | 500 | Panel title |
| `--noema-heading` | 24px / 1.22 | 500 | Section heading |
| `--noema-subhead` | 20px / 1.3 | 500 | Card heading |
| `--noema-body-l` | 17px / 1.55 | 400 | Reading |
| `--noema-body` | 15px / 1.55 | 400 | Default UI |
| `--noema-body-s` | 13px / 1.5 | 400 | Dense UI |
| `--noema-label` | 11px / 1.2 | 600 | Uppercase, tracking +0.12em |
| `--noema-mono` | 12px / 1.5 | 400 | Code, IDs |
| `--noema-data` | 13px / 1.3 | 500 | Tabular numbers |

Rules: max line length 68ch; `text-wrap: pretty` on body, `balance` on headings; `font-variant-numeric: tabular-nums` on all data; no italic for emphasis (use weight or color); no all-caps body; no 700 body; underline only links.

## 4. Space, radius, elevation, z-index

**Space** (4px base): 4, 8, 12, 16, 24, 32, 48, 64, 96. Named `--noema-space-1..9`.

**Radius:** 4 data, 8 inputs, 12 cards, 20 glass panels, 999 pills. Never mix more than two radii in one component.

**Elevation:**

| Token | Recipe | Use |
|---|---|---|
| 0 | none | Flat table, inline row |
| 1 membrane | 0 1px 0 highlight, 0 8px 24px rgba(0,0,0,.18) | Rail, toolbar |
| 2 lens | inner highlight + 0 24px 64px rgba(0,0,0,.35) | Floating panel, modal |
| 3 cavity | inset 0 1px 0 highlight + 0 1px 0 border | Reading surface |

**Z-index ladder:** 0 canvas, 10 labels, 20 HUD, 30 rail, 40 panels, 50 modal, 60 toast, 70 debug. Never invent a new layer; extend the ladder in the token file.

## 5. Glass system

Three tiers. Glass is used only where it reveals the body or creates depth. It is never used to make a plain card look premium.

### 5.1 Lens

For floating panels, modals, inspectors over the 3D canvas.

~~~css
.noema-glass-lens {
  position: relative;
  border: 1px solid transparent;
  border-radius: var(--noema-radius-panel);
  background:
    linear-gradient(135deg, var(--noema-glass-lens), transparent 62%) padding-box,
    linear-gradient(135deg, var(--noema-glass-line), transparent 55%) border-box;
  backdrop-filter: blur(20px) saturate(1.35);
  box-shadow: inset 0 1px 0 var(--noema-glass-highlight), var(--noema-shadow-lens);
}
~~~

### 5.2 Membrane

For HUD, agent rail, toolbar, sticky headers. Blur 12px, slightly more opaque, no nested blur.

### 5.3 Cavity

For forms, code, tables, transcript, settings. Opaque `--noema-glass-cavity`. No blur. This is where readability lives.

### 5.4 Rules

- Max two backdrop-filter layers visible at once. No blur on scrolling lists or large surfaces.
- Text on glass needs a `--noema-glass-veil` layer if the body behind can be bright. Test against the brightest anatomy frame.
- Fallback: if `backdrop-filter` is unsupported, use `--noema-glass-cavity`.
- Grain: inline SVG `feTurbulence` at 2-3% opacity. Never a PNG noise texture.
- Refraction is a 1px gradient border plus an inner highlight. No fake lens distortion via huge shadows.
- No glass on every card. A screen with more than three glass layers is a bug.

## 6. Motion

| Token | Value | Use |
|---|---|---|
| `--noema-motion-micro` | 120ms | Hover, press, focus |
| `--noema-motion-standard` | 200ms | Toggle, tab, tooltip |
| `--noema-motion-panel` | 320ms | Drawer, panel, route |
| `--noema-motion-scene` | 560ms | Camera, artifact reveal |
| `--noema-ease-settle` | cubic-bezier(.2,.8,.2,1) | Default enter |
| `--noema-ease-standard` | cubic-bezier(.4,0,.2,1) | State change |
| `--noema-ease-exit` | cubic-bezier(.4,0,1,1) | Exit |

Rules: animate transform and opacity only; no layout animation; no bounce; no infinite spin; stagger 24ms, max 8 items; every animation has a purpose sentence. `prefers-reduced-motion: reduce` -> opacity-only, 120ms, no parallax, no autoplay, no SVG path draw.

## 7. Layout shell

The **Spine** shell:

~~~text
+--------------------------------------------------------------+
| command bar 56px                                             |
+--------+--------------------------------------+--------------+
| rail   | canvas                               | inspector    |
| 72px   | fluid                                | 380px        |
|        |                                      |              |
+--------+--------------------------------------+--------------+
| status filament 28px                                         |
+--------------------------------------------------------------+
~~~

- Command bar: brand mark, session pill, command palette, live agents, theme.
- Rail: eight mascot buttons, active filament, tooltip, keyboard order.
- Canvas: 3D body, brain, artifact preview, dashboard. The body is the background, not a card.
- Inspector: context, tabs, settings, artifact props. Resizable, collapsible, 320-420px.
- Status filament: 1px line with live dots, fps, tokens, cost, model. Not a footer bar.

Breakpoints: 1280, 1024, 768, 480. At <1024 inspector becomes bottom sheet; at <768 rail becomes horizontal scroller. No horizontal page scroll. Reserve space for async content; CLS <0.02.

Density: comfortable default, compact for tables. Row heights 32/40/48; touch targets 44.

## 8. Components

### 8.1 Buttons

| Variant | Background | Text | Border | Use |
|---|---|---|---|---|
| Primary | artery | bone | none | One per view |
| Secondary | glass membrane | ink | glass line | Common action |
| Ghost | transparent | ink-soft | none | Toolbar, inline |
| Danger | danger | bone | none | Destructive, confirm |

Heights 28/34/40; radius 10; icon 16/20; gap 8; focus ring 2px nerve offset 2; disabled opacity .45 but still readable; loading uses a breathing dot, not a spinner. No icon-only button without `aria-label` and tooltip.

### 8.2 Inputs

Height 34/40; label 11 uppercase; helper 12 muted; error 12 danger with icon. Placeholder is never the label. Focus ring 2px nerve. Select, textarea, toggle, slider, segmented control share the same field shell. Validation on blur, not on every keystroke.

### 8.3 Navigation

Tabs use a 1px filament underline; active artery; keyboard arrows. Sidebar items 36px, active with artery-soft wash and 2px left filament. No pills for primary navigation.

### 8.4 Panels and modals

Panel: lens, radius 20, 20px padding, header 44px, close 32px. Modal: max 720px, lens, focus trap, Escape, no full-screen on desktop. Drawer: 380px, membrane, resizable. Confirm destructive actions with a verb, not "Are you sure?".

### 8.5 Tables and data

Row 32/40; sticky header; hairline separators; tabular numbers; right-align numbers; no zebra; sort indicator 12px; empty state with one action. Charts are SVG, 1.5px lines, direct labels, colorblind-safe. No 3D pies, no dual axes, no chartjunk.

### 8.6 States

- Empty: one sentence, one action, one SVG glyph. No illustration wallpaper.
- Loading: breathing skeleton for >400ms; no spinner for short waits.
- Error: what happened, why, next action. No blame. Keep the user's input.
- Stale: amber filament + timestamp + refresh.
- Success: lymph dot + short message; no confetti.

### 8.7 Command palette

Ctrl/Cmd+K. Fuzzy search, recent, actions, agents, artifacts. Keyboard complete. 12px mono for shortcuts. No mouse-only affordance.

### 8.8 Agent rail

Eight mascot buttons. Active: artery filament + name. Hover: trace animation. Focus: nerve ring. Tooltip: agent name + one-line role. Click opens the agent inspector, not a modal.

### 8.9 Artifact card

Live SVG/iframe preview, never a raster thumbnail. Title, type, model, date, status. Actions: open, fork, download, export, more. Hover reveals actions; keyboard focus reveals them too. Card height 160/220; no generic stock image.

### 8.10 Chat transcript

No bubbles. A transcript with hairline separators, mascot avatar, role, timestamp, token/cost, artifact cards, code blocks. User messages align left with a 2px artery filament; assistant messages use a 1px vein filament. Composer 44px, multiline, attachments, model selector, send. Streaming uses deterministic placeholders; no layout shift.

## 9. Iconography

- 24x24 grid, 1.5px stroke, round caps and joins, currentColor, no fills except status dots.
- One SVG sprite: `assets/icons/noema-icons.svg`. No icon font, no PNG, no emoji.
- Icons are nouns and verbs, not decoration. Every icon has a text label or `aria-label`.
- Core set: lens, heart, brain, nerve, vessel, bone, agent, artifact, dashboard, chat, template, export, download, fork, version, comment, verify, search, command, settings, close, plus, minus, chevron, arrow, check, warning, info, lock, eye, play, pause.
- New icon checklist: 24px grid, 1.5px stroke, optical alignment, 16px legibility, RTL-safe, no text, no gradient, no raster.

## 10. Mascots

Eight living glyphs. Abstract anatomy, not cartoon characters. Each is a single SVG with `body`, `trace` and `glow` groups. Body uses currentColor; trace uses one accent; glow is a soft radial gradient. No faces except Retina's iris. No emoji.

| Agent | Mascot | Silhouette | Accent | Motion |
|---|---|---|---|---|
| Cortex | The Loom | Folded cortical ribbon forming a lens | Artery | Slow fold, 4s breathe |
| Axon | The Arrow | Tapered axon with myelin beads | Nerve | Trace tail on hover |
| Thalamus | The Lantern | Relay nucleus with two apertures | Nerve | Light pass-through, 4s |
| Synapse | The Knot | Two interlocking arcs with vesicles | Vein | Vesicle drift, 4s |
| Soma | The Mason | Bone-and-muscle cube with a heartbeat seam | Artery | Heartbeat seam, 2.4s |
| Hippocampus | The Seahorse | Seahorse filament with memory ripples | Vein | Ripple, 4s |
| Retina | The Iris | Camera iris with photoreceptor ring | Nerve | Iris contract on focus |
| Aesthesis | The Prism | Glass prism splitting one filament | Artery | Spectrum trace on hover |

Sizes: 16px glyph, 24px rail, 48px profile, 96px card, 512px hero. Test at 16px first. Stroke 1.5 at 24px, 2 at 48px, 3 at 96px. No embedded raster, no external font, no script. CSS/WAAPI only. Reduced motion -> static.

## 11. Logo

### 11.1 The mark - The Lens

A 32x32 grid. Outer lens circle r=13, stroke 2.5. A 90-degree quadrant is removed at the top-right, leaving a gap. A single filament enters at the bottom-left, curves through the center and exits at the top-right, forming an N negative space. One artery dot at the center. The mark works at 16px as a monochrome glyph.

### 11.2 Wordmark

**NOEMA** in Newsreader 500, tracking +0.04em, custom O: the counter is a lens with a 1px filament. All caps. No outline, no shadow, no gradient except the optional hero filament.

### 11.3 Lockup

Mark 32px + 10px gap + wordmark, baseline aligned. Stacked variant for square spaces. Monogram N for favicon and app icon. Clearspace = height of the O. Minimum sizes: mark 16px, wordmark 96px wide, lockup 140px wide.

### 11.4 App icon and favicon

Source: `assets/brand/noema-app-icon.svg` (1024 square, Cavity background, lens mark at 60%, artery filament). Build generates ICO/ICNS/PNG into `dist/icons` or `target/icons`; never commit them. Favicon: `favicon.svg` only.

### 11.5 Misuse

No rotation, no skew, no drop shadow, no outer glow, no gradient fill on the wordmark, no mark inside another shape, no low-contrast placement, no raster export as source.

## 12. Voice and tone

**Principles:** calm, precise, human, brief. Write like a good editor, not a sales page. Sentence case. No emoji. No exclamation. No hype. No blame. No "AI magic".

**Do:** "Session restored." "Export ready." "This model is offline. Switch to a local model or retry." "No artifacts yet. Create one from a template."

**Don't:** "Oops! Something went wrong." "Unleash the power of AI." "Seamless, revolutionary, game-changing." "You failed to..."

**Numbers:** tabular, units, no fake precision. Costs in USD with two decimals; tokens with thousands separators; latency in ms; dates ISO or localized, never ambiguous.

**Buttons:** verb-first, 1-3 words. "Create artifact", "Export PDF", "Fork", "Verify". Not "Submit", not "OK".

## 13. Accessibility

- WCAG 2.2 AA. Contrast: body >=4.5:1, large >=3:1, focus >=3:1. Muted tokens only for metadata.
- Focus-visible ring: 2px nerve, 2px offset, never removed. Skip link first in DOM.
- Keyboard complete: tab order, arrows in tabs/menus, Escape closes, Enter activates, no trap.
- Semantics: landmarks, headings in order, lists, buttons are buttons, links are links.
- SVG: `role="img"` + `<title>` when meaningful; `aria-hidden="true"` when decorative. Alt text for photography.
- Touch targets >=44x44 on touch; >=24x24 desktop.
- No color-only status. No autoplay over 5s. No motion without reduced-motion path.
- Test: axe/Playwright, one manual keyboard pass, one screen-reader pass per surface.

## 14. Performance

| Surface | CSS budget | JS budget | Frame budget |
|---|---|---|---|
| Body HUD | <10KB gzip | 0KB design system | 60fps |
| Studio | <20KB | <40KB | 60fps |
| Artifact workspace | <28KB | <80KB initial | 60fps |
| Chat | <20KB | <50KB | 60fps |
| Dashboard | <15KB | <40KB | 60fps |

Global: fonts <160KB, no raster chrome, no icon font, no runtime CSS-in-JS, max two blur layers, `content-visibility` for long lists, lazy 3D, worker for heavy parse, no per-frame allocations. LCP <1.2s local, INP <100ms, CLS <0.02. Measure with `node tools/design/perf.mjs` and Playwright trace. If an effect misses the budget, cut the effect.

## 15. Artifact surfaces

See `docs/design/ARTIFACT_WORKSPACE.md`. Every artifact is editable, downloadable, exportable, forkable, commentable, versioned and verifiable. Chat, dashboard and template are artifact types, not separate products. No dead ends. No raster-only output.

## 16. Anatomy and 3D

See `docs/design/ANATOMY_AND_ASSETS.md`. The body is photoreal, physically based and neutral. Lower legs and feet need geometry + material fixes, not tint. External genitalia are removed from the default body. Evidence is WEBP turntable and close-up.

## 17. Asset law

See `docs/design/ANATOMY_AND_ASSETS.md` section 6. Summary: SVG, GLB, WOFF2, JSON, MD only; WEBP only for real photography; platform icons generated; evidence WEBP; CI audit fails on forbidden extensions.

## 18. Anti-patterns

1. Purple/blue AI gradient.
2. Glass on every card.
3. Emoji as icon or mascot.
4. Inter for everything.
5. Rounded card with a left accent border.
6. Floating blobs, hexagons or "tech" particles with no meaning.
7. Fake 3D or isometric illustrations pretending to be anatomy.
8. Spinner for a 200ms wait; skeleton forever.
9. Modal for every action.
10. All-caps body text.
11. Low-contrast muted text used for important information.
12. Infinite decorative animation.
13. Raster logo or PNG screenshot in the product.
14. Hard-coded hex outside tokens.
15. Nested backdrop-filter.
16. Icon font.
17. Chartjunk, 3D pies, dual axes, rainbow series.
18. Autoplay with sound or >5s motion.
19. Dark pattern, fake urgency, blame copy.
20. A design doc that drifts from the code.

## 19. New component checklist

Before adding a component:
1. Does an existing primitive or variant solve it?
2. Is it token-only, no hard-coded values?
3. Does it work in Cavity and Bone?
4. Is every interactive element keyboard reachable with a visible focus ring?
5. Does it pass contrast in the worst-case glass context?
6. Does it have empty, loading, error and disabled states?
7. Does it have a reduced-motion path?
8. Does it meet the surface performance budget?
9. Is its icon SVG, 24px grid, 1.5px stroke?
10. Is it documented with a usage sentence and a do/don't?
11. Is there a Playwright or unit test?
12. Is there WEBP evidence?

If any answer is no, do not ship.

## 20. Governance

- **Owner:** Aesthesis. Changes require a design decision record in `docs/design/DECISIONS.md`.
- **Versioning:** token changes bump `designSystem.version`; breaking token renames need a migration note.
- **CI:** asset audit, contrast check, Playwright visual QA, perf budget.
- **Review:** Cortex final QA; Aesthesis design QA. No silent drift.
- **Code vs doc:** doc wins for intent; code wins for current implementation until the gap is closed.
- **No clone:** Claude Design structure may be studied; implementation, naming, assets, tokens and visual identity are original NOEMA.
