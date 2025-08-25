#!/bin/bash
set -e

echo "Cross-compiling Open Reverb for Windows from macOS/Linux..."

# Check if cross is installed
if ! command -v cross &> /dev/null
then
    echo "Cross compiler not found. Installing it..."
    cargo install cross
fi

# Build common library
echo "Building common library..."
cd open-reverb-common
cross build --target x86_64-pc-windows-gnu --release
cd ..

# Build server
echo "Building server..."
cd open-reverb-server
cross build --target x86_64-pc-windows-gnu --release
cd ..

# Build client
echo "Building client..."
cd open-reverb-client
cross build --target x86_64-pc-windows-gnu --release
cd ..

# Create dist directory
mkdir -p dist/windows

# Copy binaries to dist
cp target/x86_64-pc-windows-gnu/release/open-reverb-server.exe dist/windows/
cp target/x86_64-pc-windows-gnu/release/open-reverb-client.exe dist/windows/

echo "Cross-compilation completed successfully!"
echo "Windows binaries are available in the dist/windows directory"