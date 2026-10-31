import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { RoomEnvironment } from 'three/addons/environments/RoomEnvironment.js';
import { buildBody } from './body';
import { Atlas, type AtlasLayer } from './atlas';
import { Circulation } from './circulation';
import { Swarm } from './agents';
import { BrainGraph, type BrainNode } from './brain';
import { mountStudio } from './studio';
import type { Anatomy, Agent, LayerName } from './types';
import { apiBase, apiUrl, assetBase, getJson, isTauri } from './platform';

const $ = <T extends HTMLElement>(sel: string) => document.querySelector<T>(sel)!;

interface MachineReport {
  profile: {
    platform: string; os: string; arch: string; hostname: string; cpu_brand: string;
    physical_cores: number; logical_cores: number; ram_mb: number;
    gpus: { name: string; vendor: string; kind: string; vram_mb: number | null; driver: string | null }[];
    free_disk_mb: number | null; total_disk_mb: number | null; data_dir: string; detected_at_ms: number;
  };
  tuning: {
    quality: string;
    render: { pixel_ratio_cap: number; particle_budget: number; shadows: boolean; max_texture_mb: number; lod_bias: number };
    inference: { max_context_tokens: number; parallel_requests: number; gpu_layers: number; vram_budget_mb: number; recommended_runtime: string };
    daemon: { worker_threads: number; sqlite_cache_mb: number };
    reasons: string[];
  };
  overrides: Record<string, unknown>;
  cached: boolean;
  generated_at_ms: number;
}

async function fetchMachine(refresh = false): Promise<MachineReport | null> {
  try {
    return await getJson<MachineReport>(refresh ? '/api/system/refresh' : '/api/system', isTauri() ? 12 : 1);
  } catch (e) {
    console.warn('system profile unavailable', e);
    return null;
  }
}

let machine = await fetchMachine();
const canvas = $<HTMLCanvasElement>('#scene');
const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' });
renderer.setPixelRatio(Math.min(devicePixelRatio, machine?.tuning.render.pixel_ratio_cap ?? 1.5));
renderer.setSize(innerWidth, innerHeight);
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 0.95;

const scene = new THREE.Scene();
scene.background = new THREE.Color(0x05070d);
scene.fog = new THREE.FogExp2(0x05070d, 0.02);
const camera = new THREE.PerspectiveCamera(32, innerWidth / innerHeight, 0.004, 60);
camera.position.set(0, 0.12, 2.55);
const controls = new OrbitControls(camera, canvas);
controls.target.set(0, 0.08, 0);
controls.enableDamping = true;
controls.dampingFactor = 0.075;
controls.minDistance = 0.03;
controls.maxDistance = 7;
controls.rotateSpeed = 0.6;
controls.zoomSpeed = 0.85;

const pmrem = new THREE.PMREMGenerator(renderer);
scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
pmrem.dispose();

scene.add(new THREE.HemisphereLight(0xbcd8ff, 0x06080f, 0.75));
const key = new THREE.DirectionalLight(0xfff3e8, 1.7);
key.position.set(2.2, 3.4, 2.4);
scene.add(key);
const fill = new THREE.DirectionalLight(0x8fb3ff, 1.0);
fill.position.set(-2.4, 1.2, -0.8);
scene.add(fill);
const rim = new THREE.DirectionalLight(0xff9ec4, 1.2);
rim.position.set(-0.6, 1.6, -3.2);
scene.add(rim);
const heartLight = new THREE.PointLight(0xff3b5c, 1.4, 0.9);
scene.add(heartLight);
const cameraLight = new THREE.PointLight(0xfff1f5, 0.5, 3);
scene.add(cameraLight);


const body = buildBody();
Object.values(body.layers).forEach((layer) => scene.add(layer));

const atlas = new Atlas();
atlas.root.visible = false;
scene.add(atlas.root);

const circulation = new Circulation(machine?.tuning.render.particle_budget ?? 980);
circulation.group.visible = false;
scene.add(circulation.group);

const brain = new BrainGraph(scene);
let brainMode = false;
let dragBrain: { index: number; moved: boolean } | null = null;
let brainPulseTimer: ReturnType<typeof setInterval> | null = null;
let lastDream: unknown = null;

