/// All scoring variants: BM25, BM25+, BM25L, BM199 (novel)
/// Each scorer takes the same inputs and returns a document score.

/// Standard BM25 (Lucene/Robertson)
pub fn bm25(tf: f64, df: u64, dl: u64, avgdl: f64, n: u64, k1: f64, b: f64) -> f64 {
    let idf = idf_standard(df, n);
    let tf_norm = (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * (dl as f64 / avgdl)));
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

/// BM199 — novel scoring algorithm
/// Blends: logarithmic length normalization (RankEvolve), adaptive TF saturation,
/// BM25+ delta floor, and Bayesian sigmoid calibration.
///
/// Parameters (tunable by autoresearch):
///   k1: TF saturation base (default 1.2)
///   b: length normalization strength (default 0.75)
///   delta: BM25+ lower-bounding floor (default 1.0)
///   beta_min/beta_max: adaptive saturation exponent range by IDF (default 0.7-1.0)
///   lambda_cov: coverage bonus weight (default 0.1)
///   alpha_sig/beta_sig: sigmoid calibration (default 1.0, 0.0)
///   log_base: base for logarithmic length normalization (default std::f64::consts::E)
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
            k1: 1.2,
            b: 0.75,
            delta: 1.0,
            beta_min: 0.7,
            beta_max: 1.0,
            lambda_cov: 0.1,
            alpha_sig: 1.0,
            beta_sig: 0.0,
            log_base: std::f64::consts::E,
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
    fn bm199_basic() {
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
