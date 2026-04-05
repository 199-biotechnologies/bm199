/// All scoring variants: BM25, BM25+, BM25L, BM199 (novel)
/// Each scorer takes the same inputs and returns a document score.

/// Standard BM25 (Lucene/Robertson)
pub fn bm25(tf: f64, df: u64, dl: u64, avgdl: f64, n: u64, k1: f64, b: f64) -> f64 {
    let idf = idf_standard(df, n);
    let tf_norm = (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * (dl as f64 / avgdl)));
    idf * tf_norm
}

/// BM199 — BM25 with sqrt length normalization (parameter-free length norm)
/// The ONLY difference from BM25: replaces `(1-b+b*dl/avgdl)` with `sqrt(dl/avgdl)`.
/// Eliminates the `b` hyperparameter entirely. Recommended k1=1.8.
pub fn bm199(tf: f64, df: u64, dl: u64, avgdl: f64, n: u64, k1: f64) -> f64 {
    let idf = idf_standard(df, n);
    let tf_norm = (tf * (k1 + 1.0)) / (tf + k1 * (dl as f64 / avgdl).sqrt());
    idf * tf_norm
}

/// BM25+ — adds delta floor to fix lower-bounding violation
pub fn bm25_plus(tf: f64, df: u64, dl: u64, avgdl: f64, n: u64, k1: f64, b: f64, delta: f64) -> f64 {
    let idf = idf_standard(df, n);
    let tf_norm = (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * (dl as f64 / avgdl))) + delta;
    idf * tf_norm
}

/// BM25L — adjusts TF for long documents before normalization
pub fn bm25l(tf: f64, df: u64, dl: u64, avgdl: f64, n: u64, k1: f64, b: f64, delta: f64) -> f64 {
    let idf = idf_standard(df, n);
    let ctf = tf / (1.0 - b + b * (dl as f64 / avgdl));
    let ctf_prime = ctf + delta;
    let tf_norm = (ctf_prime * (k1 + 1.0)) / (ctf_prime + k1);
    idf * tf_norm
}

/// BM199 experimental params — used for autoresearch ablation studies.
/// For the clean BM199 formula, use the standalone `bm199()` function instead.
/// Only `k1` matters in the final formula; other params are vestigial from ablations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bm199Params {
    pub k1: f64,
    pub b: f64,
    pub delta: f64,
    pub beta_min: f64,
    pub beta_max: f64,
    pub lambda_cov: f64,
    pub alpha_sig: f64,
    pub beta_sig: f64,
    pub log_base: f64,
}

impl Default for Bm199Params {
    fn default() -> Self {
        Self {
            k1: 1.8,
            b: 0.9,        // vestigial — unused with sqrt norm
            delta: 0.0,     // disabled
            beta_min: 0.8,
            beta_max: 1.0,
            lambda_cov: 0.0, // disabled
            alpha_sig: 1.0,
            beta_sig: 0.0,
            log_base: 2.0,  // vestigial — unused with sqrt norm
        }
    }
}

impl Bm199Params {
    /// Per-term score for BM199
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

    /// Score a full document against a query
    /// query_tfs: term -> (tf_in_doc, df_in_corpus)
    /// Returns raw score (before sigmoid calibration)
    pub fn score_document(
        &self,
        query_terms: &[(u64, u64)], // (df, tf_in_doc) per query term
        dl: u64,
        avgdl: f64,
        n: u64,
        query_len: usize,
    ) -> f64 {
        let mut raw = 0.0;
        let mut matched = 0usize;

        for &(df, tf) in query_terms {
            if tf > 0 {
                matched += 1;
            }
            raw += self.score_term(tf as f64, df, dl, avgdl, n);
        }

        // Coverage bonus: reward documents matching diverse query terms
        let coverage = if query_len > 1 {
            matched as f64 / query_len as f64
        } else {
            1.0
        };
        raw += self.lambda_cov * coverage * raw;

        raw
    }

    /// Bayesian sigmoid calibration: convert raw score to probability
    pub fn calibrate(&self, raw_score: f64) -> f64 {
        1.0 / (1.0 + (-self.alpha_sig * (raw_score - self.beta_sig)).exp())
    }
}

