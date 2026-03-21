# Argus Crawler Examples

This directory contains practical examples of how to use the Argus web crawler with all Phase 1 & 2 features implemented.

## Running the Examples

1. **Build the project first:**
```bash
cargo build --release
```

2. **Run individual examples:**
```bash
# Basic single-node crawl
cargo run --example basic_crawl

# Distributed crawling (requires Redis)
cargo run --example redis_distributed

# Advanced features showcase
cargo run --example advanced_features

# JavaScript rendering (requires js-render feature)
cargo run --example javascript_rendering --features js-render
```

## Example Descriptions

### 1. basic_crawl.rs
Demonstrates:
- Basic crawling configuration
- Content size limits
- Storage directory setup
- Command-line argument handling

### 2. redis_distributed.rs
Demonstrates:
- Redis-based distributed crawling
- Multiple worker coordination
- Global rate limiting
- Environment variable configuration

### 3. advanced_features.rs
Demonstrates:
- Custom fetcher configuration with retry logic
- Content deduplication using Simhash
- Metadata extraction (canonical URLs, hreflang, etc.)
- Sitemap discovery
- Graceful shutdown handling

### 4. javascript_rendering.rs
Demonstrates:
- Fallback from static to JS rendering
- Headless Chrome integration
- Dynamic content extraction
- Feature flag usage

## Real-World Use Cases

### 1. SEO Analysis
```rust
// Extract SEO metadata
let metadata = extract_metadata(&body);
println!("Canonical: {:?}", metadata.canonical_url);
println!("Title: {:?}", metadata.title);
println!("Description: {:?}", metadata.description);
```

### 2. Content Monitoring
```rust
// Track content changes
let hash = Simhash::from_text(content);
if !previous_hashes.contains(&hash) {
    notify_content_changed(url);
}
```

### 3. Site Architecture Analysis
```rust
// Discover all pages via sitemap
for sitemap_url in sitemap::discover_sitemap_urls(base_url) {
    let entries = sitemap::parse_sitemap(fetch(sitemap_url));
    // Analyze site structure
}
```

### 4. Distributed Scraping
```bash
# Scale horizontally
worker1$ ./argus crawl --redis-url redis://cluster:6379 --workers 8
worker2$ ./argus crawl --redis-url redis://cluster:6379 --workers 8
worker3$ ./argus crawl --redis-url redis://cluster:6379 --workers 8
```

## Configuration Tips

1. **For Small Sites** (< 1000 pages)
   - Use in-memory mode
   - 2-4 workers
   - 100ms delay

2. **For Medium Sites** (1000-100k pages)
   - Use Redis mode
   - 4-8 workers per machine
   - 200ms delay
   - Enable content deduplication

3. **For Large Sites** (> 100k pages)
   - Distributed Redis cluster
   - 16+ workers total
   - 500ms+ delay
   - Strict content limits
   - Monitor Redis memory

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Reduce `max_urls` limit
   - Enable `--allow-binary false`
   - Use Redis persistence

2. **Getting Blocked**
   - Increase `--crawl-delay`
   - Check robots.txt compliance
   - Rotate user agents

3. **JavaScript Not Rendering**
   - Build with `--features js-render`
   - Increase timeout
   - Check browser dependencies

4. **Redis Connection Issues**
   - Verify Redis is running
   - Check connection string
   - Ensure network connectivity
