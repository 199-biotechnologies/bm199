# Session Handoff: BM25 Length Normalization Study

**Date:** 2026-04-05 08:30
**Session:** Fixed critical issues from GPT Pro review, pivoted paper framing, built generic scoring framework, ran 250+ hypothesis sweep, found sqrt is uniquely robust.

---

## What Was Accomplished This Session

### Critical Fixes (Phase 1)
1. **Tokenizer bug fixed** — `src/beir.rs`: Removed trailing-s stripping, added proper possessive handling
2. **Clean BM199 function** — `src/scorer.rs`: Added standalone `bm199(tf,df,dl,avgdl,n,k1)` with one-line BM25 diff
3. **Fast scoring path** — `src/index.rs`: Added `Bm199` to `Bm25Variant`, eliminated 3-5x speed penalty
4. **BM25 b-parameter tuning** — eval binary now accepts `--bm25-k1`, `--bm25-b`, `--bm199-k1`
5. **Dataset split update** — heldout → validation (burned) + test (untouched)

### Generic Scoring Framework
- `src/scorer.rs`: Added `NormType` enum (11 variants), `TfMode`, `IdfMode`, `ScoringConfig`
- `src/scorer.rs`: Added `compute_norm()`, `compute_tf()`, `compute_idf()`, `score_generic()`
- `src/index.rs`: Added `search_generic()` method
- `src/bin/bench_all.rs`: Rewritten to accept `--norm`, `--alpha`, `--k1`, `--gamma` etc.
- `src/bin/eval.rs`: Added `--scorer generic` mode with full CLI config

### Hypothesis Sweep (250+ configurations)
- Tested 9 normalization families × multiple params × k1 values
- Results saved to `paper/logs/norm_sweep_20260405_070332.csv`
- IDF-conditioned results in `paper/logs/idfcond_sweep_20260405.txt`

### Validation Results
- Full validation (7 datasets) for 9 variants
- Results saved to `paper/logs/validation_results_20260405.md`

### Paper Draft v2
- `paper/drafts/bm199_paper_draft_v2.md` — Complete rewrite as systematic normalization study
- Reframed: not "novel model" but "systematic study confirming sqrt robustness"
- Cummins & O'Riordan prior art acknowledged

### Updated Research Documents
- `program.md` — New research program v2
- `autoresearch.toml` — Updated config
- `paper/data/hypotheses.md` — Comprehensive hypothesis tracker

## Key Findings

### CENTRAL RESULT: Sqrt is uniquely robust
| Variant | Tuning | Validation | Gap |
|---------|--------|------------|-----|
| **Sqrt(k1=1.5)** | **0.3994** | **0.3998** | **+0.1%** |
| BM25 default | 0.4091 | 0.3725 | -9.0% |
| Log | 0.4096 | 0.3594 | -12.3% |
| IdfCond (best tuning) | 0.4131 | 0.3525 | -14.6% |
| Hinged (best tuning) | 0.4126 | 0.3379 | -18.1% |
| BM25 tuned (b=1.0) | 0.4121 | 0.3274 | -20.6% |

### Per-Dataset Sqrt vs BM25 (validation)
| Dataset | BM25 | Sqrt | Delta |
|---------|------|------|-------|
| TREC-COVID | 0.6340 | 0.6544 | +3.2% |
| Quora | 0.7853 | 0.7809 | -0.6% |
| SciDocs | 0.1569 | 0.1564 | -0.3% |
| Touché | 0.3157 | 0.4233 | +34.1% |
| NQ | 0.2898 | 0.2986 | +3.0% |
| DBPedia | 0.2888 | 0.3019 | +4.5% |
| Climate-FEVER | 0.1368 | 0.1831 | +33.9% |
| **Average** | **0.3725** | **0.3998** | **+7.3%** |

Sqrt wins 5/7. Losses are tiny (-0.3%, -0.6%). Wins are massive on heterogeneous collections.

### Anti-Correlation Discovery
Tuning performance ANTI-correlates with validation performance across 250+ configurations.
The more a normalization fits the tuning set, the worse it generalizes.
Only sqrt(α=0.5) breaks this pattern — zero degradation.

## Critical Context

### Validation is NOT clean for sqrt
The validation set was used to choose sqrt over linear in the PREVIOUS session.
Therefore sqrt's +7.3% over BM25 on validation is contaminated.
The TRUE test (FEVER, HotpotQA) is needed for the publishable comparison.

### Novelty position
- Sqrt in BM25 is NOT novel (Cummins & O'Riordan, 2009)
- Novel contributions: (1) systematic 250+ variant comparison, (2) anti-correlation finding,
  (3) IDF-conditioned normalization, (4) hinged normalization, (5) robustness analysis

### GPT Pro review integrated
All 7 issues from the GPT Pro review have been addressed or acknowledged:
1. ✅ Held-out contamination → renamed to validation, test set reserved
2. ✅ Tokenizer bug → fixed (proper possessive handling)
3. ✅ Clean one-line replacement → standalone bm199() function
4. ✅ TF-IDF beats BM199 → compared against all baselines honestly
5. ✅ "Optimally tuned BM25" → BM25 b-parameter grid search done
6. ✅ Crossing point math → needs fix in paper (use Codex derivation)
7. ⬜ Incomplete BEIR coverage → FEVER/HotpotQA still not run

## What to Do Next

### Immediate (this session or next)
1. **Run FEVER and HotpotQA** on test set with sqrt(k1=1.5) and BM25 default — this is the clean comparison
2. **Get Codex review** of methodology (was running, may need restart)
3. **Fix crossing-point math** in paper using correct rho derivation
4. **Add per-query significance tests** to eval binary
5. **Run length heterogeneity analysis** — compute CV(doc_lengths) per dataset, correlate with gain

### Medium-term
6. **Download additional BEIR datasets** (MSMARCO, CQADupStack, etc.) for larger test set
7. **Implement leave-one-dataset-out** cross-validation
8. **Compare against Pyserini BM25** for reproducibility
9. **Run clean ablation**: BM25 default → BM25+sqrt(same k1) → BM25+sqrt(tuned k1)
10. **Explore power family r^α more** — is there something between 0.4 and 0.6 that generalizes even better?

### Paper
11. **Finalize paper draft v2** with complete results tables
12. **Target venue:** ECIR short paper or SIGIR resource track
13. **Key story:** Systematic normalization study showing sqrt is uniquely robust

## Current State
- **Branch:** `autoresearch/bm199-optimization` (needs commit with all changes)
- **Uncommitted changes:** Many (new scorer framework, fixed tokenizer, updated docs)
- **Tests passing:** Yes
- **Build status:** Clean release build
- **Background tasks:** Validation sweep may still have 1-2 variants running

## Files Modified This Session
- `src/scorer.rs` — Generic framework + IDF-conditioned norms
- `src/beir.rs` — Fixed tokenizer, updated dataset splits
- `src/index.rs` — search_generic, Bm199 variant
- `src/bin/eval.rs` — Generic scorer support, CLI args
- `src/bin/bench_all.rs` — Rewritten for generic scoring
- `program.md` — Research program v2
- `autoresearch.toml` — Updated
- `paper/drafts/bm199_paper_draft_v2.md` — Complete rewrite
- `paper/data/hypotheses.md` — Hypothesis tracker
- `paper/logs/` — All experimental results
- `scripts/sweep_norms.sh` — Normalization sweep script
