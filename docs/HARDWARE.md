# Hardware awareness

> Status: implemented 2026-10-31 (Block 5.5). Code: `crates/core/src/hardware.rs`,
> `crates/core/src/platform.rs`. Runtime profile is local-only; nothing is uploaded.

NOEMA detects the machine it runs on once, derives a full tuning profile, and lets the
owner override every field. The same row in the `settings` table feeds the 3D world,
the daemon and the CLI, so GUI and terminal never disagree.

## 1. Why

- Pick render defaults that hold 60 fps on the actual GPU instead of a fixed constant.
- Pick local-inference defaults (context, GPU layers, parallelism) that fit VRAM/RAM.
- Size daemon threads and the SQLite page cache to the CPU/RAM.
- Surface the facts in the Studio ("Machine" panel) and in `noema system profile`.
- Give the owner a documented override path for every derived number.

## 2. Storage

| Key | Scope | Content |
|---|---|---|
| `hardware.profile` | `hardware` | `HardwareProfile` JSON, stamped `detected_at_ms` |
| `hardware.overrides` | `hardware` | Sparse JSON patch, deep-merged over the derived tuning |

Cache TTL is 12 h (`CACHE_TTL_MS`). `refresh = true` bypasses it and rewrites the row.
The cache lives in the normal NOEMA database, so `NOEMA_DATA_DIR` moves it with the rest
of the state. Overrides survive profile refreshes.

## 3. Detection matrix

One probe per OS; no extra Rust dependency, no network.

| Field | Windows | macOS | Linux |
|---|---|---|---|
| hostname | `%COMPUTERNAME%` | `sysctl kern.hostname` | `/proc/sys/kernel/hostname` |
| CPU brand | `Win32_Processor.Name` (CIM) | `sysctl machdep.cpu.brand_string`, fallback `hw.model` | `/proc/cpuinfo` `model name` / `Hardware` / `Processor` |
| physical cores | `Win32_Processor.NumberOfCores` | `sysctl hw.physicalcpu` | distinct `(physical id, core id)` pairs in `/proc/cpuinfo` |
| logical cores | `Win32_Processor.NumberOfLogicalProcessors` | `sysctl hw.logicalcpu` | `std::thread::available_parallelism` |
| RAM | `Win32_ComputerSystem.TotalPhysicalMemory` | `sysctl hw.memsize` | `/proc/meminfo` `MemTotal` |
| GPUs | `Win32_VideoController` (name, AdapterRAM, DriverVersion) | `system_profiler SPDisplaysDataType -json` | `nvidia-smi` CSV, then `lspci` VGA/3D/Display, then `/sys/class/drm/card0/device/uevent` |
| VRAM | `AdapterRAM` converted to MB | `spdisplays_vram` / `spdisplays_vram_shared`; Apple Silicon = 75% of RAM (unified) | `nvidia-smi memory.total`; AMD `mem_info_vram_total` |
| data volume | `Win32_LogicalDisk` for the `NOEMA_DATA_DIR` drive | `df -kP` | `df -kP` |
| platform / arch | `std::env::consts::{OS, ARCH}` | same | same |

GPU classification (`GpuKind`): `discrete`, `integrated`, `software`, `unknown`.
Software keywords: Microsoft Basic, Remote Display, Parsec, llvmpipe, SwiftShader, "virtual".
Integrated keywords: Intel, Apple, Iris, UHD Graphics, Radeon Graphics, Vega 8, Adreno.
Discrete keywords: NVIDIA, GeForce, RTX, Quadro, Radeon RX, Arc A/B. Fallback: VRAM ≥ 2048 MB → discrete.
Duplicate names are removed; the highest (rank, VRAM) pair wins the tuning.

Windows probing is a single `powershell -NoProfile -NonInteractive` call returning one JSON
object (CPU + RAM + GPUs + data-volume space). `` is forbidden in the probe path; the
PowerShell command is fixed and receives the target directory through `NOEMA_PROBE_DIR`.

## 4. Derived tuning

`derive_tuning` is pure. Thresholds, in order:

| Quality | Condition |
|---|---|
| `ultra` | best VRAM ≥ 8000 MB and RAM ≥ 32000 MB |
| `high` | VRAM ≥ 6000 MB, or VRAM ≥ 4000 MB and RAM ≥ 16000 MB |
| `balanced` | a non-software GPU and RAM ≥ 8000 MB |
| `low` | everything else (including no GPU) |

| Field | ultra | high | balanced | low |
|---|---|---|---|---|
| `render.pixel_ratio_cap` | 2.0 | 1.75 | 1.5 | 1.0 |
| `render.particle_budget` | 4000 | 2500 | 1500 | 600 |
| `render.shadows` | true | true | false | false |
| `render.max_texture_mb` | 2048 | 1024 | 512 | 256 |
| `render.lod_bias` | 1.0 | 1.0 | 1.15 | 1.4 |

`particle_budget` is the total circulation budget; 980 is the historical default, the
UI scales all six flows proportionally when the budget changes (min 24 particles/flow).

Inference (`inference`):

