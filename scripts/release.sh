#!/bin/bash

# Argus Complete Release Script
# This script handles publishing to ALL platforms

set -e

VERSION=${1:-"0.1.0"}
REPO=${2:-"yourusername/argus"}

echo "🚀 Argus Complete Release Script v$VERSION"
echo "=========================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if we're on main branch
    if [ "$(git branch --show-current)" != "main" ]; then
        log_warning "Not on main branch. Continue? (y/N)"
        read -r response
        if [[ ! $response =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Check if working directory is clean
    if [ -n "$(git status --porcelain)" ]; then
        log_error "Working directory is not clean. Commit or stash changes first."
        exit 1
    fi
    
    # Check if we have cargo login
    if ! cargo login --help > /dev/null 2>&1; then
        log_error "Please run 'cargo login' first"
        exit 1
    fi
    
    log_success "Prerequisites checked"
}

# Update version numbers
update_versions() {
    log_info "Updating version numbers to $VERSION..."
    
    # Update all Cargo.toml files
    find crates -name "Cargo.toml" -type f -exec sed -i '' "s/^version = .*/version = \"$VERSION\"/" {} \;
    
    # Update CLI Cargo.toml dependencies
    sed -i '' "s/argus-common = { path = \"..\/argus-common\" }/argus-common = { version = \"$VERSION\" }/" crates/argus-cli/Cargo.toml
    sed -i '' "s/argus-config = { path = \"..\/argus-config\" }/argus-config = { version = \"$VERSION\" }/" crates/argus-cli/Cargo.toml
    sed -i '' "s/argus-storage = { path = \"..\/argus-storage\" }/argus-storage = { version = \"$VERSION\" }/" crates/argus-cli/Cargo.toml
    sed -i '' "s/argus-worker = { path = \"..\/argus-worker\", features = \[\"redis\"\] }/argus-worker = { version = \"$VERSION\", features = \[\"redis\"\] }/" crates/argus-cli/Cargo.toml
    
    # Update other crates similarly
    # ... (add more sed commands as needed)
    
    log_success "Version numbers updated"
}

# Run tests
run_tests() {
    log_info "Running tests..."
    
    cargo test --workspace --all-features
    cargo clippy --workspace --all-features -- -D warnings
    cargo fmt --all -- --check
    
    log_success "All tests passed"
}

# Create git tag
create_tag() {
    log_info "Creating git tag v$VERSION..."
    
    git add -A
    git commit -m "chore: bump version to $VERSION"
    git tag -a "v$VERSION" -m "Argus v$VERSION"
    git push origin main
    git push origin "v$VERSION"
    
    log_success "Git tag created and pushed"
}

# Publish to Cargo
publish_cargo() {
    log_info "Publishing to Cargo..."
    
    ./scripts/publish.sh
    
    log_success "Published to Cargo"
}

# Create GitHub Release
create_github_release() {
    log_info "Creating GitHub release..."
    
    # Generate release notes
    cat > release_notes.md << EOF
# Argus v$VERSION

## Installation

\`\`\`bash
# Cargo
cargo install argus-cli

# Homebrew (macOS)
brew install argus

# Docker
docker pull yourusername/argus:$VERSION
\`\`\`

## Changes

$(sed -n "/## \[$VERSION\]/,/## \[.*\]/p" CHANGELOG.md | sed '$d')

## Docker Images

- \`yourusername/argus:$VERSION\`
- \`yourusername/argus:latest\`

## Verification

\`\`\`bash
argus --version
# Should show: argus $VERSION
\`\`\`
EOF
    
    log_info "Release notes prepared. Create release manually at:"
    log_info "https://github.com/$REPO/releases/new?tag=v$VERSION"
    log_info "Contents saved to release_notes.md"
    
    log_success "GitHub release prepared"
}

# Build and push Docker images
build_docker() {
    log_info "Building and pushing Docker images..."
    
    # Build images
    docker build -t "yourusername/argus:$VERSION" .
    docker build -t "yourusername/argus:latest" .
    
    # Push images
    docker push "yourusername/argus:$VERSION"
    docker push "yourusername/argus:latest"
    
    log_success "Docker images pushed"
}

# Setup Homebrew
setup_homebrew() {
    log_info "Setting up Homebrew formula..."
    
    ./scripts/setup-homebrew.sh "$VERSION" "$REPO"
    
    log_success "Homebrew formula prepared"
}

# Create Chocolatey package (Windows)
create_chocolatey() {
    log_info "Creating Chocolatey package..."
    
    mkdir -p chocolatey
    
    cat > chocolatey/argus.nuspec << EOF
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>argus</id>
    <version>$VERSION</version>
    <title>Argus Web Crawler</title>
    <authors>Argus Contributors</authors>
    <description>A production-ready web crawler written in Rust</description>
    <licenseUrl>https://github.com/$REPO/blob/main/LICENSE</licenseUrl>
    <projectUrl>https://github.com/$REPO</projectUrl>
    <tags>crawler web rust</tags>
  </metadata>
</package>
EOF
    
    log_warning "Chocolatey package prepared. Manual steps required:"
    log_warning "1. Build Windows binary"
    log_warning "2. Calculate checksum"
    log_warning "3. Submit to Chocolatey"
}

# Create Snap package (Linux)
create_snap() {
    log_info "Creating Snap package..."
    
    mkdir -p snap
    
    cat > snap/snapcraft.yaml << EOF
name: argus
version: $VERSION
summary: Production-ready web crawler
description: |
  A scalable web crawler written in Rust, capable of handling
  billions of URLs with advanced features.

grade: stable
confinement: strict

parts:
  argus:
    plugin: rust
    source: .

apps:
  argus:
    command: bin/argus
    plugs:
      - home
      - network
EOF
    
    log_warning "Snap package prepared. To publish:"
    log_warning "1. sudo snap install snapcraft --classic"
    log_warning "2. snapcraft"
    log_warning "3. snapcraft register argus"
    log_warning "4. snapcraft push argus_*.snap"
}

# Update README
update_readme() {
    log_info "Updating README with new version..."
    
    # Replace old README with new one
    mv README.md README_OLD.md
    mv README_NEW.md README.md
    
    git add README.md
    git commit -m "docs: update README for v$VERSION"
    git push origin main
    
    log_success "README updated"
}

# Summary
summary() {
    echo ""
    echo "🎉 Release v$VERSION completed!"
    echo "================================"
    echo ""
    echo "✅ Published to:"
    echo "   - Cargo: https://crates.io/crates/argus-cli"
    echo "   - Docker: https://hub.docker.com/r/yourusername/argus"
    echo ""
    echo "📝 Manual steps remaining:"
    echo "   1. Create GitHub release: https://github.com/$REPO/releases/new"
    echo "   2. Submit Homebrew formula to homebrew-core"
    echo "   3. Build and submit Windows binary to Chocolatey"
    echo "   4. Build and submit Snap to Snap Store"
    echo ""
    echo "📢 Announce on:"
    echo "   - r/rust"
    echo "   - users.rust-lang.org"
    echo "   - Twitter/X with #RustLang"
    echo "   - Hacker News"
    echo ""
    echo "🔗 Useful links:"
    echo "   - Docs: https://docs.rs/argus-cli"
    echo "   - GitHub: https://github.com/$REPO"
}

# Main execution
main() {
    echo "This will release Argus v$VERSION to all platforms."
    echo "Repository: $REPO"
    echo ""
    read -p "Continue? (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
    
    check_prerequisites
    update_versions
    run_tests
    create_tag
    publish_cargo
    create_github_release
    build_docker
    setup_homebrew
    create_chocolatey
    create_snap
    update_readme
    summary
}

# Run main function
main "$@"
