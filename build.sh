#!/usr/bin/env bash
set -euo pipefail

# NotAlterra build script
# Produces: release archives in builds/
# Usage: ./build.sh          — optimized release (LTO + strip)
#        ./build.sh fast     — no LTO, faster compile

# Derive version from latest git tag, fall back to Cargo.toml
if command -v git &>/dev/null && git rev-parse --git-dir >/dev/null 2>&1; then
    TAG_VERSION=$(git describe --tags --abbrev=0 2>/dev/null | sed 's/^v//')
fi
CARGOTOML_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [ -n "${TAG_VERSION:-}" ] && [ "$TAG_VERSION" != "$CARGOTOML_VERSION" ]; then
    sed -i "s/^version = \".*\"/version = \"$TAG_VERSION\"/" Cargo.toml
    echo "  Cargo.toml version updated to $TAG_VERSION"
fi
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "=== NotAlterra v${VERSION} ==="

# Ensure Windows target is installed
if ! rustup target list --installed | grep -q x86_64-pc-windows-gnu; then
    echo "Installing Windows target..."
    rustup target add x86_64-pc-windows-gnu
fi

# Check for mingw linker
if ! command -v x86_64-w64-mingw32-gcc &>/dev/null; then
    echo "WARNING: mingw-w64 not found — Windows build will be skipped."
    echo "Install it: sudo apt install mingw-w64"
    WINDOWS_SKIP=1
else
    WINDOWS_SKIP=0
fi

echo ""
echo "Building Linux release..."
cargo build --release

if [ "$WINDOWS_SKIP" -eq 0 ]; then
    echo ""
    echo "Building Windows release..."
    cargo build --release --target x86_64-pc-windows-gnu
fi

echo ""
echo "=== Packaging release archives ==="
mkdir -p builds
LINUX_ARCHIVE="builds/notalterra-v${VERSION}-linux-amd64.tar.gz"
cp target/release/notalterra notalterra
tar -czf "$LINUX_ARCHIVE" notalterra
rm -f notalterra
echo "  $LINUX_ARCHIVE ($(du -h "$LINUX_ARCHIVE" | cut -f1))"

if [ "$WINDOWS_SKIP" -eq 0 ]; then
    WIN_ARCHIVE="builds/notalterra-v${VERSION}-windows-x64.zip"
    rm -f "$WIN_ARCHIVE"
    cp target/x86_64-pc-windows-gnu/release/notalterra.exe NotAlterra.exe
    zip -q "$WIN_ARCHIVE" NotAlterra.exe
    rm -f NotAlterra.exe
    echo "  $WIN_ARCHIVE ($(du -h "$WIN_ARCHIVE" | cut -f1))"
fi

echo ""
echo "=== Done ==="

# Remind user to sign and push to trigger CI release
echo ""
echo "--- Next: sign & push for CI release ---"
echo "  git add Cargo.toml && git commit --no-gpg-sign -m \"Bump to v${VERSION}\""
echo "  git tag -s \"v${VERSION}\" -m \"v${VERSION}\""
echo "  git push origin master && git push origin \"v${VERSION}\""
