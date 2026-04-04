/// bm199-bench-all: Benchmark binary focused on BM199 only.
/// Outputs the avg nDCG@10 as a single number for autoresearch record.
/// Usage: cargo run --release --bin bm199-bench-all
/// Only evaluates on TUNING datasets (not held-out) to prevent overfitting.

use bm199::beir::{self, BeirDataset, tokenize};
use bm199::eval;
use bm199::index::{Document, InvertedIndex};
use bm199::scorer::Bm199Params;
use std::collections::HashMap;

fn main() {
    let params: Bm199Params = if std::path::Path::new("bm199_params.json").exists() {
        let content = std::fs::read_to_string("bm199_params.json").expect("read params");
        serde_json::from_str(&content).expect("parse params")
    } else {
        Bm199Params::default()
    };

    let data_dir = beir::beir_data_dir();
    // ONLY tuning datasets — held-out is never touched during optimization
    let datasets = beir::tuning_datasets();
    let mut total_ndcg = 0.0;
    let mut ds_count = 0;

    for ds_name in &datasets {
        let ds_path = data_dir.join(ds_name);
        if !ds_path.join("corpus.jsonl").exists() { continue; }

        let dataset = BeirDataset::load(&ds_path, ds_name).expect("load");
        let documents: Vec<Document> = dataset.corpus.iter()
            .map(|(id, text)| {
                let tokens = tokenize(text);
                let length = tokens.len() as u64;
                Document { id: id.clone(), tokens, length }
            })
            .collect();
        let index = InvertedIndex::build(documents);

        let mut results_per_query: HashMap<String, Vec<String>> = HashMap::new();
        for (qid, text) in &dataset.queries {
            let qtokens = tokenize(text);
            let ranked = index.search_bm199(&qtokens, &params);
            let doc_ids: Vec<String> = ranked.iter()
                .take(100)
                .map(|(idx, _)| index.doc_ids[*idx].clone())
                .collect();
            results_per_query.insert(qid.clone(), doc_ids);
        }

        let result = eval::evaluate_all(ds_name, "bm199", &results_per_query, &dataset.qrels);
        eprintln!("{}: nDCG@10={:.4}", ds_name, result.ndcg_at_10);
        total_ndcg += result.ndcg_at_10;
        ds_count += 1;
    }

    let avg = if ds_count > 0 { total_ndcg / ds_count as f64 } else { 0.0 };
    // Print ONLY the metric value for autoresearch record
    println!("{:.6}", avg);
}
