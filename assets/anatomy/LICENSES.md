# Anatomy asset licenses

## Real anatomy atlas (CC BY-SA 4.0)

| File | Source | Author | License |
|---|---|---|---|
| `atlas/body_full.glb` | `dohunjeon51-design/web-anatomy-3d` `public/models/body.glb`, derived from Z-Anatomy | Z-Anatomy contributors (Lluís Vinent Juanico et al.) | **CC BY-SA 4.0** |

- 826 named anatomical structures (bones, muscles, cartilages, teeth) in a 1.72 m frame; 291k triangles.
- NOEMA classifies the meshes into layers by structure name and assigns PBR palette colours per structure (bone ivory, muscle red, etc.). That tinting is a derivative work, distributed under the same **CC BY-SA 4.0**.
- Attribution must remain visible in the About screen and in this file. Share-alike applies to the atlas and its derivatives.
- The processed Z-Anatomy system files were evaluated and discarded: they carry only metallic/roughness palette textures, no albedo maps, and duplicate the body surface.

## Procedural assets (CC0-1.0)

| File | Source | Author | License |
|---|---|---|---|
| `heart.glb` | procedural mesh, this repo (`tools/anatomy/build_glb.mjs`) | NOEMA contributors | CC0-1.0 |
| `brain.glb` | procedural mesh, this repo | NOEMA contributors | CC0-1.0 |

## CAD artifacts (CC0-1.0)

| File | Source | Author | License |
|---|---|---|---|
| `cad/vertebra.json`, `cad/vertebra.eval.json`, `cad/vertebra.step`, `cad/vertebra.png` | parametric part, this repo, SolidCraft CLI 0.28.0 | NOEMA contributors | CC0-1.0 |
| `cad/vertebra-section.dxf`, `cad/vertebra-section.pdf`, `cad/vertebra-section.svg` | drawing, this repo, CADCraft CLI 0.15.1 | NOEMA contributors | CC0-1.0 |

No third-party texture, scan or paid asset is committed. Code in `crates/app/ui/src/body.ts` is MIT.

Regenerate procedural meshes: `node tools/anatomy/build_glb.mjs`.
Optimise/verify an atlas GLB: `node tools/anatomy/inspect_glb.mjs <file>`.
