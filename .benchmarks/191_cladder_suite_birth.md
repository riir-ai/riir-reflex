# Bench 191 — cladder suite birth: the knowledge rung's fourth adapter, the board's causal yes/no at the board's own frozen population (issue 114 R4a queue #4)

**Date:** 2026-11-13 · **Repo:** riir-reflex (the fetch lane + builder + registry + manifest land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` R4a queue #4 (the knowledge nine triage's fourth adapter — the queue's successor to GSM8K/BBH/SATA; rethink 060 R4a's reflex-side half)
**Posture:** `LAYA_DEVICE=metal /tmp/sata-dev/release/harness --suites cladder --laya-max-questions 0 --out .benchmarks/191_cladder_suite_birth` (release, laya-riir-metal; corpus-off — the modelless lane skips loud)
**PROVENANCE:** ✓ preflight PASSED at launch (power=AC powermode=0(auto) load 5.26; the run's own box-state stamps read start load **4.61 / end 4.22** — **latency QUOTABLE**).

## What landed

1. **The licence probe PASSES at two GitHub surfaces** — the repo's
   LICENSE at the pinned tree sha `3d2d1169b4b939a09048a6a75956c8972a93cc38`
   is the full MIT text ("Copyright (c) 2023 CausalNLP"), and the
   GitHub API independently reports `spdx_id: MIT` (both read
   2026-11-13). The hub-side probe does not apply (GitHub repo, not a
   hub dataset — the bbh/sata convention).
2. **The fetch lane** (`fetch_cladder` + the `_cladder_rows` transform,
   stanza #29): the single zip at the board's pinned tree sha via the
   GitHub raw lane — 0 failed requests; zip sha256
   `9fdae052b1ebe4ee6a19fdfb3e1eb88a381c7345df394cea524bcce97e2349b3`.
   The transform ports the board's TWO board-exact steps so the Rust
   builder stays std-only: (1) its normalize builder recipe
   (prompt = model background + given_info + question joined by blank
   lines; gold = the original answer); (2) its freeze CASE_CAP
   selection (freeze.py: **5,000 of the 10,112-question balanced file,
   sha256-priority order `sha256("20260919:44:<row id>")` over the row
   ids, first 5,000**) — **verified id-for-id at fetch** (the first id
   by recomputed priority IS the envelope's first row), so OUR
   population IS the board's, byte-exact: the reads are directly
   board-comparable, no re-seating caveat (unlike gsm8k/sata's
   shuffles — there the shuffle was provenance; here the selection IS
   the population and it is ported exactly). The meta
   groundtruth/estimand columns never reach the envelope (the board's
   own request_exclusion). Envelope page test-000 blake3
   `c2bff65c37915b137fdb6614cc85cdfe8632caf9e809786e95bd759619b9e410`.
3. **`cladder`** (suite §31 of the manifest): **5,000 cases / 5,000
   questions** — ONE choice question per case; the fixed binary set
   [yes, no] in the board's own order (its gold maps yes→A/0, no→B/1);
   qid `q1` (the board's own); state EMPTY (the board's own shape —
   the prompt IS the instructions); the case id keys on the source
   question_id. Census pinned two-layer (builder + prepare, 5,000).
   **The corpus census resolves the triage's "possible corpus-ON" to
   OFF at birth**: the zip carries SEVEN EVAL question variants
   (balanced/aggregate/anticommonsense/commonsense/easy/hard/
   noncommonsense) + models + README — no train split anywhere; the
   questions themselves are eval-only (grammar-not-answers).
   `modelless_lane: false`, laya-only.
4. **The selected 5,000's answer split**: no 2,505 / yes 2,495 —
   the majority ("no") share 0.501 IS the suite's behaviour floor
   (the board's chance pin 0.5 = uniform; the §B3 re-derivation gives
   0.501 for majority-class reads, quoted as jdi_chance in the run).

## The birth read (laya[english], the only lane)

| axis | value |
|---|---|
| **accuracy (the board metric)** | **0.5346** (n 5,000) |
| chance (board pin / majority class) | 0.5 uniform · 0.501 majority |
| chance-corrected skill | **+0.067** |
| modal floor (issue 094) | 0.501 (1 key — every input is distinct, so the input-keyed floor degenerates to the global majority; disclosed, never hidden) |
| ECE(maxp) | 0.2327 (mean confidence 0.767 — overconfident) |
| determinism | ok=true, n=10 reruns clean |
| latency | p50 12 ms / p99 15 ms (support 51) — QUOTABLE posture |
| slice digest | fnv1a64-40bc4c2482663345 (5,000) |

**The read:** ABOVE floor and above chance — **the knowledge rung's
first real skill read at birth** (+0.067 vs BBH's +0.030, GSM8K's ~0,
SATA's negative). The encoder reads the causal yes/no questions at
better-than-majority with zero-shot prompting carried entirely in the
instructions. Still far below the board leaders (CLadder board rows
run 0.6–0.9): the gap is the teacher-capture lane's work (R4b,
owner-gated). The ECE 0.233 at mean confidence 0.767 says the
confidence head needs the calibration lane before any abstention
claim (the G1 law — no calibration claim when the calibrator never
moved; none ran here, corpus-off).

## Scope notes

- The birth ran on the SHARED tree (the sibling's lane landed and the
  tree was clean at the claim sha `f6e4b09` + this adapter's deltas —
  no isolation needed this time).
- The licence-table row + §31 landed in the same commit; the fetch
  stanza carries the GitHub-surface licence note.
- The board's suite.md row 44 says "5000 seeded cases" — the board's
  own freeze caps the 10,112-question balanced file at 5,000; our
  transform ports the cap byte-exact (the suite.md row is the INDEX
  number, the builder code reads the whole file — the freeze is the
  reconciliation, found and ported).
