# Hypothesis Tracker — BM25 Normalization Study

## Sweep Results Summary (2026-04-05, tuning set, fixed tokenizer)

### TOP 20 Tuning Results (avg nDCG@10)

| Rank | Variant | k1 | Params | Tuning | Notes |
|------|---------|----:|--------|--------|-------|
| 1 | Hinged(α=0.70) | 1.5 | α=0.70 | 0.4126 | **BEST. Piecewise: linear for short, power for long** |
| 2 | Hinged(α=0.80) | 1.2 | α=0.80 | 0.4123 | Near-linear long-doc regime |
| 3 | Saturation(c=5.0) | 1.5 | c=5.0 | 0.4122 | Bounded diminishing returns |
| 4 | BM25(b=1.0) | 1.2 | b=1.0 | 0.4121 | Tuned BM25 — full normalization |
| 5 | DualPivot(ss=1,sl=0.75,α=0.6) | 1.8 | 3 params | 0.4119 | Complex, marginal gain |
| 6 | Hinged(α=0.80) | 1.5 | α=0.80 | 0.4118 | |
| 7 | Hinged(α=0.80) | 1.8 | α=0.80 | 0.4116 | |
| 8 | BM25(b=0.90) | 1.2 | b=0.90 | 0.4116 | Tuned BM25 |
| 9 | Power(α=0.80) | 1.8 | α=0.80 | 0.4112 | Near-linear power |
| 10 | Power(α=0.90) | 1.8 | α=0.90 | 0.4111 | |
| 11 | Hinged(α=0.60) | 1.8 | α=0.60 | 0.4110 | |
| 12 | Asymmetric(1.0,0.60) | 1.8 | 2 params | 0.4110 | Same as hinged(0.60) |
| 13 | LogTF+Sqrt | 1.8 | tf=log | 0.4103 | Interesting TF variant |
| 14 | BM25 default | 1.2 | b=0.75 | 0.4091 | **BASELINE** |
| 15 | Log | 1.6 | 0 params | 0.4096 | Zero-param, good generalization |
| 16 | Sqrt (BM199) | 1.6 | α=0.5 | 0.3991 | Original BM199 — poor tuning |

### Validation Results (pending full run)

| Variant | Tuning | Validation | Gap | Generalizes? |
|---------|--------|------------|-----|--------------|
| Log(k1=1.6) | 0.4096 | **0.3594** | -12.3% | **BEST** |
| Power(α=0.80, k1=1.8) | 0.4112 | 0.3423 | -16.7% | OK |
| Hinged(α=0.80, k1=1.2) | 0.4123 | 0.3393 | -17.7% | Weak |
| Hinged(α=0.70, k1=1.5) | 0.4126 | 0.3379 | -18.1% | Weak |
| BM25 default | 0.4091 | (pending) | | |
| Sqrt/BM199 (k1=1.5) | 0.3994 | (pending) | | |

### Key Findings

1. **Tuning ≠ Generalization**: The tuning winner (hinged) drops 18% on validation.
   Log normalization (rank 15 on tuning) generalizes BEST.

2. **Parameter count matters for generalization**: Zero-param norms (log, sigmoid, softplus)
   generalize better than parameterized ones, even when the parameterized ones win on tuning.

3. **The "b" parameter IS the problem**: BM25's b=0.75 default is close to optimal.
   Even tuned b=1.0 (which looks great on tuning) degrades on validation.

## RankEvolve-Inspired Results (tested 2026-04-05)

Both RankEvolve-style norms UNDERPERFORM on tuning when used alone:
- Best RankEvolve log norm: 0.3749 (c=0.50, k1=1.0) — vs BM25 0.4091
- Best Bidirectional: 0.3706 (c=0.30, k1=1.0) — vs BM25 0.4091
- These are too gentle as standalone norms; RankEvolve uses them as ONE component
  among many (coverage multiplier, PMI specificity, coordination, etc.)
- The individual components need to be COMBINED, not tested in isolation

## Research Agent Findings (2025-2026 Literature)

