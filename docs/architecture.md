# Argus Architecture

Seed URLs enter the frontier (in-memory or Redis).
A pool of async workers (Tokio) pull jobs from the frontier; concurrency is set by `global_concurrency`.
Per-host politeness: before each fetch, workers enforce a minimum delay per host (`per_host_delay_ms`).
Workers fetch only when robots allow (future: real robots.txt).
Fetched pages are parsed for links.
Links are normalized and checked against the seen set (in-memory or Redis); new URLs are pushed to the frontier.
When the queue is empty and no worker is busy, the crawl finishes.

**Single-node:** frontier and seen set are in-memory; one process, many concurrent workers.

**Distributed:** frontier and seen set are in Redis (same keys for all processes). Multiple processes (or machines) run the CLI with the same `--redis-url`; they share the queue and dedupe, so work is split automatically.
