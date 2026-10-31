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
    cargo build --release -p noema-app --target aarch64-apple-darwin
    cargo build --release -p noema-app --target x86_64-apple-darwin
    lipo -create       "$ROOT/target/aarch64-apple-darwin/release/noema-app"       "$ROOT/target/x86_64-apple-darwin/release/noema-app"       -output "$BINARIES/noema-app-universal-apple-darwin"
    cargo tauri build --config "$ROOT/packaging/tauri.bundle.conf.json" \
      --target universal-apple-darwin --bundles dmg
    cp "$ROOT/target/universal-apple-darwin/release/bundle/dmg/"*.dmg "$OUT/"
    ;;
  Linux)
    TRIPLE=$(rustc -vV | sed -n 's/^host: //p')
    cp "$ROOT/target/release/noema-app" "$BINARIES/noema-app-$TRIPLE"
    cargo tauri build --config "$ROOT/packaging/tauri.bundle.conf.json" --bundles appimage,deb,rpm
    find "$ROOT/target/release/bundle" -type f \( -name '*.AppImage' -o -name '*.deb' -o -name '*.rpm' \) -exec cp {} "$OUT/" \;
    ;;
  *)
    echo "unsupported platform: $(uname -s)" >&2
    exit 1
    ;;
esac

ls -1 "$OUT"
