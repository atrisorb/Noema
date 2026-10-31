// Block 10.6/10.7 - .noema.html artifact workspace.
// One overlay: chat, live canvas, inspector, timeline, export and verify.

export type ArtifactType = 'dashboard' | 'chat' | 'template';

export interface Manifest {
  schema: 'noema.artifact/1';
  id: string;
  type: ArtifactType;
  title: string;
  version: string;
  created_at: string;
  author: string;
  model: string;
  design_system: string;
  theme: 'cavity' | 'bone';
  props: Record<string, unknown>;
  exports: string[];
  lineage: string[];
  license: string;
}

export interface Version { version: string; at: string; note: string; html: string }
export interface Comment { id: string; anchor: string; author: string; body: string; resolved: boolean }
export interface Artifact { manifest: Manifest; html: string; versions: Version[]; comments: Comment[] }

const STORE = 'noema:artifacts';
const CURRENT = 'noema:artifacts-current';
const esc = (s: string) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]!));

function runtime() {
  const n = window.__noema as any;
  const stats = n?.atlasStats?.() ?? { triangles: 0, systems: 0 };
  const debug = n?.debug?.() ?? { meshes: 0 };
  return { agents: n?.agentCount?.() ?? 0, fps: Math.round(n?.fps?.() ?? 0), tris: stats.triangles ?? 0, meshes: debug.meshes ?? 0 };
}

function dashboardBody(): string {
  const r = runtime();
  const bars = [['Cortex', 18], ['Axon', 14], ['Thalamus', 12], ['Synapse', 9], ['Soma', 8], ['Hippocampus', 6], ['Retina', 4]] as const;
  const max = 18;
  const cards = [
    ['Agents', String(r.agents)],
    ['Triangles', r.tris ? (r.tris / 1000).toFixed(0) + 'k' : '—'],
    ['Meshes', r.meshes ? String(r.meshes) : '—'],
    ['FPS', r.fps ? String(r.fps) : '—'],
  ];
  return `<main class="dash" data-noema-id="dash-root">
  <header class="dash-head" data-noema-id="dash-head">
    <p class="eyebrow">NOEMA · body runtime</p>
    <h1 data-noema-id="dash-title">Anatomy runtime</h1>
    <p class="lede">Measured values from the live body. No network, no raster, tokens only.</p>
  </header>
  <section class="stats" data-noema-id="dash-stats">${cards.map(([k, v], i) => `<article class="stat" data-noema-id="stat-${i}"><span>${k}</span><strong>${v}</strong></article>`).join('')}</section>
  <section class="panel" data-noema-id="dash-chart">
    <h2>Token share by agent</h2>
    <svg viewBox="0 0 520 200" role="img" aria-label="Token share by agent">
      ${bars.map(([name, value], i) => {
        const y = 14 + i * 25; const w = Math.round((value / max) * 330);
        return `<text x="0" y="${y + 12}" fill="var(--ink-soft)" font-size="11">${name}</text><rect x="92" y="${y}" width="${w}" height="14" rx="3" fill="var(--artery)"/><text x="${100 + w}" y="${y + 11}" fill="var(--ink-muted)" font-size="10">${value}%</text>`;
      }).join('')}
    </svg>
  </section>
</main>`;
}

function chatBody(): string {
  const rows = [
    ['Aesthesis', 'Built the Vitreous token layer. 29 color pairs, 24/24 AA.'],
    ['Cortex', 'The shell is live. Check the Bone theme and the inspector.'],
    ['Aesthesis', 'Artifact workspace is connected. Export HTML, JSON or Markdown.'],
  ];
  return `<main class="chat" data-noema-id="chat-root">
  <header class="dash-head" data-noema-id="chat-head"><p class="eyebrow">NOEMA · session</p><h1 data-noema-id="chat-title">Design session</h1><p class="lede">Hairline transcript, model and cost visible. No bubbles.</p></header>
  <section class="transcript" data-noema-id="chat-transcript">
    ${rows.map(([who, text], i) => `<article class="turn" data-noema-id="turn-${i}"><header><strong>${who}</strong><span>${who === 'Cortex' ? 'design QA' : 'design'}</span></header><p>${text}</p></article>`).join('')}
  </section>
</main>`;
}

