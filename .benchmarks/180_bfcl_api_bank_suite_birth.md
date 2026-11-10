# Bench 180 — bfcl + api_bank suite births: the JDI ladder wave-1 tools rung completes (issue 114 T4+T5)

**Date:** 2026-11-10 · **Repo:** riir-reflex (the harness/bin deltas land in the same commit as this record; the digests below pin the run) · **Box:** m3 (M3 Max, 16 cores)
**Issue:** reflex `.issues/114` T4+T5 (the reflex-side half; the rethink-side fold stays the 060 R1d/R2a booking, owner-gated amo hours — unchanged)
**Posture:** `LAYA_DEVICE=metal cargo run --release --features laya-riir-metal --bin harness -- --suites bfcl,api_bank --laya-max-questions 0 --out .benchmarks/180_bfcl_api_bank_suite_birth`
**PROVENANCE:** power=AC Power load=1.99 swap=0.00M canary=skipped powermode=2(high) (`scripts/bench_preflight.sh` PASSED — ⛔ latency figures carry this box state: High Power mode; accuracy is the deliverable, the p50s are posture-disclosed, never quotable baselines)

## What landed

The wave-1 tools rung's two remaining suites (the direct-JSON fetch lane +
the builders + the registry rows + the corpus-optional pins), completing
the decision-shaped tools three after when2call (Bench 101):

1. **bfcl** (`gorilla-llm/Berkeley-Function-Calling-Leaderboard`,
   Apache-2.0): the pick tier (simple/multiple/live_simple/live_multiple —
   the official possible_answer gold joined by exact id at fetch time) +
   the irrelevance-detect abstain tier (the fixed
   `no_appropriate_function` option appended LAST and golded — the pick
   tier never carries it). 3,033 rows fetched → **3,029 cases** built (the
   4 pinned empty-candidate live_irrelevance rows skip). ONE pinned
   upstream-typo normalization in the join (`live_multiple_1052-79-0` →
   `live_multiple_1052-279-0`; the gold id-chain …277/…278/…279 proves
   the eval file dropped a digit; any other id mismatch still fails the
   fetch loud). Per-row candidate-name universes (up to 37 on
   live_multiple), `coverage_audit: false` (the s1mb variable-breadth
   posture), corpus-optional (`modelless_lane: false` — no lawful train
   pool, grammar-not-answers).
2. **api_bank** (`liminghao1630/API-Bank`, MIT): the which-API tier
   (level-1-api whole; level-2-api ROW-LEVEL SCOREABLE-PARTIAL — the
   ToolSearcher-gated rows whose expected_output names the presented
   candidate are choice-shaped (45 kept), the downstream-API rows drop
   (90; the official eval is generation-based there — never forced into
   the pick wire). **444 cases** built (399 + 45; the prepare pin). State
   = the instruction VERBATIM; gold parsed per row from the row's own
   `expected_output` (the recorded transform).
3. **The fetch script's direct-resolve lane** (`fetch_raw_json` +
   `fetch_bfcl_join` + `_raw_convert`): the hub `/resolve/main/<file>`
   pattern for raw-JSON/JSONL mirrors the datasets-server cannot serve;
   envelope-wrapping converter (jsonl + array shapes), the idempotence
   skip law, and the all-or-nothing gold join (id-set equality both
   directions, a mismatch fails the fetch loud, never a partial suite).
   Self-test scenario 4 (jsonl/array conversion + idempotence + the join
   + the loud mismatch + the pinned-typo path) rides the existing
   hermetic loopback (layer 8 of ci_feature_guard).

## The read (this record) — laya-only, full populations, corpus-optional

| suite | lane | n | acc | macro-F1 | jdi_chance | jdi_skill | ECE(maxp) | p50 |
|---|---|---|---|---|---|---|---|---|
| bfcl | modelless | — | **SKIPPED (corpus-optional)** | — | — | — | — | — |
| bfcl | laya[english] | **3,029 (all)** | **0.5612** | 0.4341 | 0.3665 | **+0.3075** | 0.1595 | 55.0 ms |
| api_bank | modelless | — | **SKIPPED (corpus-optional)** | — | — | — | — | — |
| api_bank | laya[english] | **444 (all)** | **0.6081** | 0.3951 | 0.6059 | **+0.0057** | 0.1319 | 46.0 ms |

- cases_digests: bfcl `fnv1a64-b53dc0f20f68d79f` · api_bank
  `fnv1a64-a9d2aaea5497cd7e`; no pool/cal slices exist (the
  corpus-optional posture — `slices: None`); determinism legs
  `determinism_ok: true` both suites.
- **bfcl reads ABOVE chance with real skill** (+0.31 over the harness's
  jdi_chance 0.3665 — the option-count-weighted chance framing): the
  encoder resolves function-pick from the rendered spec stack at the
  shipped posture. A POSITIVE first read for the tools rung's pick shape.
- **api_bank reads ≈ the majority class** (acc 0.6081 vs jdi_chance
  0.6059, jdi_skill +0.0057): the kept population's plurality candidate
  (`ToolSearcher`, gold on all 45 kept lv2 rows and the largest lv1
  class) dominates — the encoder reads the majority class, an honest
  near-zero-skill first read at this posture (the when2call shape's
  lesson again: the corpus-less LLM-lane-only posture is the floor
  read, not the ceiling read; the corpus re-open paths live in the
  issue).
- ⛔ **Latency NOT QUOTABLE as baselines** — the PROVENANCE line above:
  High Power mode (powermode=2), load 1.99. The p50s are the run's own
  posture-disclosed readings, carried for continuity only.

## Gates

- 593 lib tests green (the 10 new builder-test arms: pick-gold
  resolution, the abstain tier's append-last gold, the unique-leaf rule
  incl. the ambiguous-leaf refusal, the joined-gold refusal, the
  empty-candidate drop, the api_bank gold parse + prefix-drop +
  downstream-drop); clippy `-D warnings` clean at the lib posture.
- Fetch self-test PASS (all four scenarios + the two new raw-lane arms).
- The prepare pins (the drift guards): bfcl 3,029 + 1,118 abstain golds;
  api_bank 444. A drifted fetch reds loud instead of misreporting the
  board row.
- Licence rows landed BEFORE any published number (the manifest
  §Licences table: both VERIFIED via the hub probe; BFCL Apache-2.0,
  API-Bank MIT).
- grammar-not-answers: no test rows in any training pool (both suites
  are corpus-optional — no pools exist); no training pool was invented.

## Recording convention

The birth reads are FIRST reads at the corpus-optional floor posture —
never board rows until the rethink-side fold books them (060 R1d/R2a,
owner-gated amo hours, unchanged from T6).
