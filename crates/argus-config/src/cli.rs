use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "argus")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the crawler. With --redis-url, omit --seed-url to run as a consumer only.
    Crawl(CrawlOpts),

    /// Push URLs onto the Redis frontier and exit. Use to feed a shared queue.
    Seed(SeedOpts),
}

#[derive(Debug, Parser)]
pub struct CrawlOpts {
    /// Starting URL. Required unless using --redis-url as consumer-only.
    #[arg(long)]
    pub seed_url: Option<String>,

    #[arg(long, default_value_t = 32)]
    pub global_concurrency: usize,

    #[arg(long, default_value_t = 1)]
    pub per_host_concurrency: usize,

    #[arg(long, default_value_t = 500)]
    pub per_host_delay_ms: u64,

    #[arg(long, default_value_t = 2)]
    pub max_depth: u16,

    /// Redis URL for distributed mode. If set with no value, uses redis://127.0.0.1:6379/ (matches docker-compose).
    #[arg(long, num_args = 0..=1, default_missing_value = "redis://127.0.0.1:6379/")]
    pub redis_url: Option<String>,

    /// When using Redis, use Redis for per-host rate limiting so all processes share the same limit.
    #[arg(long)]
    pub redis_rate_limit: bool,

    /// Directory to persist fetch results (metadata JSON + body files). If unset, nothing is written to disk.
    #[arg(long)]
    pub storage_dir: Option<std::path::PathBuf>,
}

#[derive(Debug, Parser)]
pub struct SeedOpts {
    /// Redis URL. No value = redis://127.0.0.1:6379/
    #[arg(long, num_args = 0..=1, default_missing_value = "redis://127.0.0.1:6379/")]
    pub redis_url: Option<String>,

    /// URLs to push onto the frontier.
    #[arg(short, long, required = true)]
    pub url: Vec<String>,
}
