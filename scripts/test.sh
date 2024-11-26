#!/usr/bin/env bash

set -eou pipefail

docker compose up -d
export LOG_LEVEL=info
cargo test --lib -- --test-threads=1 --nocapture
