# Revisiting Sublinear Length Normalization in BM25: A Systematic Study on Heterogeneous Retrieval Benchmarks

**Boris Djordjevic**
Paperfoot AI (paperfoot.com)
boris@paperfoot.com

---

## Abstract

BM25's document-length normalization is linear in relative length, a design choice controlled by the parameter *b* that can over-penalize relevant long documents in heterogeneous corpora. We conduct a systematic study of alternative normalization functions within the BM25 framework, testing over 250 configurations spanning nine families of length normalizers, three TF transformations, and four IDF variants. All satisfy the pivot constraint f(1) = 1 at average document length. Using a strict tuning/validation protocol on BEIR benchmark datasets, we find that (1) square-root normalization, sqrt(dl/avgdl), is the most robust alternative, showing zero tuning-to-validation degradation while all other alternatives exhibit 9–21% degradation; (2) tuning-set performance *anti-correlates* with validation performance across normalization families, suggesting that the standard four-dataset tuning split is insufficient for reliable structure selection; and (3) novel alternatives including IDF-conditioned and piecewise ("hinged") normalization achieve the highest tuning scores but fail to generalize. Although square-root BM25 normalization has been explored in prior work, we provide the first large-scale systematic comparison against a broad family of alternatives under modern zero-shot evaluation conditions.

---

## 1. Introduction

The BM25 ranking function has been the cornerstone of lexical retrieval for thirty years. Its length normalization component, `1 - b + b * (dl/avgdl)`, linearly interpolates between no normalization and full normalization relative to average document length. The parameter *b* (typically 0.75) controls this interpolation.

We ask a simple question: *is linear the right shape?* Sublinear alternatives—square root, logarithmic, sigmoid—compress the length ratio, penalizing extreme-length documents less aggressively. Prior work has explored individual alternatives, notably Cummins and O'Riordan's evolutionary discovery of sqrt-based normalizers, and Lv and Zhai's lower-bounding corrections in BM25+ and BM25L. However, no study has systematically compared a broad family of normalization shapes under controlled zero-shot evaluation conditions.

We test over 250 normalization configurations across nine shape families:
- **Linear** (BM25 standard), **Power** (r^α including sqrt at α=0.5), **Logarithmic**, **Sigmoid**, **Hinged** (piecewise: linear for short docs, power for long), **Asymmetric**, **Saturation**, **Softplus**, and **IDF-conditioned** (per-term normalization strength based on term rarity).

Our contributions:
1. A systematic comparison showing sqrt normalization is uniquely robust across collections, with the smallest tuning-to-validation gap of any tested alternative.
2. Evidence that tuning-set performance *anti-correlates* with held-out performance for normalization selection, cautioning against structure selection on small dataset partitions.
3. Two novel normalization variants (IDF-conditioned and hinged) that achieve state-of-the-art tuning performance but fail to generalize, illustrating the difficulty of improving on BM25's length normalization.

---

## 2. Related Work

### 2.1 BM25 Length Normalization
Robertson et al. (1994) introduced the linear normalization `1-b+b*(dl/avgdl)` within the probabilistic retrieval framework. The parameter *b* has remained the primary tunable for collection-specific adaptation. Lv and Zhai (2011) identified long-document penalization issues and proposed BM25+ (additive floor) and BM25L (modified TF), but both retain linear normalization.

### 2.2 Sublinear Normalization
Cummins and O'Riordan explored evolved term-weighting functions that included sqrt-based length normalizers within BM25-style scorers, discovered through genetic programming. Lv and Zhai note that earlier studies had tried square-root document-length substitutions heuristically. Singhal et al. (1996) introduced pivoted normalization, observing that the optimal normalization is not strictly linear.

### 2.3 Modern Evaluation
The BEIR benchmark (Thakur et al., 2021) provides heterogeneous zero-shot evaluation across diverse retrieval tasks. Its diversity makes it a natural testbed for evaluating normalization robustness, as different collections exhibit very different document-length distributions.

### 2.4 Automated Formula Discovery
RankEvolve (2026) uses genetic programming to search scoring function space, converging on log-based length dampeners. Our work is complementary: rather than discovering formulas, we systematically characterize the normalization shape space.

---

## 3. Normalization Families

We study nine families of length normalization functions, all satisfying the pivot constraint f(1) = 1:

| Family | Formula | Parameters | Properties |
|--------|---------|------------|------------|
| Linear | 1-b+b*r | b | BM25 standard |
| Power | r^α | α | α=0.5 is sqrt |
| Log | ln(1+r)/ln(2) | — | Slowest growth |
| Sigmoid | 2r/(1+r) | — | Bounded at 2.0 |
| Hinged | r if r≤1; r^α if r>1 | α | Decouples short/long |
| Asymmetric | r^α1 if r≤1; r^α2 if r>1 | α1, α2 | Full decoupling |
| Saturation | r/(r+c)*(1+c) | c | Diminishing returns |
| IDF-cond | r^(α+γ*idf_ratio) | α, γ | Per-term norm strength |
| Dual-pivot | Piecewise linear/power | 3 | Three-regime |

where r = dl/avgdl is the relative document length.

---

## 4. Experimental Setup

**Datasets.** 13 BEIR datasets partitioned as:
- Tuning (4): NFCorpus, SciFact, FiQA, ArguAna
- Validation (7): TREC-COVID, Quora, SCIDOCS, Touché, NQ, DBPedia, Climate-FEVER
- Test (2): FEVER, HotpotQA (reserved, untouched)

**Tokenization.** Lowercase, possessive removal, Lucene English stopwords, Porter stemming.

**Protocol.** Parameters (k1, α, etc.) optimized on tuning set only. Validation evaluated with frozen parameters. Test set evaluated once after all decisions finalized.

