#!/bin/bash
# Download BEIR datasets (small ones for fast benchmarking)
# NFCorpus: 3.6K docs, SciFact: 5.2K docs, FiQA: 57K docs

set -euo pipefail
DATA_DIR="data/beir"
mkdir -p "$DATA_DIR"

BASE_URL="https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets"

for DATASET in nfcorpus scifact fiqa; do
    if [ -d "$DATA_DIR/$DATASET" ] && [ -f "$DATA_DIR/$DATASET/corpus.jsonl" ]; then
        echo "✓ $DATASET already downloaded"
        continue
    fi
    echo "Downloading $DATASET..."
    curl -sL "$BASE_URL/$DATASET.zip" -o "/tmp/$DATASET.zip"
    unzip -qo "/tmp/$DATASET.zip" -d "$DATA_DIR/"
    rm "/tmp/$DATASET.zip"
    echo "✓ $DATASET downloaded to $DATA_DIR/$DATASET"
done

echo ""
echo "All datasets ready:"
for DATASET in nfcorpus scifact fiqa; do
    DOCS=$(wc -l < "$DATA_DIR/$DATASET/corpus.jsonl" 2>/dev/null || echo "?")
    QUERIES=$(wc -l < "$DATA_DIR/$DATASET/queries.jsonl" 2>/dev/null || echo "?")
    echo "  $DATASET: $DOCS docs, $QUERIES queries"
done

# Additional datasets for publication-grade evaluation
# Tuning: nfcorpus, scifact, fiqa, arguana (already have first 3)
# Held-out: trec-covid, climate-fever, fever, hotpotqa, nq, quora, scidocs, dbpedia-entity, webis-touche2020

for DATASET in arguana trec-covid climate-fever fever hotpotqa nq quora scidocs dbpedia-entity webis-touche2020; do
    if [ -d "$DATA_DIR/$DATASET" ] && [ -f "$DATA_DIR/$DATASET/corpus.jsonl" ]; then
        echo "✓ $DATASET already downloaded"
        continue
    fi
    echo "Downloading $DATASET..."
    curl -sL "$BASE_URL/$DATASET.zip" -o "/tmp/$DATASET.zip"
    unzip -qo "/tmp/$DATASET.zip" -d "$DATA_DIR/"
    rm "/tmp/$DATASET.zip"
    echo "✓ $DATASET downloaded to $DATA_DIR/$DATASET"
done

echo ""
echo "All datasets ready:"
for DATASET in nfcorpus scifact fiqa arguana trec-covid climate-fever fever hotpotqa nq quora scidocs dbpedia-entity webis-touche2020; do
    if [ -f "$DATA_DIR/$DATASET/corpus.jsonl" ]; then
        DOCS=$(wc -l < "$DATA_DIR/$DATASET/corpus.jsonl" 2>/dev/null || echo "?")
        QUERIES=$(wc -l < "$DATA_DIR/$DATASET/queries.jsonl" 2>/dev/null || echo "?")
        echo "  $DATASET: $DOCS docs, $QUERIES queries"
    else
        echo "  $DATASET: MISSING"
    fi
done
