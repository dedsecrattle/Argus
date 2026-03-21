# Scaling Argus to 1 Billion URLs

## Overview

This guide explains how to use the newly implemented scalability features to handle 1 billion+ URLs.

## Architecture Components

### 1. Distributed Deduplication with Bloom Filters

**Memory Efficiency**: 1B URLs = ~1.2GB RAM (vs. 100GB+ with naive approach)

```rust
use argus_dedupe::{BloomDeduplicator, HybridSeenSet};

// Option A: Pure Bloom Filter (99% accuracy)
let bloom = BloomDeduplicator::new(1_000_000_000, 0.01);
println!("Memory: {} MB", bloom.memory_usage() / 1_048_576);

// Option B: Hybrid (Bloom + Redis for 100% accuracy)
let seen = HybridSeenSet::new(
    "redis://localhost:6379",
    Some("argus:seen:".to_string()),
    1_000_000_000, // 1B URLs
    0.01,          // 1% false positive rate
).await?;
```

**How it works**:
1. Bloom filter does fast "probably not seen" check (O(1))
2. If bloom says "might be seen", check Redis
3. Only ~1% of URLs hit Redis (99% filtered by bloom)

### 2. Object Storage (S3/MinIO)

**Unlimited Horizontal Scaling**: Store petabytes of crawled data

```rust
use argus_storage::S3Storage;

// AWS S3
let storage = S3Storage::new(
    "my-crawl-bucket".to_string(),
    Some("crawl/".to_string())
).await?;

// MinIO (self-hosted S3-compatible)
let storage = S3Storage::new_with_endpoint(
    "my-crawl-bucket".to_string(),
    Some("crawl/".to_string()),
    "http://minio:9000".to_string(),
).await?;

storage.verify_bucket().await?;
```

**Benefits**:
- No disk space limits
- Built-in replication
- CDN integration
- Pay-per-use pricing

### 3. Redis Streams for Job Distribution

**High-Throughput Job Queue**: Consumer groups for load balancing

```rust
use argus_frontier::StreamFrontier;

let frontier = StreamFrontier::new(
    "redis://localhost:6379",
    Some("argus:jobs".to_string()),
    Some("argus:workers".to_string()),
    format!("worker-{}", std::process::id()),
).await?
.with_batch_size(100);

// Automatic load balancing across consumers
let job = frontier.pop().await;

// Acknowledge when done
frontier.ack(&message_id).await?;
```

**Features**:
- Consumer groups (automatic load balancing)
- Message acknowledgment (at-least-once delivery)
- Backpressure handling
- Dead letter queue (claim abandoned messages)

## Deployment Configurations

### Small Scale (1-10M URLs)

```bash
# Build
cargo build --release -p argus-cli

# Run with bloom filter
./target/release/argus crawl \
  --seed-url https://example.com \
  --storage-dir ./data \
  --bloom-filter \
  --bloom-capacity 10000000 \
  --workers 8
```

**Infrastructure**:
- 1 machine (16 cores, 32GB RAM)
- Local file storage
- Cost: ~$200/month

### Medium Scale (10-100M URLs)

```bash
# Build with S3 support
cargo build --release -p argus-cli --features s3

# Run with hybrid deduplication + S3
./target/release/argus crawl \
  --redis-url redis://localhost:6379 \
  --s3-bucket my-crawl-bucket \
  --bloom-capacity 100000000 \
  --workers 16
```

**Infrastructure**:
- 5 machines (32 cores, 64GB RAM each)
- Redis cluster (3 nodes)
- S3 storage
- Cost: ~$2,000/month

### Large Scale (100M-1B URLs)

```bash
# Build with all features
cargo build --release -p argus-cli --features "s3,redis"

# Run with Redis Streams + Hybrid deduplication + S3
./target/release/argus crawl \
  --redis-url redis://cluster:6379 \
  --redis-streams \
  --s3-bucket my-crawl-bucket \
  --s3-prefix crawl/ \
  --bloom-capacity 1000000000 \
  --workers 32
```

**Infrastructure**:
- 100+ machines (64 cores, 128GB RAM each)
- Redis cluster (100 nodes)
- S3 storage (1PB+)
- Cost: ~$20,000/month

## Docker Compose Setup

### MinIO + Redis Streams

```yaml
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data

  minio:
    image: minio/minio:latest
    ports:
      - "9000:9000"
      - "9001:9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    command: server /data --console-address ":9001"
    volumes:
      - minio_data:/data

  argus-worker:
    build: .
    command: >
      cargo run --release -p argus-cli --features s3 -- crawl
      --redis-url redis://redis:6379
      --redis-streams
      --s3-bucket crawl-data
      --s3-endpoint http://minio:9000
      --bloom-capacity 100000000
      --workers 16
    environment:
      AWS_ACCESS_KEY_ID: minioadmin
      AWS_SECRET_ACCESS_KEY: minioadmin
      AWS_REGION: us-east-1
      RUST_LOG: info
    depends_on:
      - redis
      - minio

volumes:
  redis_data:
  minio_data:
```

