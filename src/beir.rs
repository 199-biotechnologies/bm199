/// BEIR dataset loader — reads corpus.jsonl, queries.jsonl, qrels/test.tsv
/// Downloads datasets from HuggingFace if not cached locally.

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct CorpusEntry {
    _id: String,
    title: Option<String>,
    text: String,
}

#[derive(Deserialize)]
struct QueryEntry {
    _id: String,
    text: String,
}

/// Loaded BEIR dataset
pub struct BeirDataset {
    pub name: String,
    /// doc_id -> (title, text)
    pub corpus: Vec<(String, String)>,
    /// query_id -> query_text
    pub queries: Vec<(String, String)>,
    /// query_id -> { doc_id -> relevance }
    pub qrels: HashMap<String, HashMap<String, u32>>,
}

impl BeirDataset {
    /// Load from a local BEIR directory structure
    pub fn load(dir: &Path, name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let corpus_path = dir.join("corpus.jsonl");
        let queries_path = dir.join("queries.jsonl");
        let qrels_path = dir.join("qrels").join("test.tsv");

        // Load corpus
        let corpus = Self::load_corpus(&corpus_path)?;
        eprintln!("  Loaded {} documents", corpus.len());

        // Load queries
        let queries = Self::load_queries(&queries_path)?;
        eprintln!("  Loaded {} queries", queries.len());

        // Load qrels
        let qrels = Self::load_qrels(&qrels_path)?;
        eprintln!("  Loaded {} qrels entries", qrels.len());

        Ok(Self { name: name.to_string(), corpus, queries, qrels })
    }

    fn load_corpus(path: &Path) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut corpus = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() { continue; }
            let entry: CorpusEntry = serde_json::from_str(&line)?;
            let text = match entry.title {
                Some(t) if !t.is_empty() => format!("{} {}", t, entry.text),
                _ => entry.text,
            };
            corpus.push((entry._id, text));
        }
        Ok(corpus)
    }

    fn load_queries(path: &Path) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut queries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() { continue; }
            let entry: QueryEntry = serde_json::from_str(&line)?;
            queries.push((entry._id, entry.text));
        }
        Ok(queries)
    }

    fn load_qrels(path: &Path) -> Result<HashMap<String, HashMap<String, u32>>, Box<dyn std::error::Error>> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut qrels: HashMap<String, HashMap<String, u32>> = HashMap::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            if i == 0 && line.starts_with("query") { continue; } // skip header
            let parts: Vec<&str> = line.split('\t').collect();
            // Support both 3-column (query-id, corpus-id, score) and 4-column (query-id, Q0, corpus-id, score) formats
            let (qid, did, rel): (String, String, u32) = if parts.len() == 3 {
                (parts[0].to_string(), parts[1].to_string(), parts[2].parse().unwrap_or(0))
            } else if parts.len() >= 4 {
                (parts[0].to_string(), parts[2].to_string(), parts[3].parse().unwrap_or(0))
            } else {
                continue;
            };
            if rel > 0 {
                qrels.entry(qid).or_default().insert(did, rel);
            }
        }
        Ok(qrels)
    }
}

/// Simple whitespace + lowercase tokenizer (good enough for BM25 benchmarking)
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 1 && s.len() < 50)
        .map(|s| s.to_string())
        .collect()
}

/// Get BEIR data directory, downloading if needed
pub fn beir_data_dir() -> PathBuf {
    let dir = PathBuf::from("data/beir");
    fs::create_dir_all(&dir).ok();
    dir
}

/// Available BEIR datasets (small ones for fast iteration)
pub fn dataset_names() -> Vec<&'static str> {
    vec!["nfcorpus", "scifact", "fiqa"]
}
