2026-04-05T04:12:07.212550Z ERROR codex_core_skills::loader: failed to stat skills entry /Users/biobook/.codex/skills/nano-banana-pro/bin (symlink): No such file or directory (os error 2)
2026-04-05T04:12:07.212708Z ERROR codex_core_skills::loader: failed to stat skills entry /Users/biobook/.codex/skills/nano-banana-pro/src (symlink): No such file or directory (os error 2)
Reading additional input from stdin...
OpenAI Codex v0.118.0 (research preview)
--------
workdir: /Users/biobook/Projects/bm199
model: gpt-5.4
provider: openai
approval: never
sandbox: workspace-write [workdir, /tmp, $TMPDIR, /Users/biobook/.codex/memories]
reasoning effort: xhigh
reasoning summaries: none
session id: 019d5bd7-42e6-7b11-ab7c-92508f49ec8a
--------
user
Draft the Abstract, Introduction, and Analysis sections for a research paper about BM199.

BM199 replaces BM25's linear length normalization (1-b+b*dl/avgdl) with parameter-free sqrt(dl/avgdl).

Results on 7 held-out BEIR datasets:
  BM199 wins 6/7, avg +6.0% nDCG@10 over BM25
  Biggest wins: Touché +27%, Climate-FEVER +33% (long-doc datasets)
  Only loss: Quora -0.3% (very short docs, avgdl=6)

Mathematical insight: BM25 linear norm and sqrt cross at dl/avgdl=0.034 and 3.30 (with k1 adjustment). So BM199 is gentler on docs >3.3x average length — exactly the long argumentative/scientific documents where BM25 over-penalizes.

The tf^beta adaptive saturation is a no-op at tf=1 (common case). The entire improvement comes from sqrt.

Prior work: Singhal (1996) pivoted norm is linear. BM25+/BM25L (2011) keep linear norm. DFR family uses different framework. No known prior publication of sqrt in BM25 denominator.

Write in academic SIGIR style. Abstract should be exactly 200 words. Introduction should motivate why linear norm is the problem BM25 has had for 30 years. Analysis should include the mathematical crossing-point derivation.

Output as markdown.
codex
I’m checking the exact BM25-family formulas and prior-work claims first, so the draft lands in SIGIR style without overstating novelty. Then I’ll write the three sections directly in markdown with the crossing-point derivation spelled out.
web search: 
web search: Singhal 1996 pivoted document length normalization linear BM25 paper pdf
web search: 
web search: BM25L formula pdf long documents very long BM25 fails
web search: 
web search: BM25+ lower-bounding term frequency normalization formula linear dl/avgdl pdf
web search: 
web search: "sqrt(dl/avgdl)" BM25
web search: 
web search: Robertson Zaragoza Probabilistic Relevance Framework BM25 and Beyond pdf 2009
codex
I verified the prior-work framing enough to make the novelty claim defensible: pivoted normalization is linear, BM25L/BM25+ diagnose long-document over-penalization but keep the linear dependence, and DFR is a separate framework. I’m drafting the sections now and I’ll keep the novelty sentence as “to our knowledge” rather than absolute.
codex
## Abstract

BM25 has remained the standard lexical ranking baseline for decades, yet its document length normalization is still linear in relative length, scaling term frequency saturation by 1-b+b*dl/avgdl. We revisit this design choice and introduce BM199, a minimal variant that replaces the linear factor with the parameter free function sqrt(dl/avgdl). This single change makes length normalization gentler for documents that are substantially longer than the corpus average while retaining strong regularization for very short documents. Across seven held out BEIR datasets, BM199 outperforms BM25 on six, improving nDCG@10 by 6.0% on average. Gains are largest on long document collections, including Touché (+27%) and Climate-FEVER (+33%), while the only loss is a negligible -0.3% on Quora, whose documents are extremely short (avgdl = 6). The effect is attributable to the new length normalization rather than altered term frequency saturation: the adaptive tf^beta component is a no op when tf=1, the dominant case in sparse retrieval. Analytically, the linear and square root penalties cross after k1 adjustment near dl/avgdl = 0.034 and 3.30, explaining BM199's advantage on long documents. These results indicate that BM25's long standing weakness is not sparse lexical matching itself, but the functional form of its length penalty in ranking.

## Introduction

BM25 remains the default sparse retrieval baseline for ad hoc search, first-stage ranking, and hybrid pipelines. Its longevity is a strength, but it also means that any structural weakness in the model can persist for decades through force of convention. The most under-examined part of BM25 is its document length normalization term. If we write relative length as \(x = dl / avgdl\), BM25 penalizes length through the linear factor \(1 - b + bx\). That choice is not merely a tunable heuristic; it encodes a specific assumption that each additional unit of relative document length should incur the same additional penalty, regardless of whether a document is slightly above average length or several times longer.

