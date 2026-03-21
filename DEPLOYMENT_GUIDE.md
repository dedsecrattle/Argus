# Argus Deployment Guide

Complete guide to distributing Argus across all platforms.

## Table of Contents
1. [Cargo (crates.io)](#cargo-cratesio)
2. [GitHub Releases](#github-releases)
3. [Docker Hub](#docker-hub)
4. [Homebrew (macOS)](#homebrew-macos)
5. [Chocolatey (Windows)](#chocolatey-windows)
6. [Snap (Linux)](#snap-linux)
7. [Documentation (docs.rs)](#documentation-docsrs)

---

## Cargo (crates.io)

### Status: ✅ Ready

### Installation
```bash
cargo install argus-cli
```

### Publishing
Already configured in `scripts/publish.sh`.

### Verification
```bash
# Check if published
curl https://crates.io/api/v1/crates/argus-cli

# Install from crates.io
cargo install argus-cli
argus --version
```

---

## GitHub Releases

### Status: ✅ Ready

### Automated Release Process
1. Tag the release:
   ```bash
   git tag -a v0.1.0 -m "Argus v0.1.0: Production-ready web crawler"
   git push origin v0.1.0
   ```

2. GitHub Actions will automatically:
   - Run all tests
   - Publish to crates.io
   - Build and push Docker images

3. Manual steps:
   - Go to https://github.com/dedsecrattle/argus/releases/new
   - Auto-generate release notes from tag
   - Attach binaries (optional)

### Release Contents
- Source code (auto)
- Pre-built binaries (add to CI if needed)
- Docker images
- Documentation links

---

## Docker Hub

### Status: ✅ Ready

### Dockerfile
Already created in project root.

### Build and Push
```bash
# Build locally
docker build -t dedsecrattle/argus:latest .
docker build -t dedsecrattle/argus:v0.1.0 .

# Push to Docker Hub
docker login
docker push dedsecrattle/argus:latest
docker push dedsecrattle/argus:v0.1.0
```

### Usage
```bash
# Pull and run
docker pull dedsecrattle/argus:latest
docker run --rm dedsecrattle/argus crawl --seed-url https://example.com

# With Redis
docker run --rm --link redis:redis dedsecrattle/argus \
  crawl --redis-url redis://redis:6379
```

### Docker Compose
```yaml
version: '3.8'
services:
  argus:
    image: dedsecrattle/argus:latest
    command: crawl --seed-url https://example.com
    volumes:
      - ./data:/data
```

---

## Homebrew (macOS)

### Status: 📝 Setup Required

### Option 1: Homebrew Core (Official)
1. Fork homebrew-core:
   ```bash
   git clone https://github.com/Homebrew/homebrew-core.git
   cd homebrew-core
   git checkout -b argus-0.1.0
   ```

2. Add formula:
   ```bash
   cp /path/to/argus/homebrew/argus.rb Formula/argus.rb
   git add Formula/argus.rb
   git commit -m "argus 0.1.0"
   ```

3. Submit PR to Homebrew

### Option 2: Personal Tap (Easier)
1. Create tap repository:
   ```bash
   # On GitHub: Create new repo "homebrew-argus"
   git clone https://github.com/dedsecrattle/homebrew-argus.git
   cd homebrew-argus
   mkdir -p Formula
   cp /path/to/argus/homebrew/argus.rb Formula/
   git add .
   git commit -m "Add argus formula"
   git push origin main
   ```

2. Users install with:
   ```bash
   brew tap dedsecrattle/argus
   brew install argus
   ```

### Setup Script
```bash
./scripts/setup-homebrew.sh 0.1.0 dedsecrattle/argus
```

---

## Chocolatey (Windows)

### Status: 📝 Setup Required

### Create Package
1. Install Chocolatey:
   ```powershell
   Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
   ```

2. Create package template:
   ```bash
   choco new argus
   cd argus
   ```

3. Update `argus.nuspec`:
   ```xml
   <?xml version="1.0" encoding="utf-8"?>
   <package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
     <metadata>
       <id>argus</id>
       <version>0.1.0</version>
       <title>Argus Web Crawler</title>
       <authors>Your Name</authors>
       <description>Production-ready web crawler</description>
       <licenseUrl>https://github.com/dedsecrattle/argus/blob/main/LICENSE</licenseUrl>
       <projectUrl>https://github.com/dedsecrattle/argus</projectUrl>
       <tags>crawler web rust</tags>
     </metadata>
   </package>
   ```

4. Update `tools/chocolateyinstall.ps1`:
   ```powershell
   $ErrorActionPreference = 'Stop'
   
   $packageArgs = @{
     packageName     = 'argus'
     fileType        = 'exe'
     url             = 'https://github.com/dedsecrattle/argus/releases/download/v0.1.0/argus-x86_64-pc-windows-msvc.exe'
     checksum        = '{{checksum}}'
     checksumType    = 'sha256'
   }
   
   Install-ChocolateyPackage @packageArgs
   ```

5. Build and push:
   ```bash
   choco pack
   choco push argus.0.1.0.nupkg --source https://push.chocolatey.org/
   ```

---

## Snap (Linux)

### Status: 📝 Setup Required

### Create snapcraft.yaml
```yaml
# snap/snapcraft.yaml
name: argus
version: git
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
    build-packages:
      - pkg-config
      - libssl-dev

apps:
  argus:
    command: bin/argus
    plugs:
      - home
      - network
```

### Build and Publish
```bash
# Install snapcraft
sudo snap install snapcraft --classic

# Build
snapcraft

# Register on Snap Store
snapcraft register argus

# Push
snapcraft push argus_*.snap --release stable
```

---

## Documentation (docs.rs)

### Status: ✅ Automatic

### How it Works
- Automatically built when publishing to Cargo
- Rust doc comments become documentation
- Features are documented separately

### Best Practices
```rust
//! # Argus Web Crawler
//!
//! A production-ready web crawler written in Rust.
//!
//! ## Quick Start
//!
//! ```rust
//! use argus_cli::run_crawl;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     run_crawl(&["crawl", "--seed-url", "https://example.com"]).await
//! }
//! ```

/// Bloom filter for URL deduplication
///
/// Provides probabilistic deduplication with minimal memory usage.
/// 1 billion URLs require only ~1.2GB of memory.
///
/// # Examples
///
/// ```rust
/// use argus_dedupe::BloomDeduplicator;
///
/// let bloom = BloomDeduplicator::new(1_000_000, 0.01);
/// assert!(!bloom.might_contain("https://example.com"));
/// ```
pub struct BloomDeduplicator {
    // ...
}
```

### Verification
After publishing, check:
- https://docs.rs/argus-cli
- https://docs.rs/argus-dedupe
- etc.

---

## Complete Release Checklist

### Pre-Release
- [ ] Update all version numbers
- [ ] Update CHANGELOG.md
- [ ] Run full test suite
- [ ] Verify documentation builds
- [ ] Test installation from source

### Release Day
1. **Create Git Tag**
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

2. **Publish to Cargo**
   ```bash
   ./scripts/publish.sh
   ```

3. **Create GitHub Release**
   - Go to releases page
   - Draft new release from tag
   - Add release notes
   - Attach binaries

4. **Build Docker Images**
   ```bash
   docker build -t dedsecrattle/argus:v0.1.0 .
   docker push dedsecrattle/argus:v0.1.0
   ```

5. **Setup Homebrew**
   ```bash
   ./scripts/setup-homebrew.sh v0.1.0 dedsecrattle/argus
   ```

6. **Submit to Other Platforms**
   - Chocolatey (Windows)
   - Snap (Linux)

### Post-Release
- [ ] Update website/documentation
- [ ] Announce on social media
- [ ] Post to relevant communities
- [ ] Monitor for issues

---

## Installation Summary

Once all platforms are set up:

### macOS
```bash
# Option 1: Cargo
cargo install argus-cli

# Option 2: Homebrew
brew install argus

# Option 3: Docker
docker run dedsecrattle/argus crawl --seed-url https://example.com
```

### Linux
```bash
# Option 1: Cargo
cargo install argus-cli

# Option 2: Snap
snap install argus

# Option 3: Docker
docker run dedsecrattle/argus crawl --seed-url https://example.com
```

### Windows
```bash
# Option 1: Cargo
cargo install argus-cli

# Option 2: Chocolatey
choco install argus

# Option 3: Docker
docker run dedsecrattle/argus crawl --seed-url https://example.com
```

---

## URLs After Release

- **Cargo**: https://crates.io/crates/argus-cli
- **GitHub**: https://github.com/dedsecrattle/argus
- **Docker Hub**: https://hub.docker.com/r/dedsecrattle/argus
- **Docs**: https://docs.rs/argus-cli
- **Homebrew**: `brew install argus`
- **Chocolatey**: `choco install argus`
- **Snap**: `snap install argus`
