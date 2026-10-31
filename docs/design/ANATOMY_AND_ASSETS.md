# NOEMA - Anatomy and Asset Standard

Status: v1.0, 2026-10-31. Owner: Aesthesis. Geometry partner: Soma. Packaging partner: Axon.
Purpose: make the NOEMA body photoreal, neutral and correct; remove external genitalia; fix lower legs and feet at the geometry and material level; enforce an SVG-first, no-raster asset law.

## 1. Audit method

Sources inspected on 2026-10-31:

- <code>crates/app/ui/src/body.ts</code> - procedural body.
- <code>assets/anatomy/atlas/body_full.glb</code> - GLB JSON chunk parsed directly.
- <code>docs/evidence/block5-atlas.webp</code>, <code>block5-world.webp</code>, <code>block9-desktop.webp</code> - visual inspection.
- <code>assets/anatomy/LICENSES.md</code> - asset provenance.

## 2. Current body facts

| Fact | Value |
|---|---|
| Atlas nodes | 1,872 |
| Atlas meshes | 826 |
| Atlas materials | 2: <code>Mat_Bone_Pro</code>, <code>Mat_Muscle_Pro</code> |
| Atlas content | Bone and muscle only. No skin, fat, fascia, vessels or organs in this GLB. |
| Foot/leg bones | Present: femur, tibia, fibula, patella, calcaneus, talus, metatarsals, phalanges, foot muscles/tendons. |
| Procedural skin | One translucent <code>MeshPhysicalMaterial</code>, opacity 0.12, clearcoat 0.4. |
| Procedural legs | Three capsules per side: thigh (-0.26 to -0.60), calf (-0.60 to -0.90), foot (-0.91 to -0.92, z +0.09). |
| Procedural feet | A 0.01-long capsule. No ankle, heel, arch or toes. |
| Genitalia node names | No matches for penis/genital/phallus/scrot/testis beyond pelvic bones. A central pelvic protrusion is visible in the render; mesh-level audit required. |

## 3. Diagnosis: lower legs and feet

The lower legs and feet look wrong because of **geometry plus material**, not a single tint bug.

1. The atlas is a muscle/bone study. It has no skin or fascia surface. In the front view the tibia and fibula are exposed, so the lower leg reads as a white stick.
2. The procedural skin shell uses one translucent material for the whole body. It has no subsurface scattering, no micro-normal, no skin thickness, no tone variation, no pores.
3. The procedural foot is a 0.01-long capsule. It has no ankle joint, heel, arch, toes or nails. It cannot read as a foot.
4. The calf muscle group is behind the bone; in the front view the lower leg is mostly bone. Without a skin/fascia surface, the silhouette collapses.
5. There is no contact shadow or occlusion between the foot and the ground; the feet float.

**Do not fix this with a darker material or a blur.** Add surface geometry and a material library.

## 4. Genitalia audit and removal

The current GLB has no node named penis, genitalia, phallus, scrotum or testis beyond pelvic bones. The render still shows a central pelvic protrusion. The agent must locate it by mesh, not by name.

Audit script requirements:
- Parse the GLB JSON and binary.
- List every mesh whose bounding box intersects the pelvic region (x in [-0.18, 0.18], y in [-0.35, -0.10], z in [-0.15, 0.15]).
- Flag names containing <code>corpus</code>, <code>cavern</code>, <code>spong</code>, <code>glans</code>, <code>prepuce</code>, <code>scrot</code>, <code>testis</code>, <code>dartos</code>, <code>cremaster</code>, <code>bulb</code>, <code>crura</code>, <code>suspensory</code>, <code>perine</code>, <code>pubic</code>.
- Render a front, side and close-up pelvis view. Overlay mesh IDs.
- Produce <code>docs/evidence/design-pelvis-audit.webp</code> and a JSON report.

