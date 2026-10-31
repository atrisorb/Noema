import * as THREE from 'three';
import { apiUrl } from './platform';

export interface BrainNode {
  id: string; title: string; body: string; agent_id: string | null;
  created_at: number; importance: number; region: string;
  pos: [number, number, number];
  pulse: { hz: number; intensity: number; hue: number };
}
export interface BrainEdge { from: string; to: string; weight: number }
export interface BrainData {
  nodes: BrainNode[]; edges: BrainEdge[];
  regions: { id: string; anchor: [number, number, number] }[];
  generated_at_ms: number;
}

const CENTER = new THREE.Vector3(0, 0.8, 0.006);
const SCALE = 0.055;
const NS = 'http://www.w3.org/2000/svg';

const VERT = `
attribute vec3 aPos; attribute vec3 aColor; attribute float aSize; attribute float aPhase; attribute float aHz;
uniform float uTime; uniform float uDream;
varying vec2 vUv; varying vec3 vColor;
void main() {
  vUv = position.xy;
  float beat = 0.85 + 0.25 * sin(uTime * 6.283 * aHz + aPhase);
  vColor = mix(aColor, vec3(aColor.b, aColor.r, 0.95), uDream * 0.7) * beat;
  vec4 mv = modelViewMatrix * vec4(aPos, 1.0);
  mv.xy += position.xy * aSize * (0.85 + 0.15 * beat);
  gl_Position = projectionMatrix * mv;
}`;

const FRAG = `
varying vec2 vUv; varying vec3 vColor;
void main() {
  float d = length(vUv);
  float core = 1.0 - smoothstep(0.42, 0.62, d);
  float halo = (1.0 - smoothstep(0.62, 1.0, d)) * 0.26;
  float a = max(core, halo);
  if (a < 0.02) discard;
  vec3 col = mix(vColor, vec3(1.0), core * 0.5);
  gl_FragColor = vec4(col, a);
}`;

export class BrainGraph {
  readonly group = new THREE.Group();
  readonly center = CENTER.clone();
  data: BrainData | null = null;
  mode: 'wake' | 'dream' = 'wake';
  visible = false;
  selected: BrainNode | null = null;

  private worker: Worker | null = null;
  private positions = new Float32Array(0);
  private dirty = false;
  private posAttr: THREE.InstancedBufferAttribute | null = null;
  private colorAttr: THREE.InstancedBufferAttribute | null = null;
  private hzAttr: THREE.InstancedBufferAttribute | null = null;
  private material: THREE.ShaderMaterial | null = null;
  private baseHz = new Float32Array(0);
  private baseColors = new Float32Array(0);
  private livePulse = false;
  private edgePositions = new Float32Array(0);
  private edgeAttr: THREE.BufferAttribute | null = null;
  private edgeIndex: [number, number][] = [];
  private idIndex = new Map<string, number>();
  private labels: { el: SVGTextElement; index: number }[] = [];
  private labelRoot: SVGSVGElement | null = null;
  private tmp = new THREE.Vector3();
  private ray = new THREE.Raycaster();
  private drag: { index: number; plane: THREE.Plane; last: [number, number, number] } | null = null;

  constructor(scene: THREE.Scene) {
    this.group.position.copy(CENTER);
    this.group.scale.setScalar(SCALE);
    this.group.visible = false;
    this.group.renderOrder = 8;
    scene.add(this.group);
    this.labelRoot = document.getElementById('brain-labels') as unknown as SVGSVGElement | null;
  }