/// BM25-ATIRE variant: simplified IDF = log(N/df)
pub fn bm25_atire(tf: f64, df: u64, dl: u64, avgdl: f64, n: u64, k1: f64, b: f64) -> f64 {
    let idf = (n as f64 / df as f64).ln();
    let tf_norm = (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * (dl as f64 / avgdl)));
    idf * tf_norm
}

/// DLH13 — Divergence from Randomness, parameter-free
/// f_r = tf/dl
/// score = (1/(tf+0.5)) * [tf * log2(tf * avgdl / (dl * F/N)) + 0.5 * log2(2π * tf * (1-f_r))]
/// Requires cf (collection frequency of term)
pub fn dlh13(tf: f64, dl: u64, avgdl: f64, n: u64, cf: u64) -> f64 {
    if tf <= 0.0 { return 0.0; }
    let dl = dl as f64;
    let n_f = n as f64;
    let cf_f = cf as f64;
    let f_r = tf / dl;
    if f_r >= 1.0 { return 0.0; }
    let lambda = cf_f / n_f;
    let term1 = tf * ((tf * avgdl / (dl * lambda)).max(1e-10)).log2();
    let term2 = 0.5 * (2.0 * std::f64::consts::PI * tf * (1.0 - f_r)).max(1e-10).log2();
    (term1 + term2) / (tf + 0.5)
}

/// QLD — Query Likelihood with Dirichlet smoothing
/// score = log((tf + μ * p_c) / (dl + μ))
/// p_c = cf / total_tokens, μ = 2500 (default)
pub fn qld(tf: f64, dl: u64, cf: u64, total_tokens: u64, mu: f64) -> f64 {
    let p_c = cf as f64 / total_tokens as f64;
    ((tf + mu * p_c) / (dl as f64 + mu)).ln()
}

/// TF-IDF (sublinear TF, cosine-normalized)
pub fn tfidf(tf: f64, df: u64, n: u64) -> f64 {
    if tf <= 0.0 { return 0.0; }
    let tf_log = 1.0 + tf.ln();
    let idf = (n as f64 / df as f64).ln();
    tf_log * idf
}

/// Standard IDF: log((N - df + 0.5) / (df + 0.5) + 1)
#[inline]
pub fn idf_standard(df: u64, n: u64) -> f64 {
    let n = n as f64;
    let df = df as f64;
    ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
}

// ============================================================================
// GENERIC SCORING FRAMEWORK — for systematic hypothesis testing
// ============================================================================

/// Length normalization type. All satisfy f(1.0) = 1.0 (pivot at average doc length).
#[derive(Debug, Clone, Copy)]
pub enum NormType {
    /// BM25 standard: 1 - b + b*r
    Linear(f64),
    /// Power family: r^alpha. alpha=0.5 is sqrt, alpha=1.0 is b=1.0
    Power(f64),
    /// Logarithmic: ln(1+r)/ln(2). Slowest growth.
    Log,
    /// Bounded sigmoid: 2r/(1+r). Caps at 2.0 for infinitely long docs.
    Sigmoid,
    /// Hinged: r for short docs, r^alpha for long docs. One param.
    Hinged(f64),
    /// Asymmetric: r^a1 for short, r^a2 for long. Two params.
    Asymmetric(f64, f64),
    /// Saturation: r/(r+c)*(1+c). Bounded. c controls saturation speed.
    Saturation(f64),
    /// Softplus: smooth approximation. Normalized so f(1)=1.
    Softplus,
    /// Dual-pivot: three-regime normalization with explicit short/long slopes
    DualPivot { s_short: f64, s_long: f64, alpha_long: f64 },
    /// IDF-conditioned: rare terms get LESS length normalization (higher alpha)
    /// because a rare term in a long doc is informative, not noise.
    /// alpha_eff = base_alpha + gamma * idf_ratio
    IdfConditioned { base_alpha: f64, gamma: f64 },
    /// Hinged + IDF: combine hinged (linear short, power long) with IDF conditioning
    HingedIdf { base_alpha: f64, gamma: f64 },
    /// RankEvolve's log norm: 1 + c * ln(1 + r). Gentler than any power for long docs.
    RankEvolveLog { c: f64 },
    /// Bidirectional: penalizes deviation from avgdl in BOTH directions.
    /// norm = 1 + c * (ln(r))^2. Documents at avgdl get norm=1, both shorter and longer get penalized.
    Bidirectional { c: f64 },
}

