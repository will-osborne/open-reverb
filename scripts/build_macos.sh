#!/bin/bash
set -e

echo "Building Open Reverb for macOS..."

# Build common library
echo "Building common library..."
cd open-reverb-common
cargo build --release
cd ..

# Build server
echo "Building server..."
cd open-reverb-server
cargo build --release
cd ..

# Build client
echo "Building client..."
cd open-reverb-client
cargo build --release
cd ..

# Create dist directory
mkdir -p dist/macos

# Copy binaries to dist
cp target/release/open-reverb-server dist/macos/
cp target/release/open-reverb-client dist/macos/

echo "Build completed successfully!"
echo "Binaries are available in the dist/macos directory"