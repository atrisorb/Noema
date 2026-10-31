import { readFileSync } from 'node:fs';
for (const path of process.argv.slice(2)) {
  const buf = readFileSync(path);
  const jsonLen = buf.readUInt32LE(12);
  const g = JSON.parse(buf.subarray(20, 20 + jsonLen).toString('utf8'));
  const mats = (g.materials ?? []).map((m) => ({
    name: m.name,
    color: m.pbrMetallicRoughness?.baseColorFactor?.map((v) => +v.toFixed(3)),
    tex: !!m.pbrMetallicRoughness?.baseColorTexture,
    alpha: m.alphaMode ?? 'OPAQUE',
  }));
  let tris = 0;
  for (const m of g.meshes ?? []) for (const p of m.primitives) {
    const idx = p.indices != null ? g.accessors[p.indices]?.count ?? 0 : g.accessors[p.attributes.POSITION]?.count ?? 0;
    tris += idx / 3;
  }
  console.log('===', path, '| nodes', g.nodes?.length, '| meshes', g.meshes?.length, '| tris', Math.round(tris), '| images', g.images?.length);
  console.log('nodes:', (g.nodes ?? []).map((n) => n.name).join(' | ').slice(0, 1200));
  console.log('materials:', JSON.stringify(mats).slice(0, 1200));
  console.log('scenes:', JSON.stringify(g.scenes));
}
