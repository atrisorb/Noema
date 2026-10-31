# NOEMA Tauri v2 desktop bundles: stage the noema-app sidecar, then let
# cargo-tauri emit NSIS + MSI. Standalone zip/dmg/AppImage stay in build.ps1/build.sh.
param(
  [switch]$SkipBuild
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$ui = Join-Path $root 'crates\app\ui\dist\index.html'
$bin = Join-Path $root 'target\release'
$triple = ((rustc -vV) | Select-String '^host: ').Line.Split(':')[1].Trim()
$sidecar = Join-Path $root "src-tauri\binaries\noema-app-$triple.exe"

if (-not $SkipBuild) {
  cargo build --release -p noema-app
  if ($LASTEXITCODE -ne 0) { throw 'cargo build -p noema-app failed' }
}
if (-not (Test-Path $ui)) { throw "UI bundle missing: run 'npm run build' in crates/app/ui" }
Remove-Item (Join-Path $root 'src-tauri\binaries\*') -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force (Join-Path $root 'src-tauri\binaries') | Out-Null
Copy-Item (Join-Path $bin 'noema-app.exe') $sidecar -Force

cargo tauri build --config (Join-Path $root 'packaging\tauri.bundle.conf.json') --bundles nsis,msi
if ($LASTEXITCODE -ne 0) { throw 'cargo tauri build failed' }

$out = Join-Path $root 'dist\tauri'
New-Item -ItemType Directory -Force $out | Out-Null
Get-ChildItem (Join-Path $root 'target\release\bundle') -Recurse -Include *.exe, *.msi |
  Copy-Item -Destination $out -Force
Get-ChildItem $out | ForEach-Object { Write-Host "bundle $($_.FullName)" }
