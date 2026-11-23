# Bench 204 — i126 ag_news gold overlay: the deciding A2 record (fresh two-run, overlay posture)

**Status:** RECORD — the issue-126 verdict's step 4. The 15-row overlay (`src/harness/ag_news_gold_overlay.json`, applied at prepare time, both lanes, commit-range `2ac1fd5`..) measures its projected effect: **ag_news engine 0.9200 vs bm25 floor 0.9050 → +1.5pp — A2 CLOSED** (engine ≥ bm25, the suite's closing condition per issue 121's law). Two runs **byte-identical** (engine + bm25 + case digests). semantic_defects unchanged (0.5686 vs 0.5784, −0.9pp — the overlay does not touch it; the gap rides the pre-recorded engine lever, issue 126).

## The numbers (side by side, per the verdict's disclosure law)

| posture | engine | bm25 floor | gap | source |
|---|---|---|---|---|
| upstream-pure gold | 0.8825 (353/400) | 0.8875 (355/400) | −0.5pp (2/400, noise-level) | Bench 200 + the c80f8d28 re-sweep (`i126_resweep_run{1,2}`, byte-identical) |
| overlay (15 rows) | **0.9200 (368/400)** | **0.9050 (362/400)** | **+1.5pp (+6/400) — CLOSED** | this record (`i126_overlay_run{1,2}`, byte-identical) |

The measured overlay delta (+1.5pp) equals the symmetric audit's arithmetic projection EXACTLY (4 one-sided rows + 11 direction-neutral both-wrong rows; Bench 202 addendum) — the reconstruction was verified floor-exact before any row was read, so the closure is arithmetic the run confirmed, not a surprise.

## The run

```
cargo run --release --bin harness -- --skip-laya --suites ag_news,semantic_defects \
  --bm25-floor --nb-select --ridge-select --dump-items --out .benchmarks/i126_overlay_run1   # then run2
```

- HEAD: the overlay lane (gold_overlay module + prepare-time apply + the 3 gate tests); box amo, quiet AC, load 4.68→4.50, PROVENANCE in the run meta (Issue 021 law).
- The prepare log carries the disclosure line once per suite load: `gold overlay: 15 rows relabelled … applied to BOTH lanes …` (the Issue-126 verdict's disclosure law; the upstream-pure numbers live in the sources row above, never deleted).
- semantic_defects numbers byte-match the pre-overlay re-sweep (the overlay binds ag_news only — `OVERLAY_SUITE`, enforced by the id-prefix law + the apply gate).

## What closed, what remains

- **ag_news: A2 CLOSED** — the corpus-defect finding (engine < bm25) is resolved by the recorded gold corrections; the suite's floor row now reads an honest lexical floor (0.9050) against an engine that genuinely beats it (0.9200).
- **semantic_defects: OPEN** at −0.9pp — the engine lever (idf-weighted hashed-lexicon halves / a BM25-shaped fallback vote blended for tiny corpora), GOAT-gated, the 72-doc corpus the measurement baseline. The sweep's dumps (44 engine-wrong rows, census TSV) are its error-analysis feed.
- The overlay's drift law is armed: an upstream re-fetch that moves ids, fixes a row, or drifts a label refuses the prepare LOUD (the gate tests pin all three arms + the content of all 15 rows).

## Provenance

- Verdict: delegated-owner, session `f84657b0` (round 1 REVISE → conditions; round 2 AGREE) — recorded in issue 126.
- Audit: Bench 202 + the 202 addendum (symmetric 62-row audit; census TSVs; floor-reproduction proof).
- Implementation: `src/harness/gold_overlay.rs` + `src/harness/ag_news_gold_overlay.json` + the prepare wiring; tests `harness::gold_overlay::tests` (3/3) + the standing gates (12/12 at the landing).
- The scratch tool (`/tmp/i126_tool`, path-dep on the public seat + bm25 modules) is reproducible from the bench record's description; not committed (dev-time instrument, the 202 precedent).
