# Pyserini Official BM25 vs Our BM25 (nDCG@10)

Source: Kamalloo et al. (2024) "Resources for Brewing BEIR" + Pyserini regressions page

| Dataset | Ours | Pyserini(flat) | Pyserini(MF) | Gap vs flat |
|---------|------|----------------|--------------|-------------|
| TREC-COVID | 0.634 | 0.595 | 0.656 | +6.6% |
| Quora | 0.785 | 0.789 | 0.789 | -0.5% |
| SciDocs | 0.157 | 0.149 | 0.158 | +5.4% |
| Touché | 0.316 | 0.442 | 0.442 | **-28.5%** |
| NQ | 0.290 | 0.306 | 0.329 | -5.2% |
| DBPedia | 0.289 | 0.318 | 0.313 | -9.1% |
| Climate-FEVER | 0.137 | 0.165 | 0.213 | -17.0% |
| FEVER | 0.503 | 0.651 | 0.753 | **-22.7%** |
| HotpotQA | 0.589 | 0.633 | 0.603 | -6.9% |
| NFCorpus | 0.327 | 0.322 | 0.325 | +1.6% |
| SciFact | 0.684 | 0.679 | 0.665 | +0.7% |
| FiQA | 0.256 | 0.236 | 0.254 | +8.5% |
| ArguAna | 0.370 | 0.441 | 0.441 | **-16.1%** |

## Critical Observation
Our BM25 FEVER score (0.503) is 23% below Pyserini flat (0.651) and 33% below Pyserini MF (0.753).
Our power(0.40) FEVER score (0.646) is CLOSE TO Pyserini flat BM25 (0.651).

This means: **our "improvement" on FEVER may just be recovering what Pyserini's better tokenizer already gets with standard BM25.**

## k1 Control Test (uncontaminated test set)
| Scorer | FEVER | HotpotQA | Average |
|--------|-------|----------|---------|
| BM25 (k1=1.2, b=0.75) | 0.503 | 0.589 | 0.546 |
| BM25 (k1=1.5, b=0.75) | 0.482 | 0.573 | 0.527 (-3.5%) |
| Sqrt α=0.50 (k1=1.5) | 0.620 | 0.615 | 0.617 (+13.0%) |
| Power α=0.40 (k1=1.5) | 0.646 | 0.623 | 0.634 (+16.1%) |

k1=1.5 HURTS BM25 by -3.5%. The gain is entirely from the normalization shape.
But our power(0.40) FEVER=0.646 ≈ Pyserini flat BM25 FEVER=0.651.
