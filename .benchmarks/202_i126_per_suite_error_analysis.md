# Bench 202 — issue 126 per-suite error analysis: ag_news + semantic_defects vs the BM25 floor

**Status:** RECORD — the audit half of issue 126's claim (amo idle housekeeping, 2026-11-20). NO FIX landed; no src/ change; this record + the confusion TSVs are the deliverable. Re-sweep rides a rig job after a fix shape is chosen.
**Lane:** `harness --suites ag_news,semantic_defects --bm25-floor --nb-select --ridge-select --dump-items` at HEAD `5520ffb`, then a throwaway `/tmp/i126_tool` bin (path-dep on the public `harness::suites` + `bm25_floor` modules) that reconstructs both suites through the SAME public prepare law (`stratified_split` → builder → option-key union; `train_docs` over the cal-split rest; `synth_by_name` for the synthetic family), recomputes the BM25 floor per row, and joins on `(case_id, q_idx)` against the harness dump.
**Reproduction proof:** every headline number reproduced byte-identically on amo — ag_news engine 0.8825 / floor 0.8875 (400 q, 19800 pool docs); semantic_defects engine 0.4412 / floor 0.4706 (102 q, 18 pool docs); cross-box identical to Bench 200's shikuwa run (Windows 4090). The join is complete (502/502 rows; 0 missing).
**PROVENANCE:** amo (M5 Ultra, quiet AC) · run 2026-11-20 · accuracy is deterministic (no latency claim) · datasets `.raw/datasets` local seat.
**Per-row data in Bench 200:** NONE — `200_standing_sweep_bm25_floor/run{1,2}/results.json` carries aggregates only (hard metrics, abstain, thresholds); the BM25 floor lane report is aggregate-only by design. This bench produced the first per-row record.

## The confusion sets (bm25-right ∧ engine-wrong)

- ag_news: **17 rows** (reverse: engine-right ∧ bm25-wrong = 15; both wrong = 30)
- semantic_defects: **12 rows** (reverse = 9; both wrong = **45** — 44% of the suite is wrong under BOTH lanes)

Rows: `.benchmarks/202_i126_confusion_rows/{ag_news,semantic_defects}{,_reverse}.tsv` (case_id, q_idx, gold, engine_pick, bm25_pick, engine_conf, bm25 vote weights, bm25 top-5 retrieved doc labels+scores, state text). Verification tables: `verification_run_TABLES.md`.

## ag_news verdict — gold-side noise-dominated at the margin, engine gaps beneath

Classification of the 17 confusion rows (content-judged; every row quoted in the TSV):

