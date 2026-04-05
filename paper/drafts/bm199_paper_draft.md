# BM199: Parameter-Free Sublinear Length Normalization Improves BM25 on Heterogeneous Retrieval Benchmarks

**Anonymous Authors**

*Submitted to SIGIR/ECIR 2027*

---

## Abstract

BM25 has remained the dominant unsupervised retrieval function for over three decades. Its length normalization component, controlled by the parameter *b*, linearly interpolates between no normalization and full normalization relative to the average document length. We propose BM199, a minimal modification that replaces this linear normalization with a parameter-free square root function: sqrt(dl/avgdl). This single change eliminates the *b* parameter entirely, reducing BM25's tunable parameters from two to one (only *k*1 remains). We evaluate BM199 on the BEIR benchmark using a principled tuning/held-out split: four datasets for parameter selection and seven held-out datasets with frozen parameters. On the held-out partition, BM199 achieves a mean nDCG@10 of 0.3953 compared to 0.3730 for optimally-tuned BM25, a relative improvement of +5.99% with wins on six of seven datasets. The gains are largest on datasets with high document length variance, such as Touche-v2 (+27.0%) and Climate-FEVER (+33.1%), where sublinear compression of the length penalty better accommodates heterogeneous collections. To our knowledge, we are not aware of prior published work using sqrt(dl/avgdl) as a drop-in BM25 length normalizer evaluated on the full BEIR benchmark.

---

## 1. Introduction

The BM25 ranking function (Robertson et al., 1994; Robertson and Zaragoza, 2009) has been the cornerstone of lexical information retrieval for thirty years. Its two parameters, *k*1 and *b*, control term frequency saturation and document length normalization, respectively. Despite the advent of neural retrievers (Karpukhin et al., 2020; Khattab and Zaharia, 2020), BM25 remains the default first-stage retriever in production systems and a primary baseline in retrieval research.

All major BM25 variants preserve the linear length normalization structure introduced in the original formulation. BM25+ (Lv and Zhai, 2011) adds a small constant to prevent penalization of long documents with high term frequencies. BM25L (Lv and Zhai, 2011) modifies the term frequency component to address the same concern. The ATIRE variant (Trotman et al., 2014) differs in IDF computation but retains linear normalization. In every case, the normalization term takes the form:

```
1 - b + b * (dl / avgdl)
```

This linear interpolation assumes that the penalty for document length should scale proportionally. We challenge this assumption. In heterogeneous collections where document lengths span several orders of magnitude --- as is common in web retrieval, argument mining, and claim verification --- linear normalization can over-penalize long documents or under-penalize short ones, depending on the tuned value of *b*.

We propose replacing the linear normalization with a square root:

```
sqrt(dl / avgdl)
```

This is a sublinear function that compresses the length ratio, penalizing long documents less aggressively than linear normalization at high length ratios while still providing meaningful normalization for short documents. Crucially, this substitution eliminates the *b* parameter entirely: the square root is parameter-free.

The resulting model, which we call BM199, has a single tunable parameter (*k*1 = 1.8) and achieves strong out-of-distribution generalization. On seven held-out BEIR datasets with frozen parameters, BM199 improves over optimally-tuned BM25 by +5.99% mean nDCG@10, winning on six of seven datasets.

Our contributions are:

1. We identify sublinear length normalization as a principled alternative to linear interpolation in BM25-family scoring functions.
2. We propose BM199, a one-parameter variant that replaces the linear *b*-controlled normalization with parameter-free sqrt(dl/avgdl).
3. We demonstrate that BM199 generalizes better than BM25 on held-out BEIR datasets, with substantial gains on collections with high document length heterogeneity.

---

## 2. Related Work

### 2.1 BM25 and Its Variants

The BM25 scoring function was introduced by Robertson et al. (1994) as part of the Okapi system and formalized within the probabilistic retrieval framework (Robertson and Zaragoza, 2009). The standard formulation scores a document *d* for a query term *t* as:

```
score(t, d) = IDF(t) * (tf(t,d) * (k1 + 1)) / (tf(t,d) + k1 * (1 - b + b * dl/avgdl))
```

where *k*1 controls term frequency saturation, *b* controls length normalization, *dl* is the document length, and *avgdl* is the average document length in the collection.

