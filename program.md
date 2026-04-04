# BM199: Novel Sparse Retrieval Scoring Algorithm

## Goal
Create a scoring algorithm that beats BM25 (avg nDCG@10 = 0.4004) and all its variants
(BM25+ = 0.3969, BM25L = 0.3995) on BEIR benchmark datasets (NFCorpus, SciFact, FiQA).

## Target File
`src/scorer.rs` — the `Bm199Params` struct and its `score_term` / `score_document` methods.
Also `bm199_params.json` for parameter values.

## Eval Command
```
cargo run --release --bin bm199-bench-all 2>/dev/null
```
This outputs a single float: the average nDCG@10 across all 3 BEIR datasets.

## Metric
- **Name:** avg_ndcg_at_10
- **Direction:** higher is better
- **Baseline (BM25):** 0.4004
- **Current BM199:** 0.3820

## What to Optimize
The `Bm199Params` struct controls:
1. `k1` (0.5-2.5): TF saturation speed
2. `b` (0.0-1.0): length normalization strength
3. `delta` (0.0-2.0): BM25+ lower-bounding floor
4. `beta_min` (0.5-1.0): adaptive saturation exponent for rare terms
5. `beta_max` (0.7-1.0): adaptive saturation exponent for common terms
6. `lambda_cov` (0.0-0.5): coverage bonus weight
7. `log_base` (1.5-10.0): base for logarithmic length normalization

## Strategy
Grid search over coarse parameter space first, then hill-climb from best candidates.

### Phase 1: Parameter Tuning
- Try k1 in {0.8, 1.0, 1.2, 1.5, 2.0}
- Try b in {0.3, 0.5, 0.7, 0.8, 0.9}
- Try delta in {0.0, 0.5, 1.0, 1.5}
- Keep other params at defaults, find best k1/b/delta first.

### Phase 2: Novel Components
- Try different beta_min/beta_max ranges for adaptive saturation
- Try different log_base values
- Try different coverage bonus weights

### Phase 3: Structural Changes
- If stuck: try different saturation curves (e.g., log(1+tf) instead of tf^beta)
- Try mixing BM25L's ctf normalization into BM199
- Try entropy-weighted IDF (BMX-style)

## Rules
- One parameter change per experiment
- Always commit before eval
- Update bm199_params.json with new values
- The scorer code in src/scorer.rs can also be modified for structural changes
