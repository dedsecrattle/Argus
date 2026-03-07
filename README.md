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
| `--global-concurrency` | 32 | Max concurrent fetch tasks (Tokio workers) |
| `--per-host-concurrency` | 1 | Reserved for future per-host concurrency cap |
| `--per-host-delay-ms` | 500 | Minimum delay between requests to the same host |
| `--max-depth` | 2 | Maximum link depth from seed |
| `--redis-url` | (none) | Use Redis for frontier and seen set. Pass with no value for default `redis://127.0.0.1:6379/` (docker-compose). |
| `--redis-rate-limit` | false | When using Redis, use Redis for per-host rate limiting so all processes share the same limit. |
| `--storage-dir` | (none) | Directory to persist fetch results (metadata JSON + body files). Creates `page/` and `body/` subdirs. |

### Redis (docker-compose)

Start Redis with the included compose file:

```bash
docker compose up -d redis
```

Then run the crawler with Redis (default URL points at the compose Redis):

```bash
cargo run -p argus-cli -- --seed-url https://example.com --redis-url
```

Or pass a URL explicitly: `--redis-url redis://127.0.0.1:6379/`.

### Distributed mode

With a shared Redis instance, multiple crawler processes (or machines) share the same URL queue and seen set. Start Redis (e.g. `docker compose up -d redis`), then run any number of CLI processes with the same `--redis-url`:

```bash
# Terminal 1: docker compose up -d redis

# Terminal 2 and 3 (or more): run crawlers with the same seed and Redis URL
cargo run -p argus-cli -- --seed-url https://example.com --redis-url
```

Each process runs `global_concurrency` async workers that pull from the shared queue. Add `--redis-rate-limit` so per-host delay is enforced in Redis and shared across all processes.

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

## Testing

**Unit tests**

```bash
cargo test --workspace
```

Runs tests in all crates (e.g. URL normalization in `argus-common`, link extraction in `argus-parser`). To run tests for one crate:

```bash
cargo test -p argus-common
cargo test -p argus-parser
```

**Manual crawl**

Run the CLI with a seed URL and optional limits. Uses the network.

```bash
cargo run -p argus-cli -- --seed-url https://example.com --max-depth 1
```

`--max-depth 1` keeps the crawl small (seed + one hop). Omit it for a deeper crawl.

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Persistent storage

With `--storage-dir <dir>`, each fetched page is written under that directory:

- `page/<hash>.json` – URL, final URL, status, content-type, depth, body path, timestamp
- `body/<hash>.bin` – raw response body

Omit `--storage-dir` to run without writing to disk.

## Next

Add robots.txt parsing and optional persistent storage backends (e.g. SQLite).
