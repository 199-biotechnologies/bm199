/// bm199-eval: Run all scoring variants against BEIR datasets and output metrics.
/// Usage: cargo run --release --bin bm199-eval [-- --variant bm199]
/// Outputs JSON metrics to stdout (for autoresearch record).

use bm199::beir::{self, BeirDataset, tokenize};
use bm199::eval::{self, EvalResult};
use bm199::index::{Document, InvertedIndex, Bm25Variant};
use bm199::scorer::Bm199Params;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let variant_filter = args.iter()
        .position(|a| a == "--variant")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let params_path = args.iter()
        .position(|a| a == "--params")
        .and_then(|i| args.get(i + 1));

    // Load BM199 params from file if provided, otherwise use defaults
    let bm199_params = if let Some(path) = params_path {
        let content = std::fs::read_to_string(path).expect("Failed to read params file");
        serde_json::from_str(&content).expect("Failed to parse params JSON")
    } else if std::path::Path::new("bm199_params.json").exists() {
        let content = std::fs::read_to_string("bm199_params.json").expect("Failed to read bm199_params.json");
        serde_json::from_str(&content).expect("Failed to parse bm199_params.json")
    } else {
        Bm199Params::default()
    };

    let split = args.iter()
        .position(|a| a == "--split")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    let data_dir = beir::beir_data_dir();
    let datasets = match split {
        Some("tuning") => beir::tuning_datasets(),
        Some("heldout") => beir::heldout_datasets(),
        _ => beir::dataset_names(),
    };

    let mut all_results: Vec<EvalResult> = Vec::new();

    for ds_name in &datasets {
        let ds_path = data_dir.join(ds_name);
        if !ds_path.join("corpus.jsonl").exists() {
            eprintln!("SKIP {}: not downloaded. Run download_beir.sh first.", ds_name);
            continue;
        }

        eprintln!("=== Evaluating {} ===", ds_name);
        let dataset = BeirDataset::load(&ds_path, ds_name).expect("Failed to load dataset");

        // Build index
        let start = Instant::now();
        let documents: Vec<Document> = dataset.corpus.iter()
            .map(|(id, text)| {
                let tokens = tokenize(text);
                let length = tokens.len() as u64;
                Document { id: id.clone(), tokens, length }
            })
            .collect();
        let index = InvertedIndex::build(documents);
        let index_time = start.elapsed();
        eprintln!("  Index built in {:?} ({} docs, avgdl={:.1})", index_time, index.n, index.avgdl);

        // Tokenize queries
        let query_tokens: Vec<(String, Vec<String>)> = dataset.queries.iter()
            .map(|(qid, text)| (qid.clone(), tokenize(text)))
            .collect();

        let variants: Vec<(&str, Option<Bm25Variant>)> = match variant_filter {
            Some("bm199") => vec![("bm199", None)],
            Some("bm25") => vec![("bm25", Some(Bm25Variant::Standard))],
            Some("bm25+") => vec![("bm25+", Some(Bm25Variant::Plus))],
            Some("bm25l") => vec![("bm25l", Some(Bm25Variant::L))],
            _ => vec![
                ("bm25", Some(Bm25Variant::Standard)),
                ("bm25+", Some(Bm25Variant::Plus)),
                ("bm25l", Some(Bm25Variant::L)),
                ("bm199", None),
            ],
        };

        for (name, bm25_var) in &variants {
            let start = Instant::now();

            let mut results_per_query: HashMap<String, Vec<String>> = HashMap::new();

            for (qid, qtokens) in &query_tokens {
                let ranked = if let Some(variant) = bm25_var {
                    index.search_bm25(qtokens, 1.2, 0.75, *variant)
                } else {
                    index.search_bm199(qtokens, &bm199_params)
                };

                let doc_ids: Vec<String> = ranked.iter()
                    .take(100)
                    .map(|(idx, _)| index.doc_ids[*idx].clone())
                    .collect();
                results_per_query.insert(qid.clone(), doc_ids);
            }

            let query_time = start.elapsed();
            let qps = query_tokens.len() as f64 / query_time.as_secs_f64();

            let result = eval::evaluate_all(ds_name, name, &results_per_query, &dataset.qrels);
            eprintln!("  {} | nDCG@10={:.4} MAP={:.4} R@100={:.4} MRR={:.4} | {:.0} QPS ({:?})",
                name, result.ndcg_at_10, result.map, result.recall_at_100, result.mrr,
                qps, query_time);
            all_results.push(result);
        }
    }

    // Compute average across datasets for each variant
    let variants_seen: Vec<String> = all_results.iter().map(|r| r.variant.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();
    let datasets_seen: Vec<String> = all_results.iter().map(|r| r.dataset.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();

    eprintln!("\n=== AVERAGES ===");
    let mut avg_results: Vec<serde_json::Value> = Vec::new();
    for variant in &variants_seen {
        let variant_results: Vec<&EvalResult> = all_results.iter().filter(|r| &r.variant == variant).collect();
        let n = variant_results.len() as f64;
        if n == 0.0 { continue; }
        let avg_ndcg = variant_results.iter().map(|r| r.ndcg_at_10).sum::<f64>() / n;
        let avg_map = variant_results.iter().map(|r| r.map).sum::<f64>() / n;
        let avg_recall = variant_results.iter().map(|r| r.recall_at_100).sum::<f64>() / n;
        let avg_mrr = variant_results.iter().map(|r| r.mrr).sum::<f64>() / n;
        eprintln!("  {} | avg nDCG@10={:.4} avg MAP={:.4} avg R@100={:.4} avg MRR={:.4}",
            variant, avg_ndcg, avg_map, avg_recall, avg_mrr);
        avg_results.push(serde_json::json!({
            "variant": variant,
            "avg_ndcg_at_10": (avg_ndcg * 10000.0).round() / 10000.0,
            "avg_map": (avg_map * 10000.0).round() / 10000.0,
            "avg_recall_at_100": (avg_recall * 10000.0).round() / 10000.0,
            "avg_mrr": (avg_mrr * 10000.0).round() / 10000.0,
        }));
    }

    // Output JSON to stdout for autoresearch
    let output = serde_json::json!({
        "params": if variant_filter == Some("bm199") || variant_filter.is_none() {
            Some(&bm199_params)
        } else {
            None
        },
        "datasets": datasets_seen,
        "per_dataset": all_results,
        "averages": avg_results,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
