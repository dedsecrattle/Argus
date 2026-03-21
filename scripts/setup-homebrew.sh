#!/bin/bash

# Script to setup Homebrew formula for Argus

set -e

VERSION=${1:-"0.1.0"}
REPO=${2:-"dedsecrattle/argus"}

echo "🍺 Setting up Homebrew formula for Argus v$VERSION"
echo "=============================================="

# Create tarball and get SHA256
echo "📦 Creating release tarball..."
git archive --format=tar.gz --prefix=argus-$VERSION/ v$VERSION > argus-$VERSION.tar.gz

# Calculate SHA256
SHA256=$(shasum -a 256 argus-$VERSION.tar.gz | cut -d' ' -f1)
echo "🔑 SHA256: $SHA256"

# Update formula with actual values
sed -e "s/{{ version }}/$VERSION/g" \
    -e "s/{{ tarball_sha256 }}/$SHA256/g" \
    -e "s/dedsecrattle/$(echo $REPO | cut -d'/' -f1)/g" \
    homebrew/argus.rb > homebrew/argus.rb.tmp

mv homebrew/argus.rb.tmp homebrew/argus.rb

echo "✅ Updated formula:"
echo ""
cat homebrew/argus.rb
echo ""

# Clean up
rm argus-$VERSION.tar.gz

echo "📝 Next steps:"
echo "   1. Fork https://github.com/Homebrew/homebrew-core"
echo "   2. Create a new branch: git checkout -b argus-$VERSION"
echo "   3. Copy homebrew/argus.rb to Formula/argus.rb"
echo "   4. Commit and submit a PR"
echo ""
echo "   Or for a tap (easier):"
echo "   1. Create a new repo: https://github.com/dedsecrattle/homebrew-argus"
echo "   2. Add tap: brew tap dedsecrattle/argus"
echo "   3. Install: brew install argus"
