# Argus Distribution Guide

This document covers all the ways Argus can be distributed and installed.

## 📦 Available Distribution Channels

### 1. Cargo (crates.io) - Primary
```bash
cargo install argus-crawler
```
- **Status**: ✅ Available
- **Platform**: All platforms with Rust
- **Size**: ~10MB (binary only)

### 2. Docker Hub
```bash
docker pull dedsecrattle/argus:latest
docker run dedsecrattle/argus:latest
```
- **Status**: ✅ Available
- **Platform**: Linux, macOS, Windows (Docker)
- **Size**: ~1.2GB (includes build tools)

### 3. Homebrew (macOS)
```bash
brew tap dedsecrattle/argus
brew install argus-crawler
```
- **Status**: ✅ Available
- **Platform**: macOS, Linux
- **Size**: ~10MB (binary only)

### 4. Snap (Linux)
```bash
sudo snap install argus
```
- **Status**: 📦 Ready to publish
- **Platform**: Linux (Ubuntu, Debian, etc.)
- **Size**: ~200MB (includes dependencies)

### 5. Chocolatey (Windows)
```powershell
choco install argus
```
- **Status**: 📦 Ready to publish
- **Platform**: Windows
- **Size**: ~50MB (includes Rust)

## 🚀 Publishing Instructions

### Snap Store
1. Install snapcraft: `sudo apt install snapcraft`
2. Build: `snapcraft`
3. Register: `snapcraft register argus`
4. Upload: `snapcraft upload --release=stable argus_0.1.0_amd64.snap`

### Chocolatey
1. Create account at https://chocolatey.org/
2. Request maintainer rights for 'argus' package
3. Build: `choco pack chocolatey/argus/argus.nuspec`
4. Push: `choco push argus.0.1.0.nupkg`

### Homebrew Core (Optional)
1. Fork https://github.com/Homebrew/homebrew-core
2. Add Formula/argus_crawler.rb
3. Submit PR
4. Or continue using personal tap

## 📊 Distribution Matrix

| Platform    | Cargo | Docker | Homebrew | Snap | Chocolatey |
|-------------|-------|--------|----------|------|------------|
| Linux       | ✅    | ✅     | ✅       | ✅   | ❌         |
| macOS       | ✅    | ✅     | ✅       | ❌   | ❌         |
| Windows     | ✅    | ✅     | ❌       | ❌   | ✅         |
| FreeBSD     | ✅    | ✅     | ❌       | ❌   | ❌         |

## 🔧 Maintenance

### Version Updates
1. Update version in all package files
2. Tag release in Git
3. Update Cargo.toml files
4. Publish crates to crates.io
5. Build and push Docker image
6. Update Homebrew formula
7. Build and publish Snap/Chocolatey

### Security Scanning
- Docker images are scanned by Docker Hub
- Snap store provides automatic scanning
- Chocolatey packages are scanned on submission

## 📈 Analytics Tracking

To track downloads:
- crates.io: https://crates.io/crates/argus-crawler/downloads
- Docker Hub: https://hub.docker.com/r/dedsecrattle/argus
- Snap Store: https://snapcraft.io/argus/metrics
- Chocolatey: https://chocolatey.org/packages/argus

## 🆘 Support

For installation issues:
- Check the troubleshooting guide
- Open an issue on GitHub
- Join our Discord/Matrix community
