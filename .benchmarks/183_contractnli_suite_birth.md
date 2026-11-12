# Bench 183 — contractnli suite birth: the language rung's third suite, the T8-note licence conflict resolved upstream (issue 114 T9)

**Date:** 2026-11-12 · **Repo:** riir-reflex (the harness/bin deltas land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` T9 (the wave-3 residue's second adapter; rethink 060 R3a)
**Posture:** `LAYA_DEVICE=metal ./target/release/harness --suites contractnli --laya-max-questions 0 --out .benchmarks/183_contractnli_suite_birth` (release, laya-riir-metal; corpus-ON, both lanes)
**PROVENANCE:** ⛔ preflight REFUSED (load 12.35 > MAX_LOAD 6, swap 4.17G — Zed + sibling sessions on the box; heavier than the 181/182 windows). **Accuracy is the deliverable; the p50/p99s are posture-disclosed, never quotable baselines** (the Bench-180/181/182 convention verbatim).

## What landed

The language rung's third suite — the contract entailment decision
(board id 11, A_direct). The T8-note licence conflict is RESOLVED
upstream the same day:

1. **The licence probe PASSES at every authoritative surface** — the
   T8-note's "theainswer/contract-nli" spelling 404s (a wrong memory of
   the repo name); the authors' canonical repo is
   **`stanfordnlp/contract-nli`** (the EMNLI-2021 authors' org, default
   branch gh-pages), and it carries the full CC-BY-4.0 LICENSE (18,656
   bytes verified at HEAD). The zip's own bundled `TERMS` grants rights
   "in accordance with the terms and conditions of the Creative Commons
   Attribution 4.0 International Public License", and the dataset
   site's §License names it verbatim — no NC/ShareAlike anywhere
   upstream. **The kiddothe2b HF mirror's `cc-by-nc-sa-4.0` tag is a
   third-party curator's CONTRADICTION of the upstream** — that mirror
   is never a source (the flowaicom-witness pattern); the
   mteb/legalbench `cc-by-4.0` curation tag was right. The source
   fetches at the board's own pinned sha (`eced652`,
   `resources/contract-nli.zip`, 65,362,913 bytes, blake3 below) — the
   zip downloads ONCE and is reused for both splits.
2. **contractnli** (suite §24 of the manifest): **2,091 cases** — one
   case per (document, hypothesis) pair over the 123 test contracts ×
   17 hypotheses = **the board's own `fields` count** (the
   reconciliation, the finentity pattern). State = `{contract,
   hypothesis}` (the contract text verbatim, 1.2K–41.8K chars); options
   = the three entailment keys (`CONTRACTNLI_KEYS` exact-membership
   guard, sorted unique upstream choices: Contradiction / Entailment /
   NotMentioned — the board's panel words supported/contradicted/
   unstated are display names, never the wire keys); gold = the pair's
   choice. **The metric frame is pair-level macro-F1** — the frame the
   board's own chance arithmetic pins (analytic random-choice macro-F1
   over the fetched distribution **0.30793** vs the catalog's Monte
   Carlo **0.30852**, Δ 6e-4 = MC noise at the 46.3/43.2/10.5% class
   split; the catalog's `requests=123` documents bookkeeping repeats
   the finentity note — disclosed, never inherited).
3. **The fetch transform** (the recorded one — the digest pins the
   OUTPUT envelopes): the authors' nested per-document JSON flattens to
   pair rows `{contract, hypothesis, choice, split}` — test 2,091 rows
   (`test-000`, blake3 `68c33796…b6f99d`), train 423 × 17 = 7,191 rows
   (`train-000`, blake3 `31bf57b8…26c34`). ONE row schema reaches the
   builder; the `split` column is the eval discriminator (the ragtruth
   schema-dispatch law, explicit form — the eval population assert keys
   on every row carrying split=test). The new fetch machinery is ONE
   lane: `fetch_contractnli_pairs` (zip download + unzip extract +
   `_pairs_flatten` + the existing `_raw_convert`), the bfcl-join
   precedent for suite-shaped fetchers; the flatten is python3-only
   (PY3 resolved lazily — the jq-present boxes never resolved it
   before, caught by the first run).
4. **CORPUS-ON** (the wave-3 posture): the upstream TRAIN split is the
   task's own released training data (the SemEval-2024 paper documents
   systems fine-tuning on it; the contrast interventions were withheld
   from training only and are NOT part of the board's track — the
   original 2,091 test pairs). The corpus doc text = contract +
   hypothesis (the xnli pair-as-unit shape with the contract first: 17
   hypotheses repeat across documents, so the hypothesis alone cannot
   carry the signal — the contract is exactly what flips the label; the
   ragtruth response-alone rule does NOT transfer, disclosed in the
   builder doc). corpus_cap_per_label 40 → ≤120 docs, cal_cap 200.
   Slice-integrity OK (test 2,091 / cal 200 / pool 6,991, violations
   empty).

## The read (this record) — corpus-ON, both lanes, full population

| suite | lane | n | acc | **macro-F1 (the board metric)** | board chance | acc-frame chance/skill | ECE(maxp) | modal floor | p50 |
|---|---|---|---|---|---|---|---|---|---|
| contractnli | modelless | 2091 | 0.3367 | **0.2330** | 0.3085 (random-choice macro-F1, MC) | 0.4629 / −0.126 | 0.0013 | 0.3673 ⛔ below | 0.768 ms |
| contractnli | laya[english] | 2091 | 0.4596 | **0.2344** | 0.3085 | 0.4629 / **0.000** | 0.2135 | 0.3673 ✓ above | 29.0 ms |

- cases_digest: `fnv1a64-5a62cc9c70613c5f`; slice digests test
  `fnv1a64-e1111314a5f3295b` · cal `fnv1a64-cbfe0df2e7449d9e` · pool
  `fnv1a64-9dfd981429f37055`, violations EMPTY; determinism ✓ both
  lanes (the modelless lane's claim + the laya lane's observed repeat).
- **G1 (modelless readout ECE): PASS** — calibrated 0.0366 beats the
  raw and the conformal-naive floor.
- ⛔ **The honest first read: BOTH LANDES READ THE MAJORITY CLASS.**
  laya's acc 0.4596 = the Entailment class share 0.4629 to within a
  point (acc-frame skill **0.000**): the encoder predicts the majority
  class, so its pair-level macro-F1 0.2344 lands BELOW the board's
  random-choice chance 0.3085 (predicting one class always collapses
  the minority classes' F1). The modelless lane sits at 0.3367 acc /
  0.2330 macro-F1 — the corpus (7,191 train pairs of ~10 KB contract
  text through a 40/label cap) gives the drafter almost no leverage on
  a task whose signal is a long-document/hypothesis interaction. The
  modal-floor row splits the disclosure: laya 0.4596 > floor 0.3673
  (its per-key answers beat the input-independent census) while the
  class-prior framing shows zero skill. The corpus-less-corpus-lane
  posture is the floor read, not the ceiling read (the
  api_bank/ragtruth birth lesson at corpus-ON); the teacher
  capture/ESC re-open paths (the rethink-side booking, owner-gated) are
  where a better read comes from. The suite's VALUE lands regardless:
  the board row is now SCOREABLE by the stack, the teacher capture can
  ride it, and every future distill gets the paired our-suite read the
  B5 law requires.
- ⛔ **Latency NOT QUOTABLE** — the PROVENANCE line above: preflight
  REFUSED (load 12.35, swap 4.17G). The p50s are the run's own
  posture-disclosed readings, carried for continuity only.

## Gates

- 603 lib tests green (the 3 new contractnli builder arms: the pair
  rendering + keys + gold mapping, the train-envelope cal-front path
  never tripping the eval census, and the drifted-choice refusal);
  clippy `-D warnings` clean at the lib posture; rustfmt clean.
- The population/coverage pins: the builder's eval census assert
  (2,091, keyed on the split column) + the prepare-side pin (2,091 +
  all three gold classes present) + CONTRACTNLI_KEYS exact-membership +
  the slice-integrity audit (both directions).
- Licence rows landed BEFORE any published number (the manifest §24 +
  the licence-ledger row: CC-BY-4.0 at the authors' repo LICENSE, the
  bundled TERMS, and the site §License; the kiddothe2b mirror named
  never-a-source).
- grammar-not-answers: the corpus pool is the upstream TRAIN split only
  (the task's own released training data — the test population never
  touches it; the slice-integrity audit pins pool∩test disjoint); no
  test rows in any pool. The board's own README posture ("treat the
  suite as evaluation data; do not train on it") binds the BOARD's
  redistribution, not the task's own training data release — our pool
  is the train split the task shipped for exactly this use.
- Fetch self-test: the zip lane rides the existing self-test machinery
  (the scenario-4 direct-resolve arms cover `_raw_convert`; the
  `_pairs_flatten` python3 gate was exercised live by the real fetch,
  including its first-run refusal and repair).

## Recording convention

The birth read is the FIRST read at the corpus-ON floor posture — never
a board row until the rethink-side fold books it (060 R1d/R3a's capture
side, owner-gated amo hours, unchanged). The wave-3 remaining rows:
VAST REFUSED (no licence at any surface); NLI4CT UNDECIDED
licence-pending (the Codalab terms defer to an unstated licence); ACOS +
iSarcasmEval behind wave-2's scorers. One suite at a time per the
issue's law.