Removal rules:
- Remove any external genital geometry from the default body. Keep internal structures only if the model is used for scientific content; the default body is neutral.
- No toggle. No "scientific mode" switch. The default is neutral.
- Fill the pelvic surface with skin/fascia so there is no hole.
- If removal leaves a gap, use a neutral pelvic surface mesh or regenerate the pelvis. Do not hide it with a shadow.
- Verify with front, side and close-up renders; no protrusion; no hole; no floating geometry.

## 5. Target realism: material library

Use physically based materials. No flat colors. No raster textures in the repo; use procedural noise or KTX2 textures, generated at build time.

| Material | Base color | Roughness | Other | Notes |
|---|---|---|---|---|
| Skin | <code>#B98A72</code> warm neutral | 0.42 | SSS, clearcoat 0.06, micro-normal | Tone variants in the asset library; default neutral |
| Subcutaneous fat | <code>#E8C9A8</code> | 0.62 | SSS 0.3 | Soft, warm |
| Fascia | <code>#E6E0D4</code> | 0.38 | transmission 0.05, thickness 0.4 | Translucent sheet |
| Muscle | <code>#8E2F2F</code> | 0.44 | clearcoat 0.12, SSS 0.18, anisotropic 0.2 | Wet, striated |
| Tendon | <code>#D8CBB8</code> | 0.34 | clearcoat 0.1 | Fibrous |
| Cortical bone | <code>#E4DCC4</code> | 0.52 | clearcoat 0.04 | Micro-porosity |
| Cancellous bone | <code>#C9BFA8</code> | 0.78 | AO | Spongy |
| Cartilage | <code>#D9E2D8</code> | 0.3 | transmission 0.1, thickness 0.2 | Pearly |
| Artery | <code>#C0392B</code> | 0.36 | clearcoat 0.15 | No emissive glow |
| Vein | <code>#3F6FB5</code> | 0.4 | clearcoat 0.1 | No emissive glow |
| Nerve | <code>#E8C96A</code> | 0.4 | emissive 0.05 max | Scientific, not neon |
| Organ capsule | <code>#B46A4A</code> | 0.48 | SSS 0.1 | Per organ |

Three.js guidance:
- <code>MeshPhysicalMaterial</code> with <code>thickness</code>, <code>attenuationColor</code>, <code>attenuationDistance</code>, <code>sheen</code> and <code>clearcoat</code>.
- <code>transmission</code> is expensive. Use it only on small surfaces (cartilage, fascia) or bake SSS into the base color/normal.
- Micro-normal: procedural noise in the shader or a KTX2 normal map. No PNG.
- Skin tone is a token, not a hard-coded shader constant. Provide 3-5 neutral tones; the default is warm neutral.
- No emissive vessels or nerves beyond 0.05; the body should look lit, not neon.

## 6. Lighting and render

- Environment: procedural <code>RoomEnvironment</code> or a small KTX2 studio env map. No PNG/JPEG HDRI.
- Key: warm 5600K, intensity 1.6, upper front left.
- Fill: cool 7200K, intensity 0.7, lower front right.
- Rim: arterial 6200K, intensity 0.9, back.
- Tone mapping: ACES Filmic, exposure 0.95.
- AO, contact shadows, subtle bloom (threshold 0.85), DoF at close focus, 2% grain.
- Background: Cavity near-black with a soft radial falloff. No flat black.
- Offline renders: supersample 2x, output WEBP. If Blender/Cycles is installed, use it for the hero turntable; otherwise use Three.js with a high sample count. Do not fake photoreal with a filter.

## 7. Geometry fixes

1. Add a skin/fascia surface over the atlas. If a full-body skin mesh is not available, generate a smooth shell from the bone/muscle silhouette and blend at the joints.
2. Add a foot mesh: ankle, heel, arch, metatarsal pad, toes, nails. Correct proportions: foot length about 0.15 of body height; current capsule is 0.01.
3. Add the Achilles tendon, ankle retinacula, calf silhouette and knee transition. No floating bones.
4. Correct the leg length and stance. The current legs are three straight capsules; add a slight knee axis and ankle angle.
5. Neutral pelvis: remove external genitalia; fill the surface; keep pelvic floor and internal structures if relevant.
6. Hands: the atlas has bones/muscles; add a skin surface and nails. Do not leave skeletal hands.
7. LODs: full, medium, low. Low LOD keeps the silhouette and skin, drops micro-detail.
8. Validate: glTF-Validator clean, no degenerate triangles, no non-manifold skin shell, correct normals.

