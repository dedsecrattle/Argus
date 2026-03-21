# Scaling to 1 Billion URLs

## Current Limitations

### 1. Memory Bottlenecks
- **Seen URL Set**: In-memory deduplication won't scale to 1B URLs
- **Frontier Queue**: Memory-based queue consumes O(n) memory
- **Simhash Storage**: All hashes stored in memory

### 2. Single Points of Failure
- **Redis Single Node**: Limited by single machine memory
- **File Storage**: Local filesystem doesn't scale horizontally

### 3. Performance Issues
- **Network I/O**: Single HTTP client per worker
- **Disk I/O**: Synchronous file writes
- **CPU**: Regex processing on every page

## Required Architecture Changes

### 1. Distributed Deduplication

```rust
// Bloom filter for initial deduplication
use probabilistic_collections::bloom::BloomFilter;

struct DistributedSeenSet {
    // Fast probabilistic check
    bloom_filter: BloomFilter<String>,
    // Fallback to distributed storage
    redis: RedisClient,
    // Persistent storage
    database: DatabaseClient,
}

impl DistributedSeenSet {
    async fn insert_if_new(&self, url: &str) -> bool {
        // Quick check with bloom filter
        if self.bloom_filter.contains(url) {
            // Might be seen, check persistent storage
            !self.redis.exists(url).await.unwrap_or(false)
        } else {
            // Definitely new
            self.bloom_filter.insert(url);
            self.redis.set(url, true).await.ok();
            true
        }
    }
}
```

### 2. Sharded Frontend

```rust
// Consistent hashing for URL distribution
use consistent_hash::ConsistentHash;

struct ShardedFrontier {
    shards: Vec<RedisFrontier>,
    hasher: ConsistentHash<String>,
}

impl ShardedFrontier {
    async fn push(&self, job: CrawlJob) -> Result<()> {
        let shard = self.hasher.get(&job.normalized_url);
        shard.push(job).await
    }
    
    async fn pop(&self) -> Option<CrawlJob> {
        // Try random shards for load balancing
        let mut shards = self.shards.clone();
        shards.shuffle(&mut thread_rng());
        
        for shard in shards {
            if let Some(job) = shard.pop().await {
                return Some(job);
            }
        }
        None
    }
}
```

### 3. Object Storage

```rust
// S3/S3-compatible storage for crawled data
use aws_sdk_s3 as s3;

struct S3Storage {
    client: s3::Client,
    bucket: String,
    prefix: String,
}

impl S3Storage {
    async fn store(&self, url: &str, content: &[u8]) -> Result<()> {
        let key = format!("{}/{}.bin", self.prefix, url_to_fragment(url));
        
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(content.to_vec()))
            .send()
            .await?;
            
        Ok(())
    }
}
```

## Infrastructure Requirements

### 1. Redis Cluster
```
Minimum: 6 nodes (3 masters, 3 replicas)
Memory: 64GB per master node
Network: 10Gbps between nodes
```

### 2. Worker Cluster
```
Workers: 1000+ concurrent workers
CPU: 32 cores per worker node
Memory: 64GB per worker node
Network: 10Gbps
```

### 3. Storage
```
Object Storage: 100TB+ (S3, GCS, or MinIO)
Database: PostgreSQL cluster for metadata
Cache: Redis cluster for deduplication
```

## Performance Optimizations

### 1. Connection Pooling
```rust
use r2d2::Pool;
use r2d2_redis::RedisConnectionManager;

let redis_pool = Pool::new(RedisConnectionManager::new(redis_url)?)?;
```

### 2. Batch Operations
```rust
// Batch Redis operations
const BATCH_SIZE: usize = 1000;

let mut batch = Vec::with_capacity(BATCH_SIZE);
for url in urls {
    batch.push(url);
    if batch.len() == BATCH_SIZE {
        redis.mset(&batch).await?;
        batch.clear();
    }
}
```

### 3. Async I/O Optimization
```rust
// Use optimized async runtime
#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    // Configure tokio for high concurrency
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)
        .max_blocking_threads(32)
        .enable_all()
        .build()?;
}
```

## Estimated Costs

### AWS (1B URLs, 1TB content)
- **EC2 Workers**: 100x m5.8xlarge @ $1.53/hr = $153/hr
- **ElastiCache**: 6x r6g.16xlarge @ $4.25/hr = $25.50/hr
- **S3 Storage**: 1TB @ $23/TB/month = $23/month
- **Data Transfer**: 10TB @ $0.09/GB = $900

**Total**: ~$178/hr + storage costs

### On-Premise
- **Hardware**: $500K initial investment
- **Power/Cooling**: $50K/month
- **Network**: $20K/month
- **Staff**: $100K/month

## Implementation Roadmap

### Phase 1: Distributed Deduplication (2 weeks)
- Implement Bloom filter + Redis hybrid
- Add consistent hashing
- Test with 10M URLs

### Phase 2: Storage Migration (2 weeks)
- Implement S3 storage backend
- Add metadata database
- Migrate file storage

### Phase 3: Performance Optimization (2 weeks)
- Add connection pooling
- Implement batch operations
- Optimize regex processing

### Phase 4: Monitoring & Scaling (1 week)
- Add metrics collection
- Implement auto-scaling
- Create dashboards

## Alternative Approaches

### 1. Serverless Architecture
```yaml
# AWS Lambda + SQS + DynamoDB
- Crawler: Lambda functions
- Queue: SQS (unlimited scale)
- Storage: S3
- Deduplication: DynamoDB + DAX
```

### 2. Stream Processing
```yaml
# Kafka + Flink/Spark Streaming
- Ingest: Kafka topics
- Process: Flink cluster
- Storage: HDFS/S3
- Deduplication: Redis cluster
```

### 3. Microservices
```yaml
# Separate services for each component
- Frontend Service: URL queue management
- Fetcher Service: HTTP fetching
- Parser Service: Content extraction
- Storage Service: Data persistence
- Dedupe Service: Duplicate detection
```

## Monitoring at Scale

### Key Metrics
- URLs per second
- Error rate by domain
- Queue depth per shard
- Redis memory usage
- Storage throughput

### Alerts
- Queue depth > 1M
- Error rate > 5%
- Redis memory > 80%
- Worker failure rate > 1%

## Conclusion

Scaling to 1B URLs is achievable but requires:
1. **Distributed architecture** (no single points of failure)
2. **Probabilistic data structures** (Bloom filters)
3. **Object storage** (S3-compatible)
4. **Sharding** (consistent hashing)
5. **Significant infrastructure investment**

Current implementation gets you ~10M URLs. For 1B URLs, you need the architectural changes outlined above.
