# Deploying the Argus Web Crawler

## Production Deployment Options

### 1. Single-Node Deployment

For small to medium sites (< 100k pages):

```bash
# Build
cargo build --release -p argus-cli

# Run
./target/release/argus crawl \
  --seed-url https://yoursite.com \
  --max-depth 5 \
  --max-urls 50000 \
  --workers 8 \
  --storage-dir /data/crawl \
  --crawl-delay 100
```

### 2. Distributed Deployment with Docker

For large sites (> 100k pages):

```bash
# Start the cluster
docker-compose up -d

# Scale workers as needed
docker-compose up -d --scale argus-worker=5
```

### 3. Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: argus-worker
spec:
  replicas: 5
  selector:
    matchLabels:
      app: argus-worker
  template:
    metadata:
      labels:
        app: argus-worker
    spec:
      containers:
      - name: argus
        image: argus:latest
        command: ["argus", "crawl"]
        args:
          - "--redis-url"
          - "redis://redis-service:6379"
          - "--workers"
          - "4"
          - "--max-urls"
          - "100000"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
```

## Configuration for Production

### Environment Variables

```bash
# Redis Configuration
export REDIS_URL="redis://cluster:6379"
export REDIS_PASSWORD="your-password"

# Crawler Configuration
export CRAWL_DELAY=500
export MAX_WORKERS=8
export MAX_URLS=1000000
export STORAGE_DIR="/data/crawl"

# Logging
export RUST_LOG="info,argus_fetcher=debug"
export RUST_LOG_FORMAT="json"

# Monitoring
export PROMETHEUS_PORT=9090
export HEALTH_CHECK_PORT=8080
```

### Resource Requirements

| Scale | CPU | Memory | Storage | Network |
|-------|-----|--------|---------|---------|
| Small (1M URLs) | 2 cores | 4GB | 100GB SSD | 100Mbps |
| Medium (10M URLs) | 8 cores | 16GB | 1TB SSD | 1Gbps |
| Large (100M URLs) | 32 cores | 64GB | 10TB SSD | 10Gbps |

## Monitoring and Observability

### 1. Metrics Collection

```rust
// Add to your crawler
use prometheus::{Counter, Histogram, IntGauge};

lazy_static! {
    static ref PAGES_FETCHED: Counter = Counter::new(
        "argus_pages_fetched_total",
        "Total number of pages fetched"
    ).unwrap();
    
    static ref FETCH_DURATION: Histogram = Histogram::new(
        "argus_fetch_duration_seconds",
        "Time spent fetching pages"
    ).unwrap();
    
    static ref QUEUE_SIZE: IntGauge = IntGauge::new(
        "argus_queue_size",
        "Number of URLs in queue"
    ).unwrap();
}
```

### 2. Health Checks

```rust
// Health check endpoint
use axum::{response::Json, routing::get, Router};

async fn health_check() -> Json<HealthStatus> {
    Json(HealthStatus {
        status: "healthy",
        queue_size: queue.len(),
        active_workers: workers.len(),
    })
}

let app = Router::new()
    .route("/health", get(health_check));
```

### 3. Log Aggregation

```bash
# With structured logging
export RUST_LOG_FORMAT=json

# Example log entry
{
  "timestamp": "2024-01-01T12:00:00Z",
  "level": "info",
  "target": "argus_fetcher",
  "message": "Successfully fetched page",
  "url": "https://example.com",
  "status": 200,
  "duration_ms": 234
}
```

## Performance Optimization

### 1. Connection Pooling

```rust
// Configure connection pools
let pool_config = PoolConfig {
    max_idle_per_host: 10,
    idle_timeout: Some(Duration::from_secs(30)),
    max_lifetime_per_host: Some(Duration::from_secs(300)),
};
```

### 2. Batching Operations

```rust
// Batch Redis operations
let mut pipe = redis::pipe();
for url in urls {
    pipe.sadd("seen_urls", url);
}
pipe.query_async(&mut redis_conn).await?;
```

### 3. Async I/O Optimization

```rust
// Use tokio's optimized runtime
#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<()> {
    // Your crawler code
}
```

## Security Considerations

### 1. Network Security

```yaml
# Network policies for Kubernetes
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: argus-netpol
spec:
  podSelector:
    matchLabels:
      app: argus
  policyTypes:
  - Egress
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
  - to: []
    ports:
    - protocol: TCP
      port: 443
    - protocol: TCP
      port: 80
```

### 2. Rate Limiting

```rust
// Implement adaptive rate limiting
let mut limiter = AdaptiveRateLimiter::new(
    Duration::from_millis(100),
    2.0, // multiplier
    Duration::from_secs(10), // max delay
);

if let Some(delay) = limiter.next_delay(response_status) {
    tokio::time::sleep(delay).await;
}
```

### 3. Data Privacy

```rust
// Filter sensitive data
let sensitive_patterns = vec![
    r"\b\d{4}[-]?\d{4}[-]?\d{4}[-]?\d{4}\b", // Credit cards
    r"\b\d{3}-\d{2}-\d{4}\b", // SSN
];

for pattern in sensitive_patterns {
    content = regex::Regex::new(pattern)?.replace_all(&content, "[REDACTED]");
}
```

## Scaling Strategies

### 1. Horizontal Scaling

- Add more workers
- Shard by domain
- Use Redis Cluster

### 2. Vertical Scaling

- Increase worker count per node
- Optimize memory usage
- Use faster storage (NVMe)

### 3. Geographic Distribution

```rust
// Regional crawling
let regional_configs = vec![
    RegionConfig { region: "us-east", proxy: Some("proxy-us.example.com") },
    RegionConfig { region: "eu-west", proxy: Some("proxy-eu.example.com") },
    RegionConfig { region: "asia-pacific", proxy: Some("proxy-ap.example.com") },
];
```

## Backup and Recovery

### 1. Data Backup

```bash
# Backup crawled data
rsync -av /data/crawl/ backup-server:/backups/$(date +%Y%m%d)/

# Backup Redis state
redis-cli BGSAVE
scp /var/lib/redis/dump.rdb backup-server:/backups/redis/
```

### 2. Checkpointing

```rust
// Save crawl state
let checkpoint = CrawlCheckpoint {
    processed_urls: seen_set.len(),
    queue_snapshot: frontier.snapshot(),
    timestamp: SystemTime::now(),
};

checkpoint.save("./data/checkpoint.json").await?;
```

### 3. Recovery

```rust
// Resume from checkpoint
if let Some(checkpoint) = CrawlCheckpoint::load("./data/checkpoint.json").await? {
    frontier.restore(checkpoint.queue_snapshot)?;
    info!("Resuming from checkpoint at {:?}", checkpoint.timestamp);
}
```

## Troubleshooting Production Issues

### Common Problems

1. **High Memory Usage**
   - Check for memory leaks in URL deduplication
   - Reduce batch sizes
   - Add memory limits to containers

2. **Redis Connection Errors**
   - Monitor connection pool usage
   - Implement retry logic with exponential backoff
   - Consider Redis Sentinel for HA

3. **Slow Performance**
   - Profile with `cargo flamegraph`
   - Check disk I/O bottlenecks
   - Optimize regex patterns

### Debug Commands

```bash
# Check Redis status
redis-cli info memory
redis-cli llen crawl_queue

# Monitor system resources
htop
iotop
netstat -i

# Check crawler logs
journalctl -u argus -f
tail -f /var/log/argus/crawler.log
```