const glb = { heart: false, brain: false };
let anatomy: Anatomy | null = null;
let swarm: Swarm | null = null;
let selected: Agent | null = null;
let ready = false;
let fps = 0;
let elapsed = 0;
let paused = false;
let tween: { from: THREE.Vector3; to: THREE.Vector3; targetFrom: THREE.Vector3; targetTo: THREE.Vector3; t: number; dur: number } | null = null;
const raycaster = new THREE.Raycaster();

const rail = $('#rail');
const animaPanel = $('#anima-panel');
const machinePanel = $('#machine-panel');
const toastEl = $('#toast');
let toastTimer = 0;
function toast(message: string) {
  toastEl.textContent = message;
  toastEl.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => toastEl.classList.remove('show'), 2400);
}

const fmt = new Intl.NumberFormat('en-US');
const money = (v: number) => `$${v.toFixed(2)}`;
const el = (tag: string, cls?: string, text?: string) => {
  const node = document.createElement(tag);
  if (cls) node.className = cls;
  if (text !== undefined) node.textContent = text;
  return node;
};
const row = (k: string, v: string) => {
  const line = el('div', 'row');
  line.append(el('span', 'k', k), el('span', 'v', v));
  return line;
};
const chips = (items: string[]) => {
  if (!items.length) return el('div', 'muted', 'none');
  const wrap = el('div');
  items.forEach((i) => wrap.append(el('span', 'chip', i)));
  return wrap;
};

function renderTab(tab: string) {
  if (!selected) return;
  const bodyEl = $('#tab-body');
  bodyEl.replaceChildren();
  const t = selected.tabs;
  if (tab === 'activity') {
    if (!t.activity.events.length) bodyEl.append(el('div', 'muted', 'no usage events yet'));
    t.activity.events.forEach((e) => bodyEl.append(row(`${e.kind} · ${new Date(e.at).toLocaleTimeString()}`, `${fmt.format(e.tokens)} tok · ${money(e.cost_usd)}`)));
  } else if (tab === 'tokens') {
    bodyEl.append(el('div', 'big', fmt.format(t.tokens.total)), el('div', 'muted', 'tokens across this agent'));
    t.tokens.by_model.forEach((m) => bodyEl.append(row(m.model, fmt.format(m.tokens))));
  } else if (tab === 'costs') {
    bodyEl.append(el('div', 'big', money(t.costs.total_usd)), el('div', 'muted', 'total spend'));
    t.costs.by_model.forEach((m) => bodyEl.append(row(m.model, money(m.cost_usd))));
  } else if (tab === 'ecosystem') {
    bodyEl.append(el('div', 'muted', 'skills'), chips(t.ecosystem.skills));
    bodyEl.append(el('div', 'muted', 'plugins'), chips(t.ecosystem.plugins));
    bodyEl.append(el('div', 'muted', 'MCP servers'), chips(t.ecosystem.mcp));
  } else {
    bodyEl.append(row('cwd', t.workspace.cwd ?? '—'));
    bodyEl.append(row('repo', t.workspace.repo ?? '—'));
    bodyEl.append(row('branch', t.workspace.branch ?? '—'));
    bodyEl.append(row('last seen', t.workspace.last_seen ? new Date(t.workspace.last_seen).toLocaleString() : '—'));
  }
}

function setTab(tab: string) {
  document.querySelectorAll<HTMLButtonElement>('#tabs button').forEach((b) => b.classList.toggle('on', b.dataset.tab === tab));
  renderTab(tab);
}

function openAgent(id: string) {
  const agent = anatomy?.agents.find((a) => a.id === id);
  if (!agent) return;
  selected = agent;
  swarm?.freeze(id);
  animaPanel.classList.remove('open');
  animaPanel.setAttribute('aria-hidden', 'true');
  rail.classList.add('open');
  rail.setAttribute('aria-hidden', 'false');
  rail.style.setProperty('--dot', agent.accent);
  $('#agent-name').textContent = agent.name;
  $('#agent-role').textContent = agent.role;
  $('#greeting').textContent = agent.greeting;
  setTab('activity');
  toast(`${agent.name} frozen`);
}

