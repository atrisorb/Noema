import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { DRACOLoader } from 'three/addons/loaders/DRACOLoader.js';
import { assetUrl } from './platform';

export type AtlasLayer = 'skin' | 'muscles' | 'skeleton' | 'nerves' | 'vessels' | 'organs' | 'brain';

const LAYER_RULES: Array<[AtlasLayer, RegExp]> = [
  ['brain', /brain|cortex|cerebell|hippocamp|amygdal|thalam|hypothalam|medulla|pons|ventricle/i],
  ['vessels', /artery|arteries|arterial|vein|venous|aorta|carotid|femoral|pulmonary trunk|vena cava|capillar/i],
  ['nerves', /nerve|plexus|ganglion|spinal cord|neuraxis/i],
  ['organs', /heart|lung|liver|kidney|stomach|intestin|colon|bowel|pancreas|spleen|bladder|trachea|esophagus|thyroid|testis|ovary|uterus|eyeball|eye|tongue|gland|lymph|thymus|pericard|pleura|omentum|mesentery/i],
  ['skeleton', /bone|skeleton|cartilage|teeth|tooth|skull|cranium|vertebra|rib|femur|tibia|humerus|mandible|maxilla|scapula|clavicle|pelvis|sternum|patella|fibula|radius|ulna|metacarp|metatars|phalan|incus|malleus|stapes/i],
  ['muscles', /muscle|tendon|ligament|fascia|aponeurosis|diaphragm/i],
];

const COLORS: Array<[RegExp, number]> = [
  [/heart|myocard|ventricle|atrium/i, 0xb0182b],
  [/lung|pulmon/i, 0xd98a9e],
  [/liver|hepat/i, 0x6f3a28],
  [/kidney|renal/i, 0x8a4a4a],
  [/stomach|gastr/i, 0xc98a55],
  [/intestin|colon|bowel|jejun|ileum|duoden/i, 0xb37a52],
  [/pancreas/i, 0xd9b07a],
  [/spleen/i, 0x6b2a3a],
  [/bladder/i, 0xc9b07a],
  [/brain|cortex|cerebell|hippocamp|amygdal|thalam|medulla|pons/i, 0xd8a6bc],
  [/artery|arteries|aorta|carotid|femoral/i, 0xc0392b],
  [/vein|vena/i, 0x3f6fb5],
  [/nerve|plexus|ganglion|spinal cord/i, 0xe8c96a],
  [/bone|skeleton|cartilage|teeth|tooth|skull|cranium|vertebra|rib|femur|tibia|humerus/i, 0xe4dcc4],
  [/muscle|tendon|ligament|fascia|aponeurosis|diaphragm/i, 0x8e2f2f],
];

const HEART = /heart|ventricle|atrium|myocard/i;
const BRAIN = /brain|cortex|cerebell|hippocamp|amygdal|thalam|diencephal|brainstem|medulla|pons/i;

function classify(name: string, materialName: string): AtlasLayer {
  for (const [layer, re] of LAYER_RULES) if (re.test(name)) return layer;
  if (/bone/i.test(materialName)) return 'skeleton';
  if (/muscle/i.test(materialName)) return 'muscles';
  return 'organs';
}

function tint(material: THREE.MeshStandardMaterial, name: string): number {
  for (const [re, color] of COLORS) if (re.test(name)) return color;
  return material.color?.getHex() ?? 0xb08a7a;
}

export class Atlas {
  readonly root = new THREE.Group();
  readonly layers = new Map<AtlasLayer, THREE.Group>();
  readonly heartMeshes: THREE.Object3D[] = [];
  readonly heartAnchor = new THREE.Vector3(0.012, 0.30, 0.055);
  readonly brainAnchor = new THREE.Vector3(0, 0.78, 0.02);
  readonly errors: string[] = [];
  loadedSystems = 1;
  triangles = 0;
  meshes = 0;
  ready = false;

  private baseOpacity = new Map<THREE.Material, number>();
  private tinted = new Set<THREE.Material>();

  constructor() {
    for (const layer of ['skin', 'muscles', 'skeleton', 'nerves', 'vessels', 'organs', 'brain'] as AtlasLayer[]) {
      const group = new THREE.Group();
      group.name = layer;
      this.root.add(group);
      this.layers.set(layer, group);
    }
  }

