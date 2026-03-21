use anyhow::Result;
use argus_cli::run_crawl;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Example: Distributed crawl with Redis
    println!("=== Distributed Crawl with Redis ===");
    
    // You can run multiple instances of this with different worker counts
    let worker_count = env::var("WORKER_COUNT")
        .unwrap_or_else(|_| "4".to_string())
        .parse()?;
    
    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let args = vec![
        "crawl",
        "--redis-url", &redis_url,
        "--redis-rate-limit",
        "--workers", &worker_count.to_string(),
        "--max-urls", "1000",
        "--storage-dir", "./data/distributed",
    ];
    
    run_crawl(&args).await?;
    
    Ok(())
}
