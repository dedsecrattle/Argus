use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use argus_common::CrawlJob;
use argus_dedupe::SeenSet;
use argus_fetcher::http::HttpFetcher;
use argus_frontier::Frontier;
use argus_parser::html;
use argus_robots;
use argus_storage;
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
pub struct CrawlConfig {
    pub seed_url: String,
    pub max_depth: u16,
    pub global_concurrency: usize,
    pub per_host_concurrency: usize,
    pub per_host_delay_ms: u64,
}

/// Runs the crawl with the given frontier and seen set. Use in-memory or Redis
/// backends so the same logic works single-node or distributed.
pub async fn run<F, S>(config: CrawlConfig, frontier: F, seen: S) -> Result<()>
where
    F: Frontier + Clone + Send + Sync + 'static,
    S: SeenSet + Clone + Send + Sync + 'static,
{
    let (normalized_seed, host) = match argus_common::url::normalize_url(&config.seed_url) {
        Some(pair) => pair,
        None => anyhow::bail!("invalid seed URL: {}", config.seed_url),
    };

    argus_storage::init_storage();

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
        "crawl started seed={} concurrency={} max_depth={}",
        config.seed_url,
        config.global_concurrency,
        config.max_depth
    );

    let fetched = Arc::new(AtomicU64::new(0));
    let active = Arc::new(AtomicU64::new(0));
    let last_fetch_per_host: Arc<Mutex<HashMap<String, Instant>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let delay = Duration::from_millis(config.per_host_delay_ms);

    let concurrency = config.global_concurrency.max(1);
    let mut handles = Vec::with_capacity(concurrency);

    for _ in 0..concurrency {
        let frontier = frontier.clone();
        let seen = seen.clone();
        let fetcher = fetcher.clone();
        let config = config.clone();
        let fetched = Arc::clone(&fetched);
        let active = Arc::clone(&active);
        let last_fetch = Arc::clone(&last_fetch_per_host);
        let delay_ms = delay;

        handles.push(tokio::spawn(async move {
            loop {
                let job = match frontier.pop().await {
                    Some(j) => j,
                    None => {
                        if active.load(Ordering::SeqCst) == 0 {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        continue;
                    }
                };

                active.fetch_add(1, Ordering::SeqCst);

                if job.depth > config.max_depth {
                    active.fetch_sub(1, Ordering::SeqCst);
                    continue;
                }
                if !argus_robots::is_allowed(&job.url) {
                    active.fetch_sub(1, Ordering::SeqCst);
                    continue;
                }

                {
                    let map = last_fetch.lock().await;
                    let last = map.get(&job.host).copied();
                    drop(map);
                    if let Some(last) = last {
                        let elapsed = last.elapsed();
                        if elapsed < delay_ms {
                            tokio::time::sleep(delay_ms - elapsed).await;
                        }
                    }
                    let mut map = last_fetch.lock().await;
                    map.insert(job.host.clone(), Instant::now());
                }

                let fetch_result = match fetcher.fetch(&job).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!("fetch failed url={} error={}", job.url, e);
                        active.fetch_sub(1, Ordering::SeqCst);
                        continue;
                    }
                };

                let n = fetched.fetch_add(1, Ordering::SeqCst) + 1;
                if n == 1 || n.is_multiple_of(10) {
                    tracing::info!("fetched {} pages (current: {})", n, job.url);
                }

                if fetch_result.status != 200 {
                    active.fetch_sub(1, Ordering::SeqCst);
                    continue;
                }

                let is_html = fetch_result
                    .content_type
                    .as_deref()
                    .is_some_and(|ct| ct.starts_with("text/html"));
                if !is_html {
                    active.fetch_sub(1, Ordering::SeqCst);
                    continue;
                }

                let links = html::extract_links(&fetch_result.final_url, &fetch_result.body);

                for link in links {
                    let Some((norm_url, link_host)) =
                        argus_common::url::normalize_url(&link.to_url)
                    else {
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

                active.fetch_sub(1, Ordering::SeqCst);
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let total = fetched.load(Ordering::SeqCst);
    tracing::info!("crawl finished, fetched {} pages", total);
    Ok(())
}

/// In-memory backend for single-node runs.
pub async fn run_in_memory(config: CrawlConfig) -> Result<()> {
    let frontier = argus_frontier::InMemoryFrontier::default();
    let seen = argus_dedupe::SeenUrlSet::default();
    run(config, frontier, seen).await
}

/// Redis-backed frontier and seen set so multiple processes share the same queue.
/// Run with `cargo run -p argus-cli --features redis -- --redis-url redis://127.0.0.1/ ...`
#[cfg(feature = "redis")]
pub async fn run_redis(config: CrawlConfig, redis_url: &str) -> Result<()> {
    use argus_dedupe::RedisSeenSet;
    use argus_frontier::RedisFrontier;

    let frontier = RedisFrontier::connect(redis_url, None).await?;
    let seen = RedisSeenSet::connect(redis_url, None).await?;
    run(config, frontier, seen).await
}
