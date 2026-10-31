#!/usr/bin/env bash
# NOEMA Tauri v2 desktop bundles: dmg (macOS universal) and AppImage/deb/rpm (Linux).
# Run after packaging/build.sh so the UI dist and target/release binaries exist.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"
BINARIES="$ROOT/src-tauri/binaries"
OUT="$ROOT/dist/tauri"
rm -rf "$BINARIES"
mkdir -p "$BINARIES" "$OUT"

if [ ! -f "$ROOT/crates/app/ui/dist/index.html" ]; then
  echo "UI bundle missing: run 'npm run build' in crates/app/ui" >&2
  exit 1
fi

case "$(uname -s)" in
  Darwin)
    rustup target add aarch64-apple-darwin x86_64-apple-darwin
    cargo build --release -p noema-app -p noema-cli -p noema-core --target aarch64-apple-darwin
    cargo build --release -p noema-app -p noema-cli -p noema-core --target x86_64-apple-darwin
    for b in noema-app noema noemad; do
      cp "$ROOT/target/aarch64-apple-darwin/release/$b" "$BINARIES/${b}-aarch64-apple-darwin"
      cp "$ROOT/target/x86_64-apple-darwin/release/$b" "$BINARIES/${b}-x86_64-apple-darwin"
      lipo -create \
        "$ROOT/target/aarch64-apple-darwin/release/$b" \
        "$ROOT/target/x86_64-apple-darwin/release/$b" \
        -output "$BINARIES/${b}-universal-apple-darwin"
    done
    cargo tauri build --config "$ROOT/packaging/tauri.bundle.conf.json" \
      --target universal-apple-darwin --bundles dmg
    find "$ROOT/target" -type f -name '*.dmg' -exec cp {} "$OUT/" \;
    ;;
  Linux)
    TRIPLE=$(rustc -vV | sed -n 's/^host: //p')
    for b in noema-app noema noemad; do
      cp "$ROOT/target/release/$b" "$BINARIES/${b}-$TRIPLE"
    done
    cargo tauri build --config "$ROOT/packaging/tauri.bundle.conf.json" --bundles appimage,deb,rpm
    find "$ROOT/target/release/bundle" -type f \( -name '*.AppImage' -o -name '*.deb' -o -name '*.rpm' \) -exec cp {} "$OUT/" \;
    ;;
  *)
    echo "unsupported platform: $(uname -s)" >&2
    exit 1
    ;;
esac

ls -1 "$OUT"
