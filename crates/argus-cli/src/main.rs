use std::sync::Arc;

use anyhow::{Context, Result};
use argus_config::cli::{Cli, Command, CrawlOpts, SeedOpts};
use argus_storage::{FileStorage, NoopStorage, Storage};
use clap::Parser;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    match cli.command {
        Command::Crawl(opts) => run_crawl(opts).await,
        Command::Seed(opts) => run_seed(opts).await,
    }
}

async fn run_crawl(opts: CrawlOpts) -> Result<()> {
    let seed_url = opts.seed_url;
    let redis_url = opts.redis_url.as_deref();

    if seed_url.is_none() && redis_url.is_none() {
        anyhow::bail!("either --seed-url or --redis-url is required for crawl");
    }
    if seed_url.is_none() && redis_url.is_some() {
        tracing::info!("running as consumer only (no seed URL)");
    } else if let Some(ref u) = seed_url {
        tracing::info!("starting argus with seed {}", u);
    }

    let config = argus_worker::worker::CrawlConfig {
        seed_url,
        max_depth: opts.max_depth,
        global_concurrency: opts.global_concurrency,
        per_host_concurrency: opts.per_host_concurrency,
        per_host_delay_ms: opts.per_host_delay_ms,
    };

    let storage: Arc<dyn Storage> = match &opts.storage_dir {
        Some(dir) => {
            let s = FileStorage::new(dir);
            s.ensure_dirs().await?;
            Arc::new(s)
        }
        None => Arc::new(NoopStorage),
    };

    if let Some(redis_url) = redis_url {
        argus_worker::worker::run_redis(config, redis_url, storage, opts.redis_rate_limit).await
    } else {
        let url = config
            .seed_url
            .as_deref()
            .context("crawl without Redis requires --seed-url")?;
        // In-memory mode requires a seed
        if url.is_empty() {
            anyhow::bail!("--seed-url is required for in-memory crawl");
        }
        argus_worker::worker::run_in_memory(config, storage).await
    }
}

async fn run_seed(opts: SeedOpts) -> Result<()> {
    let redis_url = opts
        .redis_url
        .as_deref()
        .context("seed requires --redis-url")?;
    if opts.url.is_empty() {
        anyhow::bail!("seed requires at least one -u/--url");
    }
    argus_worker::worker::seed_redis(redis_url, &opts.url).await
}
