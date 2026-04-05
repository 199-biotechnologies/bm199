/// bm199-eval: Run scoring variants against BEIR datasets and output metrics.
/// Supports both legacy BM25 variants and generic scoring configs.
///
/// Usage:
///   cargo run --release --bin bm199-eval [-- --split tuning --variant bm25]
///   cargo run --release --bin bm199-eval [-- --split validation --scorer generic --norm hinged --alpha 0.7 --k1 1.5]

use bm199::beir::{self, BeirDataset, tokenize};
use bm199::eval::{self, EvalResult};
use bm199::index::{Document, InvertedIndex, Bm25Variant};
use bm199::scorer::{Bm199Params, NormType, TfMode, IdfMode, ScoringConfig};
use std::collections::HashMap;
use std::time::Instant;

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
}

fn get_f64(args: &[String], flag: &str, default: f64) -> f64 {
    get_arg(args, flag).and_then(|s| s.parse().ok()).unwrap_or(default)
}

fn parse_generic_config(args: &[String]) -> (String, ScoringConfig) {
    let norm_name = get_arg(args, "--norm").unwrap_or_else(|| "power".to_string());
    let k1 = get_f64(args, "--k1", 1.8);
    let alpha = get_f64(args, "--alpha", 0.5);
    let alpha2 = get_f64(args, "--alpha2", 0.5);
    let b = get_f64(args, "--b", 0.75);
    let c = get_f64(args, "--c", 1.0);
    let delta = get_f64(args, "--delta", 0.0);
    let s_short = get_f64(args, "--s-short", 0.75);
    let s_long = get_f64(args, "--s-long", 0.5);

    let norm = match norm_name.as_str() {
        "linear" => NormType::Linear(b),
        "power" => NormType::Power(alpha),
        "sqrt" => NormType::Power(0.5),
        "log" => NormType::Log,
        "sigmoid" => NormType::Sigmoid,
        "hinged" => NormType::Hinged(alpha),
        "asymmetric" => NormType::Asymmetric(alpha, alpha2),
        "saturation" => NormType::Saturation(c),
        "softplus" => NormType::Softplus,
        "dualpivot" => NormType::DualPivot { s_short, s_long, alpha_long: alpha },
        "idfcond" => NormType::IdfConditioned { base_alpha: alpha, gamma: get_f64(args, "--gamma", 0.3) },
        "hingedidf" => NormType::HingedIdf { base_alpha: alpha, gamma: get_f64(args, "--gamma", 0.3) },
        _ => NormType::Power(0.5),
    };

    let tf_mode = match get_arg(args, "--tf").as_deref() {
        Some("log") => TfMode::Log,
        Some("dlog") => TfMode::DoubleLog,
        Some("capped") => TfMode::Capped(get_f64(args, "--tf-cap", 5.0)),
        _ => TfMode::Standard,
    };

    let idf_mode = match get_arg(args, "--idf").as_deref() {
        Some("atire") => IdfMode::Atire,
        Some("squared") => IdfMode::Squared,
        Some("smoothed") => IdfMode::Smoothed,
        _ => IdfMode::Standard,
    };

    let label = get_arg(args, "--label").unwrap_or_else(|| {
        format!("{}(k1={},a={})", norm_name, k1, alpha)
    });

    (label, ScoringConfig { k1, norm, tf_mode, idf_mode, delta })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let scorer_mode = get_arg(&args, "--scorer").unwrap_or_else(|| "legacy".to_string());
    let variant_filter = get_arg(&args, "--variant");

    let bm25_k1 = get_f64(&args, "--bm25-k1", 1.2);
    let bm25_b = get_f64(&args, "--bm25-b", 0.75);
    let bm199_k1 = get_f64(&args, "--bm199-k1", 1.8);

    let bm199_params = if std::path::Path::new("bm199_params.json").exists() {
        let content = std::fs::read_to_string("bm199_params.json").expect("read");
        serde_json::from_str(&content).expect("parse")
    } else {
        Bm199Params::default()
    };

    let split = get_arg(&args, "--split");
    let data_dir = beir::beir_data_dir();
    let datasets = match split.as_deref() {
        Some("tuning") => beir::tuning_datasets(),
        Some("validation") | Some("heldout") => beir::validation_datasets(),
        Some("test") => beir::test_datasets(),
        _ => beir::dataset_names(),
    };

    // Parse generic config if in generic mode
    let generic_config = if scorer_mode == "generic" {
        Some(parse_generic_config(&args))
    } else {
        None
    };

    let mut all_results: Vec<EvalResult> = Vec::new();

    for ds_name in &datasets {
        let ds_path = data_dir.join(ds_name);
        if !ds_path.join("corpus.jsonl").exists() {
            eprintln!("SKIP {}: not downloaded.", ds_name);
            continue;
        }

        eprintln!("=== Evaluating {} ===", ds_name);
        let dataset = BeirDataset::load(&ds_path, ds_name).expect("Failed to load dataset");

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

        let query_tokens: Vec<(String, Vec<String>)> = dataset.queries.iter()
            .map(|(qid, text)| (qid.clone(), tokenize(text)))
            .collect();

        if let Some((ref label, ref config)) = generic_config {
            // Generic scorer mode
            let start = Instant::now();
            let mut results_per_query: HashMap<String, Vec<String>> = HashMap::new();

            for (qid, qtokens) in &query_tokens {
                let ranked = index.search_generic(qtokens, config);
                let doc_ids: Vec<String> = ranked.iter()
                    .take(100)
                    .map(|(idx, _)| index.doc_ids[*idx].clone())
                    .collect();
                results_per_query.insert(qid.clone(), doc_ids);
            }

            let query_time = start.elapsed();
            let qps = query_tokens.len() as f64 / query_time.as_secs_f64();
            let result = eval::evaluate_all(ds_name, label, &results_per_query, &dataset.qrels);
            eprintln!("  {} | nDCG@10={:.4} MAP={:.4} R@100={:.4} MRR={:.4} | {:.0} QPS ({:?})",
                label, result.ndcg_at_10, result.map, result.recall_at_100, result.mrr, qps, query_time);
            all_results.push(result);
        } else {
            // Legacy variant mode
            let variants: Vec<(&str, Option<Bm25Variant>)> = match variant_filter.as_deref() {
                Some("bm199") => vec![("bm199", Some(Bm25Variant::Bm199))],
                Some("bm199-exp") => vec![("bm199-exp", None)],
                Some("bm25") => vec![("bm25", Some(Bm25Variant::Standard))],
                Some("bm25+") => vec![("bm25+", Some(Bm25Variant::Plus))],
                Some("bm25l") => vec![("bm25l", Some(Bm25Variant::L))],
                Some("atire") => vec![("atire", Some(Bm25Variant::Atire))],
                Some("dlh13") => vec![("dlh13", Some(Bm25Variant::DLH13))],
                Some("tfidf") => vec![("tfidf", Some(Bm25Variant::TfIdf))],
                _ => vec![
                    ("bm25", Some(Bm25Variant::Standard)),
                    ("bm25+", Some(Bm25Variant::Plus)),
                    ("bm25l", Some(Bm25Variant::L)),
                    ("atire", Some(Bm25Variant::Atire)),
                    ("dlh13", Some(Bm25Variant::DLH13)),
                    ("tfidf", Some(Bm25Variant::TfIdf)),
                    ("bm199", Some(Bm25Variant::Bm199)),
                ],
            };

            for (name, bm25_var) in &variants {
                let start = Instant::now();
                let mut results_per_query: HashMap<String, Vec<String>> = HashMap::new();

                for (qid, qtokens) in &query_tokens {
                    let ranked = if let Some(variant) = bm25_var {
                        let (k1, b) = match variant {
                            Bm25Variant::Bm199 => (bm199_k1, 0.0),
                            _ => (bm25_k1, bm25_b),
                        };
                        index.search_bm25(qtokens, k1, b, *variant)
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
                    name, result.ndcg_at_10, result.map, result.recall_at_100, result.mrr, qps, query_time);
                all_results.push(result);
            }
        }
    }

    // Averages
    let variants_seen: Vec<String> = all_results.iter().map(|r| r.variant.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();
    let datasets_seen: Vec<String> = all_results.iter().map(|r| r.dataset.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();

    eprintln!("\n=== AVERAGES ===");
    let mut avg_results: Vec<serde_json::Value> = Vec::new();
    for variant in &variants_seen {
        let vr: Vec<&EvalResult> = all_results.iter().filter(|r| &r.variant == variant).collect();
        let n = vr.len() as f64;
        if n == 0.0 { continue; }
        let avg_ndcg = vr.iter().map(|r| r.ndcg_at_10).sum::<f64>() / n;
        let avg_map = vr.iter().map(|r| r.map).sum::<f64>() / n;
        let avg_recall = vr.iter().map(|r| r.recall_at_100).sum::<f64>() / n;
        let avg_mrr = vr.iter().map(|r| r.mrr).sum::<f64>() / n;
        eprintln!("  {} | avg nDCG@10={:.4} avg MAP={:.4} avg R@100={:.4} avg MRR={:.4}",
            variant, avg_ndcg, avg_map, avg_recall, avg_mrr);
        avg_results.push(serde_json::json!({
            "variant": variant,
            "avg_ndcg_at_10": (avg_ndcg * 10000.0).round() / 10000.0,
        }));
    }

    let output = serde_json::json!({
        "datasets": datasets_seen,
        "per_dataset": all_results,
        "averages": avg_results,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
