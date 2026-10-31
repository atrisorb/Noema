# Platforms

> Status: Windows + macOS + Linux are first-class targets (added 2026-10-31, Block 5.5).
> Windows is the validated build host; macOS packaging is defined; Linux has code paths,
> packaging targets and CI defined, but no physical Linux host has validated it yet.

## 1. Support matrix (v1)

| Target | Minimum | Tier | Runtime | Notes |
|---|---|---|---|---|
| Windows 10 22H2 / Windows 11 | x64 | 1, validated | WebView2 (Chromium) | Primary development host (hp-atr, Windows 10.0.26300) |
| macOS 13 Ventura+ | Intel + Apple Silicon, universal | 1, CI-defined | WKWebView | Minimum matches Tauri v2 support; test on macos-14 runners |
| Linux | x86_64, glibc 2.39+ | 1, code path + CI | WebKitGTK 4.1 | Ubuntu 24.04, Fedora 40+, Arch rolling; Wayland and X11 |

Not in v1: Android, iOS, Windows ARM64 native, musl/Alpine, 32-bit. They are not forbidden;
they are simply not built or tested.

## 2. What is platform-specific in the code

| Concern | Implementation |
|---|---|
| Data root | crates/core/src/platform.rs, data_dir() |
| Secret backend | platform::keychain_backend(): dpapi / keychain / secret-service |
| Hardware probe | crates/core/src/hardware.rs, one cfg block per OS |
| Port file ACL | mode 0600 on unix; icacls /inheritance:r /grant:r <user>:F on Windows (best-effort, warning logged) |
| Disk command | PowerShell CIM on Windows; df -kP on macOS/Linux |
| GPU command | PowerShell CIM / system_profiler / nvidia-smi + lspci + sysfs |
| Packaging | `packaging/build.ps1` (Windows), `packaging/build.sh` (macOS/Linux), docs/ARCHITECTURE.md section 8 |

Everything else (crates, JSON-RPC, SQLite, Three.js bundle, CLI) is OS-neutral Rust/JS.

## 3. Filesystem layout

| Item | Windows | macOS | Linux |
|---|---|---|---|
| Data root | %APPDATA%/noema | ~/Library/Application Support/noema | $XDG_DATA_HOME/noema, else ~/.local/share/noema |
| Database | <data>/noema.db | <data>/noema.db | <data>/noema.db |
| Daemon port file | <data>/daemon.json | <data>/daemon.json | <data>/daemon.json |
| Runtime override | NOEMA_DATA_DIR | NOEMA_DATA_DIR | NOEMA_DATA_DIR |

NOEMA_DATA_DIR wins on every OS and is the supported test/portable mode. The daemon
creates the directory, writes the port file atomically and removes it on clean shutdown.

## 4. Secrets

Provider keys, GitHub tokens and MCP OAuth tokens must live in the OS keychain, never in
the repo or a plain config file:

- Windows: DPAPI-backed credential store (dpapi).
- macOS: Keychain (keychain).
- Linux: Secret Service / libsecret (secret-service), requires gnome-keyring or kwallet
  running; headless CI uses environment fallback only.

The env fallback (NOEMA_GITHUB_TOKEN, GITHUB_TOKEN) stays for CI and is documented as
lower-trust.

## 5. Rendering path per OS

| OS | WebView | GPU API | Fallback |
|---|---|---|---|
| Windows | WebView2 / Chromium | ANGLE D3D11 (validated: 60.2 fps at 1600x900 on GTX 1650) | SwiftShader (software), profile quality low |
| macOS | WKWebView / Safari | Metal via WebKit | software WebGL |
| Linux | WebKitGTK 4.1 | Mesa OpenGL / Vulkan through WebKit | llvmpipe (software), profile quality low |

The hardware profile (docs/HARDWARE.md) sets the pixel-ratio cap and particle budget.
A software GPU is classified software and never gets a discrete/ultra profile.

## 6. Packaging

Two paths, one source tree.

1. **Standalone** — portable zip/tar with `noema`, `noemad`, `noema-app`, `ui/` and `assets/`. No WebKit requirement; the shell is the browser.
2. **Tauri v2 desktop** — `src-tauri` (`noema-desktop`) opens a native window on the same loopback Studio, bundles `noema-app` as a sidecar and `ui/`+`assets/` as resources.

