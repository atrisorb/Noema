import * as THREE from 'three';

/**
 * Creates a soft contact occlusion shadow beneath the feet at ground level (y = -0.86).
 * Anchors the anatomy visually to the virtual floor in both Cavity (dark) and Bone (light) themes.
 */
export function createContactShadow(): THREE.Mesh {
  const canvas = document.createElement('canvas');
  canvas.width = 512;
  canvas.height = 512;
  const ctx = canvas.getContext('2d');
  if (ctx) {
    ctx.clearRect(0, 0, 512, 512);

    // Overall ambient occlusion ground wash
    const ambient = ctx.createRadialGradient(256, 256, 40, 256, 256, 220);
    ambient.addColorStop(0, 'rgba(0, 0, 0, 0.28)');
    ambient.addColorStop(0.55, 'rgba(0, 0, 0, 0.10)');
    ambient.addColorStop(1, 'rgba(0, 0, 0, 0)');
    ctx.fillStyle = ambient;
    ctx.beginPath();
    ctx.ellipse(256, 256, 190, 110, 0, 0, Math.PI * 2);
    ctx.fill();

    // Two soft foot contact ellipses: left foot and right foot
    const drawFootContact = (cx: number, cy: number, rot: number) => {
      const grad = ctx.createRadialGradient(cx, cy, 8, cx, cy, 80);
      grad.addColorStop(0, 'rgba(0, 0, 0, 0.60)');
      grad.addColorStop(0.35, 'rgba(0, 0, 0, 0.38)');
      grad.addColorStop(0.7, 'rgba(0, 0, 0, 0.12)');
      grad.addColorStop(1, 'rgba(0, 0, 0, 0)');
      ctx.fillStyle = grad;
      ctx.beginPath();
      ctx.ellipse(cx, cy, 70, 42, rot, 0, Math.PI * 2);
      ctx.fill();
    };

    drawFootContact(195, 256, -0.06);
    drawFootContact(317, 256, 0.06);
  }

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;

  const geo = new THREE.PlaneGeometry(0.85, 0.65);
  const mat = new THREE.MeshBasicMaterial({
    map: texture,
    transparent: true,
    opacity: 0.85,
    depthWrite: false,
    toneMapped: false,
  });

  const mesh = new THREE.Mesh(geo, mat);
  mesh.rotation.x = -Math.PI / 2;
  mesh.position.set(0, -0.86, 0.02);
  mesh.renderOrder = 1;
  mesh.name = 'contact-shadow';
  return mesh;
}