function closeRail() {
  rail.classList.remove('open');
  rail.setAttribute('aria-hidden', 'true');
  swarm?.freeze(null);
  selected = null;
}

function heartPosition(): THREE.Vector3 {
  if (atlas.ready && atlas.heartMeshes.length > 0) return atlas.heartAnchor.clone();
  return body.heart.getWorldPosition(new THREE.Vector3());
}

function fitAtlas() {
  const box = new THREE.Box3().setFromObject(atlas.root);
  if (box.isEmpty()) return;
  const size = box.getSize(new THREE.Vector3());
  const center = box.getCenter(new THREE.Vector3());
  console.info('atlas bounds', box.min.toArray(), box.max.toArray(), 'size', size.toArray());
  controls.target.copy(center);
  const vFov = THREE.MathUtils.degToRad(camera.fov);
  const distance = (size.y / 2) / Math.tan(vFov / 2) * 1.35;
  camera.position.set(center.x, center.y + size.y * 0.02, center.z + Math.max(distance, 1.4));
  camera.near = Math.max(0.004, size.y / 500);
  camera.far = size.y * 20;
  camera.updateProjectionMatrix();
  controls.update();
}

function screenOfPoint(point: THREE.Vector3) {
  const v = point.clone().project(camera);
  return {
    x: (v.x * 0.5 + 0.5) * canvas.clientWidth + canvas.getBoundingClientRect().left,
    y: (-v.y * 0.5 + 0.5) * canvas.clientHeight + canvas.getBoundingClientRect().top,
  };
}

function openAnima() {
  if (!anatomy) return;
  rail.classList.remove('open');
  rail.setAttribute('aria-hidden', 'true');
  swarm?.freeze(null);
  animaPanel.classList.add('open');
  animaPanel.setAttribute('aria-hidden', 'false');
  const identity = $<HTMLTextAreaElement>('#anima-identity');
  const prompt = $<HTMLTextAreaElement>('#anima-prompt');
  identity.value = anatomy.anima.identity;
  prompt.value = anatomy.anima.system_prompt;
  $('#anima-status').textContent = '';
}

async function saveAnima() {
  const identity = $<HTMLTextAreaElement>('#anima-identity').value;
  const system_prompt = $<HTMLTextAreaElement>('#anima-prompt').value;
  const status = $('#anima-status');
  status.textContent = 'saving…';
  try {
    const res = await fetch(apiUrl('/api/anima'), { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ identity, system_prompt }) });
    if (!res.ok) throw new Error(await res.text());
    const saved = await res.json();
    if (anatomy) anatomy.anima = saved;
    status.textContent = 'saved to database';
    toast('Anima persisted');
  } catch (e) {
    status.textContent = 'save failed';
    toast(String(e));
  }
}


const PRESETS: Record<string, [number, number]> = { ultra: [2, 4000], high: [1.75, 2500], balanced: [1.5, 1500], low: [1, 600] };

function renderMachine() {
  const specs = $('#machine-specs');
  if (!machine) {
    specs.replaceChildren(el('div', 'muted', 'profile unavailable — app server offline'));
    return;
  }
  const p = machine.profile;
  const t = machine.tuning;
  const rows = [
    row('platform', `${p.platform} · ${p.arch}`),
    row('host', p.hostname),
    row('cpu', `${p.cpu_brand} · ${p.logical_cores} threads`),
    row('ram', `${(p.ram_mb / 1024).toFixed(0)} GB`),
    row('data dir', p.data_dir),
    row('disk free', p.free_disk_mb == null ? '—' : `${(p.free_disk_mb / 1024).toFixed(0)} / ${((p.total_disk_mb ?? 0) / 1024).toFixed(0)} GB`),
    row('quality', t.quality),
    row('pixel cap', String(t.render.pixel_ratio_cap)),
    row('particles', String(t.render.particle_budget)),
    row('runtime', t.inference.recommended_runtime),
    row('context', `${t.inference.max_context_tokens} tok`),
    row('cluster', `${t.inference.parallel_requests} req · ${t.inference.gpu_layers} gpu layers`),
    row('threads', `${t.daemon.worker_threads} · cache ${t.daemon.sqlite_cache_mb} MB`),
  ];
  p.gpus.forEach((g, i) => {
    rows.push(row(`gpu ${i + 1}`, `${g.name}${g.vram_mb ? ` · ${(g.vram_mb / 1024).toFixed(1)} GB` : ''} · ${g.kind}`));
  });
  t.reasons.forEach((reason) => rows.push(el('div', 'muted', reason)));
  specs.replaceChildren(...rows);
  $<HTMLSelectElement>('#machine-quality').value = t.quality;
  $<HTMLInputElement>('#machine-ratio').value = String(t.render.pixel_ratio_cap);
  $<HTMLInputElement>('#machine-particles').value = String(t.render.particle_budget);
}

