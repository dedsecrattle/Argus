use anyhow::Result;
use argus_config::cli::Cli;
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
    argus_worker::worker::run(config).await?;

    Ok(())
}