## 8. Asset law

Allowed source:
- <code>.svg</code> - logos, icons, mascots, diagrams.
- <code>.glb</code>, <code>.gltf</code> - 3D.
- <code>.woff2</code> - fonts.
- <code>.json</code> - tokens, manifests, data.
- <code>.md</code> - docs.
- <code>.ktx2</code> - GPU textures and environment maps.
- <code>.webp</code> - real photography only, under <code>assets/photos/**</code>.

Forbidden in the repo:
- <code>.png</code>, <code>.jpg</code>, <code>.jpeg</code>, <code>.gif</code>, <code>.bmp</code>, <code>.tif</code>, <code>.tiff</code>.
- UI screenshots as PNG/JPEG.
- Logo or icon raster exports.
- Noise textures as PNG.

Exceptions:
- Platform icons (ICO/ICNS/PNG) generated from SVG at build time into <code>dist/icons</code> or <code>target/icons</code>, git-ignored.
- Playwright screenshots captured as PNG in a temp dir, converted to WEBP, source deleted.
- Third-party tool caches outside the repo.

## 9. Migration table

| Current asset | Count | Action |
|---|---|---|
| <code>docs/evidence/*.png</code> | 20+ | Convert to WEBP, update links, delete PNG |
| <code>src-tauri/icons/*.png</code> | 3 | Generate from <code>assets/brand/noema-app-icon.svg</code>, git-ignore |
| <code>src-tauri/app-icon.png</code> | 1 | Replace with SVG source, generate, git-ignore |
| <code>assets/anatomy/cad/vertebra.webp</code> | 1 | Use <code>vertebra-section.svg</code>, delete PNG |
| <code>src-tauri/icons/icon.ico</code> | 1 | Generated build artifact, git-ignore |
| Any JPEG | 0 | None found; keep it zero |

Add <code>tools/design/asset-audit.mjs</code>:
- Scan tracked files.
- Fail on forbidden extensions.
- Fail on forbidden references in HTML/CSS/TS/MD.
- Allowlist <code>dist/**</code>, <code>target/**</code>, <code>node_modules/**</code>.
- Print a machine-readable report.
- Run in CI and in the block gate.

## 10. Evidence and acceptance

Required evidence:
- <code>docs/evidence/design-body-turntable.webp</code> - 4 views, full body.
- <code>docs/evidence/design-body-legs.webp</code> - lower legs, front/side/back.
- <code>docs/evidence/design-body-feet.webp</code> - feet close-up, front/side/top.
- <code>docs/evidence/design-pelvis-neutral.webp</code> - front/side close-up, no external genitalia, no hole.
- <code>docs/evidence/design-materials.webp</code> - material spheres or body close-up.
- <code>docs/evidence/design-perf.json</code> - 60fps trace.
- <code>docs/evidence/design-gltf-validator.json</code> - clean report.
- <code>docs/evidence/design-asset-audit.json</code> - zero forbidden files.

Acceptance:
- Lower legs and feet read as a body, not a skeleton with sticks.
- Pelvis is neutral; no external genitalia; no hole; no floating geometry.
- Materials are physically based; no flat cartoon shading.
- 60fps on the validated WebView2 host; LODs work.
- glTF-Validator clean; asset audit clean.
- All evidence is WEBP or JSON; no PNG/JPEG in the repo.

## 11. Coordination

- **Soma** owns mesh topology, rig and shaders. Aesthesis writes the audit, material recipes and acceptance criteria; Soma executes geometry edits; Aesthesis verifies.
- **Axon** owns packaging and icon generation. Aesthesis supplies the SVG source and the generation script contract.
- **Cortex** reviews the block and checks doc accuracy.
- No agent may delete another agent's geometry without a recorded decision. Fix forward.
