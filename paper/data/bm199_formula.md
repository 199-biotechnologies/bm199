# BM199 Scoring Formula

## Source: `src/scorer.rs` -- `Bm199Params::score_term`

### Rust Implementation (Final v1.0)

```rust
pub fn score_term(&self, tf: f64, df: u64, dl: u64, avgdl: f64, n: u64) -> f64 {
    let idf = idf_standard(df, n);
    let max_idf = (n as f64).ln();

    // Adaptive saturation: rare terms (high IDF) saturate faster (lower exponent)
    let idf_ratio = (idf / max_idf).clamp(0.0, 1.0);
    let beta = self.beta_max - idf_ratio * (self.beta_max - self.beta_min);
    let tf_sat = tf.powf(beta);

    // BM199 core innovation: sqrt length normalization (zero parameters)
    // Compresses both short and long doc penalties sublinearly
    // Eliminates the need for tuned `b` parameter entirely
    let len_norm = (dl as f64 / avgdl).sqrt();

    let tf_component = (tf_sat * (self.k1 + 1.0)) / (tf_sat + self.k1 * len_norm) + self.delta;

    idf * tf_component
}
```

### IDF Function

```rust
pub fn idf_standard(df: u64, n: u64) -> f64 {
    let n = n as f64;
    let df = df as f64;
    ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
}
```

## Mathematical Description

### Per-Term Score

Given a query term $t$ appearing in document $d$:

$$\text{score}(t, d) = \text{IDF}(t) \cdot \left[ \frac{tf_{\text{sat}} \cdot (k_1 + 1)}{tf_{\text{sat}} + k_1 \cdot \ell} + \delta \right]$$

where:

**IDF (Inverse Document Frequency):**

$$\text{IDF}(t) = \ln\left(\frac{N - df_t + 0.5}{df_t + 0.5} + 1\right)$$

**Adaptive TF Saturation:**

$$\text{IDF}_{\text{ratio}} = \text{clamp}\left(\frac{\text{IDF}(t)}{\ln(N)}, 0, 1\right)$$

$$\beta = \beta_{\max} - \text{IDF}_{\text{ratio}} \cdot (\beta_{\max} - \beta_{\min})$$

$$tf_{\text{sat}} = tf^{\beta}$$

Rare terms (high IDF) get lower $\beta$, causing faster sublinear saturation. Common terms (low IDF) get $\beta \approx \beta_{\max} \approx 1.0$, keeping near-linear TF scaling.

**Square-Root Length Normalization (Core Innovation):**

$$\ell = \sqrt{\frac{|d|}{\text{avgdl}}}$$

This is the key departure from BM25's linear normalization $1 - b + b \cdot \frac{|d|}{\text{avgdl}}$. The square root provides:
- Sublinear compression of length penalties
- No tunable parameter (BM25 requires $b$)
- Symmetric treatment of short and long documents
- Better generalization across diverse corpora

### Document Score

$$S(q, d) = \sum_{t \in q} \text{score}(t, d) + \lambda_{\text{cov}} \cdot \frac{|\{t \in q : tf_{t,d} > 0\}|}{|q|} \cdot \sum_{t \in q} \text{score}(t, d)$$

(In the final v1.0, $\lambda_{\text{cov}} = 0.0$, so the coverage bonus is disabled.)

### Parameters (Final Tuned Values)

| Parameter | Value | Description |
|-----------|-------|-------------|
| $k_1$ | 1.8 | TF saturation base |
| $b$ | 0.9 | (unused in v1.0 -- replaced by sqrt) |
| $\delta$ | 0.0 | BM25+ lower-bounding floor (disabled) |
| $\beta_{\min}$ | 0.8 | Adaptive saturation exponent for rare terms |
| $\beta_{\max}$ | 1.0 | Adaptive saturation exponent for common terms |
| $\lambda_{\text{cov}}$ | 0.0 | Coverage bonus weight (disabled) |
| $\alpha_{\text{sig}}$ | 1.0 | Sigmoid calibration slope |
| $\beta_{\text{sig}}$ | 0.0 | Sigmoid calibration offset |
| $\log_{\text{base}}$ | 2.0 | (unused in v1.0 -- replaced by sqrt) |

### Key Innovations vs BM25

1. **Parameter-free length normalization**: Replaces BM25's $b$ parameter with $\sqrt{|d|/\text{avgdl}}$
2. **Adaptive TF saturation**: Term-specific saturation exponent conditioned on IDF
3. **Reduced parameter count**: Only 2 effective parameters ($k_1$, $\beta_{\min}$) vs BM25's 2 ($k_1$, $b$), but sqrt norm generalizes better