function openMachine() {
  rail.classList.remove('open');
  animaPanel.classList.remove('open');
  rail.setAttribute('aria-hidden', 'true');
  animaPanel.setAttribute('aria-hidden', 'true');
  machinePanel.classList.add('open');
  machinePanel.setAttribute('aria-hidden', 'false');
  renderMachine();
}

function closeMachine() {
  machinePanel.classList.remove('open');
  machinePanel.setAttribute('aria-hidden', 'true');
}

function applyMachine() {
  const cap = machine?.tuning.render.pixel_ratio_cap ?? 1.5;
  renderer.setPixelRatio(Math.min(devicePixelRatio, cap));
  renderer.setSize(innerWidth, innerHeight);
  if (machine) circulation.setBudget(machine.tuning.render.particle_budget);
}

async function saveMachine() {
  const status = $('#machine-status');
  const overrides = {
    quality: $<HTMLSelectElement>('#machine-quality').value,
    render: {
      pixel_ratio_cap: Number($<HTMLInputElement>('#machine-ratio').value),
      particle_budget: Number($<HTMLInputElement>('#machine-particles').value),
    },
  };
  status.textContent = 'saving…';
  try {
    const res = await fetch(apiUrl('/api/system'), { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ overrides }) });
    if (!res.ok) throw new Error(await res.text());
    machine = await res.json() as MachineReport;
    applyMachine();
    renderMachine();
    status.textContent = 'saved';
    toast('hardware tuning applied');
  } catch (e) {
    status.textContent = 'save failed';
    toast(String(e));
  }
}

async function refreshMachine() {
  const status = $('#machine-status');
  status.textContent = 'probing…';
  machine = await fetchMachine(true);
  applyMachine();
  renderMachine();
  status.textContent = machine ? 're-detected' : 'probe failed';
}

function flyTo(position: THREE.Vector3, target: THREE.Vector3, dur = 900) {
  tween = { from: camera.position.clone(), to: position, targetFrom: controls.target.clone(), targetTo: target, t: 0, dur };
}

let pointerDown: { x: number; y: number; at: number } | null = null;
canvas.addEventListener('pointerdown', (e) => { pointerDown = { x: e.clientX, y: e.clientY, at: performance.now() }; });

canvas.addEventListener('pointerdown', (e) => {
  if (!brainMode) return;
  const hit = brain.pick(e.clientX, e.clientY, camera, innerWidth, innerHeight);
  if (!hit) return;
  dragBrain = { index: hit.index, moved: false };
  controls.enabled = false;
  brain.beginDrag(hit.index, e.clientX, e.clientY, camera, innerWidth, innerHeight);
  e.stopPropagation();
}, { capture: true });
canvas.addEventListener('pointermove', (e) => {
  if (!dragBrain) return;
  dragBrain.moved = true;
  brain.dragTo(e.clientX, e.clientY, camera, innerWidth, innerHeight);
});
canvas.addEventListener('pointerup', (e) => {
  if (!dragBrain) return;
  const hit = dragBrain.moved ? null : brain.pick(e.clientX, e.clientY, camera, innerWidth, innerHeight);
  brain.endDrag();
  dragBrain = null;
  controls.enabled = true;
  if (hit) openBrainCard(hit.node);
  e.stopImmediatePropagation();
});
canvas.addEventListener('pointerup', (e) => {
  if (!pointerDown || !swarm) return;
  const moved = Math.hypot(e.clientX - pointerDown.x, e.clientY - pointerDown.y);
  const quick = performance.now() - pointerDown.at < 500;
  pointerDown = null;
  if (moved > 6 || !quick) return;
  const heartAt = screenOfPoint(heartPosition());
  if (Math.hypot(e.clientX - heartAt.x, e.clientY - heartAt.y) < 30) return openAnima();
  const agent = swarm.pick(renderer, camera, e.clientX, e.clientY);
  if (agent) return openAgent(agent);
  const ndc = new THREE.Vector2((e.clientX / innerWidth) * 2 - 1, -(e.clientY / innerHeight) * 2 + 1);
  raycaster.setFromCamera(ndc, camera);
  if (atlas.ready && raycaster.intersectObject(atlas.root, true).length) {
    const hit = raycaster.intersectObject(atlas.root, true)[0];
    let node: THREE.Object3D | null = hit?.object ?? null;
    while (node) {
      if (/heart|ventricle|atrium|myocard/i.test(node.name)) return openAnima();
      node = node.parent;
    }
  }
  if (raycaster.intersectObject(body.heart, true).length) openAnima();
});

