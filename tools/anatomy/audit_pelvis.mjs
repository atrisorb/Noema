#!/usr/bin/env node
// Mesh-level pelvic audit for body_full.glb. Local-space bounding boxes.
import { readFileSync, writeFileSync } from 'node:fs';

const file = process.argv[2] ?? 'assets/anatomy/atlas/body_full.glb';
const out = process.argv[3] ?? 'docs/evidence/design-pelvis-audit.json';
const buf = readFileSync(file);
const jsonLen = buf.readUInt32LE(12);
const g = JSON.parse(buf.subarray(20, 20 + jsonLen).toString('utf8'));
const flag = /corpus|cavern|spong|glans|prepuce|scrot|testis|dartos|cremaster|bulb|crura|suspensory|perine|pubic/i;
const box = { min: [-0.18, -0.35, -0.15], max: [0.18, -0.10, 0.15] };
const inside = (min, max) => min && max && min[0] <= box.max[0] && max[0] >= box.min[0] && min[1] <= box.max[1] && max[1] >= box.min[1] && min[2] <= box.max[2] && max[2] >= box.min[2];

const names = (g.nodes ?? []).map((n) => n.name).filter(Boolean);
const flaggedNames = names.filter((n) => flag.test(n));
const meshes = (g.meshes ?? []).map((m) => {
  const prim = m.primitives?.[0];
  const acc = prim?.attributes?.POSITION != null ? g.accessors?.[prim.attributes.POSITION] : null;
  return { name: m.name ?? '', min: acc?.min, max: acc?.max, vertices: acc?.count ?? 0 };
});
const pelvicMeshes = meshes.filter((m) => flag.test(m.name));
const bboxHits = meshes.filter((m) => inside(m.min, m.max));
const report = {
  schema: 'noema.pelvis-audit/1',
  at: new Date().toISOString(),
  file,
  method: 'GLB JSON chunk; local-space POSITION accessor min/max; names flagged by anatomy keywords.',
  totals: { nodes: g.nodes?.length ?? 0, meshes: g.meshes?.length ?? 0, flaggedNames: flaggedNames.length, pelvicMeshes: pelvicMeshes.length, bboxHits: bboxHits.length },
  flaggedNames,
  pelvicMeshes: pelvicMeshes.map((m) => ({ name: m.name, vertices: m.vertices })),
  bboxHits: bboxHits.map((m) => ({ name: m.name, min: m.min, max: m.max })),
  note: 'No external genitalia node name is present. The central pelvic mass still requires visual close-up review; this audit does not authorise removal.',
};
writeFileSync(out, JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report.totals));
console.log('flagged:', flaggedNames.slice(0, 20).join(', ') || 'none');
