/// bm199-bench-all: Benchmark binary for autoresearch.
/// Outputs avg nDCG@10 as a single number.
/// Supports generic scoring configs via CLI for hypothesis testing.
///
/// Usage:
///   cargo run --release --bin bm199-bench-all
///   cargo run --release --bin bm199-bench-all -- --norm power --alpha 0.5 --k1 1.8
///   cargo run --release --bin bm199-bench-all -- --norm hinged --alpha 0.4 --k1 1.6
///   cargo run --release --bin bm199-bench-all -- --norm sigmoid --k1 1.4
///   cargo run --release --bin bm199-bench-all -- --norm log --k1 1.5 --tf log
///   cargo run --release --bin bm199-bench-all -- --norm linear --b 0.75 --k1 1.2

use bm199::beir::{self, BeirDataset, tokenize};
use bm199::eval;
use bm199::index::{Document, InvertedIndex};
use bm199::scorer::{NormType, TfMode, IdfMode, ScoringConfig};
use std::collections::HashMap;

fn parse_config(args: &[String]) -> ScoringConfig {
    let get = |flag: &str| -> Option<String> {
        args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
    };
    let getf = |flag: &str, default: f64| -> f64 {
        get(flag).and_then(|s| s.parse().ok()).unwrap_or(default)
    };

    let norm_name = get("--norm").unwrap_or_else(|| "power".to_string());
    let k1 = getf("--k1", 1.8);
    let alpha = getf("--alpha", 0.5);
    let alpha2 = getf("--alpha2", 0.5);
    let b = getf("--b", 0.75);
    let c = getf("--c", 1.0);
    let delta = getf("--delta", 0.0);
    let s_short = getf("--s-short", 0.75);
    let s_long = getf("--s-long", 0.5);

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
        "idfcond" => NormType::IdfConditioned { base_alpha: alpha, gamma: getf("--gamma", 0.3) },
        "hingedidf" => NormType::HingedIdf { base_alpha: alpha, gamma: getf("--gamma", 0.3) },
        _ => NormType::Power(0.5), // default to sqrt
    };

    let tf_name = get("--tf").unwrap_or_else(|| "standard".to_string());
    let tf_mode = match tf_name.as_str() {
        "log" => TfMode::Log,
        "dlog" => TfMode::DoubleLog,
        "capped" => TfMode::Capped(getf("--tf-cap", 5.0)),
        _ => TfMode::Standard,
    };

    let idf_name = get("--idf").unwrap_or_else(|| "standard".to_string());
    let idf_mode = match idf_name.as_str() {
        "atire" => IdfMode::Atire,
        "squared" => IdfMode::Squared,
        "smoothed" => IdfMode::Smoothed,
        _ => IdfMode::Standard,
    };

    ScoringConfig { k1, norm, tf_mode, idf_mode, delta }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = parse_config(&args);

    // Determine label for logging
    let label = args.iter().position(|a| a == "--label")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| format!("{:?}", config.norm));

    let data_dir = beir::beir_data_dir();
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
            let ranked = index.search_generic(&qtokens, &config);
            let doc_ids: Vec<String> = ranked.iter()
                .take(100)
                .map(|(idx, _)| index.doc_ids[*idx].clone())
                .collect();
            results_per_query.insert(qid.clone(), doc_ids);
        }

        let result = eval::evaluate_all(ds_name, &label, &results_per_query, &dataset.qrels);
        eprintln!("{}: nDCG@10={:.4}", ds_name, result.ndcg_at_10);
        total_ndcg += result.ndcg_at_10;
        ds_count += 1;
    }

    let avg = if ds_count > 0 { total_ndcg / ds_count as f64 } else { 0.0 };
    println!("{:.6}", avg);
}
