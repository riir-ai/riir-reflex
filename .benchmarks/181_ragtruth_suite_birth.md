# Bench 181 — ragtruth suite birth: the JDI ladder wave-3 opens, the language rung's first suite (issue 114 T7)

**Date:** 2026-11-12 · **Repo:** riir-reflex (the harness/bin deltas land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` T7 (the R3a wave-3 first suite; rethink 060 R3a's triage landed the same day — 5 scoreable / 2 partial / 0 refused)
**Posture:** `LAYA_DEVICE=metal /tmp/reflex_ragtruth/release/harness --suites ragtruth --laya-max-questions 0 --out .benchmarks/181_ragtruth_suite_birth` (release, laya-riir-metal; corpus-ON)
**PROVENANCE:** ⛔ preflight REFUSED twice (load 6.88–18.92 > MAX_LOAD 6, swap 4.17G) — Zed + sibling sessions on the box; the run's own TABLES.md carries `load 15.10/11.02 > 6`. **Accuracy is the deliverable; the p50/p99s are posture-disclosed, never quotable baselines** (the Bench-180 convention verbatim).

## What landed

The language rung's first suite — the response-level hallucination decision
(board id 59, added edition 0.3; rethink 060 R3a's wave-3 triage picked it
first: the cleanest licence of the seven + the T1-pattern gates already
discharged by 060 R1c's residue work the same day):

1. **ragtruth** (`ParticleMedia/RAGTruth` upstream-direct, MIT verified at
   the source): **2,700 cases** (all upstream test responses — the chance
   arithmetic 2·943/(943+2700) = 0.51771 pins the population, no quality
   filtering) across the three task-type splits (QA/Summary/Data2txt,
   900/900/900 — agreeing with the flowaicom mirror). Binary decision
   (grounded / hallucinated), gold = the response carries ≥1 annotated
   span. The eval option universe pinned (`RAGTRUTH_KEYS` exact-membership
   guard), the builder's 2,700 population assert is the drift guard.
2. **The upstream-direct fetch lane** (manifest §22): response.jsonl +
   source_info.jsonl joined by source_id (the recorded transform, the
   bfcl gold-join precedent; the digest pins the OUTPUT envelopes), the
   source_info context stringified per the upstream contract (string for
   Summary, dict→compact JSON for QA/Data2txt). **The mirrors are
   witnesses, not sources** — flowaicom/RAGTruth_test's `score` column is
   the INVERTED polarity (1 = grounded; verified 0-exception on all 2,700
   rows vs the upstream spans); the first build caught the dict-shape
   trap fail-loud (missing_ctx=1800 → the builder's all-build assert), the
   gate working as designed.
3. **The binary-F1 readout** (`binary_f1_of` + `HardMetrics.positive_f1`):
   the board metric is the SINGLE-class (hallucinated, index 1) F1 —
   never the two-class macro-F1 (chance row: "always hallucinated · F1";
   macro-F1 over 2 classes ≠ the board's number, so the record quotes
   `positive_f1` as the board-comparable axis and macro-F1 as the
   uniform readout).
4. **Corpus-ON** (the wave-3 posture upgrade over wave-1's corpus-optional
   rows): the lawful pool is the upstream TRAIN split (15,090 response
   rows, 6,721 hallucinated — agreeing with the wandb card; the 060 R1c
   scan had proved wandb-train ≡ upstream-train at the response level),
   projected to the same recorded transform; corpus text = the RESPONSE
   alone (the discriminative side), labelled through the processed-counts
   arm (any > 0 = hallucinated). cal_cap 200 at the clinc150 ratio;
   corpus_cap_per_label 40.
5. **One builder, two row schemas** (eval `{response, source_info, score}`
   vs pool/cal `{output, context, hallucination_labels_processed}`): the
   builder projects both to the same case shape; `train_row_label` reads
   both encodings (the slice-integrity audit labels every slice through
   that ONE rule — the eval-only first draft was refused by the audit's
   coverage law, the gate working).

## The read (this record) — corpus-ON, both lanes, full population

| suite | lane | n | acc | macro-F1 | **positive_f1 (the board metric)** | board chance | ECE(maxp) | modal floor | p50 |
|---|---|---|---|---|---|---|---|---|---|
| ragtruth | modelless | 2700 | 0.6304 | 0.3913 | **0.0099** | 0.5177 (always-hallucinated F1) | 0.1285 | 0.6493 ⛔ below | 0.129 ms |
| ragtruth | laya[english] | 2700 | 0.6430 | 0.5460 | **0.3361** | 0.5177 | 0.0670 | 0.6493 ⛔ below | 28.0 ms |

- cases_digest: `fnv1a64-` pinned in results.json (`30dbcee` on amo); slice
  digests test `fnv1a64-123a5ea1e2664c70` · cal `fnv1a64-ecb474a420a5c834` ·
  pool `fnv1a64-0417565bc6b62fda`, violations EMPTY; determinism ✓ both
  lanes (the modelless lane's claim + the laya lane's observed repeat —
  two independent runs byte-identical on every hard metric).
- **G1 (modelless readout ECE): PASS** — calibrated 0.1304 beats the raw
  0.6263 AND the conformal-naive floor 0.3844.
- ⛔ **The honest first read: BELOW the always-hallucinated floor.** The
  board scores positive-class F1 against the 0.5177 always-hallucinated
  chance; laya reads positive_f1 **0.3361** (board skill −0.377),
  modelless **0.0099** — both lanes read the MAJORITY class (grounded,
  65.1% share) heavily, so the minority (hallucinated) recall collapses.
  The modal-floor row says the same thing in the accuracy frame (both
  lanes ≤ floor 0.6493): no input-independent signal beyond the class
  prior. The harness's own jdi_skill column (accuracy-framed, chance
  0.6507) reads 0.0 both lanes — consistent.
- **The posture is the floor read, not the ceiling read** (the
  api_bank/when2call birth lesson again): the binary hallucination
  decision over long RAG contexts is the language rung's hardest shape —
  the birth records the honest floor; the corpus/teacher re-open paths
  (the rethink-side booking) are where a better read comes from. The
  suite's VALUE lands regardless: the board row is now SCOREABLE by the
  stack, the teacher capture can ride it (the R1b trip), and every
  future distill gets the paired our-suite read the B5 law requires.
- ⛔ **Latency NOT QUOTABLE** — the PROVENANCE line above: preflight
  REFUSED (load 6.88–18.92, swap 4.17G, Zed + siblings). The p50s are
  the run's own posture-disclosed readings, carried for continuity only.

## Gates

- 596 lib tests green (the 3 new binary_f1 arms: hand-computed F1, the
  perfect/degenerate postures incl. the always-positive chance shape, and
  the hard_metrics binary-vs-k>2 field law); clippy `-D warnings` clean
  at the lib posture.
- The population/coverage pins: builder 2,700 assert + RAGTRUTH_KEYS
  exact-membership + the slice-integrity audit (both gold classes covered
  on every slice — 943/1,757 eval, both classes in the cal/pool fronts).
- Licence rows landed BEFORE any published number (the manifest §22:
  upstream MIT verified at the source; the mirror witness recorded).
- grammar-not-answers: the corpus pool is the upstream TRAIN split only
  (the test population never touches it — the R1c response-level scan
  proved the split boundary clean: zero test-only content); no test rows
  in any pool.
- Fetch self-test unaffected (no new fetch-script machinery — the
  upstream-direct lane is the T7-lane transform, recorded in §22).

## Recording convention

The birth read is the FIRST read at the corpus-ON floor posture — never a
board row until the rethink-side fold books it (060 R1d/R3a's capture
side, owner-gated amo hours, unchanged). The wave-3 remaining suites
(ContractNLI / FinEntity / VAST / NLI4CT scoreable; ACOS + iSarcasmEval
waiting on wave-2's scorers) file one at a time per the issue's law.
