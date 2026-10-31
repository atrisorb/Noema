# NOEMA packaging

Local scripts and the CI matrix that produce the release artifacts.

| Target | Script | Artifact |
|---|---|---|
| Windows x64 | `packaging/build.ps1` | `noema-<ver>-windows-x64.zip`, NSIS `-setup.exe` when `makensis` exists |
| macOS | `packaging/build.sh` | `noema-<ver>-macos-<arch>.dmg` |
| Linux x64 | `packaging/build.sh` | AppImage, .deb, .rpm (when `rpmbuild` exists), .tar.gz |
| Windows desktop | `packaging/tauri.ps1` | NSIS + MSI installers (Tauri v2) |
| macOS desktop | `packaging/tauri.sh` | universal `.dmg` (Tauri v2) |
| Linux desktop | `packaging/tauri.sh` | AppImage, `.deb`, `.rpm` (Tauri v2) |

Both scripts:

1. build `noema`, `noemad` and `noema-app` in release mode;
2. build the UI with Vite (`npm ci && npm run build`);
3. stage binaries + `ui/` + `assets/` + README + LICENSE;
4. emit the platform artifact under `dist/`.

The standalone and desktop paths are independent. `build.ps1` / `build.sh` stay
the portable zip/tar path with no WebKit requirement. Both packaging scripts ensure
generated platform icons (`src-tauri/icons/`) are up-to-date from `assets/brand/noema-app-icon.svg`
via `tools/design/build-icons.mjs` without committing raster images.
`tauri.ps1` / `tauri.sh` reuse the same `target/release` binaries and UI dist, stage `noema-app`
as a sidecar, and enforce strict loopback CSP and narrowed capability origins (`main.json`).
`packaging/tauri.bundle.conf.json` is passed to cargo-tauri for the `externalBin` + `ui/`/`assets/`
resources. That override is bundle-time only, so `cargo check --workspace` works on a
clean checkout with no staged sidecar or UI dist.

CI: `.github/workflows/release.yml` runs the full test suite on Windows, macOS 14
and ubuntu-24.04, runs the standalone script, installs tauri-cli 2.12.4, then runs the
desktop bundle script. Linux installs WebKitGTK 4.1, patchelf and build deps; macOS adds
both Apple targets for the universal build. Artifacts upload from `dist/**`.

Local verification (owner host `hp-atr`, 2026-10-31): Windows zip built by
`packaging/build.ps1`; `packaging/tauri.ps1 -SkipBuild` produced
`NOEMA_0.1.0_x64-setup.exe` (5.18 MiB) and `NOEMA_0.1.0_x64_en-US.msi` (7.91 MiB) in
`dist/tauri/`, with `noema-app.exe`, `ui/` and `assets/` in the WiX file list. macOS and
Linux bundles are produced by their runners.
