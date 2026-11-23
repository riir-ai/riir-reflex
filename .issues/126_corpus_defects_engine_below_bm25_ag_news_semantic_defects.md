# Issue 126 — corpus defects: the modelless engine scores BELOW the BM25 floor on ag_news and semantic_defects

**Status:** Open — filed 2026-11-19 by the 121 standing-sweep lane (zed-b7afc49e@4090; Bench 200, both runs byte-identical). Per issue 121's A2 law: engine < bm25 is a CORPUS-DEFECT finding.
**Lease closed:** amo-idle-housekeeping@amo, 1795148823, was expires 2026-11-22T04:30Z — lapsed unlanded-past-audit; the audit half landed 2026-11-20 per the tail row
**Claimed:** zed-a029893e@amo, 1795409900, expires 2026-11-25T02:00:00+00:00
Scope of the live claim: the implementation engagement (semantic_defects feature lane + corpus growth + the fresh two-run re-sweep); work starts at the 06:15Z wake turn (amo holds the i1197 calibration boot exclusive until then — the no-cargo-on-a-poisoned-window discipline; the ag_news overlay owner call stays parked for the owner either way)
(the per-suite error-analysis lane: on each failing suite extract the bm25-right/engine-wrong rows, classify noise-vs-feature-gap, land the audit record; the re-sweep rides a rig job after the audit names the fix shape)

## The findings (per-suite, from the run tables)

- **ag_news**: engine 0.8825 < bm25 0.8875 (−0.5pp) while bm25 beats every lexicon comparator by +14.25pp (nb/ridge flat 0.7450). The engine's hashed-lexicon halves are leaving real signal on the table where a lexical floor already ranks near-ceiling — candidate corpus noise rows OR a feature-extraction gap specific to ag_news's news-wire vocabulary.
- **semantic_defects**: engine 0.4412 < bm25 0.4706 (−2.9pp, 102 rows) — the synthetic fixture suite (programmatic gold) where bm25's exact-match retrieval beats the engine's noisy halves. Small-n caveat recorded.

## Fix shape (either side closes)

Per-suite corpus audit (noise rows / gold-consistency) or an engine feature-lane gap fix; re-run the standing sweep (the 121 flags) — the suite closes when engine >= bm25 on a fresh two-run record.

## Audit record LANDED 2026-11-20 (amo idle housekeeping lane; Bench 202, `658eaab`)

The audit half of the claim is DONE — `.benchmarks/202_i126_per_suite_error_analysis.md` + `202_i126_confusion_rows/` (first per-row record; Bench-200 lane reproduced byte-identically on amo, join 502/502). Verdicts per suite:

- **ag_news — gold-side noise-dominated at the margin; CORPUS-CLEAN is the named fix.** 4 CLEAR gold mislabels (106/196/248/336 — on each, the ENGINE's pick is content-correct and bm25 "wins" by agreeing with the mislabeled gold) + 2 genuine straddles (19/295) + 10 idf-token feature gaps + 1 short-text weakness (133). Relabeling the 4 rows moves engine 0.8925 / floor 0.8775 — the A2 gap closes +1.5pp with NO engine change. **Owner call owed:** whether the floor suite carries a local corrections overlay over the 4 rows (recorded, applied to BOTH lanes, keeps the A2 comparison honest) or keeps upstream-pure gold and accepts the −0.5pp residual; the straddles stay untouched either way.
- **semantic_defects — engine feature-gap-dominated; gold clean (0 clear noise).** 9/12 confusion rows are missing signal classes (negation/failure-propagation markers; bounds idioms), 2 bm25-near-ties, 1 ambiguous (eval:27 — authoring re-look). Root constraint: 3 docs per class (18-doc corpus) makes eval↔corpus agreement unigram-only — bm25's regime. **Named fix shape (implementation lane, GOAT-gated per house law):** (1) idf-weighted variant of the hashed-lexicon halves (or a BM25-shaped fallback vote blended in for tiny corpora) targeting the two missing signal classes; (2) grow the fixture corpus 3 → ~10+/class; (3) `eval:27` authoring re-look. Engine-side alone likely cannot close −2.9pp on 18 docs — the combination plausibly does.

Remaining: the implementation engagement (semantic_defects feature lane + corpus growth) and the ag_news overlay owner call; then the fresh two-run re-sweep closes the suite per the A2 law.
