# Scoring Formula Comparison

Side-by-side mathematical definitions of all scoring algorithms evaluated.

## 1. BM25 (Robertson & Walker, 1994)

The standard probabilistic retrieval model.

$$\text{BM25}(t, d) = \text{IDF}(t) \cdot \frac{tf \cdot (k_1 + 1)}{tf + k_1 \cdot \left(1 - b + b \cdot \frac{|d|}{\text{avgdl}}\right)}$$

**IDF:**
$$\text{IDF}(t) = \ln\left(\frac{N - df_t + 0.5}{df_t + 0.5} + 1\right)$$

**Parameters:** $k_1$ (TF saturation, typically 1.2), $b$ (length normalization, typically 0.75)

**Properties:**
- Linear TF saturation via $\frac{tf \cdot (k_1+1)}{tf + k_1 \cdot L}$
- Linear length normalization: $L = 1 - b + b \cdot \frac{|d|}{\text{avgdl}}$
- 2 tunable parameters
- Violates the TF lower-bounding constraint (score can decrease with additional term occurrences for very long documents)

---

## 2. BM25+ (Lv & Zhai, 2011)

Fixes BM25's lower-bounding violation by adding a floor $\delta$.

$$\text{BM25+}(t, d) = \text{IDF}(t) \cdot \left[\frac{tf \cdot (k_1 + 1)}{tf + k_1 \cdot \left(1 - b + b \cdot \frac{|d|}{\text{avgdl}}\right)} + \delta\right]$$

**Parameters:** $k_1$, $b$, $\delta$ (floor, typically 1.0)

**Properties:**
- Adds constant $\delta$ after TF normalization
- Guarantees that adding a term occurrence never decreases the score
- 3 tunable parameters

---

## 3. BM25L (Lv & Zhai, 2011)

Adjusts TF for long documents before normalization.

$$c_{tf} = \frac{tf}{1 - b + b \cdot \frac{|d|}{\text{avgdl}}}$$

$$c_{tf}' = c_{tf} + \delta$$

$$\text{BM25L}(t, d) = \text{IDF}(t) \cdot \frac{c_{tf}' \cdot (k_1 + 1)}{c_{tf}' + k_1}$$

**Parameters:** $k_1$, $b$, $\delta$

**Properties:**
- Normalizes TF by document length first, then applies saturation
- Prevents long documents from being unfairly penalized
- 3 tunable parameters

---

## 4. BM25-ATIRE (Trotman et al., 2014)

Simplified IDF variant used in the ATIRE search engine.

$$\text{BM25\text{-}ATIRE}(t, d) = \ln\left(\frac{N}{df_t}\right) \cdot \frac{tf \cdot (k_1 + 1)}{tf + k_1 \cdot \left(1 - b + b \cdot \frac{|d|}{\text{avgdl}}\right)}$$

**IDF:** $\text{IDF}(t) = \ln\left(\frac{N}{df_t}\right)$

**Parameters:** $k_1$, $b$

**Properties:**
- Simpler IDF formula (no smoothing terms)
- Identical results to standard BM25 in practice when IDF values are similar
- 2 tunable parameters

---

## 5. DLH13 (Amati, 2006)

Divergence from Randomness model; completely parameter-free.

$$\text{DLH13}(t, d) = \frac{1}{tf + 0.5} \cdot \left[tf \cdot \log_2\left(\frac{tf \cdot \text{avgdl}}{|d| \cdot \lambda}\right) + 0.5 \cdot \log_2\left(2\pi \cdot tf \cdot \left(1 - \frac{tf}{|d|}\right)\right)\right]$$

where $\lambda = \frac{cf}{N}$ is the collection frequency ratio.

**Parameters:** None (parameter-free)

**Properties:**
- Based on information-theoretic divergence from randomness
- Requires collection frequency ($cf$) in addition to document frequency
- Zero tunable parameters
- Often competitive with tuned BM25

---

## 6. QLD (Ponte & Croft, 1998; Zhai & Lafferty, 2004)

Query Likelihood with Dirichlet smoothing.

$$\text{QLD}(t, d) = \ln\left(\frac{tf + \mu \cdot p_c}{|d| + \mu}\right)$$

