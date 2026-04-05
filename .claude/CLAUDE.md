# BM199 Project — Guardrails

## Scoring Function Invariants (MUST enforce)
1. **Pivot constraint**: ALL normalization functions MUST satisfy f(1) = 1.0 (within 1e-6). Run `validate_all_norms()` after ANY change to `compute_norm()`.
2. **IDF-conditioned sign**: Rare terms (high IDF ratio) must get LOWER alpha → LESS length normalization. Verify with the `idf_conditioned_sign_correctness` test.
3. **Monotonicity**: All norms must be monotonically non-decreasing for r > 0 (longer docs = more normalization).
4. **Run `cargo test` after ANY scorer change**. All 12+ tests must pass before committing.

## Dataset Protocol (MUST enforce)
1. **Test set (FEVER, HotpotQA) is SACRED** — never use for tuning, parameter selection, or structure selection.
2. **Validation set is BURNED for sqrt** — sqrt was previously selected using validation data. Any sqrt vs BM25 comparison on validation is contaminated.
3. **Report actual evaluated query count** — FEVER loads 123K queries but only 6,666 have qrels. Report the qrels count, not the loaded count.
4. **Compare against Pyserini BM25 numbers** — our custom tokenizer gives different results than Pyserini. Always report the gap.

## Known Tokenizer Gaps vs Pyserini
Our BM25 vs Pyserini flat BM25 (nDCG@10):
- HotpotQA: 0.589 vs 0.633 (we're 7% lower)
- Touché: 0.316 vs 0.442 (we're 29% lower!)
- NFCorpus: 0.327 vs 0.322 (close)
- FiQA: 0.256 vs 0.236 (we're higher)
These gaps mean our "improvements" over BM25 may partly compensate for tokenizer weakness.

## Paper Claims (MUST NOT overclaim)
1. sqrt in BM25 is NOT novel (Cummins & O'Riordan, 2009)
2. Do NOT report average gains dominated by one outlier dataset (FEVER)
3. Report per-dataset results alongside averages
4. Acknowledge custom tokenizer limitation in every results discussion
5. The +16.1% test result is on only 2 Wikipedia QA datasets — NOT representative of diverse retrieval

## Commit Standards
- Run `cargo test` before every commit
- Include result numbers in commit messages for traceability
- Save ALL experimental results to `paper/logs/` with dated filenames
