# BM25 Length Normalization Study — Research Program v2

## Status: PIVOTED after GPT Pro review (2026-04-05)
Original BM199 (sqrt normalization) is NOT novel — Cummins & O'Riordan published it.
Reframed as: systematic study of normalization shape families in BM25 under zero-shot BEIR.

## Goal
Find the BEST length normalization function for BM25 that generalizes across heterogeneous corpora,
using a principled tuning/validation/test protocol. The contribution is empirical and systematic,
not architectural.

## Key Discovery (Sweep 2026-04-05)
**Hinged normalization** — linear for short docs (r≤1), sublinear power for long docs (r>1) —
beats all other normalizations on tuning AND is mechanistically novel. Nobody has published a
piecewise BM25 normalization that explicitly decouples short-doc and long-doc treatment.

## Infrastructure
- **Target File:** `src/scorer.rs` — `ScoringConfig` struct with `NormType`, `TfMode`, `IdfMode`
- **Bench Binary:** `cargo run --release --bin bm199-bench-all -- --norm <type> --alpha <val> --k1 <val>`
- **Eval Binary:** `cargo run --release --bin bm199-eval -- --scorer generic --norm <type> --alpha <val> --k1 <val> --split <split>`
- **Sweep Script:** `scripts/sweep_norms.sh`
- **Results:** `paper/logs/norm_sweep_*.csv` and `paper/logs/validation_*.txt`

## Eval Command (for autoresearch)
```
cargo run --release --bin bm199-bench-all -- --norm hinged --alpha 0.70 --k1 1.5 2>/dev/null
```

## Metric
- **Name:** avg_ndcg_at_10
- **Direction:** higher is better
- **BM25 default baseline:** 0.4091 (tuning, k1=1.2, b=0.75)
- **BM25 tuned baseline:** 0.4121 (tuning, k1=1.2, b=1.0)
- **Current best:** Hinged(α=0.70, k1=1.5) = 0.4126 (tuning)

## Dataset Protocol
- **Tuning (4):** NFCorpus, SciFact, FiQA, ArguAna — for k1/alpha optimization
- **Validation (7):** TREC-COVID, Quora, SciDocs, Touché, NQ, DBPedia, Climate-FEVER — for structure selection (BURNED by sqrt comparison)
- **Test (2+):** FEVER, HotpotQA + more to download — NEVER TOUCH until frozen

## Normalization Family (all satisfy f(1)=1)
1. **Linear(b):** 1-b+b*r — BM25 standard (1 param)
2. **Power(α):** r^α — sublinear family, α=0.5 is sqrt (1 param)
3. **Log:** ln(1+r)/ln(2) — slowest growth (0 params)
4. **Sigmoid:** 2r/(1+r) — bounded at 2.0 (0 params)
5. **Hinged(α):** r for r≤1, r^α for r>1 — BEST CANDIDATE (1 param)
6. **Asymmetric(α1,α2):** r^α1 for r≤1, r^α2 for r>1 (2 params)
7. **Saturation(c):** r/(r+c)*(1+c) — diminishing returns (1 param)
8. **Softplus:** ln(1+e^(r-1))/ln(1+e^0) — smooth (0 params)
9. **DualPivot(s1,s2,α):** 3-regime with explicit slopes (3 params)

## TF Variants
- Standard (raw tf), Log (ln(1+tf)), DoubleLog, Capped

## IDF Variants
- Standard (Lucene), ATIRE, Squared, Smoothed

## What to Explore Next
### Phase 1: Validate Hinged ✓ (in progress)
- Run hinged variants on validation set
- Compare against BM25 default and tuned
- If hinged generalizes: this is the paper's contribution

### Phase 2: IDF-conditioned normalization (NOVEL)
- Different alpha for rare vs common terms
- Rare terms (high IDF) appearing in long docs = genuine signal → less normalization
- Common terms (low IDF) in long docs = noise → more normalization
- Implementation: alpha = alpha_base * (1 - gamma * idf_ratio)

### Phase 3: Query-length adaptive k1 (NOVEL)
- Single-term queries: low k1 (exact match matters)
- Multi-term queries: high k1 (allow more saturation)
- Implementation: k1_eff = k1_base * (1 + beta * (qlen - 1))

### Phase 4: Clean ablation for paper
- BM25 default → BM25 tuned → BM25+hinged(same k1) → BM25+hinged(tuned k1)
- Per-query significance tests (paired randomization)
- Length heterogeneity correlation analysis

## Rules
- One normalization family per experiment
- Record ALL results in CSV
- Never evaluate test set until formula AND params are frozen
- Get Codex review after each major finding