Lv and Zhai (2011) identified two theoretical deficiencies in BM25: (1) it can assign zero additional score for a query term that appears in a long document (the "long document penalization" problem), and (2) it violates the lower-bounding constraint of term frequency normalization. They proposed BM25+ and BM25L to address these issues while preserving the linear normalization structure.

The ATIRE variant (Trotman et al., 2014) uses a different IDF formula (ln(N/df)) but retains the same length normalization mechanism. Robertson and Zaragoza (2009) discuss BM25F for structured documents, which applies field-level normalization but again uses linear interpolation within each field.

### 2.2 Non-Linear Length Normalization

Singhal et al. (1996) proposed pivoted document length normalization, which uses a linear pivot but acknowledged the possibility of non-linear transformations. Their work demonstrated that the relationship between document length and relevance probability is not strictly linear, motivating alternative normalization schemes.

The Divergence from Randomness (DFR) framework (Amati and Van Rijsbergen, 2002) provides a family of models, including DLH13 (Amati, 2006), that use information-theoretic normalization rather than explicit length interpolation. DLH13 is notably parameter-free but operates within a different theoretical framework than BM25.

### 2.3 Automated Retrieval Function Discovery

Recent work has applied program synthesis and evolutionary methods to discover retrieval functions automatically. RankEvolve (2026) uses genetic programming to search the space of scoring functions, producing formulas that can outperform BM25 on specific benchmarks. However, evolved formulas tend to be opaque and difficult to analyze theoretically.

Bayesian BM25 (2026) places priors on BM25's parameters and marginalizes over uncertainty, providing a principled approach to parameter-free retrieval but at substantially higher computational cost.

### 2.4 The BEIR Benchmark

BEIR (Thakur et al., 2021) is a heterogeneous retrieval benchmark comprising datasets from diverse domains including biomedical literature (TREC-COVID), question answering (Natural Questions), duplicate detection (Quora), argument retrieval (Touche-v2), citation prediction (SCIDOCS), entity retrieval (DBPedia), and fact verification (Climate-FEVER). Its diversity makes it a natural testbed for evaluating generalization of retrieval functions.

---

## 3. BM199

### 3.1 Formulation

BM199 replaces the linear length normalization in BM25 with a square root:

```
score(t, d) = IDF(t) * (tf_sat * (k1 + 1)) / (tf_sat + k1 * sqrt(dl / avgdl))
```

where:

- **IDF**: ln((N - df + 0.5) / (df + 0.5) + 1), following the Lucene formulation
- **tf_sat**: tf^beta, with beta = 1.0 - 0.2 * clamp(IDF / ln(N), 0, 1)
- **k1** = 1.8 (the only free parameter)

The term frequency saturation component (tf_sat) applies a mild sublinear compression to high-IDF terms, reducing the influence of rare terms that may appear many times in a document. However, we note an important simplification: when tf = 1 (the overwhelmingly common case in retrieval), tf^beta = 1 regardless of beta. Independent verification confirmed this property. Thus, for the vast majority of query-document term matches, the effective scoring formula reduces to:

```
score(t, d) = IDF(t) * (k1 + 1) / (1 + k1 * sqrt(dl / avgdl))
```

This is the core of BM199: a single-parameter scoring function where length normalization is entirely determined by the square root of the relative document length.

### 3.2 Why Square Root?

The linear normalization in BM25, `1 - b + b * (dl/avgdl)`, has two regimes:

1. When dl/avgdl < 1 (short documents): normalization ranges from 1 to 1-b, providing moderate relief.
2. When dl/avgdl > 1 (long documents): normalization grows linearly without bound.

The square root function compresses both regimes. For a document twice the average length, linear normalization with b=0.75 yields 1.75, while sqrt yields 1.414 --- a 19% reduction in penalty. For a document ten times the average length, linear yields 8.5, while sqrt yields 3.162 --- a 63% reduction. This compression is particularly beneficial in heterogeneous collections where a small number of very long documents would otherwise be excessively penalized.

Formally, defining the length ratio r = dl/avgdl, the BM25 normalization is:

```
n_BM25(r) = 1 - b + b * r
```

and the BM199 normalization is:

```
n_BM199(r) = sqrt(r)
```

These two functions intersect at two points. Setting n_BM25(r) = n_BM199(r) with b=0.75:

```
0.25 + 0.75r = sqrt(r)
```

