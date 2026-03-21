# Using the Argus Web Crawler

## Quick Start

### Installation
```bash
# Build the CLI
cargo build --release -p argus-cli

# With JavaScript rendering support
cargo build --release -p argus-cli --features js-render
```

### Basic Usage

#### 1. Simple Crawl
```bash
# Crawl a single website
./target/release/argus crawl --seed-url https://example.com --storage-dir ./data

# With limits
./target/release/argus crawl \
  --seed-url https://example.com \
  --max-depth 3 \
  --max-urls 100 \
  --storage-dir ./data
```

#### 2. Content Controls
```bash
# Limit content sizes
./target/release/argus crawl \
  --seed-url https://example.com \
  --max-html-size 10MB \
  --max-text-size 5MB \
  --allow-binary false \
  --storage-dir ./data
```

#### 3. Distributed Crawling with Redis
```bash
# Start Redis first
docker run -d -p 6379:6379 redis

# Run distributed crawler
./target/release/argus crawl \
  --redis-url redis://localhost:6379 \
  --redis-rate-limit \
  --workers 8 \
  --max-urls 10000 \
  --storage-dir ./data
```

#### 4. JavaScript Rendering
```bash
# Build with JS support
cargo build --release -p argus-cli --features js-render

# Crawl SPAs and dynamic sites
./target/release/argus crawl \
  --seed-url https://example-spa.com \
  --js-render \
  --storage-dir ./data
```

## Advanced Usage

### Using as a Library

```rust
use argus_fetcher::{HttpFetcher, FetcherConfig, RetryConfig};
use argus_dedupe::Simhash;
use argus_parser::html::extract_metadata;
use argus_worker::worker::run_in_memory;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure fetcher with retry logic
    let config = FetcherConfig {
        user_agent: "MyBot/1.0".to_string(),
        connect_timeout: Duration::from_secs(10),
        request_timeout: Duration::from_secs(30),
        retry_config: RetryConfig::default(),
        max_redirects: 10,
    };
    
    let fetcher = Arc::new(HttpFetcher::new(config)?);
    
    // Fetch a page
    let job = CrawlJob {
        url: "https://example.com".to_string(),
        normalized_url: "https://example.com".to_string(),
        depth: 0,
    };
    
    let result = fetcher.fetch(&job).await?;
    
    // Extract metadata
    let metadata = extract_metadata(&result.body);
    println!("Title: {:?}", metadata.title);
    
    // Check for duplicates
    let hash = Simhash::from_text(std::str::from_utf8(&result.body)?);
    
    Ok(())
}
```

### Content Deduplication

```rust
use argus_dedupe::Simhash;

// Calculate similarity
let hash1 = Simhash::from_text("Page 1 content");
let hash2 = Simhash::from_text("Page 2 content");

let similarity = hash1.similarity(&hash2);
if hash1.is_near_duplicate(&hash2, 10) {
    println!("Pages are near-duplicates");
}
```

### Sitemap Processing

```rust
use argus_parser::sitemap;

// Discover sitemaps
let sitemap_urls = sitemap::discover_sitemap_urls("https://example.com");

// Parse sitemap content
let entries = sitemap::parse_sitemap(sitemap_content);
for entry in entries {
    match entry {
        sitemap::SitemapEntry::Url(url) => {
            println!("URL: {}", url.loc);
            if let Some(priority) = url.priority {
                println!("  Priority: {}", priority);
            }
        }
        sitemap::SitemapEntry::Index(url) => {
            println!("Sitemap index: {}", url);
        }
    }
}
```

### Distributed Crawling

```bash
# Terminal 1: Start first worker
WORKER_COUNT=4 REDIS_URL=redis://localhost:6379 ./target/release/argus crawl \
  --redis-url redis://localhost:6379 \
  --workers 4 \
  --storage-dir ./data

# Terminal 2: Start second worker (on another machine)
WORKER_COUNT=4 REDIS_URL=redis://your-redis-server ./target/release/argus crawl \
  --redis-url redis://your-redis-server \
  --workers 4 \
  --storage-dir ./data
```

## Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `--seed-url` | Starting URL for crawl | Required (without Redis) |
| `--redis-url` | Redis URL for distributed crawling | None |
| `--max-depth` | Maximum crawl depth | 10 |
| `--max-urls` | Maximum URLs to crawl | Unlimited |
| `--workers` | Number of concurrent workers | 4 |
| `--storage-dir` | Directory to save crawled data | ./data |
| `--respect-robots` | Respect robots.txt rules | true |
| `--crawl-delay` | Delay between requests (ms) | 100 |
| `--same-host` | Only crawl same host | false |
| `--user-agent` | Custom user agent | argus/0.1 |
| `--max-html-size` | Max HTML size to download | 10MB |
| `--max-text-size` | Max text size to download | 5MB |
| `--max-binary-size` | Max binary size to download | 50MB |
| `--allow-binary` | Download binary content | false |

## Data Storage

Crawled data is stored in two files per page:

1. **Metadata** (`page/<hash>.json`)
```json
{
  "url": "https://example.com/page",
  "final_url": "https://example.com/page",
  "status": 200,
  "content_type": "text/html",
  "depth": 0,
  "body_path": "body/<hash>.bin",
  "fetched_at_ms": 1640995200000
}
```

2. **Content** (`body/<hash>.bin`)
- Raw response body as downloaded
- Can be HTML, JSON, PDF, images, etc.

## Monitoring and Graceful Shutdown

The crawler supports graceful shutdown:

```bash
# Send SIGTERM (Ctrl+C)
# Workers will finish current jobs and exit cleanly
```

## Best Practices

1. **Be Respectful**
   - Always respect robots.txt
   - Set appropriate crawl delays
   - Use a descriptive user agent

2. **Distributed Crawling**
   - Use Redis for large crawls
   - Monitor Redis memory usage
   - Consider Redis persistence

3. **Content Handling**
   - Set appropriate size limits
   - Use content deduplication to avoid duplicates
   - Filter binary content if not needed

4. **JavaScript Sites**
   - Enable JS rendering only when needed
   - Increase timeout for heavy sites
   - Consider resource usage

5. **Error Handling**
   - Monitor retry counts
   - Check logs for 429/503 errors
   - Adjust retry configuration as needed

## Examples

See the `examples/` directory for complete working examples:
- `basic_crawl.rs` - Simple crawling
- `redis_distributed.rs` - Distributed crawling
- `advanced_features.rs` - Using all Phase 1 & 2 features
- `javascript_rendering.rs` - SPA crawling