| Field | Rule |
|---|---|
| `max_context_tokens` | VRAM ≥ 16000 → 32768; ≥ 8000 → 16384; ≥ 4000 → 8192; else 4096. Capped at 8192 when RAM < 16000 MB |
| `gpu_layers` | `-1` (full offload) when VRAM ≥ 4000 MB, else `0` |
| `vram_budget_mb` | 85% of best VRAM |
| `parallel_requests` | `clamp(logical / 2, 1, 8)` |
| `recommended_runtime` | `ollama` (VRAM ≥ 4000), `llama.cpp` (RAM ≥ 8000), `remote-only` otherwise |

Daemon (`daemon`): `worker_threads = clamp(logical − 1, 1, 16)`;
`sqlite_cache_mb = clamp(RAM / 32, 64, 1024)`.

`tuning.reasons` carries short human-readable evidence (CPU, RAM, data volume, GPU) and is
shown in the Machine panel.

## 5. Overrides

`hardware.overrides` is a JSON object deep-merged into the derived tuning:

- Objects merge recursively; scalars/arrays replace; `null` deletes a key.
- Unknown fields are rejected (`#[serde(deny_unknown_fields)]`) before writing, so a typo
  cannot silently poison the stored patch.
- The effective tuning must still deserialize, so invalid types are rejected too.

Example:

```json
{
  "quality": "balanced",
  "render": { "pixel_ratio_cap": 1.25, "particle_budget": 1200 },
  "inference": { "parallel_requests": 2, "max_context_tokens": 16384 },
  "daemon": { "worker_threads": 4 }
}
```

Every surface writes the same row:

| Surface | Call |
|---|---|
| CLI | `noema system override '{"render":{"particle_budget":1200}}'` |
| CLI read | `noema system profile [--refresh]` |
| CLI reset | `noema system reset` (drops overrides + cached profile, re-detects) |
| RPC | `system.profile {refresh}`, `system.override {overrides}`, `system.reset` |
| HTTP | `GET /api/system`, `GET /api/system/refresh`, `POST /api/system` with `{"overrides":{...}}` |
| Studio | header "machine" → Machine panel → quality preset / pixel cap / particle budget → apply |

The CLI falls back to the local database when the daemon is offline, so the profile is
always readable. `system.override` is the only mutation the Studio needs; there is no
config file to edit.

## 6. Runtime application

- `renderer.setPixelRatio(min(devicePixelRatio, render.pixel_ratio_cap))` at boot and after
  every save.
- `new Circulation(render.particle_budget)` at boot; `circulation.setBudget(n)` rebuilds the
  six flows on save (geometry and material disposed before rebuild).
- `max_texture_mb`, `shadows` and `lod_bias` are persisted for the brain/graph block and
  the Tauri packaging pass; they are visible and editable now, not yet consumed by a
  shadow pass (Block 5 renders without shadow maps).
- Inference and daemon values are the documented defaults the adapters/daemon must read;
  wiring the local runtime to them lands with the inference runtime integration.

## 7. Privacy and security

- Detection runs local OS commands only. No telemetry, no HTTP, no identifier leaves the host.
- The profile contains hostname, CPU/GPU names, RAM and free disk. It is stored in the
  local SQLite file next to session data; treat the database with the same care as a log.
- The probe never reads secrets, environment tokens or user files beyond `/proc` counters.

## 8. Known limits

- **Windows VRAM cap.** `Win32_VideoController.AdapterRAM` is a 32-bit field; cards with
  more than 4 GiB report `4095` MB (confirmed on the GTX 1650 host). The 4 GiB tier is
  still correct; higher cards fall into `high` instead of `ultra`. Reading the 64-bit
  registry `HardwareInformation.qwMemorySize` is the documented follow-up.
- **Apple Silicon.** Unified memory is reported as 75% of RAM; "VRAM" is not a physical
  figure. A 16 GB Mac therefore reports 12 GB and lands in `high`.
- **Linux.** Requires `lspci` (pciutils) for names; without it the sysfs driver name is used.
  Containers and WSL expose the host/VM GPU, not the guest's; `--refresh` after changing
  the GPU or docking station.
- **Multiple GPUs.** The best-ranked adapter drives the profile; per-process GPU selection
  is not implemented.
- **Disk.** Free space is the data-volume figure only, not every mounted volume.

## 9. Evidence

- Unit tests: quality tiers, override merge/delete, typo rejection, settings round-trip
  (`cargo test -p noema-core`, 7 tests).
- Live probe on `hp-atr`: i5-9400F (6/6), 24,456 MB RAM, GTX 1650 (4095 MB) → `high`,
  pixel cap 1.75, 2500 particles, context 8192, runtime `ollama`.
- HTTP: `GET /api/system` and `POST /api/system` verified against `noema-app serve`.
- CLI: `noema system profile` returns the same JSON with the daemon offline (local store).
- Playwright: `tests/block6-system.spec.ts` opens the Machine panel, saves an override and
  asserts the persisted value; screenshot `docs/evidence/block6-hardware.png`.
