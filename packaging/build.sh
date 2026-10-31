#!/usr/bin/env bash
# NOEMA packaging for macOS (.dmg) and Linux (AppImage, .deb, .rpm, .tar.gz).
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
VERSION=$(grep -m1 '^version = "' "$ROOT/Cargo.toml" | cut -d'"' -f2)
OUT="$ROOT/dist"
STAGE="$OUT/noema-$VERSION-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"

cargo build --release -p noema-cli -p noema-core -p noema-app
(cd "$ROOT/crates/app/ui" && npm ci && npm run build)

rm -rf "$STAGE"
mkdir -p "$STAGE"
cp "$ROOT/target/release/noema" "$ROOT/target/release/noemad" "$ROOT/target/release/noema-app" "$STAGE/"
cp -r "$ROOT/crates/app/ui/dist" "$STAGE/ui"
cp -r "$ROOT/assets" "$STAGE/assets"
cp "$ROOT/README.md" "$ROOT/LICENSE" "$STAGE/"

case "$(uname -s)" in
  Darwin)
    hdiutil create -volname "NOEMA" -srcfolder "$STAGE" -ov -format UDZO \
      "$OUT/noema-$VERSION-macos-$(uname -m).dmg"
    ;;
  Linux)
    ARCH=$(uname -m)
    tar -C "$STAGE" -czf "$OUT/noema-$VERSION-linux-$ARCH.tar.gz" .
    ;;
esac

echo "artifacts in $OUT"
ls -1 "$OUT"
