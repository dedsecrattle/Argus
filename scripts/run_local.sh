#!/usr/bin/env bash
set -euo pipefail

cargo run -p argus-cli -- crawl --seed-url https://example.com
