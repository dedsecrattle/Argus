use anyhow::Result;
use argus_cli::run_crawl;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Example 1: Basic single-node crawl
    println!("=== Example 1: Basic Crawl ===");
    let args = vec![
        "crawl",
        "--seed-url", "https://example.com",
        "--max-depth", "2",
        "--storage-dir", "./data/example",
        "--max-urls", "10",
    ];
    
    run_crawl(&args).await?;
    
    // Example 2: Crawl with content limits
    println!("\n=== Example 2: Crawl with Content Limits ===");
    let args = vec![
        "crawl",
        "--seed-url", "https://example.com",
        "--storage-dir", "./data/limited",
        "--max-html-size", "1MB",
        "--max-text-size", "500KB",
        "--allow-binary", "false",
    ];
    
    run_crawl(&args).await?;
    
    Ok(())
}