**Most promising untested ideas (from 10-area landscape scan):**
1. **RankEvolve formula (combined):** log norm + coverage + PMI + adaptive TF + leaky rectifier
2. **iDL (DLITE paper):** Information-theoretic IDF replacement, beats BM25 on 3 decades of TREC data
3. **BMX entropy-weighted IDF:** Replaces k1 saturation with entropy-augmented term
4. **Verboseness Fission (Lipani):** Decomposes b into verbosity/scope components, parameter-free
5. **BM25P proximity:** Replace TF with pseudo-TF based on proximity kernel, +5-11% MAP
6. **LexBoost:** Graph-based neighbor score smoothing, +7-17% MAP (needs neighbor graph)
7. **SPLADE-style log(1+x) saturation:** Already tested as our "log TF" variant (0.4103 tuning)
8. **BM42 attention-as-TF:** Replace TF with transformer attention weights (needs model)

## Untested Hypotheses (Phase 2)

### IDF-Conditioned Normalization (HIGH PRIORITY, NOVEL)
- **Hypothesis:** Rare terms appearing in long documents are informative signals, not noise.
  Common terms in long docs are expected. Therefore, rare terms should get LESS length
  normalization than common terms.
- **Implementation:** `alpha_eff = alpha_base * (1 - gamma * idf_ratio)` where idf_ratio∈[0,1]
- **CLI:** `--norm idf_conditioned --alpha 0.7 --gamma 0.3 --k1 1.5`
- **Expected outcome:** Better on heterogeneous corpora (Touché, Climate-FEVER)

### Query-Length Adaptive k1 (HIGH PRIORITY, NOVEL)
- **Hypothesis:** Single-term queries need different saturation than multi-term queries.
  Short queries benefit from exact matching (low k1), long queries from broader matching (high k1).
- **Implementation:** `k1_eff = k1 * (1 + beta * ln(qlen))` or `k1_eff = k1 * qlen^beta`
- **Expected outcome:** Better on NQ (short queries) and Touché (long queries)

### Collection-Adaptive Alpha (NOVEL)
- **Hypothesis:** The optimal power α correlates with collection length heterogeneity.
  Collections with high CV(doc_lengths) need lower α (more compression).
- **Implementation:** `alpha = alpha_base - gamma * CV(doc_lengths)`
- **Expected outcome:** Automatically adapts to collection characteristics

### BM25F-Style Pseudo-Title (MEDIUM PRIORITY)
- **Hypothesis:** First N tokens of a document function like a title and should get
  different normalization than the body.
- **Implementation:** Score first-50-tokens and rest-of-doc separately, combine
- **Requires index changes**

### Proximity-Weighted BM25 (MEDIUM PRIORITY)
- **Hypothesis:** When query terms appear near each other in a document, the document
  is more likely relevant. Proximity signals are underused in classical IR.
- **Implementation:** Multiply term scores by proximity factor based on min-distance between query terms
- **Requires index changes** (need to store term positions)

### Per-Term k1 (IDF-Conditioned Saturation)
- **Hypothesis:** Rare terms should saturate faster than common terms.
  Seeing a rare term once is very informative; seeing it 5 times adds less.
  Common terms need higher TF to be meaningful.
- **Implementation:** `k1_eff = k1 * (1 - gamma * idf_ratio)`
- **Expected outcome:** Better precision on exact-match tasks

### Interpolated Normalization
- **Hypothesis:** Blend linear and sublinear norms with a mixing weight.
  `norm = lambda * (1-b+b*r) + (1-lambda) * r^alpha`
- **Implementation:** Add `NormType::Interpolated { b, alpha, lambda }`
- **Expected outcome:** Smooth transition, may find sweet spot

### Exponential Decay Normalization
- **Hypothesis:** Penalize deviation from average, not absolute length.
  `norm = 1 + sign(r-1) * |r-1|^alpha`
- **Implementation:** Add `NormType::Deviation { alpha }`
- **Expected outcome:** Symmetric treatment of short/long deviations

### Quantile-Based Normalization
- **Hypothesis:** Normalize by percentile rank in the collection's length distribution,
  not by raw ratio to average. This is robust to outliers.
- **Implementation:** Pre-compute quantile function, normalize dl by its percentile
- **Requires index changes** (need CDF of doc lengths)

### Entropy-Weighted IDF
- **Hypothesis:** Standard IDF only uses document frequency. Term entropy across documents
  (how evenly distributed a term is) provides additional signal.
- **Implementation:** `idf_eff = idf * (1 + H(term))` where H is term entropy
- **Requires collection frequency** (already available)