Start with:
```bash
docker-compose up -d
docker-compose up --scale argus-worker=5
```

## Performance Benchmarks

### Deduplication Performance

| Method | Memory (1B URLs) | Lookup Time | Accuracy |
|--------|------------------|-------------|----------|
| HashMap | 100GB+ | O(1) | 100% |
| Bloom Filter | 1.2GB | O(1) | 99% |
| Hybrid (Bloom+Redis) | 1.2GB | O(1) avg | 100% |

### Storage Performance

| Backend | Write Throughput | Cost (1PB) | Scalability |
|---------|------------------|------------|-------------|
| Local Disk | 100 MB/s | $10,000 | Limited |
| S3 | 5 GB/s | $23,000/month | Unlimited |
| MinIO | 1 GB/s | $5,000 | High |

### Job Distribution Performance

| Method | Throughput | Latency | Reliability |
|--------|------------|---------|-------------|
| Redis List | 10K jobs/s | 1ms | Good |
| Redis Streams | 100K jobs/s | 1ms | Excellent |
| Kafka | 1M jobs/s | 5ms | Excellent |

## Monitoring

### Key Metrics

```rust
// Bloom filter stats
let stats = seen.stats();
println!("Bloom memory: {} MB", stats.bloom_memory_bytes / 1_048_576);
println!("Hash functions: {}", stats.bloom_hash_count);

// Stream stats
let len = frontier.stream_len().await?;
let pending = frontier.pending_count().await?;
println!("Queue length: {}", len);
println!("Pending messages: {}", pending);
```

### Prometheus Metrics

```rust
use prometheus::{Counter, Histogram, IntGauge};

lazy_static! {
    static ref BLOOM_CHECKS: Counter = Counter::new(
        "argus_bloom_checks_total",
        "Total bloom filter checks"
    ).unwrap();
    
    static ref BLOOM_HITS: Counter = Counter::new(
        "argus_bloom_hits_total",
        "Bloom filter hits (might be duplicate)"
    ).unwrap();
    
    static ref STREAM_LAG: IntGauge = IntGauge::new(
        "argus_stream_lag",
        "Number of pending messages in stream"
    ).unwrap();
}
```

## Cost Optimization

### 1. Use Spot Instances
```bash
# AWS EC2 Spot instances (70% cheaper)
aws ec2 run-instances \
  --instance-type m5.8xlarge \
  --instance-market-options MarketType=spot
```

### 2. Compress Data
```rust
// Enable compression in S3
use flate2::write::GzEncoder;

let compressed = GzEncoder::new(Vec::new(), Compression::default());
// Reduces storage costs by 70%
```

### 3. Use S3 Intelligent-Tiering
```bash
# Automatically moves data to cheaper storage tiers
aws s3api put-bucket-intelligent-tiering-configuration \
  --bucket my-crawl-bucket \
  --id default \
  --intelligent-tiering-configuration ...
```

## Troubleshooting

### High Memory Usage
```bash
# Check bloom filter size
echo "Bloom capacity: 1B URLs = 1.2GB"
echo "Actual memory: $(ps aux | grep argus | awk '{print $6/1024 "MB"}')"

# Reduce capacity if needed
--bloom-capacity 100000000  # 100M URLs = 120MB
```

### Redis Connection Issues
```bash
# Check Redis memory
redis-cli info memory

# Check connection pool
redis-cli client list | wc -l

# Increase max connections
redis-cli config set maxclients 10000
```

### S3 Throttling
```bash
# Request rate limits
# S3: 3,500 PUT/s, 5,500 GET/s per prefix

# Solution: Use multiple prefixes
--s3-prefix "crawl/shard-{0..99}/"
```

## Migration Guide

### From File Storage to S3

```rust
// 1. Read from file storage
let file_storage = FileStorage::new("./data")?;

// 2. Write to S3
let s3_storage = S3Storage::new("bucket".to_string(), None).await?;

// 3. Migrate data
for file in list_files("./data")? {
    let content = tokio::fs::read(&file).await?;
    s3_storage.put_object(&file, &content).await?;
}
```

### From In-Memory to Hybrid Deduplication

```rust
// 1. Export seen URLs
let seen_set = SeenUrlSet::default();
let urls: Vec<String> = seen_set.export();

// 2. Import to hybrid
let hybrid = HybridSeenSet::new(...).await?;
for url in urls {
    hybrid.insert_if_new(&url).await;
}
```

## Best Practices

1. **Start Small**: Test with 1M URLs before scaling to 1B
2. **Monitor Everything**: Track bloom filter hit rate, Redis memory, S3 costs
3. **Use Compression**: Compress data before storing in S3
4. **Shard Data**: Use multiple Redis instances and S3 prefixes
5. **Plan for Failures**: Implement retry logic and dead letter queues
6. **Cost Awareness**: Monitor S3 costs daily, use lifecycle policies

## Next Steps

- Implement auto-scaling based on queue depth
- Add distributed tracing (OpenTelemetry)
- Implement rate limiting per domain
- Add content deduplication with Simhash
- Implement incremental crawling