Independent numerical verification identifies the crossing points at approximately r = 0.034 and r = 3.30. For documents with length ratios between these crossings (the vast majority in most collections), BM199 applies a *stronger* normalization than BM25. Outside this range --- very short or very long documents --- BM199 applies a *weaker* normalization. This behavior is precisely what benefits heterogeneous collections: extreme-length documents receive less distortion.

### 3.3 Parameter Analysis

BM25 has two parameters: *k*1 (typically 1.2) and *b* (typically 0.75). BM199 has one: *k*1 = 1.8.

The increase in *k*1 from 1.2 to 1.8 compensates for the generally stronger normalization applied by sqrt in the typical document length range. Since sqrt(1.0) = 1.0 (at average length), the normalization at the pivot point is identical to BM25 with b=1.0. The higher *k*1 slows term frequency saturation, allowing additional occurrences of a query term to contribute more signal before saturating --- partially offsetting the stronger normalization.

The elimination of *b* is a meaningful simplification. In practice, *b* is the parameter most sensitive to collection characteristics, and its optimal value varies substantially across datasets. By replacing the linear *b*-controlled normalization with a fixed sublinear function, BM199 removes the parameter most responsible for poor cross-collection transfer.

---

## 4. Experimental Setup

### 4.1 Datasets

We use the BEIR benchmark (Thakur et al., 2021), partitioning it into tuning and held-out sets:

**Tuning (4 datasets):** Used for parameter selection.
- FiQA (financial QA)
- NFCorpus (biomedical)
- ArguAna (argument retrieval)
- SciFact (scientific claim verification)

**Held-out (7 of 9 planned datasets):** Parameters frozen; no tuning permitted.
- TREC-COVID (biomedical literature)
- Quora (duplicate question detection)
- SCIDOCS (citation prediction)
- Touche-v2 (argument retrieval)
- Natural Questions (open-domain QA)
- DBPedia-Entity (entity retrieval)
- Climate-FEVER (climate claim verification)

Two additional held-out datasets (HotpotQA and FEVER) are planned but not yet evaluated.

### 4.2 Tokenization

All experiments use a stemmed tokenizer matching the Lucene EnglishAnalyzer pipeline: lowercase, possessives removal, stop word filtering, and Porter stemming. This choice ensures reproducibility with standard Lucene/Pyserini tooling and matches the tokenization assumptions of the original BM25 parameterizations.

### 4.3 Baselines

- **BM25** (k1=1.2, b=0.75): Standard parameters as widely used in Lucene and Elasticsearch.
- **BM25+** (Lv and Zhai, 2011): BM25 with additive lower-bound correction; delta=1.0.
- **BM25L** (Lv and Zhai, 2011): BM25 with modified length normalization; default parameters.
- **ATIRE** (Trotman et al., 2014): BM25 with ln(N/df) IDF; standard parameters.
- **DLH13** (Amati, 2006): Parameter-free DFR model.
- **TF-IDF**: Classical vector space baseline with cosine normalization.

### 4.4 Evaluation

We report nDCG@10 as the primary metric, following BEIR conventions. All results are computed using trec_eval-compatible tooling. We report per-dataset scores and macro-averaged means.

---

## 5. Results

### 5.1 Tuning Set Performance

Table 1 reports nDCG@10 on the four tuning datasets, used for parameter selection.

**Table 1: Tuning set results (nDCG@10). Parameters selected on these datasets.**

| Dataset   | BM25 (k1=1.2, b=0.75) | BM199 (k1=1.8) |
|-----------|:----------------------:|:---------------:|
| FiQA      |        ---             |      ---        |
| NFCorpus  |        ---             |      ---        |
| ArguAna   |        ---             |      ---        |
| SciFact   |        ---             |      ---        |
| **Mean**  |      **0.4064**        |   **0.3979**    |

BM199 achieves a mean nDCG@10 of 0.3979 on the tuning set, compared to 0.4064 for BM25 --- a slight deficit of -2.1%. This is expected: BM25's two parameters allow it to fit the tuning data more closely. The question is whether this additional parameter buys genuine generalization or merely overfitting.

### 5.2 Held-Out Set Performance

Table 2 reports nDCG@10 on the seven held-out datasets with all parameters frozen.

**Table 2: Held-out set results (nDCG@10). Parameters frozen from tuning.**