| Target | Command | Output |
|---|---|---|
| Windows | `packaging/build.ps1` | `noema-<ver>-windows-x64.zip`; NSIS `-setup.exe` when `makensis` exists |
| macOS | `packaging/build.sh` | `noema-<ver>-macos-<arch>.dmg` |
| Linux | `packaging/build.sh` | AppImage, `.deb`, `.rpm` (when `rpmbuild` exists), `.tar.gz` |
| Windows desktop | `packaging/tauri.ps1` | NSIS `.exe` + MSI |
| macOS desktop | `packaging/tauri.sh` | universal `.dmg` |
| Linux desktop | `packaging/tauri.sh` | AppImage, `.deb`, `.rpm` |
| CLI | `cargo build --release -p noema-cli -p noema-core` | `noema[.exe]`, `noemad[.exe]` |

Standalone artifacts contain `noema`, `noemad`, `noema-app`, `ui/` (Vite build), `assets/`, README and
LICENSE. Desktop bundles contain the `noema-desktop` shell, `noema-app` as a sidecar and
`ui/` + `assets/` resources. Signing and notarisation stay deferred; CI publishes unsigned artifacts first.

### CI matrix

| Runner | Builds |
|---|---|
| windows-latest | `cargo test --workspace`, zip + NSIS setup, Tauri NSIS/MSI |
| macos-14 | `cargo test --workspace`, `.dmg`, universal Tauri `.dmg` |
| ubuntu-24.04 | `cargo test --workspace`, AppImage + `.deb` + `.rpm`, Tauri bundles |

Workflow: `.github/workflows/release.yml`. Linux installs `rpm`, `appimagetool`, WebKitGTK 4.1,
`patchelf` and the Tauri build deps; the binaries are glibc-linked, so the ubuntu-24.04
artifact targets glibc 2.39+. macOS adds `aarch64-apple-darwin` and `x86_64-apple-darwin` for the
universal desktop bundle. The standalone artifacts still need no WebKit; only the Tauri path does.

## 7. Linux runtime prerequisites

- WebKitGTK 4.1 (libwebkit2gtk-4.1-0) - Tauri v2 requires 4.1, not 4.0.
- GTK 3 and a working display (Wayland or X11); the app is a normal desktop window, no XWayland-only assumption.
- libsecret-1-0 plus a running Secret Service for keychain storage.
- Optional: libayatana-appindicator3-1 for the tray icon.
- pciutils (lspci) and nvidia-smi (NVIDIA driver) improve GPU detection; both are optional.
- Named pipes: the daemon uses loopback TCP everywhere; a unix socket is an optional future path.

### WSL

WSL2 exposes the Windows GPU through /dev/dxg and a virtual NIC. The probe sees the VM's
CPU/RAM, the host GPU name (through the driver) and the VM disk. Treat WSL profiles as
approximate; run noema system profile --refresh after a GPU driver update. The desktop
app is not a WSL target in v1; the CLI/daemon are.

## 8. Feature parity

| Feature | Windows | macOS | Linux |
|---|---|---|---|
| CLI + daemon | yes | yes | yes |
| SQLite + WAL | yes | yes | yes |
| Port file ACL | icacls | 0600 | 0600 |
| Hardware profile | CIM | sysctl + system_profiler | /proc + nvidia-smi/lspci/sysfs |
| Studio / 3D | WebView2 | WKWebView | WebKitGTK |
| Keychain | DPAPI | Keychain | Secret Service |
| Installer | NSIS/MSI | DMG | AppImage/deb/rpm |
| Auto-start | Block 8 | Block 8 | Block 8 |

## 9. Known gaps

1. No physical Linux host has run the suite yet; the code paths are compiled on Windows
   through cfg branches, so the first Linux CI run is the real proof.
2. macOS signing/notarisation need an Apple Developer account; the universal (arm64 + x64) build is configured in CI.
3. Windows ARM64 and Linux aarch64 are not built; the probe and paths are arch-neutral.
4. Linux tray/appindicator integration and GPU sandbox flags remain follow-up work.
5. Windows VRAM above 4 GiB is capped by CIM (see docs/HARDWARE.md section 8).

## 10. Verification

~~~
cargo check --workspace --all-targets
cargo test -p noema-core
NOEMA_DATA_DIR=target/cli-profile cargo run -p noema-cli -- system profile
cd crates/app/ui && npm run build && npx playwright test
~~~

Windows evidence is recorded in docs/evidence/block6-hardware.png. Linux/macOS evidence
is added by their CI jobs in Block 8.