/// TF transformation mode
#[derive(Debug, Clone, Copy)]
pub enum TfMode {
    /// Raw tf (BM25 standard)
    Standard,
    /// log(1+tf) — logarithmic saturation
    Log,
    /// log(1+log(1+tf)) — double-log compression
    DoubleLog,
    /// min(tf, cap) — hard ceiling
    Capped(f64),
}

/// IDF computation mode
#[derive(Debug, Clone, Copy)]
pub enum IdfMode {
    /// Standard Lucene: ln((N-df+0.5)/(df+0.5)+1)
    Standard,
    /// ATIRE: ln(N/df)
    Atire,
    /// Squared: IDF² — boosts rare terms more
    Squared,
    /// Smoothed: ln((N+1)/(df+1))
    Smoothed,
}

/// Generic scoring configuration for hypothesis testing
#[derive(Debug, Clone, Copy)]
pub struct ScoringConfig {
    pub k1: f64,
    pub norm: NormType,
    pub tf_mode: TfMode,
    pub idf_mode: IdfMode,
    pub delta: f64, // BM25+ floor (0 to disable)
}

impl ScoringConfig {
    /// BM25 default configuration
    pub fn bm25_default() -> Self {
        Self { k1: 1.2, norm: NormType::Linear(0.75), tf_mode: TfMode::Standard, idf_mode: IdfMode::Standard, delta: 0.0 }
    }

    /// BM199 (sqrt) configuration
    pub fn bm199_default() -> Self {
        Self { k1: 1.8, norm: NormType::Power(0.5), tf_mode: TfMode::Standard, idf_mode: IdfMode::Standard, delta: 0.0 }
    }
}

/// Compute length normalization value from ratio r = dl/avgdl
/// idf_ratio: term's IDF / max IDF, in [0,1]. Only used by IDF-conditioned norms.
#[inline]
pub fn compute_norm(norm: NormType, r: f64, idf_ratio: f64) -> f64 {
    match norm {
        NormType::Linear(b) => 1.0 - b + b * r,
        NormType::Power(alpha) => r.powf(alpha),
        NormType::Log => (1.0 + r).ln() / 2.0_f64.ln(),
        NormType::Sigmoid => 2.0 * r / (1.0 + r),
        NormType::Hinged(alpha) => if r <= 1.0 { r } else { r.powf(alpha) },
        NormType::Asymmetric(a_short, a_long) => if r <= 1.0 { r.powf(a_short) } else { r.powf(a_long) },
        NormType::Saturation(c) => r / (r + c) * (1.0 + c),
        NormType::Softplus => {
            let denom = (1.0 + 1.0_f64.exp()).ln();
            (1.0 + (r - 1.0).exp()).ln() / denom
        }
        NormType::DualPivot { s_short, s_long, alpha_long } => {
            if r <= 1.0 {
                1.0 - s_short * (1.0 - r)
            } else {
                1.0 + s_long * (r - 1.0).powf(alpha_long)
            }
        }
        NormType::IdfConditioned { base_alpha, gamma } => {
            // Rare terms (high idf_ratio) → higher alpha → LESS normalization
            // Common terms (low idf_ratio) → lower alpha → MORE normalization
            let alpha = (base_alpha + gamma * idf_ratio).clamp(0.1, 1.5);
            r.powf(alpha)
        }
        NormType::HingedIdf { base_alpha, gamma } => {
            let alpha = (base_alpha + gamma * idf_ratio).clamp(0.1, 1.5);
            if r <= 1.0 { r } else { r.powf(alpha) }
        }
        NormType::RankEvolveLog { c } => {
            // From RankEvolve (2026): 1 + c * ln(1 + r)
            // At r=1: 1 + c*ln(2) ≈ 1 + 0.693c. We normalize so f(1)=1:
            // norm = (1 + c * ln(1+r)) / (1 + c * ln(2))
            let raw = 1.0 + c * (1.0 + r).ln();
            let pivot = 1.0 + c * 2.0_f64.ln();
            raw / pivot
        }
        NormType::Bidirectional { c } => {
            // Penalizes deviation from avgdl in both directions
            // norm = 1 + c * (ln(r))^2. At r=1: norm=1. Both r<1 and r>1 increase norm.
            let log_r = r.ln();
            1.0 + c * log_r * log_r
        }
    }
}

