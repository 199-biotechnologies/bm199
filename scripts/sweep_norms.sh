#!/usr/bin/env bash
# sweep_norms.sh — Test ALL normalization hypotheses on the tuning set.
# Outputs a ranked table of results.
# Usage: ./scripts/sweep_norms.sh [--k1 1.8]

set -e
cd "$(dirname "$0")/.."

K1="${1:-1.8}"
BENCH="cargo run --release --bin bm199-bench-all --"
RESULTS_FILE="paper/logs/norm_sweep_$(date +%Y%m%d_%H%M%S).csv"
mkdir -p paper/logs

echo "hypothesis,k1,norm,alpha,extra,ndcg10" > "$RESULTS_FILE"

run() {
    local label="$1"; shift
    local score
    score=$($BENCH "$@" 2>/dev/null)
    echo "$label,$score" >> "$RESULTS_FILE"
    printf "%-45s %s\n" "$label" "$score"
}

echo "================================================================="
echo "BM25 LENGTH NORMALIZATION SWEEP (k1=$K1 unless noted)"
echo "================================================================="

# --- BASELINES ---
echo ""
echo "--- BASELINES ---"
run "bm25_default,1.2,linear,b=0.75,"       --norm linear --b 0.75 --k1 1.2
run "bm25_tuned,1.2,linear,b=0.90,"         --norm linear --b 0.90 --k1 1.2
run "bm25_tuned,1.2,linear,b=1.00,"         --norm linear --b 1.00 --k1 1.2

# --- POWER FAMILY: r^alpha ---
echo ""
echo "--- POWER FAMILY r^alpha ---"
for alpha in 0.20 0.25 0.30 0.35 0.40 0.45 0.50 0.55 0.60 0.65 0.70 0.75 0.80 0.90 1.00; do
    run "power,$K1,power,a=$alpha,"          --norm power --alpha $alpha --k1 $K1
done

# --- POWER FAMILY with different k1 ---
echo ""
echo "--- POWER FAMILY: optimal k1 per alpha ---"
for alpha in 0.30 0.40 0.50 0.60 0.70; do
    for k1 in 1.0 1.2 1.4 1.6 1.8 2.0; do
        run "power_k1,$k1,power,a=$alpha,"   --norm power --alpha $alpha --k1 $k1
    done
done

# --- LOG NORMALIZATION ---
echo ""
echo "--- LOG NORMALIZATION ---"
for k1 in 1.0 1.2 1.4 1.6 1.8 2.0; do
    run "log,$k1,log,,"                      --norm log --k1 $k1
done

# --- SIGMOID NORMALIZATION ---
echo ""
echo "--- SIGMOID (bounded) ---"
for k1 in 1.0 1.2 1.4 1.6 1.8 2.0; do
    run "sigmoid,$k1,sigmoid,,"              --norm sigmoid --k1 $k1
done

# --- HINGED NORMALIZATION ---
echo ""
echo "--- HINGED: linear for r<1, power for r>1 ---"
for alpha in 0.30 0.40 0.50 0.60 0.70 0.80; do
    for k1 in 1.2 1.5 1.8; do
        run "hinged,$k1,hinged,a=$alpha,"    --norm hinged --alpha $alpha --k1 $k1
    done
done

# --- ASYMMETRIC NORMALIZATION ---
echo ""
echo "--- ASYMMETRIC: different alpha for short vs long ---"
for a1 in 0.70 0.80 0.90 1.00; do
    for a2 in 0.30 0.40 0.50 0.60; do
        run "asymmetric,$K1,asymmetric,a1=$a1,a2=$a2" --norm asymmetric --alpha $a1 --alpha2 $a2 --k1 $K1
    done
done

# --- SATURATION NORMALIZATION ---
echo ""
echo "--- SATURATION: r/(r+c)*(1+c) ---"
for c in 0.5 1.0 2.0 3.0 5.0; do
    for k1 in 1.2 1.5 1.8; do
        run "saturation,$k1,saturation,c=$c," --norm saturation --c $c --k1 $k1
    done
done

# --- SOFTPLUS ---
echo ""
echo "--- SOFTPLUS ---"
for k1 in 1.0 1.2 1.4 1.6 1.8 2.0; do
    run "softplus,$k1,softplus,,"            --norm softplus --k1 $k1
done

# --- DUAL PIVOT ---
echo ""
echo "--- DUAL PIVOT: separate short/long slopes ---"
for ss in 0.50 0.75 1.00; do
    for sl in 0.25 0.50 0.75; do
        for alpha in 0.40 0.50 0.60; do
            run "dualpivot,$K1,dualpivot,ss=$ss,sl=$sl a=$alpha" --norm dualpivot --s-short $ss --s-long $sl --alpha $alpha --k1 $K1
        done
    done
done

# --- TF VARIANTS (with sqrt norm) ---
echo ""
echo "--- TF VARIANTS (all with sqrt norm) ---"
for k1 in 1.2 1.5 1.8; do
    run "log_tf,$k1,power,a=0.5,tf=log"     --norm power --alpha 0.5 --tf log --k1 $k1
    run "dlog_tf,$k1,power,a=0.5,tf=dlog"   --norm power --alpha 0.5 --tf dlog --k1 $k1
    run "capped_tf,$k1,power,a=0.5,tf=cap5" --norm power --alpha 0.5 --tf capped --tf-cap 5 --k1 $k1
    run "capped_tf,$k1,power,a=0.5,tf=cap3" --norm power --alpha 0.5 --tf capped --tf-cap 3 --k1 $k1
done

# --- IDF VARIANTS (with sqrt norm) ---
echo ""
echo "--- IDF VARIANTS (all with sqrt norm) ---"
for k1 in 1.5 1.8; do
    run "squared_idf,$k1,power,a=0.5,idf=sq"   --norm power --alpha 0.5 --idf squared --k1 $k1
    run "smoothed_idf,$k1,power,a=0.5,idf=sm"   --norm power --alpha 0.5 --idf smoothed --k1 $k1
    run "atire_idf,$k1,power,a=0.5,idf=atire"   --norm power --alpha 0.5 --idf atire --k1 $k1
done

# --- BM25+ DELTA with various norms ---
echo ""
echo "--- DELTA FLOOR (BM25+ style) ---"
for delta in 0.5 1.0; do
    run "delta_sqrt,$K1,power,a=0.5,d=$delta"    --norm power --alpha 0.5 --delta $delta --k1 $K1
    run "delta_linear,1.2,linear,b=0.75,d=$delta" --norm linear --b 0.75 --delta $delta --k1 1.2
done

echo ""
echo "================================================================="
echo "SWEEP COMPLETE. Results saved to: $RESULTS_FILE"
echo "================================================================="

# Sort by score descending
echo ""
echo "--- TOP 20 HYPOTHESES ---"
tail -n +2 "$RESULTS_FILE" | sort -t',' -k6 -rn | head -20