$('#rail-close').addEventListener('click', closeRail);
$('#anima-close').addEventListener('click', () => { animaPanel.classList.remove('open'); animaPanel.setAttribute('aria-hidden', 'true'); });
$('#anima-save').addEventListener('click', saveAnima);
const studio = mountStudio(toast);
$('#studio-btn').addEventListener('click', () => void studio.setOpen(!studio.isOpen()));
function applyHash() {
  const want = location.hash.replace(/^#\/?/, '') === 'studio';
  if (studio.isOpen() !== want) void studio.setOpen(want, false);
}
addEventListener('hashchange', applyHash);
if (location.hash === '#studio') void studio.setOpen(true, false);
else void studio.load().catch((e) => console.warn('studio unavailable', e));
$('#machine-btn').addEventListener('click', openMachine);
$('#machine-close').addEventListener('click', closeMachine);
$('#machine-save').addEventListener('click', () => { void saveMachine(); });
$('#machine-refresh').addEventListener('click', () => { void refreshMachine(); });
$('#machine-quality').addEventListener('change', () => {
  const preset = PRESETS[$<HTMLSelectElement>('#machine-quality').value] ?? PRESETS.balanced;
  $<HTMLInputElement>('#machine-ratio').value = String(preset[0]);
  $<HTMLInputElement>('#machine-particles').value = String(preset[1]);
});
document.querySelectorAll<HTMLButtonElement>('#tabs button').forEach((b) => b.addEventListener('click', () => setTab(b.dataset.tab!)));

const brainBtn = $('#brain-btn');
const dreamBtn = $('#dream-btn');
const brainCard = $('#brain-card');
const legend = $('#brain-legend');
const REGION_TINT = ['#7ee7ff', '#ffb86b', '#b9a2ff', '#7dffb0', '#ff7d9c'];

function renderLegend() {
  legend.replaceChildren();
  (brain.data?.regions ?? []).forEach((region, i) => {
    const item = el('span');
    const dot = el('i');
    dot.style.background = REGION_TINT[i % REGION_TINT.length];
    item.append(dot, el('span', undefined, region.id));
    legend.append(item);
  });
}

function openBrainCard(node: BrainNode) {
  $('#brain-card-title').textContent = node.title;
  $('#brain-card-meta').textContent =
    `${node.region} · ${node.agent_id ?? 'unassigned'} · ${new Date(node.created_at).toLocaleString()} · ${(node.importance * 100).toFixed(0)}% importance`;
  $('#brain-card-body').textContent = node.body;
  const links = brain.linksOf(node.id).map((n) => n.title).slice(0, 5);
  $('#brain-card-links').textContent = links.length ? `links: ${links.join(' · ')}` : 'no strong links yet';
  brain.selected = node;
  brainCard.classList.add('open');
  brainCard.setAttribute('aria-hidden', 'false');
}

function closeBrainCard() {
  brainCard.classList.remove('open');
  brainCard.setAttribute('aria-hidden', 'true');
  brain.selected = null;
}

async function setDream(on: boolean) {
  if (on === (brain.mode === 'dream')) return;
  brain.setMode(on ? 'dream' : 'wake');
  document.body.classList.toggle('dream-mode', on);
  dreamBtn.textContent = on ? 'wake' : 'dream';
  toast(on ? 'dream mode - topology relaxing' : 'wake mode - memory restored');
  if (!on) {
    try {
      const res = await fetch(apiUrl('/api/dream'), {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          summary: `dream run over ${brain.data?.nodes.length ?? 0} thoughts`,
          thought_count: brain.data?.nodes.length ?? 0,
          insights: brain.data?.edges.length ?? 0,
        }),
      });
      lastDream = await res.json();
    } catch (e) {
      console.warn('dream persistence failed', e);
    }
  }
}

