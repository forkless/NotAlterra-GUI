#!/usr/bin/env bash
set -euo pipefail

# NotAlterra release build script
# Produces: target/release/notalterra (Linux)
#           target/x86_64-pc-windows-gnu/release/notalterra.exe (Windows)

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
echo "=== Copying release binaries ==="
cp target/release/notalterra "notalterra-linux"
echo "  notalterra-linux"
if [ "$WINDOWS_SKIP" -eq 0 ]; then
    cp target/x86_64-pc-windows-gnu/release/notalterra.exe "notalterra-windows.exe"
    echo "  notalterra-windows.exe"
fi
echo ""
echo "=== Build complete ==="
ls -lh notalterra-linux
if [ "$WINDOWS_SKIP" -eq 0 ]; then
    ls -lh target/x86_64-pc-windows-gnu/release/notalterra.exe
fi
