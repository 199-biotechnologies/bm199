# GPT Pro Review Round 2 — 2026-04-05

## Overall Verdict: REJECT as submitted

## Critical Findings (shortened)

### Stopword List is Wrong
Our list has ~90 words including question words (what, which, who, when, where, why, how).
Lucene EnglishAnalyzer default has only 33 words.
On QA datasets, we're stripping question words from queries — this changes retrieval behavior.

### FEVER Gain is Tokenizer Recovery
Our BM25 FEVER=0.503 vs Pyserini=0.651. Our power(0.40) FEVER=0.646 ≈ Pyserini BM25.
The +28% "improvement" on FEVER likely just recovers what a correct tokenizer gives BM25.

### Validation is Burned
Used to select sqrt (session 1), then alpha=0.40 (session 2). Not confirmatory.

### BM25 Baseline Under-tuned
Only swept b at k1=1.2. Need full k1×b joint tuning.

### Anti-Correlation Not Evidenced
Only 9 validation data points preserved, not the full 252-point matrix.

### IDF-Conditioned Sign Bug (already fixed)
Code did opposite of stated hypothesis.

### Softplus Pivot Bug (already fixed)
f(1)=0.528 instead of 1.0.

### Paper Draft Stale
Draft still says sqrt is winner. Code says power(0.40). Numbers conflict between files.

## Minimum Path to Publication
- Fix tokenizer (use Lucene stopwords, validate against Pyserini)
- Joint BM25 k1×b tuning
- Clean ablation (BM25 same-k1 vs power)
- Nested leave-one-dataset-out CV
- More test datasets (non-Wikipedia)
- Per-query significance tests
- Regenerate all tables from code
- Target: ECIR short/resource paper

## Novel Experiment Ideas from GPT Pro
1. Verbosity injection test
2. Scope vs verbosity counterfactuals
3. Cross-tokenizer robustness matrix
4. Doc-length decile analysis
5. Rare-term sign-flip study for IDF-conditioned
