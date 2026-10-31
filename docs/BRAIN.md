# NOEMA - Brain view (Block 6)

Owner: **Hippocampus**. Code: `crates/hippocampus` (Rust math), `crates/app/src/api.rs` (`/api/brain`, `/api/dream`), `crates/app/ui/src/brain.ts` + `brain-worker.ts` (view).

## Data flow

`thoughts.embedding` -> `noema_hippocampus::project` -> `GET /api/brain` -> physics worker -> instanced SDF nodes + vector edges + SVG labels.

The database stays the source of truth. The browser never invents topology: positions and pulse values arrive from Rust. The worker only integrates the layout at 60 fps.

Live activation: Thalamus posts normalised `StreamEvent` batches to `POST /api/brain/telemetry`; `ActivationWindow` turns them into one sample. `GET /api/brain` and `GET /api/brain/telemetry` expose that sample for 30 s before the derived values return.

## Physics (reference: `crates/hippocampus/src/physics.rs`)

- Repulsion: inverse-square, `k / max(d^2, 0.0004)`. Up to 200 nodes every pair is evaluated. Above 200 a uniform grid evaluates only the 27 neighbouring cells; `cell = cbrt(4.2 / n)` clamped to `0.08..0.35`, so far-field repulsion is intentionally dropped.
- Springs: Hooke, rest length 0.32, weight from the cosine edge.
- Integration: damped velocity (0.86 wake, 0.94 dream), speed cap.
- Tether: **hard clamp** to `max_tether_radius = 1.0` for every node, including a pinned drag target. Dragging one thought moves that node only; the graph cannot cascade past the sphere.
- Dream mode relaxes spring (0.35) and raises repulsion (0.05) and damping (0.94); wake restores the defaults.

`cargo test -p noema-hippocampus` covers 600-tick clamp, a 4.0-unit drag target, and dream-mode clamp.

## Projection (reference: `projection.rs`)

UMAP-style core, implemented in-repo (no new dependency):

1. Deterministic seed from the thought id, scattered inside the anchor of its functional region.
2. Cosine kNN graph, k = 4, weights `(cos + 1) / 2`.
3. SGD iterations: attract to the neighbour centroid (0.10), pull to the region anchor (0.04), pairwise repulsion (0.0009 / d^2), then normalise to the unit sphere.

Regions and anchors: prefrontal `(0, 0.30, 0.70)`, motor `(0, 0.62, 0.05)`, Broca `(-0.62, -0.05, 0.45)`, Wernicke `(0.66, -0.10, 0.30)`, hippocampus `(0.30, -0.48, -0.12)`.

Deterministic: identical ids + embeddings produce identical coordinates across restarts. Clusters in `cluster_id` choose the region; unknown clusters fall back to prefrontal.

## Activation telemetry (`telemetry.rs`)

`Activation { logprob_entropy, thinking_density, cosine_drift }` -> `Pulse { hz, intensity, hue }`:

- `mix = 0.45 * entropy/3 + 0.35 * density + 0.20 * drift`
- `hz = 0.35 + 2.4 * mix`, `intensity = 0.35 + 0.65 * mix`, `hue = 190 + 120 * drift`.

Live path: `POST /api/brain/telemetry` accepts `{ events: StreamEvent[], cosine_drift?: 0..1, reset?: bool }`. `ActivationWindow` uses mean surprisal `-ln p` as the entropy proxy (clamped to 3), `reasoning / (reasoning + output)` as thinking density and the caller-supplied drift. The sample stays live for 30 s; after that `/api/brain` falls back to the derived values (importance and neighbour cosine). The per-node `pulse` contract does not change.

## Rendering

- Nodes: one instanced quad per thought; fragment shader draws an analytic SDF disc with `fwidth`, so edges stay crisp at any zoom. No raster text.
- Edges: vector `LineSegments` from the cosine kNN graph.
- Labels: SVG `<text>` overlay, top 12 nodes by importance, overlap culling (48 px), only when the camera is within 0.85 units of the brain centre.
- Picking and drag: screen-space nearest node (22 px); drag raycasts a camera-facing plane and pins the node in the worker. On pointer up the final local position is posted to `/api/brain/drag` and stored in `thoughts.pos_x/y/z`; stored coordinates win over the projection on the next read.
- Live pulse: while the brain view is open the UI polls `/api/brain/telemetry` every 1.5 s; a `live` sample updates the per-node `aHz` attribute and hue.
- Click: card with title, region, agent, timestamp, importance, body and strongest links.
- View: `brain` button flies to the brain; `dream` toggles Dream Mode and posts the run to `/api/dream`.

## API

`GET /api/brain` -> `{ nodes: [{ id, title, body, agent_id, created_at, importance, region, pos, pulse }], edges: [{ from, to, weight }], regions: [{ id, anchor }], telemetry: { source: 'live'|'derived', activation?, pulse?, updated_at_ms }, generated_at_ms }`.

`GET /api/brain/telemetry` -> `{ source, activation, pulse, updated_at_ms }`.

`POST /api/brain/telemetry` `{ events, cosine_drift?, reset? }` -> same shape, `source` is `live` or `empty`.

`POST /api/brain/drag` `{ id, pos: [x, y, z] }` -> `{ id, pos, persisted: true }`; clamps to the unit sphere and updates `thoughts.pos_x/y/z`.

`POST /api/dream` `{ summary, thought_count, insights }` -> `{ id, persisted: true, thought_count }`; row in `dreams`.

## Evidence

`docs/evidence/block6-brain-zoom.png`, `block6-brain-far.png`, `block6-brain-dream.png`, `block6-brain-thought.png`; `crates/app/ui/tests/block6-brain.spec.ts`.

## Limits

- The feed is push-from-Thalamus, pull-from-UI: the caller posts each normalised batch and the UI polls every 1.5 s. No SSE endpoint exists yet.
- UMAP is implemented as its kNN + SGD core, not the `umap` crate or a Python service. A future full UMAP can replace `project` behind the same API.
- Above 200 nodes repulsion is local to the 27-cell neighbourhood; long-range repulsion is dropped by design. The grid and the Rust reference use the same cell-size formula.
