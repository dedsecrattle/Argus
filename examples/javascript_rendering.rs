use anyhow::Result;
use argus_fetcher::{JsRenderer, HttpFetcher, FetcherConfig};
use argus_parser::html::extract_links;
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Example: Crawl a JavaScript-heavy website
    let url = "https://example.com SPA"; // Replace with a real SPA URL
    
    // 1. Try regular HTTP fetch first
    let config = FetcherConfig::default();
    let fetcher = HttpFetcher::new(config)?;
    
    let job = argus_common::types::CrawlJob {
        url: url.to_string(),
        normalized_url: url.to_string(),
        depth: 0,
    };
    
    match fetcher.fetch(&job).await {
        Ok(result) => {
            info!("Fetched without JS rendering");
            // Extract links from static HTML
            let links = extract_links(url, &result.body);
            info!("Found {} links without JS", links.len());
        }
        Err(_) => {
            info!("Static fetch failed, trying with JS rendering...");
            
            // 2. Use JavaScript rendering if needed
            #[cfg(feature = "js-render")]
            {
                let renderer = JsRenderer::new()
                    .with_timeout(Duration::from_secs(30))
                    .with_headless(true);
                
                match renderer.render(url).await {
                    Ok(rendered_html) => {
                        info!("Successfully rendered with JavaScript");
                        
                        // Extract links from rendered content
                        let links = extract_links(url, rendered_html.as_bytes());
                        info!("Found {} links with JS rendering", links.len());
                        
                        // Save rendered content
                        tokio::fs::write("./data/rendered.html", rendered_html).await?;
                    }
                    Err(e) => {
                        tracing::error!("JS rendering failed: {}", e);
                    }
                }
            }
            
            #[cfg(not(feature = "js-render"))]
            {
                info!("JS rendering not enabled. Build with --features js-render");
            }
        }
    }
    
    Ok(())
}
