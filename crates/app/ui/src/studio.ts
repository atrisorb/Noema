// Studio: session restore, role models, BI widget grid and filter-to-widget.
// The feed is a pull from /api/studio; every write returns the whole snapshot.

import { apiUrl } from './platform';

interface WidgetSpec { metric: string; dimension: string; filters: { dimension: string; value: string }[] }
interface Widget { id: string; name: string; chart: string; spec: WidgetSpec; x: number; y: number; w: number; h: number; pinned: boolean }
interface StudioSession { id: string; project_id: string | null; agent_id: string; model_id: string; cwd: string; repo: string | null; branch: string | null; updated_at: number }
interface Project { id: string; name: string; root_path: string }
interface Model { id: string; name: string; capability: string }
interface RoleState { global: string | null; projects: Record<string, string> }
interface UsageRow {
  agent: string | null; model: string; project: string; kind: string; peak: number;
  thinking: string | null; events: number; tokens: number; cost_usd: number; latency_ms: number | null; at: number;
}
interface StudioState {
  session: StudioSession | null; projects: Project[]; sessions: StudioSession[]; models: Model[];
  roles: Record<string, RoleState>; widgets: Widget[]; usage: UsageRow[];
}

const $ = <T extends HTMLElement>(sel: string) => document.querySelector<T>(sel)!;
const ROLES = ['embedding', 'router', 'chat', 'coding', 'dreaming'];
const DIMENSIONS = ['agent', 'model', 'project', 'peak', 'thinking', 'kind'];
const fmt = new Intl.NumberFormat('en-US');
const money = (v: number) => `$${v.toFixed(2)}`;

const h = (tag: string, cls?: string, text?: string) => {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text !== undefined) node.textContent = text;
  return node;
};

function valueOf(row: UsageRow, dim: string): string {
  switch (dim) {
    case 'agent': return row.agent ?? 'n/a';
    case 'peak': return row.peak ? 'peak' : 'off-peak';
    case 'thinking': return row.thinking ?? 'n/a';
    case 'project': return row.project || 'unassigned';
    case 'kind': return row.kind;
    default: return row.model;
  }
}

function metricValue(metric: string, value: number): string {
  if (metric === 'cost') return money(value);
  if (metric === 'latency') return `${Math.round(value)} ms`;
  return fmt.format(Math.round(value));
}