  async load(): Promise<BrainData> {
    const res = await fetch(apiUrl('/api/brain'));
    if (!res.ok) throw new Error(await res.text());
    const data = await res.json() as BrainData;
    this.data = data;
    const n = data.nodes.length;
    this.positions = new Float32Array(n * 3);
    const colors = new Float32Array(n * 3);
    const sizes = new Float32Array(n);
    const phases = new Float32Array(n);
    const hz = new Float32Array(n);
    const color = new THREE.Color();
    data.nodes.forEach((node, i) => {
      this.positions.set(node.pos, i * 3);
      color.setHSL(node.pulse.hue / 360, 0.85, 0.62);
      colors.set([color.r, color.g, color.b], i * 3);
      sizes[i] = 0.0016 + node.importance * 0.0024;
      phases[i] = (node.importance * 9.7 + i) % 6.283;
      hz[i] = node.pulse.hz;
      this.idIndex.set(node.id, i);
    });

    const geo = new THREE.InstancedBufferGeometry();
    geo.setAttribute('position', new THREE.Float32BufferAttribute([-1, -1, 0, 1, -1, 0, 1, 1, 0, -1, 1, 0], 3));
    geo.setIndex([0, 1, 2, 0, 2, 3]);
    this.posAttr = new THREE.InstancedBufferAttribute(this.positions, 3);
    this.colorAttr = new THREE.InstancedBufferAttribute(colors, 3);
    geo.setAttribute('aPos', this.posAttr);
    geo.setAttribute('aColor', this.colorAttr);
    geo.setAttribute('aSize', new THREE.InstancedBufferAttribute(sizes, 1));
    geo.setAttribute('aPhase', new THREE.InstancedBufferAttribute(phases, 1));
    this.hzAttr = new THREE.InstancedBufferAttribute(hz, 1);
    geo.setAttribute('aHz', this.hzAttr);
    this.baseHz = hz.slice();
    this.baseColors = colors.slice();
    geo.instanceCount = n;
    this.material = new THREE.ShaderMaterial({
      vertexShader: VERT, fragmentShader: FRAG,
      uniforms: { uTime: { value: 0 }, uDream: { value: 0 } },
      transparent: true, depthWrite: false, depthTest: false, blending: THREE.NormalBlending,
    });
    this.group.add(new THREE.Mesh(geo, this.material));

    this.edgePositions = new Float32Array(data.edges.length * 6);
    for (const [i, edge] of data.edges.entries()) {
      const a = this.idIndex.get(edge.from);
      const b = this.idIndex.get(edge.to);
      if (a === undefined || b === undefined) continue;
      this.edgeIndex.push([a, b]);
      this.edgePositions[i * 6] = this.positions[a * 3];
      this.edgePositions[i * 6 + 1] = this.positions[a * 3 + 1];
      this.edgePositions[i * 6 + 2] = this.positions[a * 3 + 2];
      this.edgePositions[i * 6 + 3] = this.positions[b * 3];
      this.edgePositions[i * 6 + 4] = this.positions[b * 3 + 1];
      this.edgePositions[i * 6 + 5] = this.positions[b * 3 + 2];
    }
    const eg = new THREE.BufferGeometry();
    this.edgeAttr = new THREE.BufferAttribute(this.edgePositions, 3);
    eg.setAttribute('position', this.edgeAttr);
    this.group.add(new THREE.LineSegments(eg, new THREE.LineBasicMaterial({
      color: 0x8fb3ff, transparent: true, opacity: 0.2,
      blending: THREE.NormalBlending, depthWrite: false, depthTest: false,
    })));

    const top = [...data.nodes.keys()]
      .sort((a, b) => data.nodes[b].importance - data.nodes[a].importance)
      .slice(0, 12);
    for (const i of top) {
      const el = document.createElementNS(NS, 'text') as SVGTextElement;
      el.setAttribute('class', 'brain-label');
      el.textContent = data.nodes[i].title;
      color.setHSL(data.nodes[i].pulse.hue / 360, 0.9, 0.75);
      el.style.fill = `#${color.getHexString()}`;
      this.labelRoot?.append(el);
      this.labels.push({ el, index: i });
    }

    this.worker = new Worker(new URL('./brain-worker.ts', import.meta.url), { type: 'module' });
    this.worker.onmessage = (event: MessageEvent) => {
      if (event.data?.type === 'positions') {
        this.positions = event.data.positions as Float32Array;
        this.dirty = true;
      }
    };
    this.worker.postMessage({
      type: 'init',
      positions: Array.from(this.positions),
      edges: this.edgeIndex.map(([a, b], i) => ({ a, b, weight: data.edges[i]?.weight ?? 1 })),
    });
    return data;
  }

