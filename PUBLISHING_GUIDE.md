# Argus Publishing Guide

This guide walks you through publishing Argus to all distribution channels.

## 📋 Prerequisites

### For Snap (Linux):
- Ubuntu 20.04+ or other Linux distribution
- snapd installed
- snapcraft installed: `sudo snap install snapcraft --classic`

### For Chocolatey (Windows):
- Windows 10/11
- Chocolatey installed: `Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))`

## 🚀 Publishing Steps

### 1. Snap Store (Linux)

```bash
# Clone the repository on a Linux machine
git clone https://github.com/dedsecrattle/argus.git
cd argus

# Build the snap
cd snap
snapcraft

# Register the name (first time only)
snapcraft register argus

# Upload to the store
snapcraft upload --release=stable argus_0.1.0_amd64.snap

# Verify
snap install argus
argus --help
```

### 2. Chocolatey (Windows)

```powershell
# Clone repository
git clone https://github.com/dedsecrattle/argus.git
cd argus

# Build the package
cd chocolatey
choco pack

# Test locally
choco install argus -s . -y
argus --help
choco uninstall argus -y

# Submit to Chocolatey
# 1. Create account at https://chocolatey.org/
# 2. Request maintainer rights for 'argus' package
# 3. Push package
choco push argus.0.1.0.nupkg

# Or submit for review if not a maintainer
```

### 3. Homebrew Core (Optional)

```bash
# Fork homebrew-core
git clone https://github.com/Homebrew/homebrew-core.git
cd homebrew-core

# Create formula
cat > Formula/argus_crawler.rb << 'EOF'
class ArgusCrawler < Formula
  desc "Production-ready web crawler capable of handling billions of URLs"
  homepage "https://github.com/dedsecrattle/argus"
  url "https://github.com/dedsecrattle/argus/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "c57ec979a41d741abda08119a46c9a72545aaf0e7eca72ffe9e295b6ba785f74"
  license "MIT"

  depends_on "rust" => :build
  depends_on "redis" => :optional

  def install
    system "cargo", "install", "--root", prefix, "argus-crawler"
  end

  test do
    system "#{bin}/argus", "--help"
  end
end
EOF

# Test formula
brew install --build-from-source argus-crawler

# Submit PR
git add Formula/argus_crawler.rb
git commit -m "argus-crawler 0.1.0"
git push origin your-branch
# Open PR on GitHub
```

## 📊 Publishing Checklist

- [ ] All crates published to crates.io ✅
- [ ] Docker image pushed to Docker Hub ✅
- [ ] Homebrew tap created and working ✅
- [ ] Snap package built and uploaded
- [ ] Chocolatey package created and submitted
- [ ] GitHub release created
- [ ] Documentation updated

## 🔄 Version Updates

When releasing a new version:

1. Update all version numbers:
   - Cargo.toml files
   - Docker tags
   - Homebrew formula
   - Snap version
   - Chocolatey version

2. Tag the release:
   ```bash
   git tag v0.1.1
   git push origin v0.1.1
   ```

3. Update each distribution channel

## 📞 Support

- Snap Store: https://snapcraft.io/argus
- Chocolatey: https://chocolatey.org/packages/argus
- Homebrew: https://github.com/dedsecrattle/homebrew-argus
- Issues: https://github.com/dedsecrattle/argus/issues

## 📈 Analytics

Track downloads from:
- crates.io: https://crates.io/crates/argus-crawler/downloads
- Docker Hub: https://hub.docker.com/r/dedsecrattle/argus
- Snap Store: https://snapcraft.io/argus/metrics
- Chocolatey: https://chocolatey.org/packages/argus
