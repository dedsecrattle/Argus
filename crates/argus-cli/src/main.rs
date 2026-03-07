use std::sync::Arc;

use anyhow::Result;
use argus_config::cli::Cli;
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

    tracing::info!("starting argus with seed {}", cli.seed_url);

    let config = argus_worker::worker::CrawlConfig {
        seed_url: cli.seed_url,
        max_depth: cli.max_depth,
        global_concurrency: cli.global_concurrency,
        per_host_concurrency: cli.per_host_concurrency,
        per_host_delay_ms: cli.per_host_delay_ms,
    };

    let storage: Arc<dyn Storage> = match &cli.storage_dir {
        Some(dir) => {
            let s = FileStorage::new(dir);
            s.ensure_dirs().await?;
            Arc::new(s)
        }
        None => Arc::new(NoopStorage),
    };

    if let Some(ref redis_url) = cli.redis_url {
        argus_worker::worker::run_redis(
            config,
            redis_url,
            storage,
            cli.redis_rate_limit,
        )
        .await?;
    } else {
        argus_worker::worker::run_in_memory(config, storage).await?;
    }

    Ok(())
}
