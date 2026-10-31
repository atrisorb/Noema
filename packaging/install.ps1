# NOEMA Windows Auto-Installer & PATH Manager
# Deploys NOEMA binaries, UI and assets to disk, and configures user PATH.
param(
  [string]$TargetDir = $(if (Test-Path 'F:\') { 'F:\NOEMA' } else { "$HOME\NOEMA" }),
  [switch]$ForceDownload
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

Write-Host "=== NOEMA Windows Installer ===" -ForegroundColor Cyan
Write-Host "Destination: $TargetDir"

# 1. Create target directory
if (-not (Test-Path $TargetDir)) {
  New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

# 2. Populate binaries from local stage or download from GitHub
$localStage = Get-ChildItem -Path (Join-Path $root 'dist\noema-*-windows-x64') -Directory -ErrorAction SilentlyContinue |
  Select-Object -Last 1

if ($localStage -and -not $ForceDownload) {
  Write-Host "Deploying from local build: $($localStage.FullName)" -ForegroundColor Green
  Copy-Item -Path (Join-Path $localStage.FullName '*') -Destination $TargetDir -Recurse -Force
} else {
  Write-Host "Fetching latest release metadata from GitHub..." -ForegroundColor Yellow
  $release = Invoke-RestMethod -Uri 'https://api.github.com/repos/atrisorb/noema/releases/latest' -Headers @{'User-Agent'='NOEMA-Installer'}
  $asset = $release.assets | Where-Object { $_.name -like '*windows-x64.zip' } | Select-Object -First 1
  if (-not $asset) {
    throw "No matching windows-x64.zip asset found in release $($release.tag_name)"
  }
  $zipPath = Join-Path ([System.IO.Path]::GetTempPath()) $asset.name
  Write-Host "Downloading $($asset.name) ($([math]::Round($asset.size / 1MB, 2)) MB)..." -ForegroundColor Yellow
  Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath
  Write-Host "Extracting to $TargetDir..." -ForegroundColor Yellow
  Expand-Archive -Path $zipPath -DestinationPath $TargetDir -Force
  Remove-Item $zipPath -Force -ErrorAction SilentlyContinue
}

# 3. Enroll into PATH via noema CLI
$noemaExe = Join-Path $TargetDir 'noema.exe'
if (-not (Test-Path $noemaExe)) {
  throw "Installation failed: noema.exe not found in $TargetDir"
}

Write-Host "Configuring user PATH and companion toolchain..." -ForegroundColor Cyan
$setupJson = & $noemaExe path setup
Write-Host $setupJson

Write-Host "`n=== Installation Complete ===" -ForegroundColor Green
Write-Host "NOEMA is installed at: $TargetDir"
Write-Host "Run 'noema --help' from any new PowerShell/CMD window."
Write-Host "Launch desktop app: $(Join-Path $TargetDir 'noema-app.exe')"
