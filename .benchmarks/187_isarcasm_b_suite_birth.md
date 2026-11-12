# Bench 187 — isarcasm_b suite birth: the multilabel envelope (iSarcasmEval track B), the wave-2 residue's last machinery item (issue 114 T13)

**Date:** 2026-11-12 · **Repo:** riir-reflex (the harness/builder/fetch deltas land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` T13 (the wave-2 residue's iSarcasm track-B lane; rethink 060 R2b)
**Posture:** `LAYA_DEVICE=metal ./target/release/harness --suites isarcasm_b --laya-max-questions 0 --out .benchmarks/187_isarcasm_b_suite_birth` (release, laya-riir-metal; corpus-optional — laya-only, the modelless lane skips LOUD)
**PROVENANCE:** ⛔ preflight REFUSED at launch (load 8.63 > 6 — the `grafts_cell` sibling job is on the box; swap 4.01G, AC, powermode auto). **Latency NOT QUOTABLE — the run's own box-state stamps refuse it (start load 8.58 / end 8.05, both > 6).** Accuracy is the deliverable; the p50/p99s are posture-disclosed, never quotable baselines (the Bench-180–186 convention). The picks are deterministic (determinism_ok true, n=10 reruns clean), so the accuracy axis is load-insensitive.

## What landed

1. **The multilabel scorer** (`multilabel_f1_metrics` +
   `MultilabelMetrics` in `src/harness/metrics.rs`, 6 known-answer
   tests): one row per (LABEL, gold, probs); per label, pool its rows
   across all cases, pick by the house first-index argmax, read the
   positive-class F1 through `binary_f1_of` (positive = 1 — the
   ISARCASM_KEYS convention), macro = the sequential mean over the
   pinned label order (plain `+=` fold — the float-reduction
   determinism law). A missing label refuses loud (a skipped label
   would silently inflate the macro); empty input is `None`, never a
   fabricated rate. The seam (`multilabel_f1_of` +
   `MULTILABEL_SUITES`, the CASE_EXACT_SUITES law) walks BOTH lane
   assemblies from ONE home, reads the label index as the question's
   POSITION in the pinned per-case grid, and asserts the uniform
   per-case question grid — a ragged grid would silently mislabel the
   walk and refuses loud instead. CROSS-CHECK at the landing: the
   seam's per-label F1s solve to (tp, fp) counts whose pooled read
   reproduces `hard`'s own numbers exactly — Σtp=16, Σfp=21 → acc
   929−21+16 = 924/1200 = 0.7700 = `hard.accuracy`, and pooled
   positive F1 = 32/(32+21+255) = 0.10390 = `hard.positive_f1`. The
   label walk is provably aligned with the per-question walk.

2. **The isarcasm_b suite** (manifest §27): iSarcasmEval TRACK B — the
   six sarcasm-technique columns (sarcasm, irony, satire,
   understatement, overstatement, rhetorical_question, the upstream
   header order pinned as `ISARCASM_B_LABELS`). THE WIRE: one case per
   technique-CARRYING test tweet — **200 of the 1,400** (the track-A
   sarcastic population with 20 irony-only carriers; 70 carriers hold
   >1 technique, which a single-choice question cannot express — the
   multi-question envelope IS the multilabel law here) — each with SIX
   binary questions, one per technique, positive option pinned at
   index 1 (the metric convention, deliberately not sort order — the
   six spellings sort inconsistently). The 1,200 all-zero rows (the
   track-A negatives) drop COUNTED at the builder census; per-label
   support pins [180, 20, 49, 1, 10, 11] assert at full load. The
   board does NOT score track B (board row 40's index metric is
   track-A sarcastic F1, landed as `isarcasmeval`/Bench 184) — this is
   an EXTENSION suite.

3. **CORPUS-OPTIONAL, laya-only** (the home_appliance/T3 posture): the
   modelless router binds ONE domain per input while the case needs
   six per-technique reads — there is no lawful modelless signal, so
   the lane refuses loud instead of shipping a fabricated number
   (`modelless: SKIPPED — corpus-optional` in the run log; a
   `modelless_lane: false` suite is the declared-inability machinery).
   Registry: shared `isarcasmeval` dataset dir, eval split `test_b`,
   cal_cap 0 (asserted by the corpus-optional prepare path),
   named_only, coverage_audit off, the 12-key option union pinned
   exact-membership in prepare.

4. **The fetch lane**: `fetch_isarcasmeval` gains the
   `test_b:test/task_B_En_test.csv` spec + the `_csv_b_rows` transform
   (text + the six 0/1 columns AS STRINGS — the `_csv_a_rows`
   convention `row_str` reads — + the split tag; ALL 1,400 rows go
   through faithfully, the carrier filter is the BUILDER's counted
   census, never a pre-filter). Same pinned sha `dfc708b…` the T10 MIT
   read covers (the licence gate is the repo-level one — no new probe
   needed). Verified live at the landing: test/train pages skip
   idempotently; the new `test_b-000.json` fetches 1,400 rows whose
   census matches every pin (200 carriers / 1,200 all-zero / supports
   [180, 20, 49, 1, 10, 11] / 70 multi-technique). Envelope blake3
   `d0719511682ffcfa8ef159ecba64da4a7143831936ba557c6536a355369012b2`.

## The read (laya[english]; modelless skipped loud)

| readout | value | note |
|---|---|---|
| acc (1200 binary questions) | **0.7700** | vs §B3 majority chance 0.7742 → **skill 0.0** (clipped) — the negative-majority floor posture, the same disclosure class as T10's track-A laya read |
| positive_f1 (pooled) | 0.1039 | the lane answers "not X" overwhelmingly (37 yes-predictions of 1200) |
| **multilabel macro_f1 (the suite's metric)** | **0.0813** | per-label F1 [sarcasm **0.1340**, irony 0.0, satire 0.0, understatement 0.0, overstatement **0.2**, rhetorical_question **0.1538**] |
| per-label detail | tp/fp/fn | sarcasm 13/1/167 · irony 0/?/20 · satire 0/?/49 · understatement 0/?/1 · overstatement 2/8/8 (the 0.2 fraction) · rhetorical 1/1/10 (2/13) |
| modal floor (issue 094) | 0.9125 | 1134 keys / 1092 fallbacks — mostly prior-driven on this suite, disclosed; the harness's own flag: acc ≤ floor — no input-independent signal |
| ECE(maxp) | 0.1394 | readout-ECE 0.3995 |
| determinism | ✓ n=10 | reruns clean |
| latency | NOT QUOTABLE | load 8.58/8.05 > 6 (the sibling job); p50 26.0 / p99 43.0 ms are posture-disclosed |

The honest verdict: the encoder lane reads the negative class at the
majority floor (skill 0.0) with weak per-technique residue above it on
sarcasm / overstatement / rhetorical_question — and NEVER hits irony,
satire, or the 1-support understatement positive. The multilabel
machinery is the deliverable (the last wave-2 machinery item; the
track-B row is now SCOREABLE for any future lane); the encoder's
technique read needs the corpus/teacher re-open (the rethink-side
booking, the T3/T4/T5 pattern) before it is a capability claim.

## Gates

`cargo test -p riir-reflex --lib` — 631 passed (6 new multilabel
scorer tests + 3 new builder tests); `cargo clippy --all-targets -- -D
warnings` clean; rustfmt per-file (never `cargo fmt --all`); the
fetch lane's census verified against the builder pins at the landing.
No box-state-clean latency number exists — none is quoted.