function setBrainMode(on: boolean) {
  if (!brain.data) { toast('brain graph unavailable'); return; }
  brainMode = on;
  brain.setVisible(on);
  document.body.classList.toggle('brain-mode', on);
  document.body.classList.toggle('dream-mode', on && brain.mode === 'dream');
  brainBtn.classList.toggle('on', on);
  dreamBtn.classList.toggle('hidden', !on);
  legend.classList.toggle('hidden', !on);
  if (on) {
    flyTo(brain.center.clone().add(new THREE.Vector3(0.17, 0.02, 0.17)), brain.center, 900);
    $('#hint').textContent = 'thought graph - scroll zoom - drag a thought - click for content';
    void brain.refreshPulse();
    if (!brainPulseTimer) brainPulseTimer = setInterval(() => { void brain.refreshPulse(); }, 1500);
  } else {
    if (brainPulseTimer) { clearInterval(brainPulseTimer); brainPulseTimer = null; }
    if (brain.mode === 'dream') void setDream(false);
    closeBrainCard();
    flyTo(new THREE.Vector3(0, 0.12, 2.55), new THREE.Vector3(0, 0.08, 0), 900);
    $('#hint').textContent = 'drag orbit · scroll zoom';
  }
}

brainBtn.addEventListener('click', () => setBrainMode(!brainMode));
dreamBtn.addEventListener('click', () => { void setDream(brain.mode !== 'dream'); });
$('#brain-card-close').addEventListener('click', closeBrainCard);

const atlasLayers: AtlasLayer[] = ['skin', 'muscles', 'skeleton', 'nerves', 'vessels', 'organs', 'brain'];

function applyLayer(layer: LayerName, visible: boolean) {
  if (layer === 'agents') {
    swarm?.setVisible(visible);
    return;
  }
  if (atlas.ready && atlasLayers.includes(layer as AtlasLayer)) atlas.setLayer(layer as AtlasLayer, visible);
  else if (!atlas.ready && layer in body.layers) body.layers[layer as Exclude<LayerName, 'agents'>].visible = visible;
}

function applyOpacity(layer: LayerName, value: number) {
  if (layer === 'agents') return swarm?.setOpacity(value);
  if (atlas.ready && atlasLayers.includes(layer as AtlasLayer)) atlas.setOpacity(layer as AtlasLayer, value);
  else if (layer in body.layers) body.setOpacity(layer, value);
}

document.querySelectorAll<HTMLInputElement>('[data-layer]').forEach((input) => {
  input.addEventListener('change', () => applyLayer(input.dataset.layer as LayerName, input.checked));
});
document.querySelectorAll<HTMLInputElement>('[data-opacity]').forEach((slider) => {
  slider.addEventListener('input', () => applyOpacity(slider.dataset.opacity as LayerName, Number(slider.value) / 100));
});

function applyUiLayers() {
  document.querySelectorAll<HTMLInputElement>('[data-layer]').forEach((input) => {
    applyLayer(input.dataset.layer as LayerName, input.checked);
  });
  document.querySelectorAll<HTMLInputElement>('[data-opacity]').forEach((slider) => {
    applyOpacity(slider.dataset.opacity as LayerName, Number(slider.value) / 100);
  });
}

function addRoster(agents: Agent[]) {
  const roster = $('#roster');
  agents.forEach((agent) => {
    const button = el('button') as HTMLButtonElement;
    button.style.setProperty('--dot', agent.accent);
    button.append(el('i'), el('span', undefined, agent.name));
    button.addEventListener('click', () => openAgent(agent.id));
    roster.append(button);
  });
}