  setVisible(on: boolean) {
    this.visible = on;
    this.group.visible = on;
    if (!on) for (const label of this.labels) label.el.style.display = 'none';
  }

  setMode(mode: 'wake' | 'dream') {
    this.mode = mode;
    this.worker?.postMessage({ type: 'mode', dream: mode === 'dream' });
    if (this.material) this.material.uniforms.uDream.value = mode === 'dream' ? 1 : 0;
  }

  update(time: number, camera: THREE.PerspectiveCamera, width: number, height: number) {
    if (!this.data) return;
    if (this.dirty && this.posAttr && this.edgeAttr) {
      (this.posAttr.array as Float32Array).set(this.positions);
      this.posAttr.needsUpdate = true;
      for (let i = 0; i < this.edgeIndex.length; i += 1) {
        const [a, b] = this.edgeIndex[i];
        this.edgePositions.set(this.positions.subarray(a * 3, a * 3 + 3), i * 6);
        this.edgePositions.set(this.positions.subarray(b * 3, b * 3 + 3), i * 6 + 3);
      }
      this.edgeAttr.needsUpdate = true;
      this.dirty = false;
    }
    if (this.material) this.material.uniforms.uTime.value = time;
    if (!this.visible) return;
    this.group.updateMatrixWorld(true);
    camera.updateMatrixWorld();
    const close = camera.position.distanceTo(this.center) < 0.85;
    const placed: { x: number; y: number }[] = [];
    for (const label of this.labels) {
      const p = close ? this.screenOf(label.index, camera, width, height) : null;
      const clear = p && placed.every((q) => (q.x - p.x) ** 2 + (q.y - p.y) ** 2 > 48 * 48);
      if (!p || !clear) { label.el.style.display = 'none'; continue; }
      placed.push(p);
      label.el.style.display = '';
      label.el.setAttribute('transform', `translate(${p.x.toFixed(1)} ${(p.y - 9).toFixed(1)})`);
      label.el.style.opacity = String(0.4 + this.data.nodes[label.index].importance * 0.55);
    }
  }

  pick(x: number, y: number, camera: THREE.PerspectiveCamera, width: number, height: number): { node: BrainNode; index: number } | null {
    if (!this.data) return null;
    let best = -1;
    let bestD = 22 * 22;
    this.data.nodes.forEach((_, i) => {
      const p = this.screenOf(i, camera, width, height);
      if (!p) return;
      const d = (p.x - x) ** 2 + (p.y - y) ** 2;
      if (d < bestD) { bestD = d; best = i; }
    });
    return best >= 0 ? { node: this.data.nodes[best], index: best } : null;
  }

  beginDrag(index: number, x: number, y: number, camera: THREE.PerspectiveCamera, width: number, height: number) {
    this.ray.setFromCamera(new THREE.Vector2((x / width) * 2 - 1, -(y / height) * 2 + 1), camera);
    const normal = camera.getWorldDirection(new THREE.Vector3()).negate();
    const plane = new THREE.Plane().setFromNormalAndCoplanarPoint(normal, this.nodeWorld(index));
    this.drag = { index, plane, last: [this.positions[index * 3], this.positions[index * 3 + 1], this.positions[index * 3 + 2]] };
  }