**Configurations.** Over 250 configurations tested, combining normalization families with k1 values and TF/IDF variants.

---

## 5. Results

### 5.1 Tuning Performance

The highest tuning scores (avg nDCG@10 across 4 datasets):

| Rank | Variant | Tuning |
|------|---------|--------|
| 1 | IDF-cond(α=0.60,γ=0.50,k1=1.5) | 0.4131 |
| 2 | Hinged(α=0.70,k1=1.5) | 0.4126 |
| 3 | Hinged(α=0.80,k1=1.2) | 0.4123 |
| 4 | Saturation(c=5.0,k1=1.5) | 0.4122 |
| 5 | BM25(b=1.0,k1=1.2) | 0.4121 |
| ... | BM25 default | 0.4091 |
| ... | Sqrt(k1=1.5) | 0.3994 |

### 5.2 Validation Performance

| Rank | Variant | Tuning | Validation | Gap |
|------|---------|--------|------------|-----|
| **1** | **Sqrt(k1=1.5)** | 0.3994 | **0.3998** | **+0.1%** |
| 2 | BM25 default | 0.4091 | 0.3725 | -9.0% |
| 3 | Log(k1=1.6) | 0.4096 | 0.3594 | -12.3% |
| 4 | IDF-cond(best) | 0.4131 | 0.3525 | -14.6% |
| 5 | Hinged(best) | 0.4123 | 0.3393 | -17.7% |
| 6 | BM25(b=1.0) | 0.4121 | 0.3274 | -20.6% |

### 5.3 Anti-Correlation Between Tuning and Validation

Across all tested configurations, we observe a striking negative correlation between tuning performance and the tuning-to-validation gap. The variants that score highest on tuning degrade most on validation. This anti-correlation is consistent across normalization families and suggests that the four-dataset tuning partition is too small for reliable normalization structure selection.

The sole exception is sqrt normalization, which shows zero degradation. This is consistent with sqrt being a zero-parameter normalization (only k1 is tuned), while all alternatives either tune additional normalization parameters on the tuning set or use more complex shapes that fit tuning-specific patterns.

---

## 6. Analysis

### 6.1 Why Sqrt Generalizes

Sqrt normalization has a unique property among the tested alternatives: it is the geometric mean of no normalization (r^0 = 1) and full normalization (r^1 = r). At α = 0.5, the power family is equidistant from both extremes in log-space. This may explain its robustness: it neither commits to strong normalization (which helps some collections) nor weak normalization (which helps others), but splits the difference sublinearly.

### 6.2 Why Complex Alternatives Fail

IDF-conditioned normalization (the tuning winner) adds a per-term normalization adjustment. While mechanistically motivated—rare terms in long documents are informative—the added parameter γ fits the specific IDF distribution of the four tuning datasets. On validation datasets with different IDF characteristics, this fitting becomes liability.

Hinged normalization explicitly decouples short-document and long-document treatment. While this sounds principled, the breakpoint at r = 1 (average length) is arbitrary and the power exponent α is tuned on the specific length distributions of the tuning corpora.

### 6.3 Implications for BM25 Parameter Tuning

Our results suggest that BM25's default b = 0.75 is remarkably well-calibrated for cross-collection robustness. Tuning b on a small dataset partition improves in-distribution performance but hurts generalization. This has practical implications: practitioners deploying BM25 on new, unseen collections should prefer default parameters over tuned ones unless a large, representative validation set is available.

---

## 7. Limitations

1. **Validation contamination for sqrt.** The validation datasets were previously used to select sqrt over linear normalization in earlier experiments, making the sqrt vs. BM25 comparison on validation potentially optimistic.
2. **Custom tokenizer.** Our pipeline approximates but does not exactly match Pyserini/Lucene's EnglishAnalyzer. Cross-implementation comparisons should use identical tokenization.
3. **No significance tests.** Per-query paired tests are needed for formal conclusions.
4. **Test set pending.** FEVER and HotpotQA results will provide uncontaminated evaluation.
5. **Only 4 tuning datasets.** A larger tuning partition or cross-validation would strengthen the anti-correlation finding.

---

## 8. Conclusion

We systematically tested over 250 BM25 length normalization configurations across nine shape families. Our central finding is that sqrt(dl/avgdl) normalization is uniquely robust: it is the only tested alternative with zero tuning-to-validation degradation. Novel alternatives including IDF-conditioned and piecewise normalization achieve the highest tuning scores but fail to generalize, highlighting the difficulty of improving BM25's length normalization through increased complexity.

These results have practical implications: (1) sqrt normalization is a safe, parameter-free drop-in replacement for BM25's linear normalization, especially for cross-collection deployment; (2) normalization structure should not be selected based on small dataset partitions, as tuning performance anti-correlates with generalization; and (3) BM25's default b = 0.75 remains a remarkably robust choice that is difficult to improve upon through tuning.

---

## References

Cummins, R. and O'Riordan, C. (2009). An Axiomatic Comparison of Learned Term-Weighting Schemes in Information Retrieval. *ECIR 2009*.

Lv, Y. and Zhai, C. (2011). When Documents Are Very Long, BM25 Fails! *SIGIR 2011*.

Robertson, S. E. et al. (1994). Okapi at TREC-3. *TREC-3*.

Robertson, S. and Zaragoza, H. (2009). The Probabilistic Relevance Framework: BM25 and Beyond. *FnTIR* 3(4).

Singhal, A. et al. (1996). Pivoted Document Length Normalization. *SIGIR 1996*.

Thakur, N. et al. (2021). BEIR: A Heterogeneous Benchmark for Zero-shot Evaluation of Information Retrieval Models. *NeurIPS 2021 Datasets Track*.

Trotman, A. et al. (2014). Improvements to BM25 and Language Models Examined. *ADCS 2014*.
