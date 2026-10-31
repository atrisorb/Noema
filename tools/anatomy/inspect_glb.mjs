#!/usr/bin/env node
// Summarise a GLB without loading three: materials, meshes, nodes, extensions, bounds.
import { readFileSync } from 'node:fs';

for (const path of process.argv.slice(2)) {
  const buf = readFileSync(path);
  if (buf.readUInt32LE(0) !== 0x46546c67) throw new Error(`${path}: not a GLB`);
  const jsonLen = buf.readUInt32LE(12);
  const gltf = JSON.parse(buf.subarray(20, 20 + jsonLen).toString('utf8'));
  const nodes = gltf.nodes?.length ?? 0;
  const meshes = gltf.meshes?.length ?? 0;
  const prims = (gltf.meshes ?? []).reduce((n, m) => n + m.primitives.length, 0);
  const mats = (gltf.materials ?? []).map((m) => m.name ?? '(unnamed)');
  const used = new Set([
    ...(gltf.extensionsUsed ?? []),
    ...((gltf.meshes ?? []).flatMap((m) => m.primitives.flatMap((p) => Object.keys(p.extensions ?? {})))),
  ]);
  const pos = (gltf.accessors ?? []).filter((a) => a.type === 'VEC3' && a.min && a.max);
  const min = pos.length ? [0, 1, 2].map((i) => Math.min(...pos.map((a) => a.min[i]))) : null;
  const max = pos.length ? [0, 1, 2].map((i) => Math.max(...pos.map((a) => a.max[i]))) : null;
  const names = (gltf.nodes ?? []).map((n) => n.name).filter(Boolean);
  console.log(JSON.stringify({
    file: path,
    bytes: buf.length,
    nodes,
    meshes,
    prims,
    materials: mats.length,
    materialSample: mats.slice(0, 12),
    extensions: [...used],
    bounds: min && max ? { min: min.map((v) => +v.toFixed(2)), max: max.map((v) => +v.toFixed(2)) } : null,
    nodeSample: names.slice(0, 15),
  }));
}
