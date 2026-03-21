# Publishing Argus Web Crawler

## Overview
This guide covers how to publish Argus to Cargo, GitHub, and other platforms to make it available as an open-source project.

## 1. Prepare for Publishing

### Version Management
First, ensure all crates have proper versions:

```bash
# Update versions in all Cargo.toml files
# Use semantic versioning (e.g., 0.1.0 for initial release)

# Check current versions
grep -r "version = " crates/*/Cargo.toml
```

### License
Add a proper license file:

```bash
# Create LICENSE file (MIT or Apache-2.0 recommended)
cat > LICENSE << 'EOF'
MIT License

Copyright (c) 2024 Argus Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
```

### README Enhancement
Update your README with publishing information:

```bash
cat >> README.md << 'EOF'

## Installation

### From crates.io
```bash
cargo install argus-cli
```

### From source
```bash
git clone https://github.com/yourusername/argus.git
cd argus
cargo install --path .
```

## Crates

- [argus-cli](https://crates.io/crates/argus-cli) - Command-line interface
- [argus-common](https://crates.io/crates/argus-common) - Common types and utilities
- [argus-fetcher](https://crates.io/crates/argus-fetcher) - HTTP fetching with retry logic
- [argus-parser](https://crates.io/crates/argus-parser) - HTML and sitemap parsing
- [argus-dedupe](https://crates.io/crates/argus-dedupe) - Content deduplication with Simhash
- [argus-storage](https://crates.io/crates/argus-storage) - Storage backends
- [argus-frontier](https://crates.io/crates/argus-frontier) - URL frontier implementations
- [argus-robots](https://crates.io/crates/argus-robots) - Robots.txt parsing
- [argus-worker](https://crates.io/crates/argus-worker) - Worker implementation
- [argus-config](https://crates.io/crates/argus-config) - Configuration management

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
EOF
```

## 2. Publishing to Cargo

### Setup Cargo Account
```bash
# Login to Cargo
cargo login

# Create account at https://crates.io if you don't have one
```

### Check Package Names
Verify crate names are available:

```bash
# Check if names are taken
curl https://crates.io/api/v1/crates/argus-cli
curl https://crates.io/api/v1/crates/argus-common
# ... repeat for all crates
```

### Publish Order (Dependencies First)
Publish in dependency order:

```bash
# 1. argus-common (no dependencies)
cargo publish -p argus-common

# 2. argus-config
cargo publish -p argus-config

# 3. argus-robots
cargo publish -p argus-robots

# 4. argus-dedupe
cargo publish -p argus-dedupe

# 5. argus-storage
cargo publish -p argus-storage

# 6. argus-fetcher
cargo publish -p argus-fetcher

# 7. argus-parser
cargo publish -p argus-parser

# 8. argus-frontier
cargo publish -p argus-frontier

# 9. argus-worker
cargo publish -p argus-worker

# 10. argus-cli (last, depends on all others)
cargo publish -p argus-cli
```

### Dry Run First
Test before publishing:

```bash
# Check if everything will publish correctly
cargo publish --dry-run -p argus-common
cargo publish --dry-run -p argus-config
# ... repeat for all crates
```

## 3. GitHub Repository

### Create GitHub Repository
```bash
# Initialize git if not already done
git init
git add .
git commit -m "Initial commit: Production-ready web crawler with 1B URL support"

# Add remote (replace with your repo)
git remote add origin https://github.com/yourusername/argus.git

# Push to GitHub
git push -u origin main
```

### GitHub Release
Create a comprehensive release:

```bash
# Create a tag
git tag -a v0.1.0 -m "Argus v0.1.0: Production-ready web crawler"
git push origin v0.1.0
```

### GitHub Actions CI/CD
Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ubuntu-latest
    strategy:
      matrix:
        rust:
          - stable
          - beta
          - nightly

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: ${{ matrix.rust }}
        override: true

    - name: Run tests
      run: cargo test --workspace --all-features

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: clippy
    - name: Run clippy
      run: cargo clippy --workspace --all-features -- -D warnings

  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt
    - name: Check formatting
      run: cargo fmt --all -- --check
```

## 4. Docker Hub

### Create Dockerfile
```dockerfile
# Multi-stage build for smaller image
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release -p argus-cli

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/argus /usr/local/bin/argus
ENTRYPOINT ["argus"]
```

### Build and Push to Docker Hub
```bash
# Build Docker image
docker build -t yourusername/argus:latest .
docker build -t yourusername/argus:v0.1.0 .

# Login to Docker Hub
docker login