function templateBody(): string {
  return `<main class="poster" data-noema-id="poster-root">
  <p class="eyebrow">NOEMA · anatomy poster</p>
  <h1 data-noema-id="poster-title">The body is the interface.</h1>
  <p class="lede">A template seed. Fork it, edit the tokens, keep the provenance.</p>
  <div class="poster-rule" data-noema-id="poster-rule"></div>
  <dl class="poster-facts" data-noema-id="poster-facts"><div><dt>System</dt><dd>Vitreous 1.1</dd></div><div><dt>Theme</dt><dd>Cavity</dd></div><div><dt>Format</dt><dd>.noema.html</dd></div></dl>
</main>`;
}

const BODY: Record<ArtifactType, () => string> = { dashboard: dashboardBody, chat: chatBody, template: templateBody };
const TITLE: Record<ArtifactType, string> = { dashboard: 'Anatomy runtime', chat: 'Design session', template: 'Anatomy poster' };

function styles(accent: string): string {
  return `*{box-sizing:border-box}html,body{margin:0;background:var(--bg);color:var(--ink);font-family:var(--font-ui);font-size:15px;line-height:1.55}body{padding:28px 32px;min-height:100vh}
:root{--bg:#05070B;--surface:#0B111B;--ink:#F4EFE6;--ink-soft:#D8D2C6;--ink-muted:#98A2B3;--line:rgba(244,239,230,.12);--artery:${accent};--font-ui:"Instrument Sans",system-ui,sans-serif;--font-display:"Newsreader",Georgia,serif}
[data-theme="bone"]{--bg:#F4EFE6;--surface:#FBF8F2;--ink:#0B0D12;--ink-soft:#2A2F3A;--ink-muted:#5C6472;--line:rgba(11,13,18,.12)}
h1,h2{font-family:var(--font-display);font-weight:500;letter-spacing:-.01em;margin:0}h1{font-size:clamp(34px,6vw,56px);line-height:1.05}h2{font-size:22px;margin-bottom:12px}
.eyebrow{font:600 10px/1.2 var(--font-ui);letter-spacing:.16em;text-transform:uppercase;color:var(--artery);margin:0 0 10px}
.lede{color:var(--ink-muted);max-width:62ch;margin:12px 0 26px}
.dash-head{max-width:760px}.stats{display:grid;grid-template-columns:repeat(4,1fr);gap:12px;margin:24px 0}
.stat{background:var(--surface);border:1px solid var(--line);border-radius:12px;padding:14px}.stat span{display:block;font-size:10px;letter-spacing:.12em;text-transform:uppercase;color:var(--ink-muted)}.stat strong{display:block;font-family:var(--font-display);font-size:30px;font-weight:500;margin-top:6px}
.panel{background:var(--surface);border:1px solid var(--line);border-radius:12px;padding:18px}.panel svg{width:100%;height:auto;display:block}
.transcript{display:flex;flex-direction:column}.turn{padding:14px 0;border-bottom:1px solid var(--line)}.turn header{display:flex;gap:10px;align-items:baseline}.turn header span{color:var(--ink-muted);font-size:11px}.turn p{margin:6px 0 0;color:var(--ink-soft)}
.poster{padding-top:8vh}.poster h1{font-size:clamp(44px,9vw,92px);max-width:12ch}.poster-rule{height:1px;background:var(--line);margin:34px 0}.poster-facts{display:flex;gap:36px;margin:0}.poster-facts dt{font-size:10px;letter-spacing:.12em;text-transform:uppercase;color:var(--ink-muted)}.poster-facts dd{margin:4px 0 0;font-family:var(--font-display);font-size:18px}
@media(max-width:760px){body{padding:20px}.stats{grid-template-columns:1fr 1fr}.poster-facts{flex-direction:column;gap:14px}}`;
}

