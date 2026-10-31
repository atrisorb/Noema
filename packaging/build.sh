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
    APPDIR="$OUT/NOEMA.AppDir"
    rm -rf "$APPDIR"
    mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" "$APPDIR/usr/share/noema" "$APPDIR/usr/share/icons/hicolor/256x256/apps"
    cp "$STAGE/noema" "$STAGE/noemad" "$STAGE/noema-app" "$APPDIR/usr/bin/"
    cp -r "$STAGE/ui" "$STAGE/assets" "$APPDIR/usr/share/noema/"
    cat > "$APPDIR/noema.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=NOEMA
Comment=The anatomical mind
Exec=noema-app
Icon=noema
Categories=Science;
EOF
    cp "$APPDIR/noema.desktop" "$APPDIR/usr/share/applications/"
    cat > "$APPDIR/AppRun" <<'EOF'
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH:-}"
exec "${HERE}/usr/bin/noema-app" "$@"
EOF
    chmod +x "$APPDIR/AppRun"
    if [ ! -f "$ROOT/src-tauri/icons/128x128@2x.png" ]; then
      (cd "$ROOT" && node tools/design/build-icons.mjs) || true
    fi
    ICON_SRC="$ROOT/src-tauri/icons/128x128@2x.png"
    if [ ! -f "$ICON_SRC" ]; then
      ICON_SRC="$ROOT/src-tauri/icons/128x128.png"
    fi
    cp "$ICON_SRC" "$APPDIR/usr/share/icons/hicolor/256x256/apps/noema.png" 2>/dev/null || true
    cp "$ICON_SRC" "$APPDIR/noema.png" 2>/dev/null || true
    if command -v appimagetool >/dev/null; then
      appimagetool "$APPDIR" "$OUT/noema-$VERSION-linux-$ARCH.AppImage"
    else
      echo "appimagetool not found: AppImage skipped"
    fi
    rm -rf "$OUT/deb"
    mkdir -p "$OUT/deb/DEBIAN" "$OUT/deb/usr/bin" "$OUT/deb/usr/share/noema"
    cp "$STAGE/noema" "$STAGE/noemad" "$STAGE/noema-app" "$OUT/deb/usr/bin/"
    cp -r "$STAGE/ui" "$STAGE/assets" "$OUT/deb/usr/share/noema/"
    cat > "$OUT/deb/DEBIAN/control" <<EOF
Package: noema
Version: $VERSION
Section: science
Priority: optional
Architecture: $(dpkg --print-architecture)
Maintainer: Adriano Sorbello
Description: NOEMA desktop platform and parity CLI
EOF
    dpkg-deb --build "$OUT/deb" "$OUT/noema-$VERSION-linux-$ARCH.deb"
    if command -v rpmbuild >/dev/null; then
      TOP="$OUT/rpmbuild"
      rm -rf "$TOP"
      mkdir -p "$TOP/SPECS"
      cat > "$TOP/SPECS/noema.spec" <<EOF
Name: noema
Version: $VERSION
Release: 1
Summary: NOEMA desktop platform and parity CLI
License: MIT
BuildArch: $ARCH
%description
NOEMA desktop platform and parity CLI.
%install
mkdir -p %{buildroot}/usr/bin
cp "$STAGE/noema" "$STAGE/noemad" "$STAGE/noema-app" %{buildroot}/usr/bin/
%files
/usr/bin/noema
/usr/bin/noemad
/usr/bin/noema-app
EOF
      rpmbuild --define "_topdir $TOP" -bb "$TOP/SPECS/noema.spec"
      cp "$TOP/RPMS/$ARCH"/*.rpm "$OUT/" 2>/dev/null || true
    else
      echo "rpmbuild not found: rpm skipped"
    fi
    ;;
esac

echo "artifacts in $OUT"
ls -1 "$OUT"