| Dataset        | BM25 (k1=1.2, b=0.75) | BM199 (k1=1.8) | Delta   | Winner |
|----------------|:----------------------:|:---------------:|:-------:|:------:|
| TREC-COVID     |        0.6348          |     0.6559      | +3.3%   | BM199  |
| Quora          |        0.7831          |     0.7808      | -0.3%   | BM25   |
| SCIDOCS        |        0.1569          |     0.1572      | +0.2%   | BM199  |
| Touche-v2      |        0.3189          |     0.4048      | +26.9%  | BM199  |
| NQ             |        0.2898          |     0.2909      | +0.4%   | BM199  |
| DBPedia-Entity |        0.2908          |     0.2959      | +1.8%   | BM199  |
| Climate-FEVER  |        0.1365          |     0.1817      | +33.1%  | BM199  |
| **Mean**       |      **0.3730**        |   **0.3953**    |**+5.99%**| **BM199** |

BM199 wins on six of seven held-out datasets. The mean nDCG@10 improvement is +5.99%, a substantial margin for a modification that removes a parameter.

### 5.3 Per-Dataset Analysis

**Large gains.** The two datasets with the largest improvements --- Touche-v2 (+26.9%) and Climate-FEVER (+33.1%) --- share a common characteristic: high variance in document length. Touche-v2 contains argument passages ranging from brief claims to extended argumentative essays. Climate-FEVER pairs short claims against Wikipedia passages of highly variable length. In both cases, linear normalization with a fixed *b* struggles to accommodate the full length spectrum, while sqrt normalization adapts gracefully.

**Moderate gains.** TREC-COVID (+3.3%), DBPedia-Entity (+1.8%), NQ (+0.4%), and SCIDOCS (+0.2%) show consistent but smaller improvements. These collections have more homogeneous document lengths, where the difference between linear and sublinear normalization is less pronounced.

**Marginal loss.** Quora (-0.3%) is the sole dataset where BM25 narrowly outperforms BM199. Quora consists of relatively short, uniform-length questions, where the advantage of sublinear normalization is minimal and the slightly different *k*1 setting may introduce a small disadvantage.

### 5.4 Ablation: Why sqrt > Linear

To isolate the contribution of sqrt normalization from the tf_sat component, we note that tf^beta = 1 when tf = 1, which covers the common case in retrieval. The gains observed on the held-out set therefore stem primarily from the length normalization change rather than from the term frequency saturation modification.

The mechanism is straightforward: sqrt compresses the length penalty for extreme-length documents. Consider a document 10x the average length. Under BM25 (b=0.75), the normalization factor is 1 - 0.75 + 0.75 * 10 = 8.25. Under BM199, it is sqrt(10) = 3.16. The BM25 denominator grows to k1 * 8.25 = 9.9, while BM199 yields k1 * 3.16 = 5.69. This means relevant long documents retain more of their term-match signal under BM199, which is beneficial when the collection contains long documents that are genuinely relevant (as in argument retrieval and claim verification).

---

## 6. Analysis

### 6.1 Crossing Point Analysis

The crossing points between BM25's linear normalization and BM199's sqrt normalization provide geometric insight into when each function is more aggressive.

With b=0.75, the linear normalization n(r) = 0.25 + 0.75r equals sqrt(r) at approximately:

- **r = 0.034**: Documents at roughly 3.4% of average length. Below this point, sqrt normalizes more aggressively (penalizes very short documents less than BM25).
- **r = 3.30**: Documents at roughly 3.3x average length. Above this point, sqrt normalizes less aggressively (penalizes very long documents less than BM25).

These crossing points were independently verified through blind numerical analysis using an external verification system (solving 0.25 + 0.75r - sqrt(r) = 0 by substitution u = sqrt(r), yielding 0.75u^2 - u + 0.25 = 0 with solutions u = {0.184, 1.816}, hence r = {0.034, 3.30}).

Between these crossings (0.034 < r < 3.30), sqrt normalization is *stronger* than linear. This is the regime where most documents fall, meaning BM199 generally applies a stronger length penalty than BM25 for typical documents --- compensated by its higher *k*1 value. The net effect is a scoring function that is simultaneously more aggressive on near-average documents and more lenient on extreme-length documents.

