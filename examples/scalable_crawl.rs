use anyhow::Result;
use argus_common::types::{CrawlConfig, CrawlJob};
use argus_dedupe::{BloomDeduplicator, HybridSeenSet};
use argus_frontier::StreamFrontier;
use argus_storage::S3Storage;
use argus_worker::worker::run;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting scalable crawler with 1B URL support");

    // 1. Bloom Filter Deduplication (for 1B URLs)
    let bloom = BloomDeduplicator::new(1_000_000_000, 0.01);
    info!(
        "Bloom filter initialized: {} MB memory",
        bloom.memory_usage() / 1_048_576
    );

    // 2. Hybrid Deduplication (Bloom + Redis)
    let seen = HybridSeenSet::new(
        "redis://localhost:6379",
        Some("argus:seen:".to_string()),
        1_000_000_000, // 1B URLs
        0.01,          // 1% false positive rate
    )
    .await?;

    let stats = seen.stats();
    info!(
        "Hybrid deduplication: {} MB bloom filter, {} hash functions",
        stats.bloom_memory_bytes / 1_048_576,
        stats.bloom_hash_count
    );

    // 3. Redis Streams for job distribution
    let consumer_name = format!("worker-{}", std::process::id());
    let frontier = StreamFrontier::new(
        "redis://localhost:6379",
        Some("argus:jobs".to_string()),
        Some("argus:workers".to_string()),
        consumer_name,
    )
    .await?
    .with_batch_size(100);

    info!("Redis Streams frontier initialized");

    // 4. S3 Storage (or MinIO for local testing)
    #[cfg(feature = "s3")]
    let storage: Arc<dyn argus_storage::Storage> = {
        // For AWS S3
        let s3 = S3Storage::new("my-crawl-bucket".to_string(), Some("crawl/".to_string())).await?;
        
        // OR for MinIO (local S3-compatible storage)
        // let s3 = S3Storage::new_with_endpoint(
        //     "my-crawl-bucket".to_string(),
        //     Some("crawl/".to_string()),
        //     "http://localhost:9000".to_string(),
        // ).await?;
        
        s3.verify_bucket().await?;
        info!("S3 storage verified and ready");
        Arc::new(s3)
    };

    #[cfg(not(feature = "s3"))]
    let storage: Arc<dyn argus_storage::Storage> = {
        use argus_storage::FileStorage;
        Arc::new(FileStorage::new("./data")?)
    };

    // 5. Configure crawler for high throughput
    let config = CrawlConfig {
        seed_url: Some("https://example.com".to_string()),
        max_depth: 10,
        max_urls: 1_000_000_000, // 1B URLs
        respect_robots: true,
        user_agent: "Argus/1.0 (Scalable Crawler)".to_string(),
        crawl_delay: Duration::from_millis(100),
        same_host: false,
    };

    // 6. Rate limiter
    let rate_limiter = Arc::new(argus_worker::rate_limit::InMemoryRateLimiter::default());

    // 7. Graceful shutdown
    let shutdown = argus_worker::ShutdownSignal::new();
    let shutdown_clone = shutdown.clone();
    tokio::spawn(async move {
        argus_worker::listen_for_shutdown(shutdown_clone).await;
    });

    info!("Starting crawl with scalable architecture");
    info!("- Bloom filter: 1B URLs capacity");
    info!("- Redis Streams: Consumer groups for load balancing");
    info!("- S3 Storage: Unlimited horizontal scaling");
    info!("- Workers: Can scale to 1000+ concurrent workers");

    // Run the crawler
    run(
        config,
        frontier,
        seen,
        storage,
        rate_limiter,
        Some(shutdown),
    )
    .await?;

    Ok(())
}
