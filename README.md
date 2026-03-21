# Argus Web Crawler

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/argus-crawler.svg)](https://crates.io/crates/argus-crawler)
[![Documentation](https://docs.rs/argus-crawler/badge.svg)](https://docs.rs/argus-crawler)
[![Build Status](https://github.com/dedsecrattle/argus/workflows/CI/badge.svg)](https://github.com/dedsecrattle/argus/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A production-ready web crawler written in Rust, capable of handling **billions of URLs** with advanced features like content deduplication, distributed crawling, and JavaScript rendering.

</div>

## ⚡ Quick Start

### Installation

#### 📦 Cargo (Recommended)
```bash
cargo install argus-crawler
```

#### 🍺 Homebrew (macOS)
```bash
brew tap dedsecrattle/argus
brew install argus-crawler
```

#### 🐧 Snap (Linux)
```bash
snap install argus
```

#### 🪟 Chocolatey (Windows)
```bash
choco install argus
```

#### 🐳 Docker
```bash
docker run dedsecrattle/argus crawl --seed-url https://example.com
```

### Basic Usage

```bash
# Simple crawl
argus crawl --seed-url https://example.com --storage-dir ./data

# Distributed crawling with Redis
argus crawl --redis-url redis://localhost:6379 --workers 8

# JavaScript rendering (build with js-render feature)
argus crawl --seed-url https://spa-example.com --js-render
```

## 🚀 Features

### Core Features
- ✅ **Robust Error Handling** - Automatic retry with exponential backoff
- ✅ **Robots.txt Compliance** - Full respect for crawl rules
- ✅ **Graceful Shutdown** - Clean interruption on SIGTERM/SIGINT
- ✅ **Rate Limiting** - Configurable delays per domain
- ✅ **Content Limits** - Size limits for HTML, text, and binary content

### Advanced Features
- 🔄 **Content Deduplication** - Simhash-based near-duplicate detection
- 🌐 **JavaScript Rendering** - Headless Chrome support for SPAs
- 📊 **Metadata Extraction** - Canonical URLs, hreflang, meta tags
- 🗺️ **Sitemap Parsing** - Auto-discovery and parsing of sitemaps
- 📦 **Multiple Storage Backends** - File system or S3-compatible storage

### Scalability Features
- 🧠 **Bloom Filter Deduplication** - 1B URLs in only 1.2GB RAM
- 🔀 **Distributed Crawling** - Redis-based coordination
- 🌊 **Redis Streams** - High-throughput job distribution
- ☁️ **Object Storage** - Unlimited scaling with S3/MinIO

## 📈 Performance

| Metric | Single Node | Distributed (10 nodes) |
|--------|-------------|------------------------|
| URLs/second | 100-1000 | 1000-10000 |
| Memory (1B URLs) | 1.2GB (Bloom) | 1.2GB per node |
| Storage | Local disk | S3 (unlimited) |
| Network | 1 Gbps | 10 Gbps+ |

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontier      │    │    Fetcher      │    │    Parser       │
│                 │    │                 │    │                 │
│ • URL Queue     │───▶│ • HTTP Client   │───▶│ • HTML Parser   │
│ • Prioritization│    │ • Retry Logic   │    │ • Link Extract  │
│ • Deduplication │    │ • Rate Limit    │    │ • Metadata      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Deduplication │    │    Storage      │    │   Robots.txt   │
│                 │    │                 │    │                 │
│ • Bloom Filter  │    │ • File System   │    │ • Parser        │
│ • Simhash       │    │ • S3/MinIO      │    │ • Cache         │
│ • Redis         │    │ • Metadata      │    │ • Rules         │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📚 Documentation

- [API Documentation](https://docs.rs/argus-crawler)
- [Deployment Guide](DEPLOYMENT_GUIDE.md)
- [Scaling to 1B URLs](docs/SCALING_GUIDE.md)
- [Contributing](CONTRIBUTING.md)

## 💡 Examples

### Basic Crawling
```rust
use argus_cli::run_crawl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_crawl(&[
        "crawl",
        "--seed-url", "https://example.com",
        "--max-depth", "3",
        "--storage-dir", "./data"
    ]).await
}
```

### Distributed Crawling
```rust
use argus_frontier::StreamFrontier;
use argus_dedupe::HybridSeenSet;
use argus_storage::S3Storage;

// Redis Streams for job distribution
let frontier = StreamFrontier::new(
    "redis://localhost:6379",
    Some("argus:jobs".to_string()),
    Some("workers".to_string()),
    "worker-1".to_string()
).await?;

// Bloom filter + Redis for deduplication
let seen = HybridSeenSet::new(
    "redis://localhost:6379",
    None,
    1_000_000_000, // 1B URLs
    0.01 // 1% false positive rate
).await?;

// S3 for unlimited storage
let storage = S3Storage::new(
    "my-crawl-bucket".to_string(),
    Some("crawl/".to_string())
).await?;
```

### JavaScript Rendering
```bash
# Build with JS support
cargo build --release --features js-render

# Crawl SPA sites
argus crawl \
  --seed-url https://react-app.com \
  --js-render \
  --wait-for-selector "#content"
```

## 🛠️ Development

### Setup
```bash
git clone https://github.com/dedsecrattle/argus.git
cd argus
cargo build
cargo test
```

### Features
- `redis` - Enable Redis support (default)
- `s3` - Enable S3 storage
- `js-render` - Enable JavaScript rendering
- `all-features` - Enable everything

```bash
# Build with all features
cargo build --all-features

# Run tests with all features
cargo test --all-features
```

## 📦 Crates

This is a workspace with the following crates:

- [argus-crawler](https://crates.io/crates/argus-crawler) - Command-line interface
- [argus-common](https://crates.io/crates/argus-common) - Common types and utilities
- [argus-fetcher](https://crates.io/crates/argus-fetcher) - HTTP fetching with retry logic
- [argus-parser](https://crates.io/crates/argus-parser) - HTML and sitemap parsing
- [argus-dedupe](https://crates.io/crates/argus-dedupe) - Content deduplication with Simhash
- [argus-storage](https://crates.io/crates/argus-storage) - Storage backends
- [argus-frontier](https://crates.io/crates/argus-frontier) - URL frontier implementations
- [argus-robots](https://crates.io/crates/argus-robots) - Robots.txt parsing
- [argus-worker](https://crates.io/crates/argus-worker) - Worker implementation
- [argus-config](https://crates.io/crates/argus-config) - Configuration management

## 🐳 Docker

### Basic Usage
```bash
# Pull image
docker pull dedsecrattle/argus:latest

# Run crawl
docker run -v $(pwd)/data:/data dedsecrattle/argus \
  crawl --seed-url https://example.com --storage-dir /data
```

### With Docker Compose
```yaml
version: '3.8'
services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
  
  argus:
    image: dedsecrattle/argus:latest
    command: crawl --redis-url redis://redis:6379
    volumes:
      - ./data:/data
    depends_on:
      - redis
```

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

### Quick Start
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Inspired by [Scrapy](https://scrapy.org/) and [Nutch](https://nutch.apache.org/)
- Icons by [Feather Icons](https://feathericons.com/)

## 🔗 Links

- [Website](https://dedsecrattle.github.io/argus)
- [Documentation](https://docs.rs/argus-crawler)
- [Crates.io](https://crates.io/crates/argus-crawler)
- [Docker Hub](https://hub.docker.com/r/dedsecrattle/argus)
- [GitHub](https://github.com/dedsecrattle/argus)

---

<div align="center">

**[⭐ Star us on GitHub!](https://github.com/dedsecrattle/argus)**

Made with ❤️ by the Argus contributors

</div>
