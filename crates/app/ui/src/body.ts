import * as THREE from 'three';
import type { LayerName } from './types';

const BONE = 0xd8d2c0;
const NERVE = 0xffe066;
const ARTERY = 0xff5d73;
const VEIN = 0x5b8cff;

export interface Body {
  layers: Record<Exclude<LayerName, 'agents'>, THREE.Group>;
  heart: THREE.Group;
  brain: THREE.Group;
  setOpacity(layer: LayerName, value: number): void;
  focus(distance: number): void;
}

const vec = (x: number, y: number, z: number) => new THREE.Vector3(x, y, z);

export function buildBody(): Body {
  const layers = {
    skin: new THREE.Group(),
    skeleton: new THREE.Group(),
    nerves: new THREE.Group(),
    vessels: new THREE.Group(),
    organs: new THREE.Group(),
  };
  const mats: Partial<Record<LayerName, THREE.Material[]>> = {};
  const track = (layer: LayerName, mat: THREE.Material) => {
    mat.userData.base = mat.opacity;
    (mats[layer] ??= []).push(mat);
    return mat;
  };

  const skinMat = track('skin', new THREE.MeshPhysicalMaterial({
    color: 0x9fd8ff, roughness: 0.28, metalness: 0, transparent: true,
    opacity: 0.12, depthWrite: false, side: THREE.DoubleSide, clearcoat: 0.4,
  }));
  const boneMat = track('skeleton', new THREE.MeshStandardMaterial({ color: BONE, roughness: 0.7, metalness: 0.05 }));
  const nerveMat = track('nerves', new THREE.MeshStandardMaterial({ color: NERVE, roughness: 0.45, emissive: NERVE, emissiveIntensity: 0.35 }));
  const arteryMat = track('vessels', new THREE.MeshStandardMaterial({ color: ARTERY, roughness: 0.4, emissive: ARTERY, emissiveIntensity: 0.25 }));
  const veinMat = track('vessels', new THREE.MeshStandardMaterial({ color: VEIN, roughness: 0.4, emissive: VEIN, emissiveIntensity: 0.2 }));
  const organMat = (color: number, opacity: number) =>
    track('organs', new THREE.MeshStandardMaterial({ color, roughness: 0.5, transparent: true, opacity }));

  const put = (layer: keyof typeof layers, geo: THREE.BufferGeometry, mat: THREE.Material, at?: THREE.Vector3) => {
    const mesh = new THREE.Mesh(geo, mat);
    if (at) mesh.position.copy(at);
    layers[layer].add(mesh);
    return mesh;
  };

  const tube = (layer: keyof typeof layers, points: THREE.Vector3[], radius: number, mat: THREE.Material) => {
    const curve = new THREE.CatmullRomCurve3(points);
    return put(layer, new THREE.TubeGeometry(curve, Math.max(16, points.length * 6), radius, 6, false), mat);
  };

  const capsule = (layer: keyof typeof layers, a: THREE.Vector3, b: THREE.Vector3, radius: number, mat: THREE.Material) => {
    const dir = b.clone().sub(a);
    const mesh = put(layer, new THREE.CapsuleGeometry(radius, dir.length(), 4, 10), mat, a.clone().add(b).multiplyScalar(0.5));
    mesh.quaternion.setFromUnitVectors(vec(0, 1, 0), dir.normalize());
    return mesh;
  };

  // Skin: translucent shell, faded as the camera approaches the tissue.
  const torso = put('skin', new THREE.SphereGeometry(1, 32, 24), skinMat, vec(0, 0.16, 0));
  torso.scale.set(0.235, 0.40, 0.145);
  const pelvis = put('skin', new THREE.SphereGeometry(1, 24, 18), skinMat, vec(0, -0.22, 0));
  pelvis.scale.set(0.16, 0.16, 0.12);
  const head = put('skin', new THREE.SphereGeometry(0.104, 28, 22), skinMat, vec(0, 0.80, 0.005));
  head.scale.set(1, 1.16, 1.05);
  capsule('skin', vec(0, 0.66, 0), vec(0, 0.72, 0), 0.045, skinMat);
  for (const side of [-1, 1]) {
    capsule('skin', vec(side * 0.21, 0.46, 0), vec(side * 0.33, 0.20, 0.02), 0.043, skinMat);
    capsule('skin', vec(side * 0.33, 0.20, 0.02), vec(side * 0.39, -0.03, 0.03), 0.033, skinMat);
    capsule('skin', vec(side * 0.09, -0.26, 0), vec(side * 0.12, -0.60, 0.01), 0.055, skinMat);
    capsule('skin', vec(side * 0.12, -0.60, 0.01), vec(side * 0.12, -0.90, 0.02), 0.042, skinMat);
    capsule('skin', vec(side * 0.12, -0.91, 0.02), vec(side * 0.12, -0.92, 0.09), 0.04, skinMat);
  }

  // Skeleton.
  tube('skeleton', [vec(0, -0.30, 0.02), vec(0, 0.0, 0.0), vec(0, 0.30, -0.02), vec(0, 0.52, -0.01), vec(0, 0.64, 0.0)], 0.017, boneMat);
  const skull = put('skeleton', new THREE.SphereGeometry(0.098, 24, 18), boneMat, vec(0, 0.80, 0.004));
  skull.scale.set(1, 1.14, 1.02);
  for (let i = 0; i < 8; i += 1) {
    const rib = put('skeleton', new THREE.TorusGeometry(0.118 + i * 0.006, 0.0024, 4, 28, Math.PI * 1.05), boneMat, vec(0, 0.45 - i * 0.052, 0));
    rib.rotation.set(Math.PI / 2, -0.5, 0);
    rib.scale.set(1, 1, 0.62);
  }
  put('skeleton', new THREE.TorusGeometry(0.14, 0.011, 5, 24, Math.PI * 2), boneMat, vec(0, -0.28, 0)).rotation.x = Math.PI / 2;
  for (const side of [-1, 1]) {
    capsule('skeleton', vec(side * 0.20, 0.47, 0), vec(side * 0.32, 0.21, 0.02), 0.011, boneMat);
    capsule('skeleton', vec(side * 0.32, 0.21, 0.02), vec(side * 0.385, -0.02, 0.03), 0.009, boneMat);
    capsule('skeleton', vec(side * 0.09, -0.28, 0), vec(side * 0.12, -0.60, 0.01), 0.013, boneMat);
    capsule('skeleton', vec(side * 0.12, -0.60, 0.01), vec(side * 0.12, -0.90, 0.02), 0.011, boneMat);
  }

  // Nerves: spine to limbs and skull.
  for (const side of [-1, 1]) {
    tube('nerves', [vec(0, 0.45, 0), vec(side * 0.16, 0.42, 0.01), vec(side * 0.28, 0.24, 0.02), vec(side * 0.38, -0.01, 0.02)], 0.0028, nerveMat);
    tube('nerves', [vec(0, -0.20, 0.01), vec(side * 0.09, -0.42, 0.02), vec(side * 0.12, -0.66, 0.02), vec(side * 0.12, -0.88, 0.02)], 0.0026, nerveMat);
    tube('nerves', [vec(0, 0.52, 0), vec(side * 0.04, 0.63, 0.01), vec(side * 0.06, 0.74, 0.01)], 0.0022, nerveMat);
  }

  // Vessels.
  tube('vessels', [vec(0, -0.26, 0.01), vec(0, 0.10, 0.02), vec(0, 0.34, 0.05), vec(0, 0.50, 0.03), vec(0, 0.63, 0.02)], 0.0075, arteryMat);
  tube('vessels', [vec(0.015, 0.50, 0.03), vec(0.03, 0.62, 0.01), vec(0.025, 0.72, 0.01)], 0.0035, arteryMat);
  tube('vessels', [vec(-0.015, 0.50, 0.03), vec(-0.03, 0.62, 0.01), vec(-0.025, 0.72, 0.01)], 0.0035, arteryMat);
  for (const side of [-1, 1]) {
    tube('vessels', [vec(side * 0.06, 0.02, 0.01), vec(side * 0.08, -0.32, 0.02), vec(side * 0.11, -0.62, 0.02), vec(side * 0.12, -0.86, 0.02)], 0.0042, arteryMat);
    tube('vessels', [vec(side * 0.02, 0.03, -0.01), vec(side * 0.05, -0.34, 0.0), vec(side * 0.08, -0.64, 0.01)], 0.005, veinMat);
  }

  // Organs.
  const lungMat = organMat(0xff9ec4, 0.62);
  for (const side of [-1, 1]) {
    const lung = put('organs', new THREE.SphereGeometry(1, 20, 16), lungMat, vec(side * 0.105, 0.34, 0));
    lung.scale.set(0.075, 0.15, 0.06);
  }
  const liver = put('organs', new THREE.SphereGeometry(1, 20, 16), organMat(0xb46a4a, 0.9), vec(-0.10, 0.10, 0.02));
  liver.scale.set(0.09, 0.06, 0.055);
  const stomach = put('organs', new THREE.SphereGeometry(1, 18, 14), organMat(0xd9a06b, 0.85), vec(0.09, 0.08, 0.02));
  stomach.scale.set(0.05, 0.075, 0.04);
  for (const side of [-1, 1]) {
    const kidney = put('organs', new THREE.SphereGeometry(1, 16, 12), organMat(0x9c5b5b, 0.9), vec(side * 0.105, -0.04, -0.03));
    kidney.scale.set(0.035, 0.06, 0.028);
  }

  const heart = new THREE.Group();
  heart.position.set(0.012, 0.30, 0.055);
  const heartGeo = new THREE.SphereGeometry(0.072, 36, 26);
  const pos = heartGeo.attributes.position;
  for (let i = 0; i < pos.count; i += 1) {
    const v = vec(pos.getX(i), pos.getY(i), pos.getZ(i));
    const taper = 1 - 0.35 * Math.max(0, -v.y / 0.072);
    v.x *= taper * (1 + 0.06 * Math.sin(v.y * 40));
    v.y *= 1.22;
    v.z *= 0.9;
    pos.setXYZ(i, v.x, v.y, v.z);
  }
  heartGeo.computeVertexNormals();
  const heartMesh = new THREE.Mesh(heartGeo, organMat(0xff3b5c, 0.96));
  heart.add(heartMesh);
  const aortic = new THREE.Mesh(new THREE.CylinderGeometry(0.012, 0.016, 0.14, 10), arteryMat);
  aortic.position.set(0, 0.10, -0.01);
  heart.add(aortic);
  layers.organs.add(heart);

  const brain = new THREE.Group();
  brain.position.set(0, 0.80, 0.006);
  const brainGeo = new THREE.SphereGeometry(0.082, 32, 24);
  const bpos = brainGeo.attributes.position;
  for (let i = 0; i < bpos.count; i += 1) {
    const v = vec(bpos.getX(i), bpos.getY(i), bpos.getZ(i));
    v.multiplyScalar(1 + 0.05 * Math.sin(v.x * 90) * Math.sin(v.y * 80) * Math.sin(v.z * 70));
    bpos.setXYZ(i, v.x, v.y, v.z);
  }
  brainGeo.computeVertexNormals();
  brain.add(new THREE.Mesh(brainGeo, organMat(0xe0b7c8, 0.9)));
  layers.organs.add(brain);

  let skinFactor = 1;

  function setOpacity(layer: LayerName, value: number) {
    if (layer === 'skin') skinFactor = value;
    for (const mat of mats[layer] ?? []) {
      mat.opacity = (mat.userData.base as number) * value;
      mat.transparent = true;
    }
  }

  function focus(distance: number) {
    const far = THREE.MathUtils.clamp((distance - 0.55) / 2.0, 0, 1);
    skinMat.opacity = (0.025 + 0.10 * far) * skinFactor;
    skinMat.depthWrite = false;
  }

  return { layers, heart, brain, setOpacity, focus };
}