  dragTo(x: number, y: number, camera: THREE.PerspectiveCamera, width: number, height: number) {
    if (!this.drag) return;
    this.ray.setFromCamera(new THREE.Vector2((x / width) * 2 - 1, -(y / height) * 2 + 1), camera);
    const hit = new THREE.Vector3();
    if (!this.ray.ray.intersectPlane(this.drag.plane, hit)) return;
    this.group.worldToLocal(hit);
    const len = hit.length();
    if (len > 0.98) hit.multiplyScalar(0.98 / len);
    this.drag.last = [hit.x, hit.y, hit.z];
    this.worker?.postMessage({ type: 'pin', index: this.drag.index, pos: this.drag.last });
  }

  endDrag() {
    const drag = this.drag;
    if (!drag) return;
    this.worker?.postMessage({ type: 'pin', index: -1, pos: [0, 0, 0] });
    this.drag = null;
    const node = this.data?.nodes[drag.index];
    if (!node) return;
    void fetch(apiUrl('/api/brain/drag'), {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ id: node.id, pos: drag.last }),
    }).catch(() => {});
  }

  async refreshPulse() {
    if (!this.data || !this.visible) return;
    try {
      const res = await fetch(apiUrl('/api/brain/telemetry'));
      if (!res.ok) return;
      const telemetry = await res.json() as { source?: string; pulse?: { hz: number; intensity: number; hue: number } };
      if (telemetry.source === 'live' && telemetry.pulse) this.applyPulse(telemetry.pulse);
      else if (this.livePulse) this.restorePulse();
    } catch {
      // Feed is optional; derived pulses stay in place.
    }
  }

  private applyPulse(pulse: { hz: number; intensity: number; hue: number }) {
    if (!this.data || !this.hzAttr || !this.colorAttr) return;
    const color = new THREE.Color();
    color.setHSL(pulse.hue / 360, 0.85, 0.45 + 0.3 * pulse.intensity);
    const colors = this.colorAttr.array as Float32Array;
    for (let i = 0; i < this.data.nodes.length; i += 1) {
      (this.hzAttr.array as Float32Array)[i] = pulse.hz;
      colors.set([color.r, color.g, color.b], i * 3);
    }
    this.hzAttr.needsUpdate = true;
    this.colorAttr.needsUpdate = true;
    this.livePulse = true;
  }

  private restorePulse() {
    if (!this.hzAttr || !this.colorAttr || !this.baseHz.length) return;
    this.livePulse = false;
    (this.hzAttr.array as Float32Array).set(this.baseHz);
    (this.colorAttr.array as Float32Array).set(this.baseColors);
    this.hzAttr.needsUpdate = true;
    this.colorAttr.needsUpdate = true;
  }

  nodeWorld(index: number, target = new THREE.Vector3()) {
    target.set(this.positions[index * 3], this.positions[index * 3 + 1], this.positions[index * 3 + 2]);
    return this.group.localToWorld(target);
  }

  linksOf(id: string): BrainNode[] {
    if (!this.data) return [];
    const out: BrainNode[] = [];
    for (const edge of this.data.edges) {
      if (edge.from !== id && edge.to !== id) continue;
      const other = this.data.nodes.find((n) => n.id === (edge.from === id ? edge.to : edge.from));
      if (other) out.push(other);
    }
    return out;
  }

  screenOfId(id: string, camera: THREE.PerspectiveCamera, width: number, height: number) {
    const index = this.idIndex.get(id);
    return index === undefined ? null : this.screenOf(index, camera, width, height);
  }

  maxRadius() {
    let max = 0;
    for (let i = 0; i < this.positions.length; i += 3) {
      max = Math.max(max, Math.hypot(this.positions[i], this.positions[i + 1], this.positions[i + 2]));
    }
    return max;
  }

  private screenOf(index: number, camera: THREE.PerspectiveCamera, width: number, height: number) {
    const world = this.nodeWorld(index, this.tmp);
    if (world.distanceTo(camera.position) > 1.4) return null;
    world.project(camera);
    if (world.z > 1) return null;
    return { x: (world.x * 0.5 + 0.5) * width, y: (-world.y * 0.5 + 0.5) * height };
  }
}
