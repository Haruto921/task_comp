#!/bin/bash
set -euo pipefail

cd /app

# Create logs directory if it doesn't exist
mkdir -p /logs/verifier

# Initialize the database singleton if needed (handled in test, but good practice)
# Run the specific integration test
# We capture the exit code of cargo test
cargo test --test integration_test test_read_your_writes --release -- --nocapture
rc=$?

if [ "$rc" -eq 0 ]; then
    echo 1 > /logs/verifier/reward.txt
else
    echo 0 > /logs/verifier/reward.txt
fi
