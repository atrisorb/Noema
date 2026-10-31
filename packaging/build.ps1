# NOEMA Windows packaging: stage the binaries + UI + assets and zip them.
# Optional NSIS installer when makensis is on PATH.
param(
  [ValidateSet('release', 'debug')] [string]$Profile = 'release',
  [switch]$SkipBuild,
  [switch]$SkipUiBuild
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$version = (Select-String -Path (Join-Path $root 'Cargo.toml') -Pattern '^version = "([^"]+)"' |
  Select-Object -First 1).Matches[0].Groups[1].Value
$profileDir = if ($Profile -eq 'release') { 'release' } else { 'debug' }
$out = Join-Path $root 'dist'
$stage = Join-Path $out "noema-$version-windows-x64"

if (-not $SkipBuild) {
  cargo build --profile $Profile -p noema-cli -p noema-core -p noema-app -p noema-desktop
  if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
}
if (-not $SkipUiBuild) {
  Push-Location (Join-Path $root 'crates\app\ui')
  try {
    npm ci
    if ($LASTEXITCODE -ne 0) { throw 'npm ci failed' }
    npm run build
    if ($LASTEXITCODE -ne 0) { throw 'vite build failed' }
  } finally { Pop-Location }
}

Remove-Item $stage -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $stage -Force | Out-Null
$bin = Join-Path $root "target\$profileDir"
foreach ($exe in 'noema.exe', 'noemad.exe', 'noema-app.exe', 'noema-desktop.exe') {
  if (Test-Path (Join-Path $bin $exe)) {
    Copy-Item (Join-Path $bin $exe) $stage
  }
}
Copy-Item (Join-Path $root 'crates\app\ui\dist') (Join-Path $stage 'ui') -Recurse
Copy-Item (Join-Path $root 'assets') (Join-Path $stage 'assets') -Recurse
Copy-Item (Join-Path $root 'README.md'), (Join-Path $root 'LICENSE') $stage

$zip = Join-Path $out "noema-$version-windows-x64.zip"
Remove-Item $zip -Force -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -CompressionLevel Optimal
Write-Host "staged $stage"
Write-Host "zip    $zip"

$makensis = Get-Command makensis -ErrorAction SilentlyContinue
if ($makensis) {
  $nsi = Join-Path $out 'noema-installer.nsi'
  @"
Unicode true
Name "NOEMA"
OutFile "$out\noema-$version-windows-x64-setup.exe"
InstallDir "\$PROGRAMFILES64\NOEMA"
Page directory
Page instfiles
Section
  SetOutPath "\$INSTDIR"
  File /r "$stage\*.*"
  CreateShortcut "\$DESKTOP\NOEMA.lnk" "\$INSTDIR\noema-desktop.exe"
SectionEnd
"@ | Set-Content -Path $nsi -Encoding UTF8
  & $makensis.Source $nsi
  Write-Host "setup  $out\noema-$version-windows-x64-setup.exe"
} else {
  Write-Host 'makensis not found: zip artifact only'
}
