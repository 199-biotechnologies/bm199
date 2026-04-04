/// Evaluation metrics: nDCG@K, MAP, Recall@K, MRR, Precision@K
/// Standard IR evaluation following BEIR/MTEB conventions.

use std::collections::HashMap;

/// Compute nDCG@K
pub fn ndcg_at_k(ranked_doc_ids: &[String], qrels: &HashMap<String, u32>, k: usize) -> f64 {
    let dcg: f64 = ranked_doc_ids.iter().take(k).enumerate()
        .map(|(i, doc_id)| {
            let rel = *qrels.get(doc_id).unwrap_or(&0) as f64;
            (2f64.powf(rel) - 1.0) / (i as f64 + 2.0).log2()
        })
        .sum();

    let mut ideal_rels: Vec<u32> = qrels.values().copied().collect();
    ideal_rels.sort_unstable_by(|a, b| b.cmp(a));
    let idcg: f64 = ideal_rels.iter().take(k).enumerate()
        .map(|(i, &rel)| {
            (2f64.powf(rel as f64) - 1.0) / (i as f64 + 2.0).log2()
        })
        .sum();

    if idcg == 0.0 { 0.0 } else { dcg / idcg }
}

/// Compute Mean Average Precision
pub fn map(ranked_doc_ids: &[String], qrels: &HashMap<String, u32>) -> f64 {
    let mut num_relevant = 0;
    let mut sum_precision = 0.0;
    let total_relevant = qrels.len() as f64;

    for (i, doc_id) in ranked_doc_ids.iter().enumerate() {
        if qrels.contains_key(doc_id) {
            num_relevant += 1;
            sum_precision += num_relevant as f64 / (i + 1) as f64;
        }
    }

    if total_relevant == 0.0 { 0.0 } else { sum_precision / total_relevant }
}

/// Compute Recall@K
pub fn recall_at_k(ranked_doc_ids: &[String], qrels: &HashMap<String, u32>, k: usize) -> f64 {
    let total_relevant = qrels.len() as f64;
    if total_relevant == 0.0 { return 0.0; }

    let found: f64 = ranked_doc_ids.iter().take(k)
        .filter(|doc_id| qrels.contains_key(doc_id.as_str()))
        .count() as f64;

    found / total_relevant
}

/// Compute MRR (Mean Reciprocal Rank)
pub fn mrr(ranked_doc_ids: &[String], qrels: &HashMap<String, u32>) -> f64 {
    for (i, doc_id) in ranked_doc_ids.iter().enumerate() {
        if qrels.contains_key(doc_id) {
            return 1.0 / (i + 1) as f64;
        }
    }
    0.0
}

/// Full evaluation result for a single dataset
#[derive(Debug, Clone, serde::Serialize)]
pub struct EvalResult {
    pub dataset: String,
    pub variant: String,
    pub ndcg_at_10: f64,
    pub map: f64,
    pub recall_at_100: f64,
    pub mrr: f64,
    pub num_queries: usize,
}

/// Evaluate a scoring function across all queries in a dataset
pub fn evaluate_all(
    dataset_name: &str,
    variant_name: &str,
    results_per_query: &HashMap<String, Vec<String>>, // qid -> ranked doc_ids
    qrels: &HashMap<String, HashMap<String, u32>>,
) -> EvalResult {
    let mut total_ndcg = 0.0;
    let mut total_map = 0.0;
    let mut total_recall = 0.0;
    let mut total_mrr = 0.0;
    let mut count = 0usize;

    for (qid, ranked) in results_per_query {
        if let Some(q_rels) = qrels.get(qid) {
            if q_rels.is_empty() { continue; }
            total_ndcg += ndcg_at_k(ranked, q_rels, 10);
            total_map += map(ranked, q_rels);
            total_recall += recall_at_k(ranked, q_rels, 100);
            total_mrr += mrr(ranked, q_rels);
            count += 1;
        }
    }

    let c = count as f64;
    EvalResult {
        dataset: dataset_name.to_string(),
        variant: variant_name.to_string(),
        ndcg_at_10: if count > 0 { total_ndcg / c } else { 0.0 },
        map: if count > 0 { total_map / c } else { 0.0 },
        recall_at_100: if count > 0 { total_recall / c } else { 0.0 },
        mrr: if count > 0 { total_mrr / c } else { 0.0 },
        num_queries: count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ndcg_perfect_ranking() {
        let ranked = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut qrels = HashMap::new();
        qrels.insert("a".to_string(), 3);
        qrels.insert("b".to_string(), 2);
        qrels.insert("c".to_string(), 1);
        let score = ndcg_at_k(&ranked, &qrels, 10);
        assert!((score - 1.0).abs() < 1e-10, "Perfect ranking should give nDCG=1.0");
    }

    #[test]
    fn recall_partial() {
        let ranked = vec!["a".to_string(), "x".to_string(), "b".to_string()];
        let mut qrels = HashMap::new();
        qrels.insert("a".to_string(), 1);
        qrels.insert("b".to_string(), 1);
        qrels.insert("c".to_string(), 1);
        let r = recall_at_k(&ranked, &qrels, 3);
        assert!((r - 2.0/3.0).abs() < 1e-10);
    }
}