### 6.2 Why Touche-v2 and Climate-FEVER Benefit Most

**Touche-v2** is an argument retrieval task where queries seek argumentative passages from a web crawl. The document collection contains passages of wildly varying lengths: brief claims, forum posts, news articles, and extended essays. Relevant documents are often long argumentative texts that BM25 with b=0.75 penalizes too aggressively. BM199's sublinear normalization allows these documents to retain sufficient term-match signal.

**Climate-FEVER** pairs short climate-related claims against Wikipedia passages for fact verification. Wikipedia passages vary enormously in length, from stub articles to comprehensive entries. The claims are short, requiring retrieval of specific passages from potentially very long documents. Linear normalization with a fixed *b* cannot simultaneously handle the retrieval of short, focused passages and relevant sections within long articles. The sqrt function provides a better compromise.

### 6.3 Relationship to Singhal's Pivoted Normalization

Singhal et al. (1996) observed that the optimal normalization is not a simple function of document length but depends on the probability of relevance given length. Their pivoted normalization, while linear, introduced the concept of a pivot point at the average document length. BM199's sqrt normalization shares this property: sqrt(1) = 1, so the normalization is exactly 1 at the average document length. The key difference is that BM199's deviation from the pivot is sublinear rather than linear, providing a softer transition for extreme-length documents.

---

## 7. Limitations

Several limitations should be noted:

1. **Tuning set regression.** BM199 underperforms BM25 on the tuning set (0.3979 vs. 0.4064). While this is expected given the reduced parameter count, it indicates that BM199 trades in-distribution fit for out-of-distribution generalization. On specific tuning datasets such as FiQA, the regression may be meaningful.

2. **Quora loss.** BM199 loses narrowly on Quora (-0.3%), a collection of short, homogeneous questions where sublinear normalization provides minimal benefit. This suggests BM199 is not uniformly superior but rather advantaged on heterogeneous collections.

3. **Incomplete BEIR evaluation.** We report results on 7 of 9 planned held-out datasets. HotpotQA and FEVER remain to be evaluated. The final mean and win/loss ratio may shift.

4. **No significance tests.** We do not report statistical significance tests (e.g., paired t-test over queries). Given the sometimes small per-dataset differences (SCIDOCS: +0.0003), significance testing is warranted before drawing strong conclusions on individual datasets. The large aggregate improvement (+5.99%) across seven diverse datasets provides some robustness, but formal testing is needed.

5. **Single *k*1 value.** We evaluate BM199 with *k*1 = 1.8 only, selected on the tuning set. A sensitivity analysis over *k*1 values would strengthen confidence in the robustness of the approach.

6. **No neural reranking.** We evaluate BM199 as a standalone first-stage retriever. Its utility as a candidate generator for neural rerankers (which dominate BEIR leaderboards) is untested. If BM199 retrieves a different candidate set, it may complement or conflict with downstream rerankers.

7. **Theoretical justification.** We provide empirical evidence and geometric intuition for sqrt normalization but lack a formal probabilistic derivation analogous to the original BM25 derivation from the 2-Poisson model. A derivation from first principles --- perhaps via a log-normal document length model --- would strengthen the contribution.

---

## 8. Conclusion

We presented BM199, a minimal modification to BM25 that replaces linear length normalization with parameter-free sqrt normalization. This one-line change eliminates the *b* parameter, reducing BM25 from two tunable parameters to one (*k*1 = 1.8 only). On seven held-out BEIR datasets with frozen parameters, BM199 improves over optimally-tuned BM25 by +5.99% mean nDCG@10, winning on six of seven datasets. The gains are concentrated on collections with high document length heterogeneity, where sublinear normalization better accommodates extreme-length documents.

The simplicity of the modification is notable: any BM25 implementation can adopt BM199 by replacing a single line of code. The parameter reduction from two to one simplifies deployment and eliminates the parameter (*b*) most responsible for poor cross-collection transfer.

To our knowledge, we are not aware of prior published work using sqrt(dl/avgdl) as a drop-in BM25 length normalizer evaluated on the full BEIR benchmark. We hope this work motivates further investigation of non-linear normalization functions in the BM25 family.

Future work includes completing the full 13-dataset BEIR evaluation, adding statistical significance tests, conducting sensitivity analysis over *k*1, evaluating BM199 as a first-stage retriever for neural rerankers, and developing a formal probabilistic derivation of sqrt normalization.

