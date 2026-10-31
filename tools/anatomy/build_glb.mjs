#!/usr/bin/env node
// Procedural anatomy GLBs: zero-cost, self-authored, CC0. No Blender needed.
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const outDir = join(here, '..', '..', 'assets', 'anatomy');

const align4 = (n) => (n + 3) & ~3;

function blob(segU, segV, scale, shape) {
  const positions = [];
  const normals = [];
  const indices = [];
  for (let v = 0; v <= segV; v += 1) {
    const phi = (v / segV) * Math.PI;
    for (let u = 0; u <= segU; u += 1) {
      const theta = (u / segU) * Math.PI * 2;
      const dir = [Math.sin(phi) * Math.cos(theta), Math.cos(phi), Math.sin(phi) * Math.sin(theta)];
      const r = shape(theta, phi);
      positions.push(dir[0] * r * scale[0], dir[1] * r * scale[1], dir[2] * r * scale[2]);
      const n = [dir[0] / scale[0], dir[1] / scale[1], dir[2] / scale[2]];
      const len = Math.hypot(...n) || 1;
      normals.push(n[0] / len, n[1] / len, n[2] / len);
    }
  }
  for (let v = 0; v < segV; v += 1) {
    for (let u = 0; u < segU; u += 1) {
      const a = v * (segU + 1) + u;
      const b = a + 1;
      const c = a + segU + 1;
      const d = c + 1;
      indices.push(a, c, b, b, c, d);
    }
  }
  return { positions, normals, indices };
}

function glb(mesh, color, name) {
  const pos = new Float32Array(mesh.positions);
  const nor = new Float32Array(mesh.normals);
  const idx = new Uint32Array(mesh.indices);
  const posBytes = pos.byteLength;
  const norBytes = nor.byteLength;
  const idxBytes = idx.byteLength;
  const bin = Buffer.alloc(align4(posBytes) + align4(norBytes) + align4(idxBytes));
  Buffer.from(pos.buffer).copy(bin, 0);
  Buffer.from(nor.buffer).copy(bin, align4(posBytes));
  Buffer.from(idx.buffer).copy(bin, align4(posBytes) + align4(norBytes));
  const min = [Infinity, Infinity, Infinity];
  const max = [-Infinity, -Infinity, -Infinity];
  for (let i = 0; i < pos.length; i += 3) {
    for (let k = 0; k < 3; k += 1) {
      min[k] = Math.min(min[k], pos[i + k]);
      max[k] = Math.max(max[k], pos[i + k]);
    }
  }
  const json = {
    asset: { version: '2.0', generator: 'noema-anatomy 0.3.0' },
    scene: 0,
    scenes: [{ nodes: [0] }],
    nodes: [{ name, mesh: 0 }],
    meshes: [{ name, primitives: [{ attributes: { POSITION: 0, NORMAL: 1 }, indices: 2, material: 0 }] }],
    materials: [{
      name: 'tissue',
      pbrMetallicRoughness: { baseColorFactor: [...color, 1], metallicFactor: 0.0, roughnessFactor: 0.55 },
      doubleSided: true,
    }],
    accessors: [
      { bufferView: 0, componentType: 5126, count: pos.length / 3, type: 'VEC3', min, max },
      { bufferView: 1, componentType: 5126, count: nor.length / 3, type: 'VEC3' },
      { bufferView: 2, componentType: 5125, count: idx.length, type: 'SCALAR' },
    ],
    bufferViews: [
      { buffer: 0, byteOffset: 0, byteLength: posBytes, target: 34962 },
      { buffer: 0, byteOffset: align4(posBytes), byteLength: norBytes, target: 34962 },
      { buffer: 0, byteOffset: align4(posBytes) + align4(norBytes), byteLength: idxBytes, target: 34963 },
    ],
    buffers: [{ byteLength: bin.length }],
  };
  let jsonBuf = Buffer.from(JSON.stringify(json), 'utf8');
  jsonBuf = Buffer.concat([jsonBuf, Buffer.alloc(align4(jsonBuf.length) - jsonBuf.length, 0x20)]);
  const total = 12 + 8 + jsonBuf.length + 8 + bin.length;
  const header = Buffer.alloc(12);
  header.writeUInt32LE(0x46546c67, 0);
  header.writeUInt32LE(2, 4);
  header.writeUInt32LE(total, 8);
  const jsonHead = Buffer.alloc(8);
  jsonHead.writeUInt32LE(jsonBuf.length, 0);
  jsonHead.writeUInt32LE(0x4e4f534a, 4);
  const binHead = Buffer.alloc(8);
  binHead.writeUInt32LE(bin.length, 0);
  binHead.writeUInt32LE(0x004e4942, 4);
  return { bytes: Buffer.concat([header, jsonHead, jsonBuf, binHead, bin]), json, vertices: pos.length / 3, triangles: idx.length / 3 };
}

const heart = blob(48, 32, [0.09, 0.108, 0.078], (theta, phi) => {
  const groove = 1 - 0.10 * Math.exp(-Math.pow((phi - Math.PI / 2) * 3.2, 2)) * Math.abs(Math.cos(theta));
  const lobes = 1 + 0.09 * Math.sin(3 * theta) * Math.sin(phi) - 0.16 * Math.max(0, Math.cos(phi)) * Math.abs(Math.sin(theta));
  return groove * lobes;
});
const brain = blob(48, 32, [0.086, 0.076, 0.096], (theta, phi) => {
  return 1 + 0.045 * Math.sin(9 * theta) * Math.sin(7 * phi) + 0.02 * Math.cos(13 * theta + 2 * phi);
});

mkdirSync(outDir, { recursive: true });
for (const [file, mesh, color, name] of [
  ['heart.glb', heart, [0.85, 0.18, 0.28], 'heart'],
  ['brain.glb', brain, [0.86, 0.68, 0.76], 'brain'],
]) {
  const result = glb(mesh, color, name);
  writeFileSync(join(outDir, file), result.bytes);
  console.log(`${file} ${result.vertices} verts, ${result.triangles} tris, ${result.bytes.length} bytes`);
  const check = JSON.parse(result.bytes.subarray(20, 20 + result.bytes.readUInt32LE(12)).toString('utf8'));
  if (check.asset.version !== '2.0' || check.accessors[0].count !== result.vertices) throw new Error(`${file}: bad GLB`);
  if (check.accessors[2].count !== result.triangles * 3) throw new Error(`${file}: bad indices`);
  const maxIndex = Math.max(...mesh.indices);
  if (maxIndex >= result.vertices) throw new Error(`${file}: index out of range`);
}
console.log('OK all GLBs parse and are index-safe');
