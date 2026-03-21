use anyhow::Result;
use argus_common::types::{CrawlJob, FetchResult};
use argus_dedupe::{SeenUrlSet, Simhash};
use argus_fetcher::{HttpFetcher, FetcherConfig, RetryConfig, ContentLimits};
use argus_parser::{html::extract_metadata, sitemap};
use argus_storage::FileStorage;
use argus_worker::worker::run_in_memory;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Configure fetcher with Phase 1 features
    let config = FetcherConfig {
        user_agent: "MyBot/1.0".to_string(),
        connect_timeout: std::time::Duration::from_secs(5),
        request_timeout: std::time::Duration::from_secs(20),
        max_redirects: 5,
        retry_config: RetryConfig {
            max_retries: 3,
            initial_backoff: std::time::Duration::from_millis(100),
            max_backoff: std::time::Duration::from_secs(10),
            backoff_multiplier: 2.0,
        },
    };

    let fetcher = Arc::new(HttpFetcher::new(config)?);

    // 2. Content deduplication with Simhash
    let mut content_hashes: Vec<Simhash> = Vec::new();
    
    // 3. Fetch and analyze a page
    let url = "https://example.com";
    let job = CrawlJob {
        url: url.to_string(),
        normalized_url: url.to_string(),
        depth: 0,
    };

    let result = fetcher.fetch(&job).await?;
    
    // 4. Extract metadata (Phase 2 feature)
    let metadata = extract_metadata(&result.body);
    info!("Page title: {:?}", metadata.title);
    info!("Canonical URL: {:?}", metadata.canonical_url);
    info!("Alternate URLs: {:?}", metadata.alternate_urls);

    // 5. Check for near-duplicates
    let text_content = std::str::from_utf8(&result.body)?;
    let hash = Simhash::from_text(text_content);
    
    for existing_hash in &content_hashes {
        if hash.is_near_duplicate(existing_hash, 10) {
            info!("Found near-duplicate content!");
            break;
        }
    }
    content_hashes.push(hash);

    // 6. Discover sitemaps
    let sitemap_urls = sitemap::discover_sitemap_urls(url);
    info!("Potential sitemaps: {:?}", sitemap_urls);

    // 7. Run a full crawl with all features
    let crawl_config = argus_common::types::CrawlConfig {
        seed_url: Some(url.to_string()),
        max_depth: 3,
        max_urls: 100,
        respect_robots: true,
        user_agent: "MyBot/1.0".to_string(),
        crawl_delay: std::time::Duration::from_millis(100),
        same_host: true,
    };

    let storage = Arc::new(FileStorage::new("./data/advanced")?);
    
    // Create shutdown signal
    let shutdown = argus_worker::ShutdownSignal::new();
    let shutdown_clone = shutdown.clone();
    
    tokio::spawn(async move {
        argus_worker::listen_for_shutdown(shutdown_clone).await;
    });

    info!("Starting crawl...");
    run_in_memory(crawl_config, storage, Some(shutdown)).await?;
    
    Ok(())
}