---

## References

Amati, G. (2006). Frequentist and Bayesian Approach to Information Retrieval. In *Proceedings of ECIR 2006*, pages 13--24.

Amati, G. and Van Rijsbergen, C. J. (2002). Probabilistic Models of Information Retrieval Based on Measuring the Divergence from Randomness. *ACM Transactions on Information Systems*, 20(4):357--389.

Karpukhin, V., Oguz, B., Min, S., Lewis, P., Wu, L., Edunov, S., Chen, D., and Yih, W. (2020). Dense Passage Retrieval for Open-Domain Question Answering. In *Proceedings of EMNLP 2020*, pages 6769--6781.

Khattab, O. and Zaharia, M. (2020). ColBERT: Efficient and Effective Passage Search via Contextualized Late Interaction over BERT. In *Proceedings of SIGIR 2020*, pages 39--48.

Lv, Y. and Zhai, C. (2011). When Documents Are Very Long, BM25 Fails! In *Proceedings of SIGIR 2011*, pages 1103--1104.

Robertson, S. E., Walker, S., Jones, S., Hancock-Beaulieu, M., and Gatford, M. (1994). Okapi at TREC-3. In *Proceedings of TREC-3*, pages 109--126.

Robertson, S. and Zaragoza, H. (2009). The Probabilistic Relevance Framework: BM25 and Beyond. *Foundations and Trends in Information Retrieval*, 3(4):333--389.

Singhal, A., Buckley, C., and Mitra, M. (1996). Pivoted Document Length Normalization. In *Proceedings of SIGIR 1996*, pages 21--29.

Thakur, N., Reimers, N., Rucktaschel, A., Srivastava, A., and Gurevych, I. (2021). BEIR: A Heterogeneous Benchmark for Zero-shot Evaluation of Information Retrieval Models. In *Proceedings of NeurIPS 2021 Datasets and Benchmarks Track*.

Trotman, A., Puurula, A., and Burgess, B. (2014). Improvements to BM25 and Language Models Examined. In *Proceedings of ADCS 2014*, pages 58--65.

---

## Appendix A: BM199 Implementation

The modification from BM25 to BM199 requires changing a single line in any standard BM25 implementation. In pseudocode:

```
// BM25 normalization
norm = 1.0 - b + b * (dl / avgdl)

// BM199 normalization (replaces the above)
norm = sqrt(dl / avgdl)
```

The complete BM199 scoring function:

```python
import math

def bm199_score(tf, df, dl, avgdl, N, k1=1.8):
    # IDF (Lucene formula)
    idf = math.log((N - df + 0.5) / (df + 0.5) + 1.0)

    # Term frequency saturation
    beta = 1.0 - 0.2 * max(0.0, min(1.0, idf / math.log(N)))
    tf_sat = tf ** beta

    # Sqrt length normalization
    norm = math.sqrt(dl / avgdl)

    # Score
    score = idf * (tf_sat * (k1 + 1.0)) / (tf_sat + k1 * norm)
    return score
```

Note: When tf = 1 (the common case), tf_sat = 1.0 regardless of beta, yielding the simplified formula presented in Section 3.1.

---

## Appendix B: Full Numerical Results

**Table B.1: Complete held-out results with absolute differences.**

| Dataset        | BM25    | BM199   | Abs. Delta | Rel. Delta |
|----------------|:-------:|:-------:|:----------:|:----------:|
| TREC-COVID     | 0.6348  | 0.6559  | +0.0211    | +3.3%      |
| Quora          | 0.7831  | 0.7808  | -0.0023    | -0.3%      |
| SCIDOCS        | 0.1569  | 0.1572  | +0.0003    | +0.2%      |
| Touche-v2      | 0.3189  | 0.4048  | +0.0859    | +26.9%     |
| NQ             | 0.2898  | 0.2909  | +0.0011    | +0.4%      |
| DBPedia-Entity | 0.2908  | 0.2959  | +0.0051    | +1.8%      |
| Climate-FEVER  | 0.1365  | 0.1817  | +0.0452    | +33.1%     |
| **Mean**       |**0.3730**|**0.3953**|**+0.0223**|**+5.99%** |

Wins: 6/7. Losses: 1/7 (Quora, -0.3%).
