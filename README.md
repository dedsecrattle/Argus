# Argus

A distributed web crawler written in Rust.

## Prerequisites

- Rust stable toolchain (see `rust-toolchain.toml`)

## Build

```bash
cargo build
```

## Run

```bash
cargo run -p argus-cli -- --seed-url https://example.com
```

Or use the convenience script:

```bash
./scripts/run_local.sh
```

### CLI Options

| Flag | Default | Description |
|------|---------|-------------|
| `--seed-url` | required | Starting URL for the crawl |
| `--global-concurrency` | 32 | Max concurrent fetch tasks |
| `--per-host-concurrency` | 1 | Max concurrent requests per host |
| `--per-host-delay-ms` | 500 | Delay between requests to the same host |
| `--max-depth` | 2 | Maximum link depth from seed |

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `argus-common` | Shared types and URL normalization |
| `argus-config` | CLI argument parsing |
| `argus-frontier` | URL queue management |
| `argus-fetcher` | HTTP client for page retrieval |
| `argus-parser` | HTML link extraction |
| `argus-robots` | robots.txt handling |
| `argus-dedupe` | URL deduplication |
| `argus-storage` | Crawl data persistence |
| `argus-worker` | Crawl loop orchestration |
| `argus-cli` | Command-line entry point |

## Configuration

See `configs/local.toml` for an example configuration file.

## Architecture

See `docs/architecture.md` for a high-level overview of the crawl pipeline.

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Next

Wire the crawl loop across crates, then add host politeness enforcement and persistent storage.