export function mountStudio(toast: (message: string) => void) {
  const panel = $('#studio-panel');
  panel.innerHTML = `
    <header class="studio-head">
      <div><h2>Studio</h2><p id="studio-sub" class="muted">restoring last session…</p></div>
      <button id="studio-close" aria-label="close">×</button>
    </header>
    <div class="studio-body">
      <section id="studio-session" class="studio-block"></section>
      <section class="studio-block">
        <h3>role models</h3>
        <label class="studio-scope">scope <select id="role-scope"></select></label>
        <div id="role-grid"></div>
      </section>
      <section class="studio-block">
        <h3>projects · sessions</h3>
        <div id="studio-projects"></div>
        <div id="studio-sessions"></div>
      </section>
      <section class="studio-block">
        <h3>BI widgets</h3>
        <form id="widget-form">
          <input id="widget-name" placeholder="custom widget name" autocomplete="off" />
          <select id="widget-metric" aria-label="metric">
            <option value="tokens">tokens</option>
            <option value="cost">cost</option>
            <option value="latency">latency</option>
          </select>
          <select id="widget-dim" aria-label="dimension"></select>
          <select id="widget-filter-dim" aria-label="filter dimension"></select>
          <select id="widget-filter-val" aria-label="filter value"><option value="">any</option></select>
          <button id="widget-add" type="submit">add</button>
        </form>
        <div id="widget-grid"></div>
      </section>
    </div>`;

  let state: StudioState | null = null;
  let scope = '';
  let open = false;
  let drag: {
    id: string; mode: 'move' | 'size'; sx: number; sy: number;
    x0: number; y0: number; w0: number; h0: number; moved: boolean;
  } | null = null;

  const metricSel = $<HTMLSelectElement>('#widget-metric');
  const dimSel = $<HTMLSelectElement>('#widget-dim');
  const filterDim = $<HTMLSelectElement>('#widget-filter-dim');
  const filterVal = $<HTMLSelectElement>('#widget-filter-val');
  DIMENSIONS.forEach((d) => {
    dimSel.add(new Option(d, d));
    filterDim.add(new Option(d, d));
  });

  async function post(body: unknown): Promise<StudioState> {
    const res = await fetch(apiUrl('/api/studio'), {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(await res.text());
    return await res.json() as StudioState;
  }

  function pill() {
    const node = $('#session-pill');
    const s = state?.session;
    node.textContent = s
      ? `${s.repo ?? s.cwd}@${s.branch ?? '—'} · ${s.agent_id} · ${s.model_id.split('/').pop()}`
      : 'no session yet';
  }

  function renderSession() {
    const box = $('#studio-session');
    box.replaceChildren();
    const s = state?.session;
    $('#studio-sub').textContent = s
      ? `restored ${s.repo ?? s.cwd}@${s.branch ?? '—'} · ${s.agent_id} · ${new Date(s.updated_at).toLocaleString()}`
      : 'no session stored yet';
    if (!s) {
      box.append(h('div', 'muted', 'No session in the database. The CLI creates one on first run.'));
      return;
    }
    const card = h('div', 'session-card');
    card.append(h('div', 'session-title', `${s.repo ?? 'workspace'}${s.branch ? '@' + s.branch : ''}`));
    card.append(h('div', 'muted', `cwd ${s.cwd}`));
    card.append(h('div', 'session-meta', `agent ${s.agent_id} · model ${s.model_id} · ${s.id.slice(0, 8)}`));
    box.append(card);
  }

  function renderProjects() {
    const projects = $('#studio-projects');
    const sessions = $('#studio-sessions');
    projects.replaceChildren();
    sessions.replaceChildren();
    (state?.projects ?? []).forEach((p) => {
      const line = h('div', 'project-row');
      line.append(h('span', 'project-name', p.name), h('span', 'muted', p.root_path));
      projects.append(line);
    });
    if (!state?.projects.length) projects.append(h('div', 'muted', 'no projects'));
    (state?.sessions ?? []).forEach((s) => {
      const button = h('button', s.id === state?.session?.id ? 'session-row current' : 'session-row') as HTMLButtonElement;
      button.dataset.session = s.id;
      button.append(
        h('span', 'project-name', `${s.repo ?? s.cwd}@${s.branch ?? '—'}`),
        h('span', 'muted', `${s.agent_id} · ${new Date(s.updated_at).toLocaleTimeString()}`),
      );
      sessions.append(button);
    });
  }

  function renderRoles() {
    const roles = $('#role-grid');
    const scopeSel = $<HTMLSelectElement>('#role-scope');
    scopeSel.replaceChildren(new Option('global default', ''));
    (state?.projects ?? []).forEach((p) => scopeSel.add(new Option(`project · ${p.name}`, p.id)));
    scopeSel.value = scope;
    roles.replaceChildren();
    ROLES.forEach((role) => {
      const row = h('label', 'role-row');
      row.append(h('span', 'role-name', role));
      const select = document.createElement('select');
      select.dataset.role = role;
      select.add(new Option('— unassigned —', ''));
      (state?.models ?? []).forEach((m) => select.add(new Option(`${m.name} · ${m.capability}`, m.id)));
      const current = scope ? state?.roles[role]?.projects[scope] : state?.roles[role]?.global;
      select.value = current ?? '';
      select.addEventListener('change', () => void assignRole(role, select.value));
      row.append(select);
      roles.append(row);
    });
  }

  async function assignRole(role: string, model: string) {
    if (!model) return;
    try {
      state = await post({ action: 'role', capability: role, model, project: scope || undefined });
      renderRoles();
      toast(`${role} → ${model.split('/').pop()}${scope ? ' (project)' : ''}`);
    } catch (e) {
      toast(`role save failed: ${e}`);
    }
  }

  function groups(w: Widget) {
    const rows = (state?.usage ?? []).filter((r) =>
      w.spec.filters.every((f) => valueOf(r, f.dimension) === f.value));
    const map = new Map<string, { value: number; weight: number }>();
    rows.forEach((r) => {
      const label = valueOf(r, w.spec.dimension);
      const raw = w.spec.metric === 'tokens' ? r.tokens : w.spec.metric === 'cost' ? r.cost_usd : r.latency_ms;
      if (raw == null) return;
      const g = map.get(label) ?? { value: 0, weight: 0 };
      if (w.spec.metric === 'latency') {
        g.value += raw * r.events;
        g.weight += r.events;
      } else {
        g.value += raw;
      }
      map.set(label, g);
    });
    return [...map.entries()]
      .map(([label, g]) => ({ label, value: g.weight ? g.value / g.weight : g.value }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 6);
  }

  function widgetCard(w: Widget) {
    const card = h('article', 'widget-card');
    card.dataset.id = w.id;
    card.style.gridColumn = `${w.x + 1} / span ${w.w}`;
    card.style.gridRow = `${w.y + 1} / span ${w.h}`;
    const head = h('header', 'widget-head');
    head.append(
      h('span', 'widget-name', w.name),
      h('span', 'widget-meta', `${w.spec.metric} by ${w.spec.dimension}${w.spec.filters.length ? ' · filtered' : ''}`),
    );
    const remove = h('button', 'widget-remove', '×') as HTMLButtonElement;
    remove.dataset.remove = w.id;
    remove.title = 'remove widget';
    head.append(remove);
    card.append(head);
    const bars = h('div', 'widget-bars');
    const data = groups(w);
    const max = Math.max(1, ...data.map((d) => d.value));
    if (!data.length) bars.append(h('div', 'muted', 'no matching usage'));
    data.forEach((d) => {
      const line = h('div', 'bar-row');
      line.append(h('span', 'bar-label', d.label));
      const track = h('div', 'bar-track');
      const fill = h('div', 'bar-fill');
      fill.style.width = `${Math.max(2, (d.value / max) * 100)}%`;
      track.append(fill);
      line.append(track, h('span', 'bar-value', metricValue(w.spec.metric, d.value)));
      bars.append(line);
    });
    card.append(bars);
    const size = h('span', 'widget-size');
    size.title = 'resize';
    card.append(size);
    return card;
  }

  function renderWidgets() {
    const grid = $('#widget-grid');
    grid.replaceChildren();
    (state?.widgets ?? []).forEach((w) => grid.append(widgetCard(w)));
    renderFilterValues();
  }

  function renderFilterValues() {
    const dim = filterDim.value;
    const previous = filterVal.value;
    const values = new Set<string>();
    (state?.usage ?? []).forEach((r) => values.add(valueOf(r, dim)));
    filterVal.replaceChildren(new Option('any', ''));
    [...values].sort().forEach((v) => filterVal.add(new Option(v, v)));
    filterVal.value = values.has(previous) ? previous : '';
  }

  function render() {
    if (!state) return;
    renderSession();
    renderProjects();
    renderRoles();
    renderWidgets();
    pill();
  }

  async function load() {
    const res = await fetch(apiUrl('/api/studio'));
    if (!res.ok) throw new Error(await res.text());
    state = await res.json() as StudioState;
    render();
  }

  function setHash(on: boolean) {
    try {
      history.replaceState(null, '', on ? '#studio' : location.pathname + location.search);
    } catch {
      location.hash = on ? 'studio' : '';
    }
  }

  async function setOpen(on: boolean, syncHash = true) {
    open = on;
    panel.classList.toggle('open', on);
    panel.setAttribute('aria-hidden', on ? 'false' : 'true');
    document.body.classList.toggle('studio-mode', on);
    if (on) {
      if (syncHash) setHash(true);
      try {
        await load();
      } catch (e) {
        toast(`studio unavailable: ${e}`);
      }
    } else if (syncHash && location.hash) {
      setHash(false);
    }
  }

  async function saveLayout() {
    if (!state) return;
    try {
      state = await post({
        action: 'layout',
        widgets: state.widgets.map((w) => ({ id: w.id, x: w.x, y: w.y, w: w.w, h: w.h })),
      });
      toast('layout saved');
    } catch (e) {
      toast(`layout save failed: ${e}`);
    }
  }

  function gridStep() {
    const grid = $('#widget-grid');
    const cs = getComputedStyle(grid);
    const gap = parseFloat(cs.gap) || 8;
    const col = (grid.clientWidth - gap * 11) / 12;
    const row = parseFloat(cs.gridAutoRows) || 58;
    return { col: col + gap, row: row + gap };
  }

  function renderDrag() {
    if (!drag || !state) return;
    const w = state.widgets.find((x) => x.id === drag!.id);
    if (!w) return;
    const card = document.querySelector<HTMLElement>(`[data-id="${w.id}"]`);
    if (!card) return;
    card.style.gridColumn = `${w.x + 1} / span ${w.w}`;
    card.style.gridRow = `${w.y + 1} / span ${w.h}`;
  }

  panel.querySelector('#widget-grid')!.addEventListener('pointerdown', (event) => {
    const e = event as PointerEvent;
    const target = e.target as HTMLElement;
    if (target.closest('.widget-remove')) return;
    const card = target.closest('.widget-card') as HTMLElement | null;
    if (!card || !state) return;
    const w = state.widgets.find((x) => x.id === card.dataset.id);
    if (!w) return;
    drag = {
      id: w.id, mode: target.closest('.widget-size') ? 'size' : 'move',
      sx: e.clientX, sy: e.clientY, x0: w.x, y0: w.y, w0: w.w, h0: w.h, moved: false,
    };
    card.classList.add('dragging');
    e.preventDefault();
  });

  addEventListener('pointermove', (e) => {
    if (!drag || !state) return;
    const w = state.widgets.find((x) => x.id === drag!.id);
    if (!w) return;
    const { col, row } = gridStep();
    const dx = Math.round((e.clientX - drag.sx) / col);
    const dy = Math.round((e.clientY - drag.sy) / row);
    if (dx || dy) drag.moved = true;
    if (drag.mode === 'move') {
      w.x = Math.max(0, Math.min(11, drag.x0 + dx));
      w.x = Math.min(w.x, 12 - w.w);
      w.y = Math.max(0, drag.y0 + dy);
    } else {
      w.w = Math.max(2, Math.min(12 - w.x, drag.w0 + dx));
      w.h = Math.max(2, Math.min(8, drag.h0 + dy));
    }
    renderDrag();
  });

  addEventListener('pointerup', () => {
    if (!drag) return;
    const card = document.querySelector<HTMLElement>(`[data-id="${drag.id}"]`);
    card?.classList.remove('dragging');
    const moved = drag.moved;
    drag = null;
    if (moved) void saveLayout();
  });

  panel.querySelector('#widget-grid')!.addEventListener('click', (event) => {
    const target = event.target as HTMLElement;
    const id = target.dataset.remove;
    if (!id) return;
    void (async () => {
      try {
        state = await post({ action: 'widget_remove', id });
        renderWidgets();
        toast('widget removed');
      } catch (e) {
        toast(`remove failed: ${e}`);
      }
    })();
  });

  panel.querySelector('#studio-sessions')!.addEventListener('click', (event) => {
    const target = (event.target as HTMLElement).closest('[data-session]') as HTMLElement | null;
    if (!target) return;
    void (async () => {
      try {
        state = await post({ action: 'session', id: target.dataset.session! });
        render();
        toast('session restored');
      } catch (e) {
        toast(`session restore failed: ${e}`);
      }
    })();
  });

  $('#role-scope').addEventListener('change', () => {
    scope = $<HTMLSelectElement>('#role-scope').value;
    renderRoles();
  });

  filterDim.addEventListener('change', renderFilterValues);

  $('#widget-form').addEventListener('submit', (event) => {
    event.preventDefault();
    if (!state) return;
    const metric = metricSel.value;
    const dimension = dimSel.value;
    const filters = filterVal.value ? [{ dimension: filterDim.value, value: filterVal.value }] : [];
    const name = $<HTMLInputElement>('#widget-name').value.trim()
      || `${metric} by ${dimension}${filterVal.value ? ' · ' + filterVal.value : ''}`;
    const bottom = state.widgets.reduce((max, w) => Math.max(max, w.y + w.h), 0);
    const widget: Widget = {
      id: `widget-${Date.now().toString(36)}`, name, chart: 'bars',
      spec: { metric, dimension, filters }, x: 0, y: bottom, w: 6, h: 3, pinned: false,
    };
    void (async () => {
      try {
        state = await post({ action: 'widget', widget });
        $<HTMLInputElement>('#widget-name').value = '';
        renderWidgets();
        toast(`widget "${name}" added`);
      } catch (e) {
        toast(`widget save failed: ${e}`);
      }
    })();
  });

  $('#studio-close').addEventListener('click', () => void setOpen(false));

  return {
    load,
    setOpen,
    isOpen: () => open,
    snapshot: () => ({
      open,
      session: state?.session ?? null,
      widgets: state?.widgets ?? [],
      roles: state?.roles ?? {},
      projects: state?.projects.length ?? 0,
      sessions: state?.sessions.length ?? 0,
      usage: state?.usage.length ?? 0,
    }),
  };
}
