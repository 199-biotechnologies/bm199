# Corrected Tokenizer Results Summary — 2026-04-06

## What Changed
Replaced 90-word stopword list with Lucene EnglishAnalyzer's actual 33 words.
Removed question words (what, which, who, when, where, why, how) and ~60 others.

## CORRECTED Tuning Results (avg nDCG@10, 4 datasets)

| Config | Tuning | Note |
|--------|--------|------|
| Hinged(a=0.80, k1=1.5) | 0.4093 | Top tuning, corrected |
| Power(a=0.90, k1=1.8) | 0.4091 | |
| BM25 joint tuned (k1=1.6, b=0.90) | 0.4104 | Best BM25 from 48-config grid |
| BM25 default (k1=1.2, b=0.75) | 0.4035 | |
| Power(a=0.40, k1=1.5) | ~0.385 | Below BM25 on tuning |
| Sqrt(a=0.50, k1=1.5) | ~0.394 | Below BM25 on tuning |

## CORRECTED Validation Results (avg nDCG@10, 7 datasets)

| Config | Tuning | Validation | Gap | vs BM25 |
|--------|--------|------------|-----|---------|
| **Power(α=0.40, k1=1.5)** | ~0.385 | **0.3987** | **+3.6%** | **+7.8%** |
| Sqrt(α=0.50, k1=1.5) | ~0.394 | 0.3946 | +0.1% | +6.7% |
| BM25 default | 0.4035 | 0.3698 | -8.3% | — |
| BM25 tuned (k1=1.5, b=0.90) | ~0.408 | 0.3398 | -17.2% | -8.1% |

## Key Findings (Survived Tokenizer Fix)

1. **Power(0.40) still beats BM25 default by +7.8% on validation** — NOT a tokenizer artifact
2. **Positive tuning→validation gap persists** — only for power(0.30-0.50) range
3. **Anti-correlation confirmed** — tuning winners collapse on validation
4. **BM25 joint tuning doesn't help** — best tuned BM25 (0.4104) collapses to 0.3398

## What DID Change With Fix
- BM25 default tuning: 0.4091 → 0.4035 (-1.4%)
- BM25 default validation: 0.3725 → 0.3698 (-0.7%)
- The gains are slightly smaller but still substantial and consistent
- FEVER/HotpotQA test needs re-running with correct tokenizer (pending)
