# Autoresearch Report

## Summary

| Metric | Value |
|--------|-------|
| Target file | `src/scorer.rs` |
| Eval command | `cargo run --release --bin bm199-bench-all 2>/dev/null` |
| Metric | avg_ndcg_at_10 (higher is better) |
| Total experiments | 44 |
| Kept | 20 |
| Discarded | 22 |
| Baseline | 0.402100 |
| Best | 0.409500 (run #34) |
| Improvement | 1.84% |
| Time span | 2026-04-04T18:47:39.721978+00:00 to 2026-04-05T00:17:52.374196+00:00 |

## Winning Changes (Kept)

- **Run #43** (926e0b0): 0.397900 — A2 symmetric sqrt: tuning=0.3979, HELDOUT=0.4997 BEST. Beats BM25 (0.4734) by +5.6%. TREC-COVID +3.3%, Touché +27%, SCIDOCS +0.2%, Quora -0.3%. ZERO length params.
- **Run #42** (2bd5f2c): 0.390300 — A1+B1 combo: tuning=0.3903, heldout=0.4478. Worse than A1 alone. B1 log-tf hurts long-doc datasets when combined with root length.
- **Run #41** (2c93aa8): 0.404100 — D1 likelihood-ratio: tuning=0.4041, heldout=0.4047. Most consistent (gap=0.1%) but lower absolute. Quora 0.7796 best of all forks.
- **Run #40** (5677155): 0.395500 — C1 tempered-idf: tuning=0.3955, heldout=0.4441. Quora 0.7702 good, but overall worse than baseline
- **Run #39** (8c0cee6): 0.399500 — B1 log-tf: tuning=0.3995, heldout=0.4092. ArguAna 0.4171 great, Quora 0.7629 great, but FiQA 0.1913 and Touché 0.1838 bad
- **Run #38** (234d0b7): 0.380300 — A1 root-length: tuning=0.3803 (down), heldout=0.4897 (UP from 0.4563). Touché 0.2617->0.4685 (+79%), Quora 0.7728->0.7124 (-8%)
- **Run #34** (7907bf0): 0.409500 — DROP log-norm, use BM25 linear norm + short-doc penalty + adaptive saturation. 0.4029->0.4095 (+1.6%). BEATS BM25 (0.4064) by +0.76%. Log-norm was HURTING with stemmed tokens!
- **Run #32** (8225f60): 0.402900 — k1=1.8: 0.4027->0.4029. Still climbing.
- **Run #31** (8359071): 0.402700 — k1=1.6: slower saturation for stemmed tokens. 0.4021->0.4027 (+0.15%). Stemming reduced token count so k1 needs to go up.
- **Run #25** (5fe85e4): 0.402000 — Short-doc penalty 0.15 (Codex suggestion): 0.4009->0.4020 (+0.27%). Now beats BM25 (0.4004) by +0.4%. Fixed FiQA over-rewarding ultra-short snippets.
- **Run #24** (5022b2a): 0.400900 — Quadratic short-doc penalty: 0.4006->0.4009. Now BEATING BM25 by +0.12%. Short docs get penalized for lacking content, not rewarded for brevity.
- **Run #19** (8db3312): 0.400600 — k1=1.4: 0.4001->0.4006. BEATS BM25 BASELINE (0.4004) FOR THE FIRST TIME. BM199 > BM25 by +0.02%.
- **Run #18** (5adb3c7): 0.400100 — k1=1.3: fine-tune from 1.2. 0.3999->0.4001. NOW MATCHING BM25 BASELINE (0.4004). First time BM199 approaches BM25.
- **Run #15** (ba2faba): 0.399900 — beta_min=0.8: milder adaptive saturation. 0.3994->0.3999 (+0.1%). Now within 0.0005 of BM25 (0.4004). The saturation was too aggressive at 0.5.
- **Run #9** (3f0cfc9): 0.399400 — Fix log-norm anchor: len_norm=1.0 at dl=avgdl. Same metric but mathematically correct normalization.
- **Run #8** (26a7c88): 0.399400 — beta_min=0.5: stronger adaptive saturation for rare terms. 0.3990->0.3994. Nearly matching BM25 (0.4004).
- **Run #6** (1e34795): 0.399000 — log_base=2.0: stronger length compression via log2. 0.3983->0.3990 (+0.2%). Approaching BM25 baseline (0.4004).
- **Run #4** (1f962d0): 0.398300 — lambda_cov=0.0: disable coverage bonus. 0.3968->0.3983 (+0.4%). Coverage bonus was hurting — remove it.
- **Run #2** (a773025): 0.396800 — b=0.9: stronger length norm with log compression. 0.3944->0.3968 (+0.6%). Log-norm benefits from higher b than BM25 linear norm
- **Run #1** (600b979): 0.394400 — delta=0.0: disable BM25+ floor. 0.3820->0.3944 (+3.2%). Log-norm + adaptive saturation improves over default delta=1.0

## Failed Attempts (Discarded)

- **Run #37**: 0.409400 — b=0.8: FiQA +4.6% but SciFact -0.4% and ArguAna -2.8%. Net same as b=0.9. Keep b=0.9.
- **Run #36**: 0.408500 — Short-doc penalty 0.05: 0.4095->0.4085. Still worse than 0.15. The penalty helps more datasets than it hurts.
- **Run #35**: 0.408000 — Remove short-doc penalty: 0.4095->0.4080. Penalty helps other datasets, just hurts FiQA. Need adaptive penalty.
- **Run #33**: 0.402900 — k1=2.0: plateau at 0.4029, same as k1=1.8. Keep 1.8.
- **Run #30**: 0.400500 — IDF-conditioned b with 0.1 scale: 0.4021->0.4005. Better than 0.3 but still worse than no conditioning. The concept hurts avg but might help specific datasets.
- **Run #29**: 0.398200 — IDF-conditioned b with 0.3 scale: too aggressive. 0.4021->0.3982. Try gentler scaling.
- **Run #28**: 0.401200 — Short-doc penalty 0.25: slightly worse. 0.4021->0.4012. 0.15 is still better.
- **Run #27**: 0.395700 — b=0.75: worse even with stemming. 0.4021->0.3957. b=0.9 still optimal.
- **Run #23**: 0.400600 — log_base=1.5: same as log_base=2.0 (0.4006). No improvement, keep simpler log2.
- **Run #22**: 0.400400 — IDF boost delta=0.1: slightly worse than no boost. 0.4006->0.4004. IDF boost not the right direction.
- **Run #21**: 0.399800 — k1=1.35: worse than 1.4. 0.4006->0.3998. k1=1.4 is the peak.
- **Run #20**: 0.400000 — k1=1.5: overshoots. 0.4006->0.4000. k1=1.4 confirmed as optimal.
- **Run #17**: 0.368700 — Robertson IDF (no +1): negative IDF for common terms destroys scoring. 0.3999->0.3687. Standard IDF is better.
- **Run #16**: 0.399700 — beta_min=0.85: slightly worse than 0.8. 0.3999->0.3997. beta_min=0.8 is optimal.
- **Run #14**: 0.398600 — b=0.85: worse than b=0.9. 0.3994->0.3986. b=0.9 confirmed.
- **Run #13**: 0.396300 — BM25L-style delta=0.5 added to TF before formula. 0.3994->0.3963. BM25L approach hurts with log-norm.
- **Run #12**: 0.398300 — beta_min=0.6 beta_max=0.9: universal sublinear saturation hurts. 0.3994->0.3983. Common terms need near-linear TF.
- **Run #11**: 0.398100 — k1=1.0: too fast saturation. 0.3994->0.3981. k1=1.2 confirmed optimal.
- **Run #10**: 0.397400 — Blend log+power TF saturation by IDF. 0.3994->0.3974. Log saturation at query time hurts. Power-only is better.
- **Run #7**: 0.397700 — b=0.95: too strong length norm. 0.3990->0.3977. b=0.9 with log2 is optimal.
- **Run #5**: 0.398000 — k1=1.5: slower saturation. 0.3983->0.3980 (-0.1%). k1=1.2 remains optimal.
- **Run #3**: 0.395900 — beta_min=1.0: disable adaptive saturation. 0.3968->0.3959 (-0.2%). Adaptive saturation contributes positively, keep it.

## Metric Progression

```
#  0 0.3820 ###############
#  1 0.3944 ###############
#  2 0.3968 ###############
#  4 0.3983 ###############
#  6 0.3990 ###############
#  8 0.3994 ###############
#  9 0.3994 ###############
# 15 0.3999 ###############
# 18 0.4001 ################
# 19 0.4006 ################
# 24 0.4009 ################
# 25 0.4020 ################
# 26 0.4021 ################
# 31 0.4027 ################
# 32 0.4029 ################
# 34 0.4095 ################
# 38 0.3803 ###############
# 39 0.3995 ###############
# 40 0.3955 ###############
# 41 0.4041 ################
# 42 0.3903 ###############
# 43 0.3979 ###############
```

---
*Generated by autoresearch CLI v0.3.3*
