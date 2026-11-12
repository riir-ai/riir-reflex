# Bench 184 — isarcasmeval suite birth: the language rung's fourth suite, the track-A-English board-metric lane (issue 114 T10)

**Date:** 2026-11-12 · **Repo:** riir-reflex (the harness/bin deltas land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` T10 (the wave-3 residue's third adapter; rethink 060 R3a — the first of the two "scoreable-partial" rows to land, WITHOUT new machinery)
**Posture:** `LAYA_DEVICE=metal ./target/release/harness --suites isarcasmeval --laya-max-questions 0 --out .benchmarks/184_isarcasmeval_suite_birth` (release, laya-riir-metal; corpus-ON, both lanes)
**PROVENANCE:** ⛔ preflight PASSED at launch (load 5.97 ≤ ceiling 6.0, swap 4.17G — the grafts_cell sibling job contending) but the run's own box-state stamps read start load **10.16 / end 9.11** — **latency NOT QUOTABLE** (load > 6 at the run). **Accuracy is the deliverable; the p50/p99s are posture-disclosed, never quotable baselines** (the Bench-180/181/182/183 convention verbatim).

## What landed

The language rung's fourth suite — intended sarcasm detection, track A
English (board id 40, A_direct). The first of the wave-3 triage's two
"scoreable-partial sharing wave-2's scorers" rows to land — and it
needed NO new machinery:

1. **The licence probe PASSES at the board's own pinned sha** —
   `iabufarha/iSarcasmEval @dfc708b` (the tree sha read live 2026-11-12
   matches the board's suite.md pin EXACTLY): the repo carries the full
   MIT LICENSE (1,074 bytes, "Copyright (c) 2022 Ibrahim Abu Farha"),
   GitHub's repo API independently reports `spdx_id: MIT`, and the
   recursive tree shows NO separate data-terms file — the CSV data dirs
   ride the repo licence (the FinEntity two-surface PASS shape; the
   board's row said "see repository LICENSE"). The `third-party
   annotations/` dir is a supplementary release, NOT the board track —
   never fetched.
2. **The scope is the triage's sanctioned partial, narrowed by the
   metric frame: TRACK A ENGLISH ONLY.** The board row's index metric
   AND its chance (0.2227) are defined on exactly that track ("track A
   English: F1 of random guessing on the sarcastic class"), so the
   suite's `positive_f1` (binary, index 1 = "sarcastic" — the sorted
   upstream spellings land the class exactly where
   `hard_metrics::positive_f1` reads it) IS the board number, read
   straight off the aggregate — no new readout, no per-type split. THE
   RE-OPENS, recorded loud in the builder doc + manifest §25 (the
   triage's "never a silent drop" law): **track C** (pair pick —
   shape-fine plain choice, but its second option universe would break
   the one-suite-one-universe router law and dilute the binary
   positive_f1 the board scores), **track B** (the six-label multilabel
   — needs the wave-2 multi-question envelope), **the Arabic tracks**
   (the triage's English-first posture decision).
3. **isarcasmeval** (suite §25 of the manifest): **1,400 cases** — one
   case per track-A-English test tweet, the fetched census (200
   sarcastic / 1,200 non-sarcastic — the 1-in-7 share the board's
   chance pins: analytic random-guess sarcastic-F1 0.2222 vs the
   catalog's Monte Carlo 0.2227, Δ 5e-4 = MC noise). State = `{tweet}`
   verbatim; options = `ISARCASM_KEYS` (sorted upstream spellings,
   exact-membership guard); gold = the author's own label (the
   collection method's whole point — no annotator proxies). The board
   catalog's cases 4,600 books all five test tracks — the
   contractnli/finentity reconciliation law (disclosed, never
   inherited). CORPUS-ON: the upstream train split (3,467 fetched rows
   — one empty-tweet row of 3,468 dropped by the converter's rule,
   disclosed) is the task's own released training data; corpus doc =
   the tweet alone; corpus_cap_per_label 200, cal_cap 200.
4. **The fetch lane** (`fetch_isarcasmeval` + `_csv_a_rows` in
   scripts/fetch_datasets.sh): the track-A CSVs at the pinned sha via
   the GitHub raw lane (the hub /resolve does not serve GitHub), a csv
   DictReader transform (tweets carry embedded commas/newlines — naive
   splitting is wrong), idempotent envelopes skip on a valid cached
   page. Census: test 1,400 / train 3,467; slice-integrity OK
   (pool∩cal∩test disjoint).

## The read (the birth's honest numbers)

| lane | n | acc | macro F1 | **positive F1 (= the board metric)** | ECE(maxp) | p50 |
|---|---|---|---|---|---|---|
| modelless | 1,400 | 0.2186 | 0.2168 | **0.2538** | 0.2923 | 0.080 ms |
| laya english | 1,400 | 0.8057 | 0.4893 | **0.0872** | 0.0627 | 9.0 ms |

Board chance (track-A random sarcastic F1): **0.2227**.

- **modelless positive_f1 0.2538 — ABOVE chance (+0.031)**: the corpus
  pool (867 sarcastic / 2,600 non tweets) gives the drafter a real —
  if thin — sarcastic-class signal. The lane's overall accuracy 0.2186
  is BELOW the always-non-sarcastic floor 0.8571 (it over-predicts the
  sarcastic class), and its G1 readout FAIL (raw 0.2138 vs the
  conformal-naive floor 0.3704) — the calibration read is the honest
  disclosure, never buried.
- **laya positive_f1 0.0872 — BELOW chance**: the encoder reads the
  majority class (acc 0.8057 ≈ the non-sarcastic share 0.8571 to
  within a point; acc-frame skill ≈ 0), so its sarcastic-class F1
  collapses — the api_bank/contractnli-class floor disclosure verbatim.
  The modal-floor row splits it: laya acc 0.8057 ≤ the input-keyed
  modal floor 0.8593 (no input-independent signal), modelless 0.2186 ≤
  floor likewise — both lanes disclosed at/below the modal floor on
  accuracy.
- The suite's VALUE lands regardless: the board row is now SCOREABLE
  by the stack, the teacher capture can ride it, and every future
  distill gets the paired our-suite read. The birth read is the FIRST
  read at the corpus-ON floor posture — never a board row until the
  rethink-side fold books it (060 R1d/R3a's capture side, owner-gated
  amo hours, unchanged).
- ⛔ **Latency NOT QUOTABLE** — the PROVENANCE line above: the run's
  box-state stamps read load 10.16/9.11 (the grafts_cell sibling job
  surged between the preflight and the run). The p50s are the run's
  own posture-disclosed readings, carried for continuity only.

## Gates

- 606 lib tests green (the 3 new isarcasmeval builder arms: the tweet
  rendering + the sarcastic-positive key order + gold mapping, the
  train-envelope cal-front path never tripping the eval census, and
  the drifted-label refusal); clippy `-D warnings` clean at the lib
  posture; rustfmt clean.
- The population/coverage pins: the builder's eval census assert
  (1,400, keyed on the split column) + the prepare-side pin (1,400 +
  both gold classes present — a fetch that lost the 200-tweet
  sarcastic minority would misreport the metric) + ISARCASM_KEYS
  exact-membership + the slice-integrity audit (both directions).
- Licence rows landed BEFORE any published number (the manifest §25 +
  the licence-ledger row: MIT at the authors' repo LICENSE + GitHub's
  API spdx, both at the board's pinned tree sha).
- grammar-not-answers: the corpus pool is the upstream TRAIN split
  only (the task's own released training data — the shared task ran
  on it; the test population never touches it; the slice-integrity
  audit pins pool∩test disjoint); no test rows in any pool.

## Recording convention

The birth read is the FIRST read at the corpus-ON floor posture —
never a board row until the rethink-side fold books it. The wave-3
remaining rows: VAST REFUSED (no licence); NLI4CT UNDECIDED
(licence-pending); ACOS (the all-exact case scorer + the
multi-question envelope — the wave-2 machinery, its own claim);
iSarcasmEval tracks B/C/Arabic (the recorded re-opens, manifest §25).
One suite at a time per the issue's law.

Per-file digests (blake3): `test-000.json` 1,400 rows
`b5779bce92f11a1ccbdae031847f59ccc1233348aff11aeb7f74f884f9eed49f` ·
`train-000.json` 3,467 rows
`92bf913f1a527a747f1370e49fe37423dfeb3f8a384931798fe23a3c24950101`.