export function buildHtml(m: Manifest, body: string, accent = '#FF5A5F'): string {
  return `<!doctype html>
<html lang="en" data-noema-artifact data-theme="${m.theme}">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>${esc(m.title)}</title>
<style id="noema-styles">${styles(accent)}</style></head>
<body><template id="noema-template">${body}</template>
<script type="application/noema+json" id="noema-manifest">${JSON.stringify(m, null, 2)}</script>
<script type="module" id="noema-logic"></script></body></html>`;
}

export function parseHtml(html: string): { manifest: Manifest; body: string; styles: string } | null {
  const doc = new DOMParser().parseFromString(html, 'text/html');
  if (!doc.documentElement.hasAttribute('data-noema-artifact')) return null;
  const raw = doc.querySelector('#noema-manifest')?.textContent ?? '';
  try {
    const manifest = JSON.parse(raw) as Manifest;
    const body = doc.querySelector('#noema-template')?.innerHTML ?? '';
    const style = doc.querySelector('#noema-styles')?.textContent ?? '';
    if (!manifest.schema || !body) return null;
    return { manifest, body, styles: style };
  } catch { return null; }
}

export function verify(html: string): { ok: boolean; at: string; checks: { id: string; pass: boolean; detail: string }[] } {
  const parsed = parseHtml(html);
  const checks = [
    { id: 'parse', pass: !!parsed, detail: parsed ? 'manifest and template parse' : 'missing manifest or template' },
    { id: 'schema', pass: parsed?.manifest.schema === 'noema.artifact/1', detail: parsed?.manifest.schema ?? 'none' },
    { id: 'title', pass: !!parsed?.manifest.title, detail: parsed?.manifest.title ?? 'missing' },
    { id: 'anchors', pass: (html.match(/data-noema-id="/g) ?? []).length >= 3, detail: `${(html.match(/data-noema-id="/g) ?? []).length} anchors` },
    { id: 'no-network', pass: !/(https?:)?\/\/[^\s"']+/.test(html), detail: 'no absolute URL' },
    { id: 'no-raster', pass: !/\.(png|jpe?g|gif|bmp|ico)/i.test(html), detail: 'no raster reference' },
    { id: 'tokens', pass: html.includes('var(--') && html.includes('--artery'), detail: 'token variables present' },
    { id: 'license', pass: !!parsed?.manifest.license, detail: parsed?.manifest.license ?? 'missing' },
  ];
  return { ok: checks.every((c) => c.pass), at: new Date().toISOString(), checks };
}

const seed = (): Artifact[] => {
  const mk = (type: ArtifactType, title: string, id: string): Artifact => {
    const m: Manifest = { schema: 'noema.artifact/1', id, type, title, version: '1.0.0', created_at: new Date().toISOString(), author: 'atr', model: 'deepseek/deepseek-flash', design_system: 'noema-vitreous@1.1.0', theme: 'cavity', props: {}, exports: ['html', 'json', 'md'], lineage: [], license: 'MIT' };
    const html = buildHtml(m, BODY[type]());
    return { manifest: m, html, versions: [{ version: '1.0.0', at: m.created_at, note: 'Seed', html }], comments: [] };
  };
  return [mk('dashboard', 'Anatomy runtime', 'art_runtime'), mk('chat', 'Design session', 'art_session'), mk('template', 'Anatomy poster', 'art_poster')];
};

function load(): Artifact[] {
  try { const raw = localStorage.getItem(STORE); if (raw) return JSON.parse(raw); } catch { /* ignore */ }
  return seed();
}
function save(list: Artifact[]) { try { localStorage.setItem(STORE, JSON.stringify(list)); } catch { /* ignore */ } }

export function mountArtifacts(toast: (s: string) => void) {
  let list = load();
  let current: Artifact | null = null;
  let currentId: string | null = null;
  try { currentId = localStorage.getItem(CURRENT); } catch { /* ignore */ }
  let pane = 'props';

  const panel = document.createElement('section');
  panel.id = 'artifact-panel';
  panel.setAttribute('aria-hidden', 'true');
  panel.innerHTML = `
    <header class="aw-head">
      <div><p class="aw-eyebrow">Artifact workspace</p><h2 id="aw-title">Artifacts</h2></div>
      <div class="aw-actions">
        <button id="aw-gallery-btn">gallery</button>
        <button id="aw-verify">verify</button>
        <button id="aw-export-html">html</button>
        <button id="aw-export-json">json</button>
        <button id="aw-export-md">md</button>
        <button id="aw-close" aria-label="Close artifact workspace">×</button>
      </div>
    </header>
    <div class="aw-body">
      <aside class="aw-chat"><h3>Chat</h3><div id="aw-transcript"></div><form id="aw-compose"><input id="aw-input" placeholder="Ask for a change" aria-label="Message" /><button>send</button></form></aside>
      <main class="aw-canvas"><div id="aw-gallery"></div><iframe id="aw-frame" title="Artifact preview" sandbox="allow-same-origin"></iframe><div id="aw-verify-out" hidden></div></main>
      <aside class="aw-inspector"><nav id="aw-tabs"><button data-pane="props" class="on">props</button><button data-pane="tokens">tokens</button><button data-pane="layers">layers</button><button data-pane="code">code</button><button data-pane="comments">comments</button></nav><div id="aw-pane"></div></aside>
    </div>
    <footer class="aw-timeline"><span>timeline</span><div id="aw-versions"></div><button id="aw-save-version">save version</button></footer>`;
  document.body.append(panel);

  const $ = <T extends HTMLElement>(s: string) => panel.querySelector<T>(s)!;
  const frame = $<HTMLIFrameElement>('#aw-frame');
  const gallery = $('#aw-gallery');
  const paneEl = $('#aw-pane');

  const select = (a: Artifact | null) => {
    if (a?.manifest.type === 'dashboard') a.html = buildHtml(a.manifest, dashboardBody(), (a.manifest.props.accent as string) ?? '#FF5A5F');
    current = a;
    currentId = a?.manifest.id ?? null;
    try { if (currentId) localStorage.setItem(CURRENT, currentId); else localStorage.removeItem(CURRENT); } catch { /* ignore */ }
    render();
  };
  const open = (type?: ArtifactType) => {
    panel.classList.add('open');
    panel.setAttribute('aria-hidden', 'false');
    if (type) select(list.find((a) => a.manifest.type === type) ?? null);
    else select(currentId ? list.find((a) => a.manifest.id === currentId) ?? null : null);
  };
  const close = () => { panel.classList.remove('open'); panel.setAttribute('aria-hidden', 'true'); };

  const preview = (a: Artifact) => {
    const parsed = parseHtml(a.html);
    if (!parsed) return '';
    const accent = (a.manifest.props.accent as string) ?? '#FF5A5F';
    return `<!doctype html><html data-theme="${a.manifest.theme}"><head><style>${styles(accent)}</style></head><body>${parsed.body}</body></html>`;
  };

  const render = () => {
    $('#aw-title').textContent = current?.manifest.title ?? 'Artifacts';
    gallery.innerHTML = current ? '' : `<div class="aw-gallery-grid">${list.map((a) => `<button class="aw-card" data-art="${a.manifest.id}"><span>${a.manifest.type}</span><strong>${esc(a.manifest.title)}</strong><small>v${a.manifest.version} · ${a.manifest.theme}</small></button>`).join('')}</div>`;
    gallery.hidden = !!current;
    frame.hidden = !current;
    if (current) frame.srcdoc = preview(current);
    renderPane();
    renderVersions();
    renderTranscript();
    gallery.querySelectorAll<HTMLButtonElement>('[data-art]').forEach((b) => b.addEventListener('click', () => select(list.find((a) => a.manifest.id === b.dataset.art) ?? null)));
  };

  const renderVersions = () => {
    const box = $('#aw-versions');
    box.replaceChildren();
    (current?.versions ?? []).forEach((v) => {
      const b = document.createElement('button');
      b.className = 'aw-version';
      b.textContent = `v${v.version}`;
      b.title = v.note;
      b.addEventListener('click', () => {
        if (!current) return;
        const parsed = parseHtml(v.html);
        if (!parsed) return;
        current.html = v.html; current.manifest = parsed.manifest; render(); toast('Version restored');
      });
      box.append(b);
    });
  };

  const renderTranscript = () => {
    const box = $('#aw-transcript');
    box.replaceChildren();
    (current ? [['Aesthesis', 'Artifact open. Edit props, code or tokens. Verify before export.'], ['Cortex', 'Provenance and version are visible. No network.']] : []).forEach(([who, text]) => {
      const row = document.createElement('p'); row.className = 'aw-turn'; row.innerHTML = `<strong>${who}</strong> ${text}`; box.append(row);
    });
  };

  const renderPane = () => {
    paneEl.replaceChildren();
    if (!current) { paneEl.textContent = 'Select an artifact.'; return; }
    if (pane === 'props') {
      paneEl.innerHTML = `<label>Title<input id="aw-prop-title" value="${esc(current.manifest.title)}" /></label><label>Theme<select id="aw-prop-theme"><option value="cavity">Cavity</option><option value="bone">Bone</option></select></label><label>Accent<input id="aw-prop-accent" value="${(current.manifest.props.accent as string) ?? '#FF5A5F'}" /></label><label>Type<select id="aw-prop-type"><option value="dashboard">dashboard</option><option value="chat">chat</option><option value="template">template</option></select></label>`;
      const theme = $<HTMLSelectElement>('#aw-prop-theme'); theme.value = current.manifest.theme;
      const type = $<HTMLSelectElement>('#aw-prop-type'); type.value = current.manifest.type;
      $('#aw-prop-title').addEventListener('change', (e) => { if (!current) return; current.manifest.title = (e.target as HTMLInputElement).value; persist(); });
      theme.addEventListener('change', () => { if (!current) return; current.manifest.theme = theme.value as 'cavity' | 'bone'; persist(); });
      type.addEventListener('change', () => { if (!current) return; current.manifest.type = type.value as ArtifactType; persist(); });
      $('#aw-prop-accent').addEventListener('change', (e) => { if (!current) return; const v = (e.target as HTMLInputElement).value; if (!/^#[0-9a-fA-F]{6}$/.test(v)) { toast('Accent must be #RRGGBB'); return; } current.manifest.props.accent = v; persist(); });
    } else if (pane === 'tokens') {
      const keys = ['--noema-artery', '--noema-ink', '--noema-ink-muted', '--noema-surface', '--noema-glass-line', '--noema-radius-card', '--noema-motion-panel', '--noema-font-display'];
      paneEl.innerHTML = `<dl class="aw-tokens">${keys.map((k) => `<div><dt>${k}</dt><dd>${getComputedStyle(document.documentElement).getPropertyValue(k).trim() || '—'}</dd></div>`).join('')}</dl>`;
    } else if (pane === 'layers') {
      const anchors = [...(parseHtml(current.html)?.body.matchAll(/data-noema-id="([^"]+)"/g) ?? [])].map((m) => m[1]);
      paneEl.innerHTML = `<ul class="aw-layers">${anchors.map((a) => `<li>${a}</li>`).join('')}</ul>`;
    } else if (pane === 'code') {
      paneEl.innerHTML = `<textarea id="aw-code" spellcheck="false"></textarea><button id="aw-apply">apply</button>`;
      $<HTMLTextAreaElement>('#aw-code').value = current.html;
      $('#aw-apply').addEventListener('click', () => {
        if (!current) return;
        const parsed = parseHtml($<HTMLTextAreaElement>('#aw-code').value);
        if (!parsed) { toast('Invalid .noema.html'); return; }
        current.html = $<HTMLTextAreaElement>('#aw-code').value; current.manifest = parsed.manifest; persist();
      });
    } else {
      paneEl.innerHTML = `<div id="aw-comments"></div><form id="aw-comment-form"><input id="aw-comment" placeholder="Comment on first anchor" /><button>add</button></form>`;
      const box = $('#aw-comments');
      (current.comments ?? []).forEach((c) => {
        const row = document.createElement('p'); row.className = 'aw-comment' + (c.resolved ? ' resolved' : '');
        row.innerHTML = `<strong>${c.author}</strong> ${esc(c.body)} <button data-resolve="${c.id}">${c.resolved ? 'reopen' : 'resolve'}</button>`;
        row.querySelector('button')!.addEventListener('click', () => { c.resolved = !c.resolved; persist(); });
        box.append(row);
      });
      $('#aw-comment-form').addEventListener('submit', (e) => {
        e.preventDefault();
        if (!current) return;
        const input = $<HTMLInputElement>('#aw-comment'); if (!input.value.trim()) return;
        current.comments.push({ id: 'c' + Date.now(), anchor: 'first', author: 'atr', body: input.value.trim(), resolved: false });
        input.value = ''; persist();
      });
    }
  };

  const persist = () => { save(list); render(); };
  const active = () => current;

  const exportText = (format: 'html' | 'json' | 'md') => {
    if (!current) return '';
    if (format === 'html') return current.html;
    if (format === 'json') return JSON.stringify({ manifest: current.manifest, versions: current.versions.map(({ html, ...v }) => v), comments: current.comments }, null, 2);
    return `# ${current.manifest.title}\n\n- type: ${current.manifest.type}\n- version: ${current.manifest.version}\n- theme: ${current.manifest.theme}\n- design system: ${current.manifest.design_system}\n- license: ${current.manifest.license}\n\n${parseHtml(current.html)?.body ?? ''}\n`;
  };
  const download = (format: 'html' | 'json' | 'md') => {
    const text = exportText(format);
    if (!text) return;
    const name = (current?.manifest.id ?? 'artifact') + (format === 'html' ? '.noema.html' : '.' + format);
    const a = document.createElement('a'); a.href = URL.createObjectURL(new Blob([text], { type: 'text/plain' })); a.download = name; a.click(); URL.revokeObjectURL(a.href);
  };

  $('#aw-close').addEventListener('click', close);
  $('#aw-gallery-btn').addEventListener('click', () => select(null));
  $('#aw-verify').addEventListener('click', () => {
    if (!current) return;
    const report = verify(current.html);
    const out = $('#aw-verify-out'); out.hidden = false;
    out.textContent = JSON.stringify(report, null, 2);
    toast(report.ok ? 'Verify passed' : 'Verify failed');
    (window as any).__noemaArtifacts.lastVerify = report;
  });
  $('#aw-export-html').addEventListener('click', () => download('html'));
  $('#aw-export-json').addEventListener('click', () => download('json'));
  $('#aw-export-md').addEventListener('click', () => download('md'));
  $('#aw-save-version').addEventListener('click', () => {
    if (!current) return;
    const next = (parseFloat(current.manifest.version) + 0.1).toFixed(1);
    current.manifest.version = next;
    current.versions.push({ version: next, at: new Date().toISOString(), note: 'Manual save', html: current.html });
    persist(); toast('Version saved');
  });
  panel.querySelectorAll<HTMLButtonElement>('#aw-tabs button').forEach((b) => b.addEventListener('click', () => { pane = b.dataset.pane!; panel.querySelectorAll('#aw-tabs button').forEach((x) => x.classList.toggle('on', x === b)); renderPane(); }));
  $('#aw-compose').addEventListener('submit', (e) => {
    e.preventDefault();
    const input = $<HTMLInputElement>('#aw-input'); if (!input.value.trim()) return;
    const box = $('#aw-transcript'); const p = document.createElement('p'); p.className = 'aw-turn'; p.innerHTML = `<strong>atr</strong> ${esc(input.value)}`; box.append(p); input.value = '';
    const reply = document.createElement('p'); reply.className = 'aw-turn'; reply.innerHTML = '<strong>Aesthesis</strong> Draft updated. Save a version to keep it.'; box.append(reply);
  });

  const api = {
    open, close, list: () => list.map((a) => ({ id: a.manifest.id, title: a.manifest.title, type: a.manifest.type, version: a.manifest.version })),
    current: () => active() ? { ...active()!.manifest } : null,
    verify: () => active() ? verify(active()!.html) : null,
    exportText,
    lastVerify: null as any,
  };
  (window as any).__noemaArtifacts = api;
  return { open, close, isOpen: () => panel.classList.contains('open') };
}
