#!/bin/bash

# Argus Publishing Script
# This script helps publish all crates in the correct order

set -e

echo "🚀 Publishing Argus to crates.io"
echo "================================"

# Check if we're logged in to Cargo
if ! cargo login --help > /dev/null 2>&1; then
    echo "❌ Please run 'cargo login' first"
    exit 1
fi

# Check if we have a token
if [ -z "$CARGO_REGISTRY_TOKEN" ]; then
    echo "⚠️  CARGO_REGISTRY_TOKEN not set, using stored token"
fi

# Function to publish a crate
publish_crate() {
    local crate_name=$1
    echo "📦 Publishing $crate_name..."
    
    # Dry run first
    if cargo publish --dry-run -p $crate_name; then
        echo "✅ Dry run passed for $crate_name"
        
        # Actually publish
        cargo publish -p $crate_name
        echo "✅ Published $crate_name"
        
        # Wait for the crate to be available
        echo "⏳ Waiting 30 seconds for $crate_name to be available..."
        sleep 30
    else
        echo "❌ Dry run failed for $crate_name"
        exit 1
    fi
}

# Check if we're on a git tag
if [ -z "$(git tag --points-at HEAD)" ]; then
    echo "⚠️  Warning: Not on a git tag. Consider creating a tag first:"
    echo "   git tag -a v0.1.0 -m 'Argus v0.1.0'"
    echo "   git push origin v0.1.0"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Publish in dependency order
echo ""
echo "📋 Publishing crates in dependency order..."
echo ""

publish_crate "argus-common"
publish_crate "argus-config"
publish_crate "argus-robots"
publish_crate "argus-dedupe"
publish_crate "argus-storage"
publish_crate "argus-fetcher"
publish_crate "argus-parser"
publish_crate "argus-frontier"
publish_crate "argus-worker"
publish_crate "argus-cli"

echo ""
echo "🎉 All crates published successfully!"
echo ""
echo "📝 Next steps:"
echo "   1. Create a GitHub release: https://github.com/yourusername/argus/releases/new"
echo "   2. Build and push Docker image: docker build -t yourusername/argus:latest . && docker push yourusername/argus:latest"
echo "   3. Announce the release in the Rust community"
echo ""
echo "🔗 Links:"
echo "   - Cargo: https://crates.io/crates/argus-cli"
echo "   - Docs: https://docs.rs/argus-cli"
echo "   - GitHub: https://github.com/yourusername/argus"
