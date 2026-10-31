#!/usr/bin/env node
// World-space mesh inventory for body_full.glb. No three.js dependency.
import { readFileSync } from 'node:fs';

const file = process.argv[2] ?? 'assets/anatomy/atlas/body_full.glb';
const buf = readFileSync(file);
if (buf.readUInt32LE(0) !== 0x46546c67) throw new Error('not a GLB');
const jsonLen = buf.readUInt32LE(12);
const g = JSON.parse(buf.subarray(20, 20 + jsonLen).toString('utf8'));
const nodes = g.nodes ?? [];
const meshes = g.meshes ?? [];
const acc = g.accessors ?? [];

function mat4FromNode(n) {
  if (n.matrix) return n.matrix.slice();
  const t = n.translation ?? [0, 0, 0];
  const r = n.rotation ?? [0, 0, 0, 1];
  const s = n.scale ?? [1, 1, 1];
  const [x, y, z, w] = r;
  const x2 = x + x, y2 = y + y, z2 = z + z;
  const xx = x * x2, xy = x * y2, xz = x * z2;
  const yy = y * y2, yz = y * z2, zz = z * z2;
  const wx = w * x2, wy = w * y2, wz = w * z2;
  const [sx, sy, sz] = s;
  return [
    (1 - (yy + zz)) * sx, (xy + wz) * sx, (xz - wy) * sx, 0,
    (xy - wz) * sy, (1 - (xx + zz)) * sy, (yz + wx) * sy, 0,
    (xz + wy) * sz, (yz - wx) * sz, (1 - (xx + yy)) * sz, 0,
    t[0], t[1], t[2], 1,
  ];
}
function mul(a, b) {
  const o = new Array(16).fill(0);
  for (let c = 0; c < 4; c += 1) for (let r = 0; r < 4; r += 1) {
    let sum = 0;
    for (let k = 0; k < 4; k += 1) sum += a[k * 4 + r] * b[c * 4 + k];
    o[c * 4 + r] = sum;
  }
  return o;
}
function apply(m, p) {
  return [
    m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12],
    m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13],
    m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14],
  ];
}
const world = new Map();
function walk(i, parent) {
  const n = nodes[i];
  const m = mul(parent, mat4FromNode(n));
  world.set(i, m);
  for (const c of n.children ?? []) walk(c, m);
}
const I = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
for (const r of g.scenes?.[g.scene ?? 0]?.nodes ?? [0]) walk(r, I);

const rows = [];
const gmin = [1e9, 1e9, 1e9];
const gmax = [-1e9, -1e9, -1e9];
for (let i = 0; i < nodes.length; i += 1) {
  const n = nodes[i];
  if (n.mesh == null) continue;
  const m = world.get(i);
  const mesh = meshes[n.mesh];
  const prim = mesh?.primitives?.[0];
  const pa = prim ? acc[prim.attributes?.POSITION] : null;
  if (!m || !pa?.min || !pa?.max) continue;
  const mn = [1e9, 1e9, 1e9];
  const mx = [-1e9, -1e9, -1e9];
  for (const x of [pa.min[0], pa.max[0]]) for (const y of [pa.min[1], pa.max[1]]) for (const z of [pa.min[2], pa.max[2]]) {
    const p = apply(m, [x, y, z]);
    for (let k = 0; k < 3; k += 1) {
      mn[k] = Math.min(mn[k], p[k]);
      mx[k] = Math.max(mx[k], p[k]);
      gmin[k] = Math.min(gmin[k], p[k]);
      gmax[k] = Math.max(gmax[k], p[k]);
    }
  }
  const tris = prim.indices != null ? acc[prim.indices].count / 3 : pa.count / 3;
  rows.push({ node: i, name: n.name || mesh.name || '', tris, min: mn, max: mx });
}
const round = (v) => v.map((x) => +x.toFixed(3));
console.log('FILE', file);
console.log('GLOBAL min', round(gmin), 'max', round(gmax), 'height', (gmax[1] - gmin[1]).toFixed(3), 'meshes', rows.length);
const kw = /(skin|cutis|epiderm|derm|subcut|fascia|integument)/i;
console.log('SKIN/FASCIA', rows.filter((r) => kw.test(r.name)).map((r) => r.name + ' ' + r.tris).join(' | ') || 'NONE');
const regions = {
  legL: (r) => /tibia|fibula|femur|patella|leg|soleus|gastrocnem|tibialis|fibularis|perone|poplite|sural|sartorius|adductor/i.test(r.name) && r.max[0] < 0.02,
  legR: (r) => /tibia|fibula|femur|patella|leg|soleus|gastrocnem|tibialis|fibularis|perone|poplite|sural|sartorius|adductor/i.test(r.name) && r.max[0] > 0.02,
  footL: (r) => /tarsal|metatars|phalanx of .*foot|calcane|talus|navicular|cuboid|cuneiform|sesamoid/i.test(r.name) && r.max[0] < 0.02,
  footR: (r) => /tarsal|metatars|phalanx of .*foot|calcane|talus|navicular|cuboid|cuneiform|sesamoid/i.test(r.name) && r.max[0] > 0.02,
  handL: (r) => /carpal|metacarp|phalanx of .*hand|scaphoid|lunate|triquet|pisiform|trapez|capitate|hamate|palmar|thenar|hypothenar|digiti|interossei|lumbrical/i.test(r.name) && r.max[0] < 0.02,
  handR: (r) => /carpal|metacarp|phalanx of .*hand|scaphoid|lunate|triquet|pisiform|trapez|capitate|hamate|palmar|thenar|hypothenar|digiti|interossei|lumbrical/i.test(r.name) && r.max[0] > 0.02,
};
for (const [key, test] of Object.entries(regions)) {
  const list = rows.filter(test);
  const mn = [1e9, 1e9, 1e9];
  const mx = [-1e9, -1e9, -1e9];
  let tris = 0;
  for (const r of list) {
    tris += r.tris;
    for (let k = 0; k < 3; k += 1) { mn[k] = Math.min(mn[k], r.min[k]); mx[k] = Math.max(mx[k], r.max[k]); }
  }
  console.log(key, 'meshes', list.length, 'tris', Math.round(tris), 'min', round(mn), 'max', round(mx));
}
console.log('TOP TRIANGLES');
rows.slice().sort((a, b) => b.tris - a.tris).slice(0, 15).forEach((r) => console.log('  ', r.tris, r.name));