- **(a) Corpus-side: 6 rows** — 4 CLEAR gold-label errors + 2 genuine straddles:
  - `ag_news:106` gold=**world**, text "Stocks Climb on Drop in Consumer Prices… CPI… gasoline prices falling…" — a Business story. Engine said business (conf 0.0518 — its highest in the set); bm25's top-5 retrieved docs are mostly world-class.
  - `ag_news:196` gold=**world**, "Stock Prices Climb Ahead of Google IPO… Nasdaq… Dow… initial public offering" — Business. Engine business (conf 0.0725 — the set's highest); bm25 top docs world.
  - `ag_news:248` gold=**sci_tech**, "Caterpillar… acquire a remanufacturer of engines and automatic transmissions… employment base" — Business (industrial M&A, not tech). Engine business; bm25 top docs sci_tech.
  - `ag_news:336` gold=**business**, "Chavez victory confirmed… recall referendum… Organization of American States" — World/politics. Engine world; bm25 top docs business.
  - Ambiguous straddles: `ag_news:19` (HP earnings vs storage/servers — gold sci_tech defensible, business defensible), `ag_news:295` ("Producer sues for Rings profits" — entertainment; no clean home in the 4-class scheme; bm25's own margin world 90.0 vs sci_tech 82.1).
  - **Key mechanism:** on the 4 clear rows the ENGINE's pick is the content-correct class and bm25 "wins" only by agreeing with the mislabeled gold through incidental surface overlap (its top retrieved docs are mostly the wrong class). The floor's +0.5pp lead is substantially an artifact of gold noise.
- **(b) Gold-consistent, engine feature gap: 10 rows** — the missing signal is BM25's idf weighting of rare discriminative tokens against surface-word bulk: `ag_news:235` (coach/offensive coordinator → sports vs Stanford/hired), `ag_news:231` (doping → sports vs scandal/journalists), `ag_news:213` (cricket/Test match → sports vs government/security delegation), `ag_news:270` (LPGA/championship → sports vs dates/Kraft), `ag_news:365` (video games/Frogger/Pong → sci_tech vs "games" polysemy eaten by the sports half), `ag_news:323` (plasma TVs → sci_tech vs shipments/prices), `ag_news:18` (fossil/sediments → sci_tech vs Oxford/researcher), `ag_news:280` (suing/Attorney General/business practices → business vs electronics retailer), `ag_news:310` (e-commerce sales → business vs clicking/online; bm25 margin thin 200.4 vs 189.2), `ag_news:337` (Wall St./housing headline vs Olympics body — bm25 weighted the business core).
- **(c) Genuine engine weakness: 1 row** — `ag_news:133` "Kerry-Kerrey Confusion Trips Up Campaign… John Kerry, Bob Kerrey. It's easy to get confused." — a 13-token micro-article: hashed-lexicon features starve on tiny inputs; bm25 wins on the few rare name tokens.

**Quantified fix effect (corpus-clean path):** relabeling the 4 clear noise rows moves engine 353→357 (0.8925) and floor 355→351 (0.8775) — engine ≥ bm25 by +1.5pp and the suite closes on the A2 bar WITHOUT any engine change. The engine's conf ordering supports the content read: its three highest-confusion-set confidences are exactly the three clear-business noise rows.
**Recommended fix shape (ag_news): CORPUS CLEAN** — audit + relabel the four named rows at the fetch/authoring source (`seed`/mirror fix or a documented gold-override file, per wherever the ag_news mirror's gold lands), leave the two straddles as-is (they are defensible golds; any relabel there is an owner call), then re-run the standing sweep. No engine work is indicated for this suite at −0.5pp.

## semantic_defects verdict — engine feature-gap-dominated, gold clean

Classification of the 12 confusion rows:

- **(a) Corpus-side: 1 row, no clear noise** — `semantic_defects:eval:27` (gold off_by_one: `&history[history.len() - 1 ..]` under a "last two revisions" comment) is ALSO defensible as swapped_lookup (the code reads a different range than the documented intent — that taxonomy's criteria fit). Gold-consistent under off_by_one's own criteria; flag as ambiguous-gold, not an error. The programmatic gold (issue 061, audited, dual-grounding bar) shows: real noise ≈ 0.
- **(b) Gold-consistent, engine feature gap: 9 rows** — two named signal classes:
  - *Negation / failure-propagation markers* (bm25 keyed them idf-high; the engine's count halves diluted them): `eval:86` ("`.ok();` … accumulate **silently**" — engine said clean), `eval:100` ("`let _ = file.set_len(0);` … a **failed** truncate leaves the old tail…", engine swapped_lookup), `eval:101` ("`metrics.export().ok();` … unrecorded", engine swapped_lookup), `eval:35` ("`if !token.verified { grant(session); }`" negated guard vs "admit-when-verified" docstring, engine swallowed_error), `eval:38` ("`if a.hash != b.hash { merge }`" vs "merge when equal", engine unwrapped_none), `eval:70` ("rows[col][row] is **transposed**", engine inverted_condition), `eval:79` ("`ports[name]` … keys on `ports[alias]`", engine unwrapped_none).
  - *Bounds idioms*: `eval:17` ("`for i in 0 ..= samples.len()` … sentinel slot after the buffer", engine inverted_condition), `eval:18` ("final tile's end index lands outside the buffer", engine inverted_condition).
- **(c) Genuine engine weakness: 2 rows** — `eval:10` and `eval:19`: bm25's own vote margins are near-ties (clean 5.18 vs swallowed 5.02; off_by_one 15.53 vs clean 15.26) — heavy multi-class surface overlap; the engine picked plausible runner-ups (in its bm25-vote rank, the engine's pick was 2nd or 3rd in ALL 12 rows — it is losing argmaxes, not producing unrelated answers).

**Root constraint:** the corpus is **3 docs per class** (18 total) vs 102 eval fixtures authored 3-gram-fresh (zero shared word 3-grams with corpus∪cal) — so eval↔corpus agreement is UNIGRAM-only, which is precisely bm25's regime and precisely what the count-table/hashed-lexicon halves underweight. 45/102 both-wrong rows corroborate the corpus-size ceiling: neither lane can anchor a 6-way distinction on 18 docs.
**Recommended fix shape (semantic_defects): ENGINE FEATURE LANE + corpus growth**, in that order — (1) an idf-weighted variant of the hashed-lexicon halves (tf-idf scaled bag counts — or a BM25-shaped fallback vote blended into the scorer for tiny corpora), GOAT-gated per the house law; the two named signal classes (negation/failure markers `!`, `!=`, `.ok()`, `silently`, `transposed`; bounds idioms `..=`, `len()`-arithmetic) are the concrete recall targets; (2) grow the fixture corpus per class (3 → ~10+) so both lanes have more than docstring prose to key on; (3) re-examine `eval:27`'s gold (one-line authoring tweak if the owner agrees it straddles). Engine-side alone may not close −2.9pp; corpus growth + idf weighting together plausibly do.

## What this bench is NOT

- No engine change, no corpus change, no fix implementation (issue 126's audit half only).
- No relabel list is authoritative until a human/owner pass confirms the content reads above (the four clear rows are content-obvious; the TSV quotes each in full).
- The `both_wrong` populations (ag_news 30, semantic_defects 45) bound any fix: closing the A2 gap needs only the confusion rows, but suite accuracy ceilings are set by the both-wrong set + corpus size.
