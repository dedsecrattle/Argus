<p align="center">
  <img src="assets/logo.png" alt="Argus" width="200" />
</p>

# Argus

**Argus** is a distributed web crawler written in Rust. Run it as a single process or scale out across many workers sharing a Redis-backed queue and seen set.

[![CI](https://github.com/dedsecrattle/argus/actions/workflows/ci.yml/badge.svg)](https://github.com/dedsecrattle/argus/actions)

---

## Overview

- **Single-node or distributed** — In-memory for local runs; Redis for a shared frontier and deduplication across processes.
- **Async** — Tokio-based worker pool with configurable concurrency and per-host rate limiting (in-memory or Redis).
- **Persistent storage** — Optional file-based storage for fetched pages (metadata JSON + raw body).
- **CLI** — `crawl` to run the crawler; `seed` to push URLs into Redis for worker-based setups.

## Requirements

- [Rust](https://www.rust-lang.org/) stable (see [rust-toolchain.toml](rust-toolchain.toml))
- For distributed mode: [Redis](https://redis.io/) (e.g. via [Docker](https://www.docker.com/))

## Installation

```bash
git clone https://github.com/your-username/argus.git
cd argus
cargo build --release -p argus-cli
```

The binary is at `target/release/argus-cli`. Symlink or copy it to a directory on your `PATH` if you want to run it as `argus`.

## Quick start

**One-off crawl (single process):**

```bash
cargo run -p argus-cli -- crawl --seed-url https://example.com --max-depth 2
```

**With Redis (e.g. for multiple workers):**

```bash
docker compose up -d redis
cargo run -p argus-cli -- crawl --seed-url https://example.com --redis-url
```

See [Deployment](docs/deployment.md) for multi-worker and continuous-crawl setups.

## Usage

### Commands

| Command | Description                                                                                                                                                                      |
| ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crawl` | Run the crawler. Seeds the queue (or Redis) with `--seed-url` if given, then consumes until the queue is empty. With `--redis-url`, omit `--seed-url` to run as a consumer only. |
| `seed`  | Push one or more URLs into the Redis frontier and exit. Use to feed a shared queue without running a crawl.                                                                      |

### Examples

```bash
# Single process, limit depth
argus crawl --seed-url https://example.com --max-depth 1

# Persist fetched pages to disk
argus crawl --seed-url https://example.com --storage-dir ./crawl-data

# Feed Redis and run workers (distributed)
argus seed --redis-url -u https://example.com -u https://iana.org
argus crawl --redis-url --redis-rate-limit
```

### Crawl options

| Option                     | Default                               | Description                                                                |
| -------------------------- | ------------------------------------- | -------------------------------------------------------------------------- |
| `--seed-url <URL>`         | required (unless Redis consumer-only) | Starting URL. Omit when using `--redis-url` to run as consumer only.       |
| `--max-depth <N>`          | 2                                     | Maximum link depth from seed.                                              |
| `--global-concurrency <N>` | 32                                    | Number of concurrent fetch tasks per process.                              |
| `--per-host-delay-ms <MS>` | 500                                   | Minimum delay between requests to the same host.                           |
| `--redis-url [URL]`        | (none)                                | Use Redis for frontier and seen set. No value = `redis://127.0.0.1:6379/`. |
| `--redis-rate-limit`       | false                                 | Use Redis for per-host rate limiting (shared across workers).              |
| `--storage-dir <DIR>`      | (none)                                | Directory for persisted pages (`page/*.json`, `body/*.bin`).               |

### Seed options

| Option               | Description                                      |
| -------------------- | ------------------------------------------------ |
| `--redis-url [URL]`  | Redis URL. No value = `redis://127.0.0.1:6379/`. |
| `-u, --url <URL>...` | One or more URLs to push onto the frontier.      |

## Redis and Docker

A Redis instance is required for distributed mode. Use the included Compose file:

```bash
docker compose up -d redis
```

Then run the CLI with `--redis-url` (or `--redis-url redis://127.0.0.1:6379/`).

## Persistent storage

With `--storage-dir <dir>`, each fetched page is written under that directory:

- `page/<hash>.json` — URL, final URL, status, content-type, depth, body path, timestamp
- `body/<hash>.bin` — raw response body

Omit `--storage-dir` to run without writing to disk.

## Project structure

| Crate                                   | Description                     |
| --------------------------------------- | ------------------------------- |
| [argus-common](crates/argus-common)     | Shared types, URL normalization |
| [argus-config](crates/argus-config)     | CLI and config types            |
| [argus-frontier](crates/argus-frontier) | URL queue (in-memory, Redis)    |
| [argus-fetcher](crates/argus-fetcher)   | HTTP client                     |
| [argus-parser](crates/argus-parser)     | HTML link extraction            |
| [argus-robots](crates/argus-robots)     | robots.txt (stub)               |
| [argus-dedupe](crates/argus-dedupe)     | Seen-URL set (in-memory, Redis) |
| [argus-storage](crates/argus-storage)   | Persistence (no-op, file)       |
| [argus-worker](crates/argus-worker)     | Crawl loop and rate limiting    |
| [argus-cli](crates/argus-cli)           | CLI entrypoint                  |

## Documentation

- [Architecture](docs/architecture.md) — Crawl pipeline and data flow
- [Deployment](docs/deployment.md) — Single-node, distributed workers, containers, continuous crawl

## Development

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

CI runs the same steps (see [.github/workflows/ci.yml](.github/workflows/ci.yml)).

## Contributing

Contributions are welcome. Open an issue or a pull request; for larger changes, discuss in an issue first.

## License

See [LICENSE](LICENSE) in the repository root.
