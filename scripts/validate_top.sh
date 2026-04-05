#!/usr/bin/env bash
# validate_top.sh — Run top tuning candidates on validation set
set -e
cd "$(dirname "$0")/.."

EVAL="cargo run --release --bin bm199-eval --"
RESULTS="paper/logs/validation_top_$(date +%Y%m%d_%H%M%S).txt"
mkdir -p paper/logs

run_val() {
    local label="$1"; shift
    echo "=== $label ===" | tee -a "$RESULTS"
    $EVAL --split validation "$@" 2>&1 | grep -E '(nDCG@10=|avg)' | tee -a "$RESULTS"
    echo "" >> "$RESULTS"
}

echo "VALIDATION SET EVALUATION — $(date)" | tee "$RESULTS"
echo "=============================================" | tee -a "$RESULTS"

# We need to update eval binary to support generic scorer too.
# For now, use the existing variants for BM25 and BM199,
# and add a quick generic eval mode.

echo "Running BM25 default (k1=1.2, b=0.75)..."
run_val "BM25 default (k1=1.2, b=0.75)" --variant bm25

echo "Running BM25 tuned (k1=1.2, b=1.0)..."
run_val "BM25 tuned (k1=1.2, b=1.0)" --variant bm25 --bm25-b 1.0

echo "Running BM199 sqrt (k1=1.5)..."
run_val "BM199 sqrt (k1=1.5)" --variant bm199 --bm199-k1 1.5

echo "Running BM199 sqrt (k1=1.8)..."
run_val "BM199 sqrt (k1=1.8)" --variant bm199 --bm199-k1 1.8

echo "============================================="
echo "Done. Results in $RESULTS"