where $p_c = \frac{cf}{\sum_{d'} |d'|}$ is the collection probability.

**Parameters:** $\mu$ (Dirichlet prior, typically 2500)

**Properties:**
- Language modeling approach
- Smooths document model with collection model
- 1 tunable parameter
- Performs poorly with standard IDF (needs careful tuning of $\mu$)

---

## 7. TF-IDF (Salton & Buckley, 1988)

Classic vector space model with sublinear TF.

$$\text{TF\text{-}IDF}(t, d) = (1 + \ln tf) \cdot \ln\left(\frac{N}{df_t}\right)$$

**Parameters:** None

**Properties:**
- Sublinear TF via logarithm
- No length normalization (in the per-term formulation)
- Simple and interpretable
- Generally weaker than BM25 family

---

## 8. BM199 (This Work, 2026)

Novel scoring algorithm with sqrt length normalization and adaptive TF saturation.

$$\text{BM199}(t, d) = \text{IDF}(t) \cdot \left[\frac{tf^{\beta(t)} \cdot (k_1 + 1)}{tf^{\beta(t)} + k_1 \cdot \sqrt{\frac{|d|}{\text{avgdl}}}} + \delta\right]$$

**Adaptive saturation exponent:**

$$\beta(t) = \beta_{\max} - \text{clamp}\left(\frac{\text{IDF}(t)}{\ln N}, 0, 1\right) \cdot (\beta_{\max} - \beta_{\min})$$

**IDF:** Same as standard BM25: $\text{IDF}(t) = \ln\left(\frac{N - df_t + 0.5}{df_t + 0.5} + 1\right)$

**Parameters:** $k_1 = 1.8$, $\beta_{\min} = 0.8$, $\beta_{\max} = 1.0$, $\delta = 0.0$

**Properties:**
- **Square-root length normalization** (core innovation): $\sqrt{|d|/\text{avgdl}}$ replaces BM25's linear $1 - b + b \cdot |d|/\text{avgdl}$
- **Zero length normalization parameters**: no $b$ to tune
- **Adaptive TF saturation**: rare terms (high IDF) saturate faster via lower $\beta$; common terms scale near-linearly
- 2 effective parameters ($k_1$, $\beta_{\min}$), though $\beta_{\min}$'s effect is marginal
- Sublinear compression of length penalties benefits long-document and argumentative corpora

---

## Comparison Table

| Algorithm | Year | TF Component | Length Norm | IDF | Params | Needs CF? |
|-----------|------|-------------|-------------|-----|--------|-----------|
| TF-IDF | 1988 | $1 + \ln tf$ | None | $\ln(N/df)$ | 0 | No |
| BM25 | 1994 | $\frac{tf(k_1+1)}{tf+k_1 L}$ | $1-b+b\frac{|d|}{\text{avgdl}}$ | Standard | 2 | No |
| QLD | 1998 | $\ln(tf+\mu p_c)$ | $|d|+\mu$ | Implicit | 1 | Yes |
| DLH13 | 2006 | Info-theoretic | Implicit | Implicit | 0 | Yes |
| BM25+ | 2011 | $\frac{tf(k_1+1)}{tf+k_1 L}+\delta$ | $1-b+b\frac{|d|}{\text{avgdl}}$ | Standard | 3 | No |
| BM25L | 2011 | $\frac{c'(k_1+1)}{c'+k_1}$ | Pre-normalize TF | Standard | 3 | No |
| BM25-ATIRE | 2014 | Same as BM25 | Same as BM25 | $\ln(N/df)$ | 2 | No |
| **BM199** | **2026** | $\frac{tf^{\beta}(k_1+1)}{tf^{\beta}+k_1\ell}$ | $\sqrt{|d|/\text{avgdl}}$ | Standard | **2** | No |

### Length Normalization Comparison

For a document 4x the average length ($|d|/\text{avgdl} = 4$):

| Algorithm | $b$ | Length penalty |
|-----------|-----|---------------|
| BM25 ($b=0.75$) | 0.75 | $1-0.75+0.75 \cdot 4 = 3.25$ |
| BM25 ($b=0.9$) | 0.9 | $1-0.9+0.9 \cdot 4 = 3.7$ |
| **BM199** | N/A | $\sqrt{4} = 2.0$ |

For a document 1/4 the average length ($|d|/\text{avgdl} = 0.25$):

| Algorithm | $b$ | Length penalty |
|-----------|-----|---------------|
| BM25 ($b=0.75$) | 0.75 | $1-0.75+0.75 \cdot 0.25 = 0.4375$ |
| BM25 ($b=0.9$) | 0.9 | $1-0.9+0.9 \cdot 0.25 = 0.325$ |
| **BM199** | N/A | $\sqrt{0.25} = 0.5$ |

**Key insight**: BM199's sqrt compression is less aggressive than BM25 for both long and short documents. This prevents over-penalizing long documents (e.g., argumentative text in Touche) and over-rewarding short documents (e.g., short snippets in FiQA).
