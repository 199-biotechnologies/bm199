# BM25 Length Normalization Study

**Can we beat BM25 by changing one line?**

A systematic study of 250+ length normalization alternatives within the BM25 scoring framework, evaluated on 13 BEIR benchmark datasets.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024_edition-orange.svg)](https://www.rust-lang.org/)

## Key Finding

Replacing BM25's linear length normalization with a sublinear power function improves retrieval quality on heterogeneous corpora:

```
// BM25 (standard)
norm = 1.0 - b + b * (dl / avgdl)           // 2 params: k1, b

// Power normalization (this study)
norm = (dl / avgdl).powf(0.40)              // 1 param: k1 only
```

On an uncontaminated test set (FEVER + HotpotQA):

| Scorer | FEVER | HotpotQA | Average |
|--------|-------|----------|---------|
| BM25 (k1=1.2, b=0.75) | 0.503 | 0.589 | 0.546 |
| **Power α=0.40 (k1=1.5)** | **0.646** | **0.623** | **0.634** |

> **Caveat**: Both test datasets are Wikipedia-based QA. The gain is asymmetric (+28% FEVER, +6% HotpotQA). Additional diverse test datasets are needed before claiming general improvement. See [Limitations](#limitations).

## What We Tested

13 normalization families, each satisfying f(1) = 1 at average document length:

| Family | Formula | Params | Best Validation |
|--------|---------|--------|-----------------|
| Linear | 1-b+b·r | b | 0.372 (default) |
| **Power** | **r^α** | **α** | **0.404 (α=0.40)** |
| Log | ln(1+r)/ln(2) | — | 0.359 |
| Sigmoid | 2r/(1+r) | — | 0.393 |
| Hinged | r if r≤1, r^α if r>1 | α | 0.339 |
| Saturation | r/(r+c)·(1+c) | c | 0.343 |
| IDF-conditioned | r^(α-γ·idf_ratio) | α, γ | 0.353 |
| Softplus | ln(1+e^(r-1))/ln(2) | — | 0.406 |
| + 5 more | See paper | — | — |

250+ configurations tested across normalization type × k1 × TF mode × IDF mode.

## Anti-Correlation Discovery

Tuning performance **negatively correlates** with validation performance. Configs that win on tuning collapse on held-out data:

| Config | Tuning | Validation | Gap |
|--------|--------|------------|-----|
| IDF-cond (tuning winner) | **0.413** | 0.353 | -14.6% |
| BM25 tuned (b=1.0) | 0.412 | 0.327 | -20.6% |
| BM25 default | 0.409 | 0.372 | -9.0% |
| **Power α=0.40** | 0.390 | **0.404** | **+3.6%** |

## Quick Start

```bash
# Build
cargo build --release

# Run BM25 baseline on tuning set
cargo run --release --bin bm199-eval -- --split tuning --variant bm25

# Run power(α=0.40) on tuning set
cargo run --release --bin bm199-bench-all -- --norm power --alpha 0.40 --k1 1.5

# Run any normalization on any split
cargo run --release --bin bm199-eval -- \
  --split validation \
  --scorer generic \
  --norm power --alpha 0.40 --k1 1.5

# Full sweep (250+ configs, ~15 min)
bash scripts/sweep_norms.sh
```

### Available Normalizations

```
--norm linear    --b 0.75           # BM25 standard
--norm power     --alpha 0.50       # sqrt (α=0.5)
--norm power     --alpha 0.40       # best performer
--norm log                          # zero-parameter log
--norm sigmoid                      # bounded at 2.0
--norm hinged    --alpha 0.60       # piecewise
--norm saturation --c 5.0           # diminishing returns
--norm idfcond   --alpha 0.6 --gamma 0.3   # per-term norm
--norm softplus                     # smooth
--norm bidir     --c 0.06           # bidirectional penalty
--norm relog      --c 0.15          # RankEvolve-inspired
```

### TF and IDF Variants

```
--tf standard    # raw tf (default)
--tf log         # ln(1+tf)
--tf dlog        # ln(1+ln(1+tf))
--tf capped --tf-cap 5  # min(tf, 5)

--idf standard   # Lucene: ln((N-df+0.5)/(df+0.5)+1)
--idf atire      # ln(N/df)
--idf squared    # IDF²
--idf smoothed   # ln((N+1)/(df+1))
```

## Dataset Protocol

```
Tuning (4):     NFCorpus, SciFact, FiQA, ArguAna
Validation (7): TREC-COVID, Quora, SciDocs, Touché, NQ, DBPedia, Climate-FEVER
Test (2):       FEVER, HotpotQA
```

- Parameters tuned ONLY on tuning set
- Normalization structure selected on validation
- Test set evaluated ONCE with frozen everything

## Limitations

1. **Custom tokenizer** — approximates but does not match Lucene EnglishAnalyzer. Our BM25 baseline diverges from Pyserini's by up to 29% on some datasets (Touché: 0.316 vs 0.442).
2. **2-dataset test set** — both Wikipedia QA. Not representative of diverse retrieval tasks.
3. **FEVER dominates the average** — +28% on FEVER vs +6% on HotpotQA. The headline gain is outlier-driven.
4. **No significance tests** — per-query paired tests are needed.
5. **Prior art** — sqrt(dl/avgdl) in BM25 was published by Cummins & O'Riordan (2009). The power family and systematic comparison are our contribution.

## Project Structure

```
src/
  scorer.rs          # All scoring functions + generic framework
  beir.rs            # Tokenizer + BEIR dataset loader
  index.rs           # Inverted index + search methods
  eval.rs            # nDCG@10, MAP, Recall@100, MRR
  bin/eval.rs        # Full evaluation binary
  bin/bench_all.rs   # Single-metric output for autoresearch
scripts/
  sweep_norms.sh     # Run full 250+ config sweep
paper/
  drafts/            # Paper drafts
  logs/              # All experimental results (CSV, logs)
  data/              # Hypothesis tracker, formulas
```

## Citation

```bibtex
@article{djordjevic2026bm25norm,
  title={Revisiting Sublinear Length Normalization in {BM25}: 
         A Systematic Study on Heterogeneous Retrieval Benchmarks},
  author={Djordjevic, Boris},
  year={2026},
  note={Paperfoot AI. Code: github.com/199-biotechnologies/bm199}
}
```

## Author

**Boris Djordjevic** — [Paperfoot AI](https://paperfoot.com) — [@longevityboris](https://x.com/longevityboris)

## License

MIT
