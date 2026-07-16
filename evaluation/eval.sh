#!/bin/bash
set -e

cd "$(dirname "$0")/../environment"
export PATH="/usr/local/cargo/bin:$PATH"

RESULT="PASS"
SCORE=0

echo "=============================================="
echo "  Stale Read Fix - Evaluation"
echo "=============================================="
echo ""

# Build check
echo "[1/4] Checking build..."
if cargo build --release 2>&1 | tee /tmp/build.log; then
    echo "  ✓ Build successful"
    SCORE=$((SCORE + 20))
else
    echo "  ✗ Build failed"
    RESULT="FAIL"
fi
echo ""

# Basic test
echo "[2/4] Running test_read_your_writes_basic..."
if cargo test --test integration_test test_read_your_writes_basic --release -- --nocapture 2>&1; then
    echo "  ✓ Basic test passed"
    SCORE=$((SCORE + 20))
else
    echo "  ✗ Basic test failed"
    RESULT="FAIL"
fi
echo ""

# Cross-task test
echo "[3/4] Running test_cross_task_token_persistence..."
if cargo test --test integration_test test_cross_task_token_persistence --release -- --nocapture 2>&1; then
    echo "  ✓ Cross-task test passed"
    SCORE=$((SCORE + 30))
else
    echo "  ✗ Cross-task test failed"
    RESULT="FAIL"
fi
echo ""

# Concurrent sessions
echo "[4/4] Running test_concurrent_sessions..."
if cargo test --test integration_test test_concurrent_sessions --release -- --nocapture 2>&1; then
    echo "  ✓ Concurrent sessions test passed"
    SCORE=$((SCORE + 30))
else
    echo "  ✗ Concurrent sessions test failed"
    RESULT="FAIL"
fi
echo ""

# Check for thread_local usage
echo "[Bonus] Checking for thread_local removal..."
if grep -r "thread_local" src/ 2>/dev/null; then
    echo "  ! thread_local still present (may or may not be an issue)"
else
    echo "  ✓ No thread_local usage"
    SCORE=$((SCORE + 10))
fi
echo ""

echo "=============================================="
echo "  Final Score: $SCORE/110"
echo "  Result: $RESULT"
echo "=============================================="

[ "$RESULT" = "PASS" ] && exit 0 || exit 1
