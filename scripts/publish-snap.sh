#!/bin/bash
# Script to publish Argus to Snap Store
# Must be run on Linux with snapcraft installed

set -e

echo "=== Publishing Argus to Snap Store ==="

# Check if snapcraft is installed
if ! command -v snapcraft &> /dev/null; then
    echo "Error: snapcraft not installed. Install with:"
    echo "  sudo snap install snapcraft --classic"
    exit 1
fi

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "Error: Snap publishing must be done on Linux"
    exit 1
fi

# Build the snap
echo "Building snap package..."
cd snap
snapcraft

# Check if snap was built
if [ ! -f "argus_0.1.0_amd64.snap" ]; then
    echo "Error: Snap build failed"
    exit 1
fi

# Register the name (first time only)
echo "Registering snap name..."
snapcraft register argus || echo "Name already registered"

# Upload to store
echo "Uploading to Snap Store..."
snapcraft upload --release=stable argus_0.1.0_amd64.snap

echo "✅ Snap published successfully!"
echo ""
echo "Install with: sudo snap install argus"
