use anyhow::Result;
use argus_common::CrawlJob;
use argus_dedupe::seen::SeenUrlSet;
use argus_fetcher::http::HttpFetcher;
use argus_frontier::in_memory::InMemoryFrontier;
use argus_parser::html;
use argus_robots;
use argus_storage;

#[derive(Clone, Debug)]
pub struct CrawlConfig {
    pub seed_url: String,
    pub max_depth: u16,
    pub global_concurrency: usize,
    pub per_host_concurrency: usize,
    pub per_host_delay_ms: u64,
}

pub async fn run(config: CrawlConfig) -> Result<()> {
    let (normalized_seed, host) = match argus_common::url::normalize_url(&config.seed_url) {
        Some(pair) => pair,
        None => {
            anyhow::bail!("invalid seed URL: {}", config.seed_url);
        }
    };

    argus_storage::init_storage();

    let frontier = InMemoryFrontier::default();
    let seen = SeenUrlSet::default();
    let fetcher = HttpFetcher::new()?;

    let seed_job = CrawlJob {
        url: config.seed_url.clone(),
        normalized_url: normalized_seed.clone(),
        host: host.clone(),
        depth: 0,
    };

    if !seen.insert_if_new(normalized_seed).await {
        tracing::info!("seed URL already seen, nothing to do");
        return Ok(());
    }
    frontier.push(seed_job).await;

    tracing::info!(
        "crawl started seed={} max_depth={}",
        config.seed_url,
        config.max_depth
    );

    let mut fetched: u64 = 0;

    while let Some(job) = frontier.pop().await {
        if job.depth > config.max_depth {
            continue;
        }
        if !argus_robots::is_allowed(&job.url) {
            continue;
        }

        let fetch_result = match fetcher.fetch(&job).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("fetch failed url={} error={}", job.url, e);
                continue;
            }
        };

        fetched += 1;
        if fetched.is_multiple_of(10) || fetched == 1 {
            tracing::info!("fetched {} pages (current: {})", fetched, job.url);
        }

        if fetch_result.status != 200 {
            continue;
        }

        let is_html = fetch_result
            .content_type
            .as_deref()
            .is_some_and(|ct| ct.starts_with("text/html"));
        if !is_html {
            continue;
        }

        let links = html::extract_links(&fetch_result.final_url, &fetch_result.body);

        for link in links {
            let Some((norm_url, link_host)) = argus_common::url::normalize_url(&link.to_url) else {
                continue;
            };
            if !seen.insert_if_new(norm_url.clone()).await {
                continue;
            }
            let new_job = CrawlJob {
                url: link.to_url,
                normalized_url: norm_url,
                host: link_host,
                depth: job.depth + 1,
            };
            frontier.push(new_job).await;
        }
    }

    tracing::info!("crawl finished, fetched {} pages", fetched);
    Ok(())
}
