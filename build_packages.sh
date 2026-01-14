#!/bin/bash
set -e

echo "Starting build process..."

# 1. Build Release Binary
echo "Building release binary..."
cargo build --release

# 2. Check and Install cargo-deb
if ! cargo deb --version &> /dev/null; then
    echo "cargo-deb not found. Installing..."
    cargo install cargo-deb
fi

# 3. Build .deb
echo "Building .deb package..."
cargo deb

# 4. Check and Install cargo-generate-rpm
if ! cargo generate-rpm --version &> /dev/null; then
    echo "cargo-generate-rpm not found. Installing..."
    cargo install cargo-generate-rpm
fi

# 5. Build .rpm
echo "Building .rpm package..."
cargo generate-rpm

echo "Build complete!"
echo "DEB package: target/debian/*.deb"
echo "RPM package: target/generate-rpm/*.rpm"
