# Issue 126 — corpus defects: the modelless engine scores BELOW the BM25 floor on ag_news and semantic_defects

**Status:** Open — filed 2026-11-19 by the 121 standing-sweep lane (zed-b7afc49e@4090; Bench 200, both runs byte-identical). Per issue 121's A2 law: engine < bm25 is a CORPUS-DEFECT finding.
**Claimed:** amo-idle-housekeeping@amo, 1795148823, expires 2026-11-22T04:30Z
(the per-suite error-analysis lane: on each failing suite extract the bm25-right/engine-wrong rows, classify noise-vs-feature-gap, land the audit record; the re-sweep rides a rig job after the audit names the fix shape)

## The findings (per-suite, from the run tables)

- **ag_news**: engine 0.8825 < bm25 0.8875 (−0.5pp) while bm25 beats every lexicon comparator by +14.25pp (nb/ridge flat 0.7450). The engine's hashed-lexicon halves are leaving real signal on the table where a lexical floor already ranks near-ceiling — candidate corpus noise rows OR a feature-extraction gap specific to ag_news's news-wire vocabulary.
- **semantic_defects**: engine 0.4412 < bm25 0.4706 (−2.9pp, 102 rows) — the synthetic fixture suite (programmatic gold) where bm25's exact-match retrieval beats the engine's noisy halves. Small-n caveat recorded.

## Fix shape (either side closes)

Per-suite corpus audit (noise rows / gold-consistency) or an engine feature-lane gap fix; re-run the standing sweep (the 121 flags) — the suite closes when engine >= bm25 on a fresh two-run record.