let last = performance.now();
let acc = 0;
let frames = 0;
function frame(now: number) {
  requestAnimationFrame(frame);
  const dt = Math.min(0.05, (now - last) / 1000);
  last = now;
  elapsed += dt;
  acc += dt;
  frames += 1;
  if (acc >= 0.5) {
    fps = frames / acc;
    acc = 0;
    frames = 0;
    $('#fps').textContent = `${fps.toFixed(0)} fps`;
  }
  if (tween) {
    tween.t = Math.min(1, tween.t + (dt * 1000) / tween.dur);
    const k = tween.t * tween.t * (3 - 2 * tween.t);
    camera.position.lerpVectors(tween.from, tween.to, k);
    controls.target.lerpVectors(tween.targetFrom, tween.targetTo, k);
    if (tween.t >= 1) tween = null;
  }
  controls.update();
  const distance = camera.position.distanceTo(controls.target);
  body.focus(distance);
  swarm?.update(dt);
  circulation.update(dt);
  brain.update(elapsed, camera, innerWidth, innerHeight);
  const beat = Math.sin(elapsed * 6.4);
  const pulse = Math.pow(Math.max(0, beat), 2);
  if (atlas.ready) atlas.beat(pulse);
  else body.heart.scale.setScalar(1 + 0.07 * pulse);
  const anchor = heartPosition();
  heartLight.position.copy(anchor).add(new THREE.Vector3(0, 0.02, 0.16));
  heartLight.intensity = 1.0 + 0.9 * pulse;
  cameraLight.position.copy(camera.position);
  if (!paused) renderer.render(scene, camera);
}

addEventListener('resize', () => {
  camera.aspect = innerWidth / innerHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(innerWidth, innerHeight);
});

async function boot() {
  anatomy = await getJson<Anatomy>('/api/anatomy', isTauri() ? 12 : 1);
  swarm = new Swarm(anatomy.agents);
  scene.add(swarm.group);
  addRoster(anatomy.agents);
  try {
    await brain.load();
    renderLegend();
  } catch (e) {
    console.warn('brain graph unavailable', e);
  }

  const hint = $('#hint');
  try {
    await atlas.load((label) => { hint.textContent = `loading anatomy · ${label}`; });
  } catch (e) {
    console.error(e);
  }
  if (atlas.ready) {
    fitAtlas();
    for (const layer of Object.values(body.layers)) layer.visible = false;
    scene.attach(body.heart);
    body.heart.visible = true;
    atlas.root.visible = true;
    circulation.group.visible = true;
    glb.heart = true;
    glb.brain = atlas.layers.get('brain')!.children.length > 0;
    hint.textContent = `real anatomy · ${(atlas.triangles / 1e6).toFixed(1)}M tris · drag orbit · scroll zoom · click heart or agent`;
  } else {
    hint.textContent = 'procedural fallback · atlas unavailable · drag orbit · scroll zoom';
    toast('anatomy atlas unavailable — procedural fallback');
  }
  applyUiLayers();
  ready = true;
  document.body.dataset.ready = '1';
  requestAnimationFrame(frame);
}

boot().catch((e) => { toast(`boot failed: ${e}`); console.error(e); });

