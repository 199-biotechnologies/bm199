/// Per-query significance test: BM25 default vs Power(alpha, k1) on a dataset split.
/// Outputs per-query nDCG@10 for both scorers and a paired randomization p-value.
///
/// Usage: cargo run --release --bin significance -- --split validation --alpha 0.40 --k1 1.5

use bm199::beir::{self, BeirDataset, tokenize};
use bm199::eval;
use bm199::index::{Document, InvertedIndex};
use bm199::scorer::{NormType, TfMode, IdfMode, ScoringConfig};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |flag: &str| -> Option<String> {
        args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
    };
    let getf = |flag: &str, default: f64| -> f64 {
        get(flag).and_then(|s| s.parse().ok()).unwrap_or(default)
    };

    let split = get("--split").unwrap_or_else(|| "validation".to_string());
    let alpha = getf("--alpha", 0.40);
    let k1 = getf("--k1", 1.5);
    let n_perms: usize = get("--perms").and_then(|s| s.parse().ok()).unwrap_or(100_000);

    let config_power = ScoringConfig {
        k1,
        norm: NormType::Power(alpha),
        tf_mode: TfMode::Standard,
        idf_mode: IdfMode::Standard,
        delta: 0.0,
    };
    let config_bm25 = ScoringConfig {
        k1: 1.2,
        norm: NormType::Linear(0.75),
        tf_mode: TfMode::Standard,
        idf_mode: IdfMode::Standard,
        delta: 0.0,
    };

    let data_dir = bm199::beir::beir_data_dir();
    let datasets = match split.as_str() {
        "tuning" => bm199::beir::tuning_datasets(),
        "validation" | "heldout" => bm199::beir::validation_datasets(),
        "test" => bm199::beir::test_datasets(),
        _ => bm199::beir::dataset_names(),
    };

    eprintln!("=== Paired Randomization Test: BM25(k1=1.2,b=0.75) vs Power(α={},k1={}) ===", alpha, k1);
    eprintln!("Split: {}, Permutations: {}", split, n_perms);

    for ds_name in &datasets {
        let ds_path = data_dir.join(ds_name);
        if !ds_path.join("corpus.jsonl").exists() {
            eprintln!("SKIP {}", ds_name);
            continue;
        }

        let dataset = BeirDataset::load(&ds_path, ds_name).expect("load");
        let documents: Vec<Document> = dataset.corpus.iter()
            .map(|(id, text)| {
                let tokens = tokenize(text);
                let length = tokens.len() as u64;
                Document { id: id.clone(), tokens, length }
            })
            .collect();
        let index = InvertedIndex::build(documents);
        let query_tokens: Vec<(String, Vec<String>)> = dataset.queries.iter()
            .map(|(qid, text)| (qid.clone(), tokenize(text)))
            .collect();

        // Score with BM25
        let mut bm25_results: HashMap<String, Vec<String>> = HashMap::new();
        for (qid, qtokens) in &query_tokens {
            let ranked = index.search_generic(qtokens, &config_bm25);
            let doc_ids: Vec<String> = ranked.iter().take(100)
                .map(|(idx, _)| index.doc_ids[*idx].clone()).collect();
            bm25_results.insert(qid.clone(), doc_ids);
        }

        // Score with Power
        let mut power_results: HashMap<String, Vec<String>> = HashMap::new();
        for (qid, qtokens) in &query_tokens {
            let ranked = index.search_generic(qtokens, &config_power);
            let doc_ids: Vec<String> = ranked.iter().take(100)
                .map(|(idx, _)| index.doc_ids[*idx].clone()).collect();
            power_results.insert(qid.clone(), doc_ids);
        }

        // Per-query nDCG@10
        let bm25_pq = eval::per_query_ndcg(&bm25_results, &dataset.qrels);
        let power_pq = eval::per_query_ndcg(&power_results, &dataset.qrels);

        assert_eq!(bm25_pq.len(), power_pq.len(), "query count mismatch");
        let n_queries = bm25_pq.len();

        let bm25_scores: Vec<f64> = bm25_pq.iter().map(|(_, s)| *s).collect();
        let power_scores: Vec<f64> = power_pq.iter().map(|(_, s)| *s).collect();

        let bm25_mean = bm25_scores.iter().sum::<f64>() / n_queries as f64;
        let power_mean = power_scores.iter().sum::<f64>() / n_queries as f64;
        let delta = power_mean - bm25_mean;
        let rel_delta = if bm25_mean > 0.0 { delta / bm25_mean * 100.0 } else { 0.0 };

        // Win/loss/tie counts
        let mut wins = 0usize;
        let mut losses = 0usize;
        let mut ties = 0usize;
        for i in 0..n_queries {
            if (power_scores[i] - bm25_scores[i]).abs() < 1e-10 {
                ties += 1;
            } else if power_scores[i] > bm25_scores[i] {
                wins += 1;
            } else {
                losses += 1;
            }
        }

        // Significance test
        let p_value = eval::paired_randomization_test(&bm25_scores, &power_scores, n_perms);
        let sig = if p_value < 0.001 { "***" } else if p_value < 0.01 { "**" } else if p_value < 0.05 { "*" } else { "ns" };

        println!("{:<20} | n={:<6} | BM25={:.4} | Power={:.4} | Δ={:+.4} ({:+.1}%) | W/L/T={}/{}/{} | p={:.4} {}",
            ds_name, n_queries, bm25_mean, power_mean, delta, rel_delta, wins, losses, ties, p_value, sig);
    }
}