/// Compute IDF using specified mode
#[inline]
pub fn compute_idf(mode: IdfMode, df: u64, n: u64) -> f64 {
    match mode {
        IdfMode::Standard => idf_standard(df, n),
        IdfMode::Atire => (n as f64 / df as f64).ln(),
        IdfMode::Squared => { let base = idf_standard(df, n); base * base },
        IdfMode::Smoothed => ((n as f64 + 1.0) / (df as f64 + 1.0)).ln(),
    }
}

/// Transform TF using specified mode
#[inline]
pub fn compute_tf(mode: TfMode, tf: f64) -> f64 {
    match mode {
        TfMode::Standard => tf,
        TfMode::Log => (1.0 + tf).ln(),
        TfMode::DoubleLog => (1.0 + (1.0 + tf).ln()).ln(),
        TfMode::Capped(cap) => tf.min(cap),
    }
}

/// Generic BM25-family scorer. Composes norm, TF, and IDF modes.
pub fn score_generic(config: &ScoringConfig, tf: f64, df: u64, dl: u64, avgdl: f64, n: u64) -> f64 {
    let r = dl as f64 / avgdl;
    let idf = compute_idf(config.idf_mode, df, n);
    let max_idf = (n as f64).ln().max(1.0);
    let idf_ratio = (idf / max_idf).clamp(0.0, 1.0);
    let len_norm = compute_norm(config.norm, r, idf_ratio);
    let tf_val = compute_tf(config.tf_mode, tf);
    let tf_component = (tf_val * (config.k1 + 1.0)) / (tf_val + config.k1 * len_norm) + config.delta;
    idf * tf_component
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bm25_basic() {
        let score = bm25(3.0, 10, 100, 120.0, 1000, 1.2, 0.75);
        assert!(score > 0.0);
    }

    #[test]
    fn bm25_plus_geq_bm25() {
        let s1 = bm25(1.0, 5, 500, 100.0, 10000, 1.2, 0.75);
        let s2 = bm25_plus(1.0, 5, 500, 100.0, 10000, 1.2, 0.75, 1.0);
        assert!(s2 >= s1, "BM25+ must be >= BM25");
    }

    #[test]
    fn bm199_clean_basic() {
        let score = bm199(3.0, 10, 100, 120.0, 1000, 1.8);
        assert!(score > 0.0);
    }

    #[test]
    fn bm199_clean_vs_bm25_at_avgdl() {
        // At dl == avgdl: sqrt(1.0) = 1.0 and (1-b+b*1) = 1.0
        // So the only difference is k1
        let s25 = bm25(2.0, 10, 100, 100.0, 1000, 1.2, 0.75);
        let s199 = bm199(2.0, 10, 100, 100.0, 1000, 1.2);
        assert!((s25 - s199).abs() < 1e-10, "At avgdl with same k1, BM25 and BM199 must agree");
    }

    #[test]
    fn bm199_one_param_fewer() {
        // BM199 has 6 params, BM25 has 7 (extra `b`)
        // At dl=300, avgdl=100: sqrt(3.0)=1.732 vs (0.25+0.75*3)=2.5
        // BM199 penalizes long docs LESS than BM25
        let s25 = bm25(1.0, 10, 300, 100.0, 1000, 1.8, 0.75);
        let s199 = bm199(1.0, 10, 300, 100.0, 1000, 1.8);
        assert!(s199 > s25, "BM199 should penalize long docs less than BM25");
    }

    #[test]
    fn bm199_exp_basic() {
        let p = Bm199Params::default();
        let score = p.score_term(3.0, 10, 100, 120.0, 1000);
        assert!(score > 0.0);
    }

    #[test]
    fn bm199_coverage_bonus() {
        let p = Bm199Params::default();
        // 2 query terms, doc matches both
        let s1 = p.score_document(&[(10, 2), (20, 3)], 100, 120.0, 1000, 2);
        // 2 query terms, doc matches only 1
        let s2 = p.score_document(&[(10, 2), (20, 0)], 100, 120.0, 1000, 2);
        assert!(s1 > s2, "Full coverage should score higher");
    }
}
