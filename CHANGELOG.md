# Changelog

All notable changes to Argus will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Scalability features for 1B+ URL support
- Bloom filter-based distributed deduplication
- S3/MinIO object storage backend
- Redis Streams for job distribution
- Comprehensive documentation and examples

## [0.1.0] - 2024-03-22

### Added
- Initial release of Argus web crawler
- Core crawling functionality with Phase 1 features:
  - Robots.txt compliance with caching
  - Automatic retry with exponential backoff
  - Graceful shutdown on SIGTERM/SIGINT
  - Error categorization (retryable vs permanent)
  - Configurable timeouts and limits

- Content and extraction features (Phase 2):
  - Content deduplication with Simhash
  - Enhanced link extraction (canonical, hreflang, meta tags)
  - Sitemap parsing and auto-discovery
  - Content type detection with size limits
  - Optional JavaScript rendering support

- Distributed crawling support:
  - Redis-based frontier for distributed crawling
  - Worker coordination via Redis
  - Rate limiting and backpressure handling

- Storage backends:
  - File-based storage with metadata
  - S3-compatible object storage (optional)

- CLI interface:
  - Single-node and distributed crawling modes
  - Comprehensive configuration options
  - Docker support with docker-compose

### Performance
- Capable of crawling 10-50M URLs out of the box
- Scales to 1B+ URLs with distributed architecture
- Memory-efficient Bloom filter deduplication (1B URLs = 1.2GB RAM)
- High-throughput job distribution via Redis Streams

### Documentation
- Comprehensive README with usage examples
- API documentation on docs.rs
- Examples for common use cases
- Docker deployment guides
- Scaling documentation for large deployments

## [0.0.1] - Development

### Added
- Initial project structure
- Basic crawling functionality
- Test suite
- CI/CD pipeline setup