  async load(onStep?: (label: string) => void): Promise<void> {
    onStep?.('body');
    const draco = new DRACOLoader().setDecoderPath(assetUrl('/draco/'));
    const loader = new GLTFLoader().setDRACOLoader(draco);
    const gltf = await loader.loadAsync(assetUrl('/assets/anatomy/atlas/body_full.glb'));
    const holder = new THREE.Group();
    holder.add(gltf.scene);
    holder.updateMatrixWorld(true);

    // Normalise once: feet at -0.86, height 1.72 m.
    const box = new THREE.Box3().setFromObject(holder);
    const size = box.getSize(new THREE.Vector3());
    const center = box.getCenter(new THREE.Vector3());
    const scale = size.y > 0.1 ? 1.72 / size.y : 1;
    holder.scale.setScalar(scale);
    holder.position.set(-center.x * scale, -box.min.y * scale - 0.86, -center.z * scale);
    holder.updateMatrixWorld(true);

    const counts: Record<string, number> = {};
    const entries: Array<{ mesh: THREE.Mesh; name: string }> = [];
    gltf.scene.traverse((node) => {
      const mesh = node as THREE.Mesh;
      if (!mesh.isMesh) return;
      const names: string[] = [];
      let cursor: THREE.Object3D | null = mesh;
      for (let i = 0; i < 4 && cursor; i += 1) { names.push(cursor.name); cursor = cursor.parent; }
      entries.push({ mesh, name: names.filter(Boolean).join(' ') });
    });

    for (const { mesh, name } of entries) {
      const source = Array.isArray(mesh.material) ? mesh.material[0] : mesh.material;
      const layer = classify(name, (source as THREE.Material)?.name ?? '');
      const group = this.layers.get(layer)!;
      const geo = mesh.geometry;
      if (geo?.index) this.triangles += geo.index.count / 3;
      else if (geo?.attributes.position) this.triangles += geo.attributes.position.count / 3;

      const originals = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
      const cloned = originals.map((original) => {
        const m = (original as THREE.MeshStandardMaterial).clone() as THREE.MeshStandardMaterial;
        if ('metalness' in m) m.metalness = 0;
        if ('roughness' in m) m.roughness = layer === 'muscles' ? 0.44 : layer === 'skeleton' ? 0.52 : layer === 'vessels' || layer === 'organs' ? 0.42 : 0.5;
        if ('clearcoat' in m) (m as THREE.MeshPhysicalMaterial).clearcoat = layer === 'skeleton' ? 0.04 : layer === 'vessels' ? 0.35 : 0.12;
        if ('envMapIntensity' in m) m.envMapIntensity = 0.9;
        m.side = THREE.DoubleSide;
        m.color.setHex(tint(m, name));
        m.needsUpdate = true;
        this.baseOpacity.set(m, m.opacity ?? 1);
        return m;
      });
      mesh.material = Array.isArray(mesh.material) ? cloned : cloned[0];

      if (HEART.test(name)) this.heartMeshes.push(mesh);
      counts[layer] = (counts[layer] ?? 0) + 1;
      group.attach(mesh);
      this.meshes += 1;
    }
    holder.removeFromParent();
    const normScale = scale;
    const normPos = holder.position.clone();

    // Optional internal systems from the Z-Anatomy atlas (same metre frame).
    // Internal systems are opt-in until their textures/palette maps are fixed.
    const extras: Array<[AtlasLayer, string, number]> = [];
    for (const [layer, file, fallbackColor] of extras) {
      try {
        const extra = await loader.loadAsync(assetUrl('/assets/anatomy/atlas-opt/' + file));
        extra.scene.scale.setScalar(normScale);
        extra.scene.position.copy(normPos);
        extra.scene.updateMatrixWorld(true);
        const group = this.layers.get(layer)!;
        const list: Array<{ mesh: THREE.Mesh; name: string }> = [];
        extra.scene.traverse((node) => {
          const mesh = node as THREE.Mesh;
          if (mesh.isMesh) list.push({ mesh, name: mesh.name });
        });
        for (const { mesh, name } of list) {
          const geo = mesh.geometry;
          if (geo?.index) this.triangles += geo.index.count / 3;
          else if (geo?.attributes.position) this.triangles += geo.attributes.position.count / 3;
          const originals = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
          const cloned = originals.map((original) => {
            const m = (original as THREE.MeshStandardMaterial).clone() as THREE.MeshStandardMaterial;
            m.metalness = 0;
            m.roughness = layer === 'vessels' ? 0.38 : 0.45;
            m.side = THREE.DoubleSide;
            m.color.setHex(tint(m, name) || fallbackColor);
            m.needsUpdate = true;
            this.baseOpacity.set(m, 1);
            return m;
          });
          mesh.material = Array.isArray(mesh.material) ? cloned : cloned[0];
          if (HEART.test(name)) this.heartMeshes.push(mesh);
          if (BRAIN.test(name)) group.add(new THREE.Object3D());
          group.attach(mesh);
          this.meshes += 1;
        }
        this.loadedSystems += 1;
      } catch (e) {
        this.errors.push(file + ': ' + String(e));
      }
    }

    this.root.updateMatrixWorld(true);
    if (this.layers.get('skin')!.children.length === 0) this.layers.get('skin')!.visible = false;
    this.root.updateMatrixWorld(true);

    const heartBox = new THREE.Box3();
    for (const mesh of this.heartMeshes) heartBox.expandByObject(mesh);
    if (!heartBox.isEmpty()) heartBox.getCenter(this.heartAnchor);

    const brainBox = new THREE.Box3();
    for (const mesh of this.layers.get('brain')!.children) brainBox.expandByObject(mesh);
    if (!brainBox.isEmpty()) brainBox.getCenter(this.brainAnchor);

    for (const [layer, group] of this.layers) if (layer !== 'skin') group.visible = true;
    this.ready = this.meshes > 0;
    console.info('atlas loaded', counts, 'meshes', this.meshes, 'tris', Math.round(this.triangles));
  }

  setLayer(layer: AtlasLayer, visible: boolean) {
    const group = this.layers.get(layer);
    if (group) group.visible = visible;
  }

  layerVisible(layer: AtlasLayer): boolean {
    return this.layers.get(layer)?.visible ?? false;
  }

  setOpacity(layer: AtlasLayer, value: number) {
    const group = this.layers.get(layer);
    if (!group) return;
    group.traverse((node) => {
      const mesh = node as THREE.Mesh;
      if (!mesh.isMesh) return;
      const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
      for (const material of materials) {
        const base = this.baseOpacity.get(material) ?? 1;
        material.transparent = layer === 'skin' || value < 1;
        material.opacity = base * value;
        material.depthWrite = layer === 'skin' ? false : value >= 1;
        material.needsUpdate = true;
      }
    });
  }

  beat(amount: number) {
    const k = 1 + 0.02 * amount;
    for (const mesh of this.heartMeshes) {
      if (!mesh.userData.baseScale) mesh.userData.baseScale = mesh.scale.clone();
      const base = mesh.userData.baseScale as THREE.Vector3;
      mesh.scale.set(base.x * k, base.y * k, base.z * k);
    }
  }
}