This linearity is difficult to justify for the kinds of collections on which modern retrieval systems are deployed. Long argumentative, scientific, and evidence-heavy documents are often long because they contain more claims, more supporting passages, and more opportunities for relevant lexical evidence to appear. A linear penalty treats these additional tokens as if they were uniformly verbosity. Tuning \(b\) cannot solve that problem. Changing \(b\) only changes the slope of the line, not its curvature: \(\frac{d}{dx}(1 - b + bx) = b\) is constant everywhere. Thus, for roughly thirty years, BM25 has inherited a rigid bias against documents whose length substantially exceeds the corpus average.

Prior work has repeatedly identified the symptom while leaving the functional form largely intact. Pivoted document length normalization is linear (Singhal et al., 1996). BM25L and BM25+ explicitly observe that BM25 can over-penalize very long documents, but address the issue by shifting or lower-bounding the score rather than replacing the linear dependence on \(dl / avgdl\) (Lv and Zhai, 2011a; Lv and Zhai, 2011b). The DFR family addresses length effects within a different probabilistic framework rather than by modifying BM25's denominator directly (Amati and Van Rijsbergen, 2002). To our knowledge, prior published BM25 variants have not studied the simple substitution of the linear normalizer with \(\sqrt{dl / avgdl}\).

This paper studies exactly that substitution. BM199 replaces BM25's \(1 - b + b\,dl/avgdl\) term with the parameter free function \(\sqrt{dl/avgdl}\). The modification is intentionally minimal: it removes one parameter, preserves the overall BM25 structure, and changes only the geometry of length normalization. The empirical pattern is consistent with the hypothesis that BM25's main weakness is linear over-penalization. On seven held-out BEIR datasets, BM199 outperforms BM25 on six, improving average nDCG@10 by 6.0%, with especially large gains on Touché (+27%) and Climate-FEVER (+33%). The only loss is a negligible -0.3% on Quora, a collection with extremely short documents (\(avgdl = 6\)). The central claim is correspondingly narrow but consequential: most of the improvement comes not from rethinking sparse term weighting in general, but from replacing a three-decade-old linear length penalty with a square-root one.

## Analysis

To isolate the source of the gain, it is useful to separate term-frequency saturation from length normalization. Let \(x = dl / avgdl\). For a single query term, BM25 uses the denominator

\[
D_{25}(tf, x) = tf + k_1(1 - b + bx).
\]

BM199 replaces only the length term:

\[
D_{199}(tf, x) = tf^\beta + k_1' \sqrt{x}.
\]

The key observation is that the adaptive saturation term is inert in the most common sparse-retrieval case. When \(tf = 1\), we have \(1^\beta = 1\), so

\[
D_{199}(1, x) = 1 + k_1' \sqrt{x}.
\]

Thus, for singleton term matches, the entire difference between BM25 and BM199 reduces to the shape of the length penalty. This is important because singleton matches dominate lexical retrieval; consequently, the observed effectiveness gains are best interpreted as a length-normalization effect, not a general term-frequency effect.

The geometric difference is immediate. BM25's penalty grows linearly with slope \(b\). BM199's penalty grows as a square root, with diminishing marginal cost:

\[
\frac{d}{dx}(1 - b + bx) = b,
\qquad
\frac{d}{dx}\bigl(\rho \sqrt{x}\bigr) = \frac{\rho}{2\sqrt{x}},
\]