# Push images
docker push yourusername/argus:latest
docker push yourusername/argus:v0.1.0
```

## 5. Documentation

### Docs.rs
Documentation is automatically built when publishing to Cargo. Ensure good docs:

```rust
//! # Argus Web Crawler
//!
//! A production-ready web crawler written in Rust, capable of handling
//! billions of URLs with advanced features like content deduplication,
//! distributed crawling, and JavaScript rendering.
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

/// A Bloom filter for URL deduplication
///
/// Provides probabilistic deduplication with minimal memory usage.
/// 1 billion URLs require only ~1.2GB of memory.
pub struct BloomDeduplicator {
    // ...
}
```

### Website Documentation
Create a documentation site:

```bash
# Using mdBook
cargo install mdbook
mdbook build docs/book

# Or using GitHub Pages
mkdir -p docs
# Add comprehensive documentation here
```

## 6. Community Building

### Contributing Guidelines
Create `CONTRIBUTING.md`:

```markdown
# Contributing to Argus

We welcome contributions! Here's how to get started:

## Development Setup

```bash
git clone https://github.com/yourusername/argus.git
cd argus
cargo build
cargo test
```

## How to Contribute

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write tests for new features
- Update documentation

## Issues

- Bug reports: Use GitHub issues
- Feature requests: Use GitHub discussions
- Security issues: Email security@yourproject.com
```

### Code of Conduct
Create `CODE_OF_CONDUCT.md`:

```markdown
# Code of Conduct

## Our Pledge

We are committed to providing a welcoming and inclusive environment
for all contributors.

## Our Standards

- Use welcoming and inclusive language
- Respect different viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
```

## 7. Promotion

### Rust Community
- Announce on [users.rust-lang.org](https://users.rust-lang.org/)
- Post on r/rust subreddit
- Share on Twitter/X with #RustLang hashtag

### Tech Communities
- Hacker News
- Lobste.rs
- Dev.to
- Medium

### Presentations
- RustConf
- Local meetups
- Conference talks

## 8. Maintenance

### Release Process
```bash
# For each release:
1. Update version numbers
2. Update CHANGELOG.md
3. Create git tag
4. Publish to Cargo
5. Create GitHub release
6. Update Docker images
7. Announce release
```

### Issue Triage
- Label issues (bug, enhancement, question)
- Create project boards
- Respond to issues promptly
- Thank contributors

## 9. Alternative Publishing Options

### Homebrew (macOS)
Create a formula for easy installation:

```ruby
# argus.rb
class Argus < Formula
  desc "Production-ready web crawler"
  homepage "https://github.com/yourusername/argus"
  url "https://github.com/yourusername/argus/archive/v#{version}.tar.gz"
  license "MIT"
  head "https://github.com/yourusername/argus.git"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--path", ".", "--root", prefix
  end
end
```

### Snap (Linux)
Create a snap package:

```yaml
# snap/snapcraft.yaml
name: argus
version: git
summary: Production-ready web crawler
description: |
  A scalable web crawler written in Rust

grade: stable
confinement: strict

parts:
  argus:
    plugin: rust
    source: .
```

### Chocolatey (Windows)
Create a Chocolatey package:

```xml
<!-- argus.nuspec -->
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>argus</id>
    <version>0.1.0</version>
    <title>Argus Web Crawler</title>
    <authors>Your Name</authors>
    <description>Production-ready web crawler</description>
    <licenseUrl>https://github.com/yourusername/argus/blob/main/LICENSE</licenseUrl>
    <projectUrl>https://github.com/yourusername/argus</projectUrl>
  </metadata>
</package>
```

## 10. Monitoring and Analytics

### Crates.io Stats
- Monitor downloads
- Track reverse dependencies
- Watch for issues

### GitHub Stars/Forks
- Track repository growth
- Identify popular features
- Engage with contributors

### Usage Metrics
- Add telemetry (optional)
- Survey users
- Collect feedback

## Checklist Before Publishing

- [ ] All tests pass
- [ ] Documentation is complete
- [ ] License is added
- [ ] README is comprehensive
- [ ] Version numbers are set
- [ ] CHANGELOG is updated
- [ ] CI/CD is configured
- [ ] Security review done
- [ ] Performance benchmarks
- [ ] Examples are working

## Post-Publishing

1. Monitor for issues
2. Respond to feedback
3. Fix bugs quickly
4. Release regular updates
5. Engage with the community
6. Write blog posts
7. Create tutorials
8. Build a roadmap

Good luck with your release! 🚀
