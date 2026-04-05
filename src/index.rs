/// In-memory inverted index for benchmarking.
/// Not optimized for production — optimized for clarity and correctness
/// so autoresearch can iterate on the scoring function, not the index.

use rustc_hash::FxHashMap;
use std::collections::HashMap;

/// A document in the corpus
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub tokens: Vec<String>,
    pub length: u64,
}

/// Posting list entry
#[derive(Debug, Clone)]
pub struct Posting {
    pub doc_idx: usize,
    pub tf: u32,
}

/// Inverted index with corpus statistics
pub struct InvertedIndex {
    /// term -> posting list
    pub postings: FxHashMap<String, Vec<Posting>>,
    /// Document lengths
    pub doc_lengths: Vec<u64>,
    /// Document IDs (for mapping back to BEIR IDs)
    pub doc_ids: Vec<String>,
    /// Total documents
    pub n: u64,
    /// Average document length
    pub avgdl: f64,
    /// Document frequency per term
    pub df: FxHashMap<String, u64>,
    /// Collection frequency per term (total occurrences across all docs)
    pub cf: FxHashMap<String, u64>,
    /// Total tokens in corpus
    pub total_tokens: u64,
}

impl InvertedIndex {
    pub fn build(documents: Vec<Document>) -> Self {
        let n = documents.len() as u64;
        let total_len: u64 = documents.iter().map(|d| d.length).sum();
        let avgdl = if n > 0 { total_len as f64 / n as f64 } else { 1.0 };

        let mut postings: FxHashMap<String, Vec<Posting>> = FxHashMap::default();
        let mut df: FxHashMap<String, u64> = FxHashMap::default();
        let mut doc_lengths = Vec::with_capacity(documents.len());
        let mut doc_ids = Vec::with_capacity(documents.len());

        for (doc_idx, doc) in documents.iter().enumerate() {
            doc_lengths.push(doc.length);
            doc_ids.push(doc.id.clone());

            // Count term frequencies within this document
            let mut tf_map: HashMap<&str, u32> = HashMap::new();
            for token in &doc.tokens {
                *tf_map.entry(token.as_str()).or_insert(0) += 1;
            }

            for (term, tf) in tf_map {
                *df.entry(term.to_string()).or_insert(0) += 1;
                postings
                    .entry(term.to_string())
                    .or_default()
                    .push(Posting { doc_idx, tf });
            }
        }

        // Compute collection frequencies
        let mut cf: FxHashMap<String, u64> = FxHashMap::default();
        for (term, plist) in &postings {
            let total: u64 = plist.iter().map(|p| p.tf as u64).sum();
            cf.insert(term.clone(), total);
        }

        Self { postings, doc_lengths, doc_ids, n, avgdl, df, cf, total_tokens: total_len }
    }

    /// Retrieve documents matching query terms with BM25/BM25+/BM25L/BM199 scoring
    pub fn search_bm25(
        &self,
        query_tokens: &[String],
        k1: f64,
        b: f64,
        variant: Bm25Variant,
    ) -> Vec<(usize, f64)> {
        let mut scores: FxHashMap<usize, f64> = FxHashMap::default();

        for term in query_tokens {
            let df = match self.df.get(term) {
                Some(&d) => d,
                None => continue,
            };
            let postings = match self.postings.get(term) {
                Some(p) => p,
                None => continue,
            };

            for posting in postings {
                let tf = posting.tf as f64;
                let dl = self.doc_lengths[posting.doc_idx];

                let term_score = match variant {
                    Bm25Variant::Standard => {
                        crate::scorer::bm25(tf, df, dl, self.avgdl, self.n, k1, b)
                    }
                    Bm25Variant::Plus => {
                        crate::scorer::bm25_plus(tf, df, dl, self.avgdl, self.n, k1, b, 1.0)
                    }
                    Bm25Variant::L => {
                        crate::scorer::bm25l(tf, df, dl, self.avgdl, self.n, k1, b, 0.5)
                    }
                    Bm25Variant::Atire => {
                        crate::scorer::bm25_atire(tf, df, dl, self.avgdl, self.n, k1, b)
                    }
                    Bm25Variant::DLH13 => {
                        let cf = self.cf.get(term).copied().unwrap_or(1);
                        crate::scorer::dlh13(tf, dl, self.avgdl, self.n, cf)
                    }
                    Bm25Variant::QLD => {
                        let cf = self.cf.get(term).copied().unwrap_or(1);
                        crate::scorer::qld(tf, dl, cf, self.total_tokens, 2500.0)
                    }
                    Bm25Variant::TfIdf => {
                        crate::scorer::tfidf(tf, df, self.n)
                    }
                    Bm25Variant::Bm199 => {
                        crate::scorer::bm199(tf, df, dl, self.avgdl, self.n, k1)
                    }
                };

                *scores.entry(posting.doc_idx).or_insert(0.0) += term_score;
            }
        }

        let mut results: Vec<(usize, f64)> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Search with BM199 scoring
    pub fn search_bm199(
        &self,
        query_tokens: &[String],
        params: &crate::scorer::Bm199Params,
    ) -> Vec<(usize, f64)> {
        let mut doc_term_data: FxHashMap<usize, Vec<(u64, u64)>> = FxHashMap::default();

        // Gather (df, tf) per query term per document
        for term in query_tokens {
            let df = match self.df.get(term) {
                Some(&d) => d,
                None => {
                    // Term not in corpus — still need to record 0 tf for coverage calculation
                    continue;
                }
            };
            let postings = match self.postings.get(term) {
                Some(p) => p,
                None => continue,
            };

            for posting in postings {
                doc_term_data
                    .entry(posting.doc_idx)
                    .or_default()
                    .push((df, posting.tf as u64));
            }
        }

        let query_len = query_tokens.len();
        let mut results: Vec<(usize, f64)> = doc_term_data
            .into_iter()
            .map(|(doc_idx, term_data)| {
                let dl = self.doc_lengths[doc_idx];
                let raw = params.score_document(&term_data, dl, self.avgdl, self.n, query_len);
                (doc_idx, raw)
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

impl InvertedIndex {
    /// Generic search using ScoringConfig — for hypothesis testing
    pub fn search_generic(
        &self,
        query_tokens: &[String],
        config: &crate::scorer::ScoringConfig,
    ) -> Vec<(usize, f64)> {
        let mut scores: FxHashMap<usize, f64> = FxHashMap::default();

        for term in query_tokens {
            let df = match self.df.get(term) { Some(&d) => d, None => continue };
            let postings = match self.postings.get(term) { Some(p) => p, None => continue };

            for posting in postings {
                let tf = posting.tf as f64;
                let dl = self.doc_lengths[posting.doc_idx];
                let term_score = crate::scorer::score_generic(config, tf, df, dl, self.avgdl, self.n);
                *scores.entry(posting.doc_idx).or_insert(0.0) += term_score;
            }
        }

        let mut results: Vec<(usize, f64)> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Bm25Variant {
    Standard,
    Plus,
    L,
    Atire,
    DLH13,
    QLD,
    TfIdf,
    /// Clean BM199: sqrt length normalization, no b parameter
    Bm199,
}
