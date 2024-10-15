#!/usr/bin/env bash

set -eou pipefail

docker compose up wait-for-services-to-be-ready -d
export LOG_LEVEL=info
cargo test --lib -- --test-threads=1 --nocapture