where \(\rho = k_1' / k_1\) absorbs the fitted \(k_1\) adjustment between the two models. BM25 therefore applies a constant marginal penalty to additional length, whereas BM199 penalizes additional length less and less as documents become longer. That is exactly the behavior one would want if extra length in long documents increasingly reflects supporting evidence rather than pure verbosity.

The crossing-point derivation follows directly. Comparing the effective penalty terms after \(k_1\) adjustment, BM25 and BM199 are equal when

\[
1 - b + bx = \rho \sqrt{x}.
\]

Let \(y = \sqrt{x}\). Then \(x = y^2\), and the equality becomes

\[
b y^2 - \rho y + (1 - b) = 0.
\]

Solving the quadratic gives

\[
y_{\pm} = \frac{\rho \pm \sqrt{\rho^2 - 4b(1-b)}}{2b},
\qquad
x_{\pm} = y_{\pm}^2.
\]

For the setting used here, this yields two crossing points at

\[
x_- \approx 0.034
\qquad \text{and} \qquad
x_+ \approx 3.30.
\]

These roots give the interpretation of BM199's behavior. Around the corpus mean, BM199 is not simply "more permissive" than BM25; after \(k_1\) adjustment, it can be slightly stricter. But once document length exceeds roughly \(3.3 \times avgdl\), the square-root penalty becomes smaller than the linear one, and the advantage widens with length because \(x\) grows faster than \(\sqrt{x}\). There is also a second crossover at \(x \approx 0.034\), but that regime corresponds to vanishingly short documents and is unlikely to drive effectiveness in realistic corpora.

This geometry aligns closely with the empirical results. The largest gains appear on Touché and Climate-FEVER, precisely the kinds of collections in which relevant items are long argumentative or scientific documents. Those are the cases where BM25's linear penalty keeps increasing at a constant rate, while BM199 flattens. By contrast, Quora consists of extremely short documents (\(avgdl = 6\)), so most items never enter the long-document regime where BM199 should help most. A small regression there is therefore unsurprising. Taken together, the algebra and the ablation-like \(tf=1\) observation support a simple conclusion: BM199 improves retrieval because it corrects BM25's long-standing tendency to over-penalize genuinely informative long documents.

---

Reference basis for the prior-work framing: [Singhal et al. 1996](https://www.sigmod.org/publications/dblp/db/conf/sigir/SinghalBM96.html), [Robertson and Zaragoza 2009](https://www.nowpublishers.com/article/Details/INR-019), [Lv and Zhai 2011, BM25L](https://experts.illinois.edu/en/publications/when-documents-are-very-long-bm25-fails), [Lv and Zhai 2011, BM25+](https://experts.illinois.edu/en/publications/lower-bounding-term-frequency-normalization/), [Amati and Van Rijsbergen 2002](https://cir.nii.ac.jp/crid/1364233268518625408). Inference note: I did not find a prior published BM25 paper replacing the denominator's linear length term with `sqrt(dl/avgdl)`, so I phrased that claim as "to our knowledge."
tokens used
65,987
## Abstract

BM25 has remained the standard lexical ranking baseline for decades, yet its document length normalization is still linear in relative length, scaling term frequency saturation by 1-b+b*dl/avgdl. We revisit this design choice and introduce BM199, a minimal variant that replaces the linear factor with the parameter free function sqrt(dl/avgdl). This single change makes length normalization gentler for documents that are substantially longer than the corpus average while retaining strong regularization for very short documents. Across seven held out BEIR datasets, BM199 outperforms BM25 on six, improving nDCG@10 by 6.0% on average. Gains are largest on long document collections, including Touché (+27%) and Climate-FEVER (+33%), while the only loss is a negligible -0.3% on Quora, whose documents are extremely short (avgdl = 6). The effect is attributable to the new length normalization rather than altered term frequency saturation: the adaptive tf^beta component is a no op when tf=1, the dominant case in sparse retrieval. Analytically, the linear and square root penalties cross after k1 adjustment near dl/avgdl = 0.034 and 3.30, explaining BM199's advantage on long documents. These results indicate that BM25's long standing weakness is not sparse lexical matching itself, but the functional form of its length penalty in ranking.

## Introduction

BM25 remains the default sparse retrieval baseline for ad hoc search, first-stage ranking, and hybrid pipelines. Its longevity is a strength, but it also means that any structural weakness in the model can persist for decades through force of convention. The most under-examined part of BM25 is its document length normalization term. If we write relative length as \(x = dl / avgdl\), BM25 penalizes length through the linear factor \(1 - b + bx\). That choice is not merely a tunable heuristic; it encodes a specific assumption that each additional unit of relative document length should incur the same additional penalty, regardless of whether a document is slightly above average length or several times longer.

This linearity is difficult to justify for the kinds of collections on which modern retrieval systems are deployed. Long argumentative, scientific, and evidence-heavy documents are often long because they contain more claims, more supporting passages, and more opportunities for relevant lexical evidence to appear. A linear penalty treats these additional tokens as if they were uniformly verbosity. Tuning \(b\) cannot solve that problem. Changing \(b\) only changes the slope of the line, not its curvature: \(\frac{d}{dx}(1 - b + bx) = b\) is constant everywhere. Thus, for roughly thirty years, BM25 has inherited a rigid bias against documents whose length substantially exceeds the corpus average.

Prior work has repeatedly identified the symptom while leaving the functional form largely intact. Pivoted document length normalization is linear (Singhal et al., 1996). BM25L and BM25+ explicitly observe that BM25 can over-penalize very long documents, but address the issue by shifting or lower-bounding the score rather than replacing the linear dependence on \(dl / avgdl\) (Lv and Zhai, 2011a; Lv and Zhai, 2011b). The DFR family addresses length effects within a different probabilistic framework rather than by modifying BM25's denominator directly (Amati and Van Rijsbergen, 2002). To our knowledge, prior published BM25 variants have not studied the simple substitution of the linear normalizer with \(\sqrt{dl / avgdl}\).

This paper studies exactly that substitution. BM199 replaces BM25's \(1 - b + b\,dl/avgdl\) term with the parameter free function \(\sqrt{dl/avgdl}\). The modification is intentionally minimal: it removes one parameter, preserves the overall BM25 structure, and changes only the geometry of length normalization. The empirical pattern is consistent with the hypothesis that BM25's main weakness is linear over-penalization. On seven held-out BEIR datasets, BM199 outperforms BM25 on six, improving average nDCG@10 by 6.0%, with especially large gains on Touché (+27%) and Climate-FEVER (+33%). The only loss is a negligible -0.3% on Quora, a collection with extremely short documents (\(avgdl = 6\)). The central claim is correspondingly narrow but consequential: most of the improvement comes not from rethinking sparse term weighting in general, but from replacing a three-decade-old linear length penalty with a square-root one.

## Analysis

To isolate the source of the gain, it is useful to separate term-frequency saturation from length normalization. Let \(x = dl / avgdl\). For a single query term, BM25 uses the denominator

\[
D_{25}(tf, x) = tf + k_1(1 - b + bx).
\]

BM199 replaces only the length term:

\[
D_{199}(tf, x) = tf^\beta + k_1' \sqrt{x}.
\]

The key observation is that the adaptive saturation term is inert in the most common sparse-retrieval case. When \(tf = 1\), we have \(1^\beta = 1\), so

\[
D_{199}(1, x) = 1 + k_1' \sqrt{x}.
\]

Thus, for singleton term matches, the entire difference between BM25 and BM199 reduces to the shape of the length penalty. This is important because singleton matches dominate lexical retrieval; consequently, the observed effectiveness gains are best interpreted as a length-normalization effect, not a general term-frequency effect.

The geometric difference is immediate. BM25's penalty grows linearly with slope \(b\). BM199's penalty grows as a square root, with diminishing marginal cost:

\[
\frac{d}{dx}(1 - b + bx) = b,
\qquad
\frac{d}{dx}\bigl(\rho \sqrt{x}\bigr) = \frac{\rho}{2\sqrt{x}},
\]

where \(\rho = k_1' / k_1\) absorbs the fitted \(k_1\) adjustment between the two models. BM25 therefore applies a constant marginal penalty to additional length, whereas BM199 penalizes additional length less and less as documents become longer. That is exactly the behavior one would want if extra length in long documents increasingly reflects supporting evidence rather than pure verbosity.

The crossing-point derivation follows directly. Comparing the effective penalty terms after \(k_1\) adjustment, BM25 and BM199 are equal when

\[
1 - b + bx = \rho \sqrt{x}.
\]

Let \(y = \sqrt{x}\). Then \(x = y^2\), and the equality becomes

\[
b y^2 - \rho y + (1 - b) = 0.
\]

Solving the quadratic gives

\[
y_{\pm} = \frac{\rho \pm \sqrt{\rho^2 - 4b(1-b)}}{2b},
\qquad
x_{\pm} = y_{\pm}^2.
\]

For the setting used here, this yields two crossing points at

\[
x_- \approx 0.034
\qquad \text{and} \qquad
x_+ \approx 3.30.
\]

These roots give the interpretation of BM199's behavior. Around the corpus mean, BM199 is not simply "more permissive" than BM25; after \(k_1\) adjustment, it can be slightly stricter. But once document length exceeds roughly \(3.3 \times avgdl\), the square-root penalty becomes smaller than the linear one, and the advantage widens with length because \(x\) grows faster than \(\sqrt{x}\). There is also a second crossover at \(x \approx 0.034\), but that regime corresponds to vanishingly short documents and is unlikely to drive effectiveness in realistic corpora.

This geometry aligns closely with the empirical results. The largest gains appear on Touché and Climate-FEVER, precisely the kinds of collections in which relevant items are long argumentative or scientific documents. Those are the cases where BM25's linear penalty keeps increasing at a constant rate, while BM199 flattens. By contrast, Quora consists of extremely short documents (\(avgdl = 6\)), so most items never enter the long-document regime where BM199 should help most. A small regression there is therefore unsurprising. Taken together, the algebra and the ablation-like \(tf=1\) observation support a simple conclusion: BM199 improves retrieval because it corrects BM25's long-standing tendency to over-penalize genuinely informative long documents.

---

Reference basis for the prior-work framing: [Singhal et al. 1996](https://www.sigmod.org/publications/dblp/db/conf/sigir/SinghalBM96.html), [Robertson and Zaragoza 2009](https://www.nowpublishers.com/article/Details/INR-019), [Lv and Zhai 2011, BM25L](https://experts.illinois.edu/en/publications/when-documents-are-very-long-bm25-fails), [Lv and Zhai 2011, BM25+](https://experts.illinois.edu/en/publications/lower-bounding-term-frequency-normalization/), [Amati and Van Rijsbergen 2002](https://cir.nii.ac.jp/crid/1364233268518625408). Inference note: I did not find a prior published BM25 paper replacing the denominator's linear length term with `sqrt(dl/avgdl)`, so I phrased that claim as "to our knowledge."
