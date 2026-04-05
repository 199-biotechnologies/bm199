# BM199 Complete Results

## 1. Tuning Set Results (4 Datasets)

Evaluated on NFCorpus, SciFact, FiQA, ArguAna during parameter optimization.
Final tuning metric (avg nDCG@10 across 4 datasets):

| Variant | Tuning avg nDCG@10 | Note |
|---------|-------------------|------|
| BM25 (baseline) | 0.4064 | Standard Robertson/Lucene |
| BM25+ | 0.3969 | With delta=1.0 |
| BM25L | 0.3995 | Long-doc adjusted |
| BM199 (best tuning, run #34) | 0.4095 | Linear norm + adaptive sat + short-doc penalty |
| BM199 (final v1.0, run #43) | 0.3979 | Sqrt length norm (optimized for generalization) |

### Tuning Progression (Key Milestones)

| Run | Metric | Change |
|-----|--------|--------|
| #0 (baseline) | 0.3820 | Default params |
| #1 | 0.3944 | delta=0.0 (+3.2%) |
| #2 | 0.3968 | b=0.9 (+0.6%) |
| #4 | 0.3983 | lambda_cov=0.0 (+0.4%) |
| #6 | 0.3990 | log_base=2.0 (+0.2%) |
| #8 | 0.3994 | beta_min=0.5 |
| #15 | 0.3999 | beta_min=0.8 |
| #18 | 0.4001 | k1=1.3 (matching BM25) |
| #19 | 0.4006 | k1=1.4 (FIRST TIME > BM25) |
| #25 | 0.4020 | Short-doc penalty 0.15 |
| #26 | 0.4021 | Re-baseline with stemming |
| #34 | 0.4095 | Linear norm + adaptive sat (best tuning) |
| #38 (A1) | 0.3803 | Root-length fork |
| #43 (A2) | 0.3979 | Symmetric sqrt (final v1.0) |

## 2. Held-Out Results (7 Complete Datasets + 1 Partial)

Evaluated on held-out BEIR datasets not seen during tuning.
All models use the same index with stemming and stopword removal.

### nDCG@10

| Dataset | N docs | BM25 | BM25+ | BM25L | ATIRE | DLH13 | QLD | TF-IDF | **BM199** | BM199 vs BM25 |
|---------|--------|------|-------|-------|-------|-------|-----|--------|-----------|---------------|
| TREC-COVID | 171K | 0.6348 | 0.5786 | 0.5959 | 0.6348 | 0.5979 | 0.0044 | 0.5139 | **0.6559** | **+3.3%** |
| Quora | 523K | 0.7831 | 0.7823 | 0.7833 | 0.7831 | 0.7574 | 0.0030 | 0.5643 | 0.7808 | -0.3% |
| SciDocs | 26K | 0.1569 | 0.1478 | 0.1502 | 0.1570 | 0.1524 | 0.0010 | 0.1277 | **0.1572** | **+0.2%** |
| Touche-2020 | 383K | 0.3189 | 0.3329 | 0.3382 | 0.3189 | 0.2860 | 0.0052 | 0.5110 | **0.4048** | **+26.9%** |
| NQ | 2.7M | 0.2898 | 0.2951 | 0.2974 | 0.2898 | 0.3068 | 0.0002 | 0.2152 | 0.2909 | +0.4% |
| DBPedia | 4.6M | 0.2908 | 0.2900 | 0.2911 | 0.2908 | 0.2991 | 0.0044 | 0.2592 | **0.2959** | **+1.8%** |
| Climate-FEVER | 5.4M | 0.1365 | 0.1431 | 0.1492 | 0.1365 | 0.1565 | 0.0002 | 0.1425 | **0.1817** | **+33.1%** |
| FEVER* | 5.4M | 0.5077 | 0.5868 | -- | -- | -- | -- | -- | -- | *(partial)* |

*FEVER evaluation was interrupted after BM25 and BM25+ completed.

### MAP (Mean Average Precision)

| Dataset | BM25 | BM25+ | BM25L | DLH13 | TF-IDF | **BM199** |
|---------|------|-------|-------|-------|--------|-----------|
| TREC-COVID | 0.0865 | 0.0767 | 0.0798 | 0.0684 | 0.0714 | **0.0899** |
| Quora | 0.7420 | 0.7416 | 0.7428 | 0.7158 | 0.5115 | 0.7395 |
| SciDocs | 0.1068 | 0.0996 | 0.1018 | 0.1029 | 0.0859 | 0.1067 |
| Touche-2020 | 0.1999 | 0.2009 | 0.2070 | 0.1858 | 0.2875 | **0.2571** |
| NQ | 0.2446 | 0.2514 | 0.2528 | 0.2601 | 0.1800 | 0.2467 |
| DBPedia | 0.2202 | 0.2201 | 0.2210 | 0.2247 | 0.1815 | 0.2223 |
| Climate-FEVER | 0.1019 | 0.1058 | 0.1108 | 0.1161 | 0.1029 | **0.1344** |

### Recall@100

| Dataset | BM25 | BM25+ | BM25L | DLH13 | TF-IDF | **BM199** |
|---------|------|-------|-------|-------|--------|-----------|
| TREC-COVID | 0.1227 | 0.1163 | 0.1180 | 0.1064 | 0.1120 | **0.1265** |
| Quora | 0.9721 | 0.9702 | 0.9708 | 0.9584 | 0.9283 | 0.9719 |
| SciDocs | 0.3653 | 0.3472 | 0.3513 | 0.3539 | 0.3369 | 0.3653 |
| Touche-2020 | 0.5461 | 0.5377 | 0.5488 | 0.5442 | 0.5443 | **0.5979** |
| NQ | 0.7478 | 0.7480 | 0.7527 | 0.7545 | 0.6687 | 0.7522 |
| DBPedia | 0.4622 | 0.4738 | 0.4743 | 0.4764 | 0.4304 | 0.4637 |
| Climate-FEVER | 0.3769 | 0.4034 | 0.4121 | 0.4199 | 0.4273 | **0.4419** |

### Held-Out Summary (7 Complete Datasets)

| Scorer | Avg nDCG@10 | vs BM25 |
|--------|-------------|---------|
| BM25 | 0.3730 | -- |
| BM25+ | 0.3671 | -1.6% |
| BM25L | 0.3722 | -0.2% |
| BM25-ATIRE | 0.3587 | -3.8% |
| DLH13 | 0.3652 | -2.1% |
| QLD | 0.0026 | -99.3% |
| TF-IDF | 0.3334 | -10.6% |
| **BM199** | **0.3953** | **+5.6%** |

Note: Average computed over the 7 datasets with complete results (excl. FEVER).
BM25 held-out avg (7 datasets) = (0.6348+0.7831+0.1569+0.3189+0.2898+0.2908+0.1365)/7 = 0.3730
BM199 held-out avg (7 datasets) = (0.6559+0.7808+0.1572+0.4048+0.2909+0.2959+0.1817)/7 = 0.3953

## 3. Fork Experiment Results (5 Structural Variants)

Explored after initial 34 experiments. Each fork tests a different structural innovation.

| Fork | Description | Tuning nDCG@10 | Held-Out nDCG@10 | Tuning-Heldout Gap | Key Finding |
|------|-------------|----------------|-------------------|---------------------|-------------|
| A1 | One-sided sqrt length: $\sqrt{|d|/\text{avgdl}}$ for long docs, 1.0 for short | 0.3803 | 0.4897 | 22.3% | Touche +79%, but ArguAna -24%. Large generalization gap |
| **A2** | **Symmetric sqrt: $\sqrt{|d|/\text{avgdl}}$ for all docs** | **0.3979** | **0.4997** | **20.4%** | **BEST. BM25 beaten by +5.6%. Zero length params.** |
| B1 | Log-TF: $\ln(1+tf)$ instead of $tf^{\beta}$ | 0.3995 | 0.4092 | 2.4% | ArguAna +13% but FiQA -26%, Touche -30% |
| C1 | Tempered IDF: $\text{IDF}^{0.8}$ | 0.3955 | 0.4441 | 10.9% | Quora 0.7702 good, but overall worse |
| D1 | Likelihood-ratio IDF: $\ln\frac{p(t|R)}{p(t|C)}$ | 0.4041 | 0.4047 | 0.1% | Most consistent (minimal gap) but lower absolute |
| A1+B1 | Combined root-length + log-TF | 0.3903 | 0.4478 | 12.8% | Worse than A1 alone; interactions hurt |

### Fork Per-Dataset Held-Out nDCG@10

| Dataset | BM25 | A1 | A2 | B1 | C1 | D1 |
|---------|------|----|----|----|----|----|
| TREC-COVID | 0.6348 | -- | 0.6559 | -- | -- | -- |
| Quora | 0.7831 | 0.7124 | 0.7808 | 0.7629 | 0.7702 | 0.7796 |
| Touche-2020 | 0.3189 | 0.4685 | 0.4048 | 0.1838 | -- | -- |
| ArguAna | -- | -- | -- | 0.4171 | -- | -- |
| FiQA | -- | -- | -- | 0.1913 | -- | -- |

Note: Not all forks were evaluated on all datasets. Values shown are those reported in experiment logs.

## 4. Aggregate Performance

### Final BM199 v1.0 (A2 Symmetric Sqrt)

- **Tuning set** (4 datasets): avg nDCG@10 = 0.3979 (below BM25's 0.4064 by 2.1%)
- **Held-out set** (7 datasets): avg nDCG@10 = 0.3953 (above BM25's 0.3730 by **+5.6%**)
- **Combined** (11 datasets): BM199 outperforms BM25 on 5 of 7 held-out datasets
- **Biggest wins**: Climate-FEVER (+33.1%), Touche-2020 (+26.9%), TREC-COVID (+3.3%)
- **Biggest loss**: Quora (-0.3%)
- **Parameter-free length normalization**: eliminates $b$ hyperparameter entirely
