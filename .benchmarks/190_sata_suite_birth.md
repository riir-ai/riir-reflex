# Bench 190 — sata suite birth: the knowledge rung's third adapter, the select-all case at the board's own whole-case wire (issue 114 R4a queue #3)

**Date:** 2026-11-13 · **Repo:** riir-reflex (the fetch lane + builder + registry + manifest land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` R4a queue #3 (the knowledge nine triage's third adapter — the queue's successor to GSM8K/BBH; rethink 060 R4a's reflex-side half)
**Posture:** `LAYA_DEVICE=metal ./target/release/harness --suites sata --laya-max-questions 0 --datasets-dir <main>/.raw/datasets --out .benchmarks/190_sata_suite_birth` (release, laya-riir-metal; run from an ISOLATED worktree at the claim commit `24f72cb` — a sibling lane (issue 117 T1) was mid-flight in the shared tree, so the build + run carry ONLY this adapter's deltas at that sha; corpus-optional — the modelless lane skips loud)
**PROVENANCE:** ✓ preflight PASSED at launch (power=AC powermode=0(auto) load 5.79; the run's own box-state stamps read start load **4.89 / end 3.02** — **latency QUOTABLE**).

## What landed

1. **The licence probe PASSES at two GitHub surfaces** — the repo's
   LICENSE at the pinned tree sha `371dd0c18fe75a96fbbcf2d1507ceeaf0d5263c5`
   is the full MIT text ("Copyright (c) 2025 sata-bench"), and the
   GitHub API independently reports `spdx_id: MIT` (both read
   2026-11-13). The hub-side probe does not apply (this source is a
   GitHub repo, not a hub dataset — the fetch stanza names the
   surfaces, the bbh convention).
2. **The fetch lane** (`fetch_sata` + the `_sata_rows` transform,
   stanza #28): the single prepared jsonl at the board's pinned tree
   sha via the GitHub raw lane — 1,650 rows = the board's population
   exactly ("All prepared cases retained"), 0 failed requests. THE
   LEGACY `answer`/`symbol` COLUMNS NEVER LAND ON DISK: the board's own
   gold_rule names them a DIFFERENT permutation ("legacy answer letters
   refer to a different permutation; matching upstream
   sata_prompt_provider.py the per-candidate bool is the gold") — a
   trap column, dropped at the transform so no future reader can trust
   it. Source sha256 `adb8ad3f7330a659e7b38a4ff1c67b3c51e0fd7c708b0a24476f216e86aa9a74`
   (read at fetch); envelope page test-000 blake3
   `4ffcbc15bbdcd4e63e55d34cca70edc5c817459ccafaffcacf5242c27056ab64`.
3. **`sata`** (suite §30 of the manifest): **1,650 cases / 15,517
   questions** — ONE case per source row; every candidate becomes an
   independent binary yes/no question (the board's own wire: criteria
   `{"no": "No", "yes": "Yes"}`, the instructions string verbatim);
   the case passes only when EVERY candidate's yes/no is right — the
   board's `case exact accuracy` (the `case_exact` column arms for
   this suite, the CASE_EXACT_SUITES law). k per case = the row's
   candidate count (3..16; the k-histogram is the board's own:
   {3:52, 4:52, 5:82, 6:486, 7:61, 8:312, 9:15, 10:13, 11:3, 12:1,
   13:1, 15:571, 16:1}). Census pinned loud at build + prepare (1,650
   / 15,517; the home_appliance two-layer form).
4. **The shuffle**: the board's per-source-index seeded shuffle
   (`random.Random(f"20260918:SATA:{i}")`, MT19937) RE-SEATED on the
   house fold — key-sort on `house_fold("20260918:SATA:{pos}:{orig}")`
   (the gsm8k disclosure verbatim: METHOD-identical, NOT
   byte-identical; and the case-exact metric is PERMUTATION-INVARIANT —
   every candidate gets its own yes/no question — so the shuffle here
   is provenance fidelity, never score-bearing; fixture-tested
   deterministic).
5. **Corpus-optional** (`modelless_lane: false`): no train split
   exists and the candidates are case-specific texts
   (grammar-not-answers) — only the LLM lanes answer; the fetch is
   test-only, the whole census is asserted at load.

## The birth read (laya[english], the only lane)

| axis | value |
|---|---|
| **case exact accuracy (the board metric)** | **10/1,650 = 0.0061** |
| case exact chance (∏(1/2) per case re-derived on the kept set) | 0.0131290 (board pin 0.0131 — Δ 0.2%, the §B3 law) |
| per-question (field) accuracy | 0.4538 (n 15,517) |
| per-question chance (majority class, the §B3 column) | 0.6166 (the "no" share — yes is 0.3834) |
| per-question chance-corrected skill | −0.425 |
| modal floor (issue 094) | 0.6117 (9,683 keys, 8,750 fallbacks, LOO over eval) |
| ECE(maxp) | 0.3537 (mean confidence 0.807 on a 61.7%-no population — the lane over-says "yes") |
| best case | 10 cases passed all their questions; the all-exact bar is what the board scores |
| determinism | ok=true, n=10 reruns clean |
| latency | p50 153 ms / p99 387 ms (support 17) — QUOTABLE posture |

**The read:** BELOW chance on the board metric and below the
per-question chance — the floor disclosure, honestly. A zero-shot
encoder reading each candidate sentence against the paragraph cannot
do the per-candidate fact check the task demands: it answers "yes" at
0.807 mean confidence against a 61.7%-no population, so the per-field
read (0.4538 < 0.5 < 0.6166) compounds through the all-questions-exact
bar to 0.0061 < 0.0131. The shape mirrors home_appliance's birth
([Bench 185](185_home_appliance_suite_birth.md)): the whole-case bar is
BRUTAL by design (the board prices random whole-case guesses at
0.0131), and a wire-join bug would read as EXACT zero — the 10 exact
cases + the determinism ✓ + the census pins prove the join end to end.
The suite is SCOREABLE: the row is live on the harness, the teacher
capture can ride it (R4b, owner-gated). Board comparability: the
option ORDER is our re-seated shuffle (not byte-identical to the
board's MT19937 permutation) — but the case-exact metric is
permutation-invariant, so SATA rows are board-comparable REGARDLESS
(the gsm8k comparability caveat does not even arise here).

## Scope notes

- The run posture: an isolated worktree at the claim commit — the
  sibling issue-117 lane was mid-flight in the shared tree; the binary
  carries only this adapter's deltas at `24f72cb`. The harness's own
  census lines (1,650 cases / 15,517 questions, fnv1a64-18ffe71609d4dfcd)
  pin the slice.
- The licence-table row + §30 landed in the same commit; the fetch
  stanza carries the GitHub-surface licence note (the hub probe does
  not apply to a GitHub-repo source).
