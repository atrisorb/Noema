/// <reference lib="webworker" />
// Mirrors crates/hippocampus/src/physics.rs: Hooke springs, repulsion, damping
// and a hard tether clamp. Runs at 60 fps off the main thread.

interface Edge { a: number; b: number; weight: number }
interface Params { repulsion: number; spring: number; damping: number; tether: number; maxSpeed: number; rest: number }

const wake = (): Params => ({ repulsion: 0.02, spring: 1.2, damping: 0.86, tether: 1, maxSpeed: 0.35, rest: 0.32 });
const dream = (): Params => ({ repulsion: 0.05, spring: 0.35, damping: 0.94, tether: 1, maxSpeed: 0.12, rest: 0.32 });

let n = 0;
let pos = new Float32Array(0);
let vel = new Float32Array(0);
let edges: Edge[] = [];
let params = wake();
let pinned = -1;
const pin: number[] = [0, 0, 0];
let timer: ReturnType<typeof setTimeout> | null = null;

const SPATIAL_MIN = 200;

function repel(i: number, j: number, f: Float32Array) {
  const dx = pos[i * 3] - pos[j * 3];
  const dy = pos[i * 3 + 1] - pos[j * 3 + 1];
  const dz = pos[i * 3 + 2] - pos[j * 3 + 2];
  const k = params.repulsion / Math.max(0.0004, dx * dx + dy * dy + dz * dz);
  f[i * 3] += dx * k; f[i * 3 + 1] += dy * k; f[i * 3 + 2] += dz * k;
  f[j * 3] -= dx * k; f[j * 3 + 1] -= dy * k; f[j * 3 + 2] -= dz * k;
}

function repulsionAll(f: Float32Array) {
  for (let i = 0; i < n; i += 1) {
    for (let j = i + 1; j < n; j += 1) repel(i, j, f);
  }
}

// Only neighbours within one cell interact above 200 nodes; cell size tracks
// mean spacing so the bucket count stays near 27 per node.
function repulsionGrid(f: Float32Array) {
  const cell = Math.min(0.35, Math.max(0.08, Math.cbrt(4.2 / n)));
  const grid = new Map<string, number[]>();
  for (let i = 0; i < n; i += 1) {
    const key = `${Math.floor(pos[i * 3] / cell)},${Math.floor(pos[i * 3 + 1] / cell)},${Math.floor(pos[i * 3 + 2] / cell)}`;
    const bucket = grid.get(key);
    if (bucket) bucket.push(i); else grid.set(key, [i]);
  }
  for (const [key, bucket] of grid) {
    const [cx, cy, cz] = key.split(',').map(Number);
    for (const i of bucket) {
      for (let dx = -1; dx <= 1; dx += 1) {
        for (let dy = -1; dy <= 1; dy += 1) {
          for (let dz = -1; dz <= 1; dz += 1) {
            const neighbours = grid.get(`${cx + dx},${cy + dy},${cz + dz}`);
            if (!neighbours) continue;
            for (const j of neighbours) if (j > i) repel(i, j, f);
          }
        }
      }
    }
  }
}

function step(dt: number) {
  if (!n) return;
  const f = new Float32Array(n * 3);
  if (n > SPATIAL_MIN) repulsionGrid(f); else repulsionAll(f);
  for (const e of edges) {
    const a = e.a * 3;
    const b = e.b * 3;
    const dx = pos[b] - pos[a];
    const dy = pos[b + 1] - pos[a + 1];
    const dz = pos[b + 2] - pos[a + 2];
    const len = Math.max(0.0001, Math.hypot(dx, dy, dz));
    const p = params.spring * (len - params.rest) * e.weight / len;
    f[a] += dx * p; f[a + 1] += dy * p; f[a + 2] += dz * p;
    f[b] -= dx * p; f[b + 1] -= dy * p; f[b + 2] -= dz * p;
  }
  for (let i = 0; i < n; i += 1) {
    if (i === pinned) {
      const len = Math.hypot(pin[0], pin[1], pin[2]);
      const s = len > params.tether ? params.tether / len : 1;
      for (let c = 0; c < 3; c += 1) { pos[i * 3 + c] = pin[c] * s; vel[i * 3 + c] = 0; }
      continue;
    }
    for (let c = 0; c < 3; c += 1) {
      vel[i * 3 + c] = (vel[i * 3 + c] + f[i * 3 + c] * dt) * params.damping;
      pos[i * 3 + c] += vel[i * 3 + c] * dt;
    }
    const len = Math.hypot(pos[i * 3], pos[i * 3 + 1], pos[i * 3 + 2]);
    if (len > params.tether) {
      const s = params.tether / len;
      for (let c = 0; c < 3; c += 1) { pos[i * 3 + c] *= s; vel[i * 3 + c] *= s; }
    }
    const speed = Math.hypot(vel[i * 3], vel[i * 3 + 1], vel[i * 3 + 2]);
    if (speed > params.maxSpeed) {
      const s = params.maxSpeed / speed;
      for (let c = 0; c < 3; c += 1) vel[i * 3 + c] *= s;
    }
  }
}

function post() {
  const copy = pos.slice();
  (self as unknown as Worker).postMessage({ type: 'positions', positions: copy }, [copy.buffer]);
}

function loop() {
  step(1 / 60);
  post();
  timer = setTimeout(loop, 16);
}

self.onmessage = (event: MessageEvent) => {
  const msg = event.data;
  if (msg.type === 'init') {
    n = msg.positions.length / 3;
    pos = Float32Array.from(msg.positions);
    vel = new Float32Array(n * 3);
    edges = msg.edges;
    if (!timer) loop();
  } else if (msg.type === 'mode') {
    params = msg.dream ? dream() : wake();
  } else if (msg.type === 'pin') {
    pinned = msg.index;
    if (msg.index >= 0) { pin[0] = msg.pos[0]; pin[1] = msg.pos[1]; pin[2] = msg.pos[2]; }
  }
};
