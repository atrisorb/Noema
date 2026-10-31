import * as THREE from 'three';

type Flow = { curve: THREE.CatmullRomCurve3; color: number; count: number; speed: number; size: number };

const V = (x: number, y: number, z: number) => new THREE.Vector3(x, y, z);

const FLOWS: Flow[] = [
  {
    color: 0xff2d55, count: 220, speed: 0.055, size: 0.0085,
    curve: new THREE.CatmullRomCurve3([
      V(0.012, 0.30, 0.06), V(0.02, 0.40, 0.045), V(0.012, 0.52, 0.03), V(0.0, 0.60, 0.02), V(-0.03, 0.72, 0.01),
    ]),
  },
  {
    color: 0xff2d55, count: 220, speed: 0.052, size: 0.008,
    curve: new THREE.CatmullRomCurve3([
      V(0.0, 0.30, 0.02), V(0.0, 0.14, -0.01), V(0.0, -0.12, -0.02), V(0.0, -0.36, -0.01), V(0.0, -0.55, 0.0),
    ]),
  },
  {
    color: 0xff2d55, count: 120, speed: 0.05, size: 0.0072,
    curve: new THREE.CatmullRomCurve3([V(0, -0.55, 0), V(0.08, -0.66, 0.01), V(0.12, -0.82, 0.02), V(0.12, -0.885, 0.03)]),
  },
  {
    color: 0xff2d55, count: 120, speed: 0.05, size: 0.0072,
    curve: new THREE.CatmullRomCurve3([V(0, -0.55, 0), V(-0.08, -0.66, 0.01), V(-0.12, -0.82, 0.02), V(-0.12, -0.885, 0.03)]),
  },
  {
    color: 0x4d7cff, count: 160, speed: 0.045, size: 0.008,
    curve: new THREE.CatmullRomCurve3([
      V(-0.02, 0.36, 0.035), V(-0.12, 0.38, 0.02), V(-0.17, 0.33, 0.0), V(-0.2, 0.12, -0.01), V(-0.16, -0.18, 0.0), V(-0.10, -0.50, 0.0),
    ]),
  },
  {
    color: 0x4d7cff, count: 140, speed: 0.042, size: 0.0075,
    curve: new THREE.CatmullRomCurve3([
      V(-0.16, -0.18, 0.0), V(-0.06, -0.34, 0.01), V(-0.02, -0.48, 0.01), V(0.0, -0.55, 0.0),
    ]),
  },
];

function glowTexture(): THREE.Texture {
  const canvas = document.createElement('canvas');
  canvas.width = 32;
  canvas.height = 32;
  const ctx = canvas.getContext('2d')!;
  const g = ctx.createRadialGradient(16, 16, 0, 16, 16, 16);
  g.addColorStop(0, 'rgba(255,255,255,1)');
  g.addColorStop(0.35, 'rgba(255,255,255,.7)');
  g.addColorStop(1, 'rgba(255,255,255,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, 32, 32);
  const tex = new THREE.CanvasTexture(canvas);
  tex.colorSpace = THREE.SRGBColorSpace;
  return tex;
}

export class Circulation {
  readonly group = new THREE.Group();
  private systems: Array<{ flow: Flow; points: THREE.Points; offsets: Float32Array }> = [];
  private clock = 0;
  private texture = glowTexture();

  constructor(budget = 980) {
    this.build(budget);
  }

  /** Rebuild every flow for a new particle budget (1 unit = 980 particles). */
  setBudget(budget: number) {
    for (const { points } of this.systems) {
      points.geometry.dispose();
      (points.material as THREE.PointsMaterial).dispose();
      this.group.remove(points);
    }
    this.systems = [];
    this.build(budget);
  }

  private build(budget: number) {
    const scale = Math.max(0.25, Math.min(2, budget / 980));
    for (const base of FLOWS) {
      const flow = { ...base, count: Math.max(24, Math.round(base.count * scale)) };
      const positions = new Float32Array(flow.count * 3);
      const offsets = new Float32Array(flow.count);
      for (let i = 0; i < flow.count; i += 1) offsets[i] = (i / flow.count + Math.random() * 0.01) % 1;
      const geo = new THREE.BufferGeometry();
      geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
      const mat = new THREE.PointsMaterial({
        color: flow.color, size: flow.size, map: this.texture, transparent: true,
        blending: THREE.AdditiveBlending, depthWrite: false, depthTest: false,
        opacity: 0.55, sizeAttenuation: true,
      });
      const points = new THREE.Points(geo, mat);
      this.group.add(points);
      this.systems.push({ flow, points, offsets });
    }
    this.update(0);
  }

  update(dt: number) {
    this.clock += dt;
    for (const { flow, points, offsets } of this.systems) {
      const attr = points.geometry.getAttribute('position') as THREE.BufferAttribute;
      for (let i = 0; i < flow.count; i += 1) {
        const t = (offsets[i] + this.clock * flow.speed) % 1;
        const p = flow.curve.getPointAt(t);
        attr.setXYZ(i, p.x, p.y, p.z);
      }
      attr.needsUpdate = true;
    }
  }

  setOpacity(value: number) {
    for (const { points } of this.systems) (points.material as THREE.PointsMaterial).opacity = 0.55 * value;
  }
}
