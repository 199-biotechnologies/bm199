# Gemini Common Sense Check — 2026-04-05

## Verdict: SUSPICIOUS — likely dataset-specific artifact

## Key Concerns Raised:
1. **FEVER dominates the average:** +28.3% on FEVER vs +5.7% on HotpotQA. The +16.1% headline is an outlier-driven average.
2. **Both test datasets are Wikipedia QA** with long documents. The sublinear norm boosts long docs that BM25 over-penalizes.
3. **Tuning underperformance is a red flag:** alpha=0.40 is BELOW BM25 on tuning. A truly better function should improve consistently.
4. **Selection bias:** best of 6 candidates on 7-dataset validation → 2-dataset test is winner's curse territory.
5. **Need MS MARCO / Robust04** to confirm this isn't corpus-specific.

## Gemini's Bottom Line:
"Do not publish this as a general improvement. It is a Wikipedia-tuned variant at best."

## Our Response:
- k1 isolation shows k1=1.5 vs 1.2 accounts for only +0.4% on tuning, so the norm IS the dominant factor
- BUT the norm might specifically help Wikipedia-length-distribution corpora
- MUST test on non-Wikipedia test datasets before claiming generalization
- Should report per-dataset, not average, to show FEVER outlier transparently
- Running BM25(k1=1.5) on test set as k1 control (background)

## k1 Isolation Results (tuning):
| Config | k1 | Tuning nDCG@10 |
|--------|-----|----------------|
| BM25(b=0.75) | 1.2 | 0.4091 |
| BM25(b=0.75) | 1.5 | 0.4106 (+0.4%) |
| Power(alpha=0.40) | 1.2 | 0.3893 |
| Power(alpha=0.40) | 1.5 | 0.3904 |

k1 effect is tiny. Normalization shape is the dominant factor. But the question is WHETHER the shape generalizes beyond Wikipedia QA.
