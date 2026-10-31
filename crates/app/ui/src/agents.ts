import * as THREE from 'three';
import type { Agent } from './types';

interface Item {
  data: Agent;
  sprite: THREE.Sprite;
  proxy: THREE.Mesh;
  curve: THREE.CatmullRomCurve3;
  t: number;
  speed: number;
  phase: number;
}

const PATHS: [number, number, number][][] = [
  [[0, 0.83, 0.12], [0.09, 0.80, 0.06], [0.05, 0.71, 0.0], [-0.06, 0.73, -0.02], [-0.10, 0.82, 0.05]],
  [[0, 0.46, 0.14], [0.15, 0.30, 0.11], [0.09, 0.04, 0.17], [-0.11, 0.01, 0.13], [-0.17, 0.29, 0.09]],
  [[0.16, 0.42, -0.04], [0.2, 0.2, -0.08], [0.1, 0.12, -0.12], [-0.02, 0.3, -0.12], [0.0, 0.44, -0.08]],
  [[0.2, 0.48, 0], [0.32, 0.25, 0.06], [0.39, 0.0, 0.05], [0.31, 0.22, -0.02], [0.19, 0.46, -0.01]],
  [[-0.2, 0.48, 0], [-0.32, 0.25, 0.06], [-0.39, 0.0, 0.05], [-0.31, 0.22, -0.02], [-0.19, 0.46, -0.01]],
  [[0.08, -0.2, 0.06], [0.13, -0.5, 0.09], [0.12, -0.86, 0.05], [0.11, -0.5, -0.01], [0.05, -0.24, 0.0]],
  [[-0.08, -0.2, 0.06], [-0.13, -0.5, 0.09], [-0.12, -0.86, 0.05], [-0.11, -0.5, -0.01], [-0.05, -0.24, 0.0]],
];

function glow(): THREE.Texture {
  const canvas = document.createElement('canvas');
  canvas.width = 64;
  canvas.height = 64;
  const ctx = canvas.getContext('2d')!;
  const g = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
  g.addColorStop(0, 'rgba(255,255,255,1)');
  g.addColorStop(0.25, 'rgba(255,255,255,.85)');
  g.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, 64, 64);
  const tex = new THREE.CanvasTexture(canvas);
  tex.colorSpace = THREE.SRGBColorSpace;
  return tex;
}

export class Swarm {
  readonly group = new THREE.Group();
  private readonly pickScene = new THREE.Scene();
  private readonly target = new THREE.WebGLRenderTarget(2, 2, { depthBuffer: true, stencilBuffer: false });
  private readonly items: Item[] = [];
  private frozenId: string | null = null;
  private clock = 0;
  private texture = glow();

  constructor(agents: Agent[]) {
    agents.forEach((data, i) => {
      const sprite = new THREE.Sprite(new THREE.SpriteMaterial({
        map: this.texture, color: new THREE.Color(data.accent), transparent: true,
        blending: THREE.AdditiveBlending, depthWrite: false, depthTest: true, opacity: 0.95,
      }));
      sprite.scale.setScalar(0.075);
      const bits = i + 1;
      const proxy = new THREE.Mesh(
        new THREE.SphereGeometry(0.05, 8, 6),
        new THREE.MeshBasicMaterial({
          color: new THREE.Color(bits & 1 ? 1 : 0, bits & 2 ? 1 : 0, bits & 4 ? 1 : 0),
        }),
      );
      const curve = new THREE.CatmullRomCurve3(PATHS[i % PATHS.length].map((p) => new THREE.Vector3(...p)), true, 'catmullrom', 0.4);
      const item: Item = { data, sprite, proxy, curve, t: (i / agents.length + 0.13) % 1, speed: 0.022 + (i % 3) * 0.007, phase: i * 1.3 };
      this.items.push(item);
      this.group.add(sprite);
      this.pickScene.add(proxy);
      this.place(item);
    });
  }

  private place(item: Item) {
    const point = item.curve.getPointAt(item.t % 1);
    item.sprite.position.copy(point);
    item.proxy.position.copy(point);
  }

  update(dt: number) {
    this.clock += dt;
    for (const item of this.items) {
      if (item.data.id !== this.frozenId) item.t += dt * item.speed;
      this.place(item);
      const pulse = item.data.id === this.frozenId ? 0.11 : 0.07 + Math.sin(this.clock * 2.6 + item.phase) * 0.009;
      item.sprite.scale.setScalar(pulse);
    }
  }

  freeze(id: string | null) { this.frozenId = id; }
  get frozen() { return this.frozenId; }
  get visible() { return this.group.visible; }
  setVisible(on: boolean) { this.group.visible = on; }
  setOpacity(v: number) { this.group.children.forEach((c) => { const m = (c as THREE.Sprite).material as THREE.SpriteMaterial; m.opacity = 0.95 * v; }); }
  positionOf(id: string) { return this.items.find((i) => i.data.id === id)?.sprite.position.clone() ?? null; }

  screenOf(id: string, camera: THREE.Camera, renderer: THREE.WebGLRenderer) {
    const item = this.items.find((i) => i.data.id === id);
    if (!item) return null;
    const canvas = renderer.domElement;
    const v = item.sprite.position.clone().project(camera);
    return {
      x: (v.x * 0.5 + 0.5) * canvas.clientWidth + canvas.getBoundingClientRect().left,
      y: (-v.y * 0.5 + 0.5) * canvas.clientHeight + canvas.getBoundingClientRect().top,
    };
  }

  /** GPU picking: render ID proxies into a target, read the pixel under the pointer. */
  pick(renderer: THREE.WebGLRenderer, camera: THREE.Camera, clientX: number, clientY: number): string | null {
    const canvas = renderer.domElement;
    const rect = canvas.getBoundingClientRect();
    const size = renderer.getDrawingBufferSize(new THREE.Vector2());
    this.target.setSize(size.x, size.y);
    const prevTarget = renderer.getRenderTarget();
    const prevClear = renderer.getClearColor(new THREE.Color());
    const prevAlpha = renderer.getClearAlpha();
    renderer.setRenderTarget(this.target);
    renderer.setClearColor(0x000000, 1);
    renderer.clear();
    const prevAspect = camera instanceof THREE.PerspectiveCamera ? camera.aspect : 1;
    if (camera instanceof THREE.PerspectiveCamera) {
      camera.aspect = rect.width / rect.height;
      camera.updateProjectionMatrix();
    }
    renderer.render(this.pickScene, camera);
    if (camera instanceof THREE.PerspectiveCamera) {
      camera.aspect = prevAspect;
      camera.updateProjectionMatrix();
    }
    const x = Math.floor(((clientX - rect.left) / rect.width) * size.x);
    const y = Math.floor(((rect.height - (clientY - rect.top)) / rect.height) * size.y);
    const px = new Uint8Array(4);
    renderer.readRenderTargetPixels(this.target, Math.min(size.x - 1, Math.max(0, x)), Math.min(size.y - 1, Math.max(0, y)), 1, 1, px);
    renderer.setRenderTarget(prevTarget);
    renderer.setClearColor(prevClear, prevAlpha);
    const id = (px[0] > 127 ? 1 : 0) + (px[1] > 127 ? 2 : 0) + (px[2] > 127 ? 4 : 0);
    return id ? this.items[id - 1]?.data.id ?? null : null;
  }
}
