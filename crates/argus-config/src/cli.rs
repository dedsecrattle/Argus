use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "argus")]
pub struct Cli {
    #[arg(long)]
    pub seed_url: String,

    #[arg(long, default_value_t = 32)]
    pub global_concurrency: usize,

    #[arg(long, default_value_t = 1)]
    pub per_host_concurrency: usize,

    #[arg(long, default_value_t = 500)]
    pub per_host_delay_ms: u64,

    #[arg(long, default_value_t = 2)]
    pub max_depth: u16,
}
