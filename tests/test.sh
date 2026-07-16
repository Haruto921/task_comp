#!/bin/bash
set -e

cd "$(dirname "$0")/../environment"

export PATH="/usr/local/cargo/bin:$PATH"

echo "=== Running Integration Tests ==="
cargo test --test integration_test --release -- --nocapture
