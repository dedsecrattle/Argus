use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::Result;
use argus_common::CrawlJob;
use argus_dedupe::SeenSet;
use argus_fetcher::http::HttpFetcher;
use argus_frontier::Frontier;
use argus_parser::html;
use argus_robots;
use argus_storage::Storage;

use crate::rate_limit::{InMemoryRateLimiter, RateLimiter};

#[derive(Clone, Debug)]
pub struct CrawlConfig {
    /// If Some, push this URL as the seed job before running. If None (e.g. Redis consumer-only), just drain the queue.
    pub seed_url: Option<String>,
    pub max_depth: u16,
    pub global_concurrency: usize,
    pub per_host_concurrency: usize,
    pub per_host_delay_ms: u64,
}

/// Runs the crawl with the given frontier, seen set, storage, and rate limiter.
pub async fn run<F, S>(
    config: CrawlConfig,
    frontier: F,
    seen: S,
    storage: Arc<dyn Storage>,
    rate_limiter: Arc<dyn RateLimiter>,
) -> Result<()>
where
    F: Frontier + Clone + Send + Sync + 'static,
    S: SeenSet + Clone + Send + Sync + 'static,
{
    argus_storage::init_storage();

    if let Some(ref seed_url) = config.seed_url {
        let (normalized_seed, host) = match argus_common::url::normalize_url(seed_url) {
            Some(pair) => pair,
            None => anyhow::bail!("invalid seed URL: {}", seed_url),
        };
        let seed_job = CrawlJob {
            url: seed_url.clone(),
            normalized_url: normalized_seed.clone(),
            host: host.clone(),
            depth: 0,
        };
        if !seen.insert_if_new(normalized_seed).await {
            tracing::info!("seed URL already seen, skipping push");
        } else {
            frontier.push(seed_job).await;
        }
        tracing::info!(
            "crawl started seed={} concurrency={} max_depth={}",
            seed_url,
            config.global_concurrency,
            config.max_depth
        );
    } else {
        tracing::info!(
            "crawl started (consumer only) concurrency={} max_depth={}",
            config.global_concurrency,
            config.max_depth
        );
    }

    let fetcher = HttpFetcher::new()?;

    let fetched = Arc::new(AtomicU64::new(0));
    let active = Arc::new(AtomicU64::new(0));
    let concurrency = config.global_concurrency.max(1);
    let mut handles = Vec::with_capacity(concurrency);

    for _ in 0..concurrency {
        let frontier = frontier.clone();
        let seen = seen.clone();
        let fetcher = fetcher.clone();
        let storage = Arc::clone(&storage);
        let rate_limiter = Arc::clone(&rate_limiter);
        let config = config.clone();
        let fetched = Arc::clone(&fetched);
        let active = Arc::clone(&active);

        handles.push(tokio::spawn(async move {
            loop {
                let job = match frontier.pop().await {
                    Some(j) => j,
                    None => {
                        if active.load(Ordering::SeqCst) == 0 {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
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

                rate_limiter
                    .wait_for_host(&job.host, config.per_host_delay_ms)
                    .await;

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

                if let Err(e) = storage.record_fetch(&job, &fetch_result).await {
                    tracing::warn!("storage record failed url={} error={}", job.url, e);
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
pub async fn run_in_memory(config: CrawlConfig, storage: Arc<dyn Storage>) -> Result<()> {
    let frontier = argus_frontier::InMemoryFrontier::default();
    let seen = argus_dedupe::SeenUrlSet::default();
    let rate_limiter = Arc::new(InMemoryRateLimiter::default());
    run(config, frontier, seen, storage, rate_limiter).await
}

/// Redis-backed frontier and seen set; optional Redis-backed rate limiter for global per-host delay.
#[cfg(feature = "redis")]
pub async fn run_redis(
    config: CrawlConfig,
    redis_url: &str,
    storage: Arc<dyn Storage>,
    use_redis_rate_limit: bool,
) -> Result<()> {
    use argus_dedupe::RedisSeenSet;
    use argus_frontier::RedisFrontier;

    use crate::rate_limit::RedisRateLimiter;

    let frontier = RedisFrontier::connect(redis_url, None).await?;
    let seen = RedisSeenSet::connect(redis_url, None).await?;
    let rate_limiter: Arc<dyn RateLimiter> = if use_redis_rate_limit {
        Arc::new(RedisRateLimiter::connect(redis_url).await?)
    } else {
        Arc::new(InMemoryRateLimiter::default())
    };
    run(config, frontier, seen, storage, rate_limiter).await
}

/// Push URLs onto the Redis frontier (and mark them in the seen set). Exits after pushing; no crawl.
#[cfg(feature = "redis")]
pub async fn seed_redis(redis_url: &str, urls: &[String]) -> Result<()> {
    use argus_dedupe::RedisSeenSet;
    use argus_frontier::RedisFrontier;

    let frontier = RedisFrontier::connect(redis_url, None).await?;
    let seen = RedisSeenSet::connect(redis_url, None).await?;

    for url in urls {
        let Some((normalized_url, host)) = argus_common::url::normalize_url(url) else {
            tracing::warn!("invalid URL, skipping: {}", url);
            continue;
        };
        let job = CrawlJob {
            url: url.clone(),
            normalized_url: normalized_url.clone(),
            host,
            depth: 0,
        };
        if seen.insert_if_new(normalized_url).await {
            frontier.push(job).await;
            tracing::info!("seeded: {}", url);
        }
    }
    Ok(())
}
