# NOEMA — STUDIO CONTRACT

Block 7 surface: session restore, projects, role-model assignment, BI widgets.
Owner: Retina. UI: `crates/app/ui/src/studio.ts`. Server: `crates/app/src/api.rs`.

## 1. Routes

| Route | Behaviour |
|---|---|
| `/` | 3D body. HUD shows the restored session pill. |
| `/#studio` | Studio overlay opens on boot; hash is the route. |
| `studio` button | Toggles the same overlay and sets/clears the hash. |

The 3D scene keeps rendering behind the overlay. No empty landing page exists in either route.

## 2. Snapshot — `GET /api/studio`

```json
{
  "session": {"id","project_id","agent_id","model_id","cwd","repo","branch","updated_at"},
  "projects": [{"id","name","root_path"}],
  "sessions": [ ...same shape as session, 25 newest... ],
  "models": [{"id","name","capability"}],
  "roles": {"chat": {"global": "model-id", "projects": {"project-id": "model-id"}}},
  "widgets": [{"id","name","chart","spec","x","y","w","h","pinned"}],
  "usage": [{"agent","model","project","kind","peak","thinking","events","tokens","cost_usd","latency_ms","at"}]
}
```

- `session` is the newest row in `sessions.updated_at`; the CLI and the Studio share it.
- `roles` always contains the five capabilities: embedding, router, chat, coding, dreaming.
- `usage` is grouped by agent, model, project, kind, peak and thinking; `latency_ms` is the event-weighted average and is null when no sample exists.
- When `widgets` is empty the server seeds four defaults: tokens by agent, cost by model, latency by peak band, tokens by thinking.

## 3. Writes — `POST /api/studio`

Every action returns the full snapshot above.

| Action | Body |
|---|---|
| `session` | `{"action":"session","id":"..."}` — bumps `updated_at`, proving restore. |
| `role` | `{"action":"role","capability":"chat","model":"...","project":"..."}` — omit `project` for the global default. |
| `widget` | `{"action":"widget","widget":{...}}` — upsert by id; missing id gets a generated one. |
| `widget_remove` | `{"action":"widget_remove","id":"..."}`. |
| `layout` | `{"action":"layout","widgets":[{"id","x","y","w","h"}]}` — clamped to a 12-column grid. |

Role keys are stored in `settings`: `role.<capability>@global` and `role.<capability>@project:<id>` (scope `studio`). No new table.

## 4. Widget spec

```json
{
  "metric": "tokens | cost | latency",
  "dimension": "agent | model | project | peak | thinking | kind",
  "filters": [{"dimension": "agent", "value": "soma"}]
}
```

- A filter matches when `String(row[dimension]) === value` after the projection in `valueOf`.
- `peak` renders `peak` / `off-peak`; `project` renders the project name or `unassigned`; `thinking` renders `on` / `off` / `n/a`.
- The client aggregates up to six groups, sorts by the chosen metric descending and derives bar width from the largest group. Tokens are summed, cost is summed, latency is event-weighted.
- Filter-to-widget stores the arbitrary filter list, so any future dimension composes without a schema change.

## 5. Schema v2

`crates/core/migrations/0002_studio.sql` adds to `usage_events`:

- `latency_ms INTEGER` — nullable; the BI latency metric ignores null samples.
- `project_id TEXT` — BI attribution; joins to `projects.name`.
- `thinking TEXT` — `on` / `off`; unknown is `n/a`.

The migration runner applies it once; `migrations_apply_once` asserts the count from the migration list.

## 6. Verified behaviour

- `cargo test -p noema-app`: `studio_restores_session_and_round_trips_roles_widgets_layout` covers snapshot, global and project role writes, widget upsert, layout clamp, session restore and removal.
- Playwright `tests/block7-studio.spec.ts`: opens `/#studio`, asserts session repo/branch, changes a role, creates a filtered widget, drags it, reloads and asserts role + widget + layout, then removes it.
- Evidence: `docs/evidence/block7-studio.webp`, `docs/evidence/block7-widget-filter.webp`.

## 7. Decisions and limits

1. Boot stays on the 3D body; `#studio` is the Studio route. This keeps the Block 6 visual QA valid and still restores the session state in the HUD.
2. No chart dependency: bars are DOM/CSS; the grid is CSS grid with pointer math. React/uPlot stay out until a chart needs axes.
3. The feed is a pull; no SSE. Thalamus pushes telemetry into `/api/brain/telemetry`; the Studio reads `/api/studio` on open and after each write.
4. `thinking` and `latency_ms` are populated by the demo seed today. Live adapters must write them in the same `usage_events` insert to make the BI exact.

## 8. CLI parity

Every Studio action has a `noema` command. The CLI talks to the local bridge on
`NOEMA_APP_PORT` (default 4173) with one HTTP/1.1 request per invocation.

| Action | Command |
|---|---|
| Snapshot (session, projects, roles, widgets, usage) | `noema studio` |
| Focus a session | `noema studio session <id>` |
| Assign a role model | `noema studio role <capability> <model> [--project <id>]` |
| Upsert a widget | `noema studio widget '<json>'` |
| Remove a widget | `noema studio widget-remove <id>` |
| Persist a layout | `noema studio layout '[{"id","x","y","w","h"}]'` |

Body/daemon actions mirror the rest of the GUI: `health`, `anatomy`, `anima`, `brain`
(`telemetry`, `drag`, `dream`), `system` and `session`. `noema methods` prints the daemon RPC
catalog; `noema rpc <method> [json]` reaches any of them.

Parity is enforced by tests, not prose: `every_gui_route_has_a_cli_command` and
`every_rpc_method_has_a_cli_path` fail the build when a new route or method has no CLI path.