declare global {
  interface Window { __noema: Record<string, (...args: any[]) => any> }
}
window.__noema = {
  ready: () => ready,
  fps: () => fps,
  cameraDistance: () => camera.position.distanceTo(controls.target),
  layerVisible: (layer: LayerName) => {
    if (layer === 'agents') return swarm?.visible ?? false;
    if (atlas.ready && atlasLayers.includes(layer as AtlasLayer)) return atlas.layerVisible(layer as AtlasLayer);
    return body.layers[layer as Exclude<LayerName, 'agents'>].visible;
  },
  heartScreen: () => screenOfPoint(heartPosition()),
  agentScreen: (id: string) => swarm?.screenOf(id, camera, renderer) ?? null,
  agentCount: () => anatomy?.agents.length ?? 0,
  greeting: (id: string) => anatomy?.agents.find((a) => a.id === id)?.greeting ?? null,
  zoomToCellular: () => {
    const p = heartPosition();
    atlas.setLayer('muscles', false);
    atlas.setOpacity('skeleton', 0.4);
    const muscles = document.querySelector<HTMLInputElement>('[data-layer="muscles"]');
    if (muscles) muscles.checked = false;
    flyTo(p.clone().add(new THREE.Vector3(0, 0.03, 0.36)), p, 1100);
  },
  glbLoaded: () => ({ ...glb }),
  atlasReady: () => atlas.ready,
  atlasStats: () => ({ systems: atlas.loadedSystems, triangles: Math.round(atlas.triangles), errors: atlas.errors }),
  debug: () => {
    const box = new THREE.Box3().setFromObject(atlas.root);
    const size = box.getSize(new THREE.Vector3());
    const center = box.getCenter(new THREE.Vector3());
    let meshes = 0;
    let visibleMeshes = 0;
    let transparent = 0;
    atlas.root.traverse((o) => {
      if (!(o as THREE.Mesh).isMesh) return;
      meshes += 1;
      if (o.visible) visibleMeshes += 1;
      const m = (o as THREE.Mesh).material as THREE.Material;
      if (m && m.transparent) transparent += 1;
    });
    return {
      rootVisible: atlas.root.visible, box: { min: box.min.toArray(), max: box.max.toArray() },
      size: size.toArray(), center: center.toArray(), meshes, visibleMeshes, transparent,
      camera: camera.position.toArray(), target: controls.target.toArray(),
      layers: Object.fromEntries([...atlas.layers].map(([k, g]) => [k, g.visible])),
    };
  },
  fit: () => fitAtlas(),
  pause: (on: boolean) => { paused = on; },
  direct: () => { renderer.render(scene, camera); },
  normals: (on: boolean) => { scene.overrideMaterial = on ? new THREE.MeshNormalMaterial({ side: THREE.DoubleSide }) : null; },
  sample: () => {
    const out: any[] = [];
    atlas.root.traverse((o) => {
      const mesh = o as THREE.Mesh;
      if (!mesh.isMesh || out.length >= 6) return;
      const m = mesh.material as THREE.MeshStandardMaterial;
      out.push({
        name: mesh.name, parent: mesh.parent?.name, visible: mesh.visible,
        type: m?.type, color: m?.color?.getHexString?.(), opacity: m?.opacity,
        transparent: m?.transparent, side: m?.side, map: !!m?.map, vertexColors: m?.vertexColors,
        metalness: m?.metalness, roughness: m?.roughness,
      });
    });
    return out;
  },
  renderInfo: () => ({ calls: renderer.info.render.calls, triangles: renderer.info.render.triangles, frame: renderer.info.render.frame }),
  railOpen: () => rail.classList.contains('open'),
  tabCount: () => document.querySelectorAll('#tabs button').length,
  animaOpen: () => animaPanel.classList.contains('open'),
  machineOpen: () => machinePanel.classList.contains('open'),
  machineProfile: () => machine,
  activeTab: () => document.querySelector('#tabs button.on')?.textContent ?? null,
  animaText: () => $<HTMLTextAreaElement>('#anima-identity').value,
  brain: () => ({
    available: !!brain.data,
    visible: brain.visible,
    mode: brain.mode,
    nodes: brain.data?.nodes.length ?? 0,
    nodeList: (brain.data?.nodes ?? []).map((n) => ({ id: n.id, title: n.title, region: n.region })),
    edges: brain.data?.edges.length ?? 0,
    labels: document.querySelectorAll('#brain-labels text').length,
    maxRadius: brain.maxRadius(),
    selected: brain.selected?.title ?? null,
    dream: lastDream,
    cameraDistance: camera.position.distanceTo(controls.target),
  }),
  brainScreen: (id: string) => brain.screenOfId(id, camera, innerWidth, innerHeight),
  brainMode: (on: boolean) => setBrainMode(on),
  dreamMode: (on: boolean) => setDream(on),
  studio: () => studio.snapshot(),
  studioOpen: (on: boolean) => studio.setOpen(on),
  platform: () => ({ tauri: isTauri(), apiBase: apiBase(), assetBase: assetBase(), hash: location.hash }),
};
