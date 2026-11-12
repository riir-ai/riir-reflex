# Bench 185 — home_appliance suite birth: the wave-2 case-exact machinery's first consumer, the board's own simulator track (issue 114 T11)

**Date:** 2026-11-12 · **Repo:** riir-reflex (the harness/builder/fetch deltas land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` T11 (the wave-2 scorer machinery + the ACOS lane that resolved REFUSED; rethink 060 R2b/R3a)
**Posture:** `LAYA_DEVICE=metal ./target/release/harness --suites home_appliance --laya-max-questions 0 --dump-items --out .benchmarks/185_home_appliance_suite_birth` (release, laya-riir-metal; corpus-optional — laya-only)
**PROVENANCE:** ⛔ preflight REFUSED at launch (load 7.99 > 6 — the `grafts_cell` sibling job at ~408% CPU is on the box; swap 4.17G, AC, powermode auto). **Latency NOT QUOTABLE — the run's own box-state stamps refuse it (start load 8.41 / end 7.28, both > 6).** Accuracy is the deliverable; the p50/p99s are posture-disclosed, never quotable baselines (the Bench-180–184 convention verbatim). The case-exact picks are deterministic (determinism_ok true, n=10 reruns clean), so the accuracy axis is load-insensitive.

## What landed

The JDI ladder wave-2's machinery — the board's all-sub-answers-exact
case scorer (`scoring/index.py` n==1 arm, ported) — plus its first
CONSUMING suite: board row 9, the home-appliance simulator. And the
claim's other half resolved REFUSED (the ACOS licence probe) before
any fetch:

1. **The ACOS licence probe: REFUSED, no build.** `NUSTM/ACOS @45d179a`
   (the board's own suite.md pin) carries NO licence at any
   authoritative surface: GitHub's repo API reports `license: null`;
   the README at the pinned sha is a citation REQUEST only ("If you
   use the data and code in your research, please cite our paper" — a
   citation is not a grant); the repo tree at that sha has no LICENSE
   or terms file at any level (`Extract-Classify-ACOS/`, `README.md`,
   `data/`, `img/` — the full tree, truncated=false). The board's own
   suite.md row 38 independently records "not stated in the
   repository". The underlying data compounds it: Restaurant-ACOS
   extends the SemEval-2014 set, Laptop-ACOS is collected from the
   Amazon domain. Same verdict class as VAST (no licence at any
   surface → REFUSED, distinct from NLI4CT's UNDECIDED where terms
   exist but defer). **No ACOS suite exists; the named re-open is an
   AUTHORS' STATEMENT — an owner call, never filed unilaterally.** The
   wave-3 triage's "ACOS behind wave-2's scorers" parked the MACHINERY
   gate; the licence gate fires first and stays shut. ACOS joins VAST
   in issue 114's refused residue; the machinery lands regardless (the
   home_appliance consumer below).

2. **The wave-2 case-exact scorer** (`case_exact_metrics` in
   `src/harness/metrics.rs`, 6 known-answer tests): a case passes iff
   EVERY question's forced pick is exact; `exact_accuracy` = the
   fraction of passing cases; `exact_chance` = the board's
   `random_exact_case` expectation — mean over cases of ∏(1/kᵢ) over
   the case's presented option counts (first-index argmax ties, the
   house rule; sequential fold in case order; empty input None, never a
   fabricated zero; misaligned arrays refuse loud). This is the
   board's n==1 scorer verbatim: one row per field, the case passes
   only if all fields are right, per-case chance = the exact
   expectation product. Emitted as an ADDITIVE `case_exact` column on
   `LaneResult` (serde skip-at-None — the leak-field wire law), computed
   by ONE shared helper (`case_exact_of`, gated by `CASE_EXACT_SUITES`)
   from BOTH lane assemblies (the modelless walk and the shared laya
   assembler — no duplicated metric code). k per question =
   `question_option_count` (choice: the criteria entry count; score:
   the level array; noul: 2).

3. **The home_appliance suite** (manifest §26): board row 9, the
   home-appliance simulator — C_semantic_options, the board's OWN
   generated track (MIT, "this repository"), 88 cases × ~18 choice
   questions = 1,590 questions. One case per kept household request;
   the questions are the board's own wire (the k=9 resolution
   decision, per-device target-membership k=2, the outcome summary, the
   state read, the post-handling field reads k=3–6); opaque `k_<hash>`
   option KEYS with the option TEXTS in the criteria values (the
   engine renders `{key}: {value}` — `render_options` — the texts are
   the signal); gold = the criteria KEY the board's `expected` names,
   mapped to its insertion-order position. State = the household JSON
   + the request text, verbatim.

4. **The fetch lane** (`fetch_home_appliance` in
   scripts/fetch_datasets.sh): the board's hub rows file
   (`multimodalart/decision-index-suite-0.3`) is PRIVATE (the HF lane
   refuses anonymous reads — probed 2026-11-12), so the lane
   REGENERATES the rows with the board's own generator
   (`decision_index/suite/build/home_appliance.py` at the clone sha
   `f5a6e4fd`, stdlib-only, deterministic seed 2026091807) and filters
   to the release-v2.1 subset's kept set. THE KEPT SET IS
   CRYPTOGRAPHICALLY PINNED: the subset file's 88 run_ids newline-join
   to sha256 `f76d0ef2…` = the file's own `run_ids_sha256` pin
   (verified matching). Regeneration proofs, all verified live at the
   landing: 240 rows (160 test / 80 dev = the suite.md census); the
   duplicate-state census 24 = the subset's `duplicate_rows_dropped:
   24` exactly; kept∩test = 88/88; kept∩dev-state collisions = 0;
   1,590 questions. Envelope blake3
   `52572e3451649d7a1ec034ebb401790a1b911f34c4c3fad988dee2919ab19556`.

5. **CORPUS-OPTIONAL** (`modelless_lane: false`): the board publishes
   no train pool for the track (the dev split is the DEDUPE reference,
   never a corpus — grammar-not-answers); only the LLM lanes answer.
   The builder + prepare layers pin the population (88 cases / 1,590
   questions — a drifted refetch fails loud, the when2call form);
   builder unit tests carry the four-arm discipline (the board-row
   mapping, the gold-key refusal, the score-type refusal, the
   population pin).

## The numbers (laya[english], the only lane)

| axis | value |
|---|---|
| **case exact accuracy (the board metric)** | **0/88 = 0.0000** |
| case exact chance (∏(1/kᵢ) re-derived on the kept set) | 2.1583e-07 (board pin 2.1031e-07 over the full 160-row population — Δ 2.6%, the §B3 re-derivation law: the actual presented options win) |
| per-question (field) accuracy | 0.4717 (642+66+19+1+22 = 750/1,590) |
| per-question chance (majority-class, the §B3 column) | 0.4472 |
| per-question chance-corrected skill | +0.0444 |
| per-k breakdown | k=2: 0.5071 · k=3: 0.4342 · k=5: 0.2969 · k=6: 0.0500 · k=9 (resolution): 0.2500 (random 1/9 = 0.111) |
| best case | juniper:08 — 15/17 exact (the honest near-miss; the all-exact bar is what the board scores) |
| determinism | ok=true, n=10 reruns clean |
| latency | p50 435 ms / p99 482 ms — ⛔ NOT QUOTABLE (load > 6 throughout) |

**The read:** the floor disclosure is the headline, honestly — the
laya encoder reads the sim-state wire at per-question near-chance
(+0.044 skill) and the ALL-QUESTIONS-EXACT bar compounds that to
zero: no request passed all ~18 questions. This is the board's own
metric doing its job (the whole-case bar is exactly why the board
scores `random_exact_case` at 2.1e-07 — the metric is BRUTAL by
design), and the shape mirrors the first-wave laya-only births
(when2call/bfcl/api_bank: honestly below the floor, the corpus-less
zero-shot disclosure). The per-k gradient is monotone and the k=9
resolution decision reads 2.25× random — the wiring is proven sound
(a gold-join bug would flatten every bucket to chance; the 15/17
best case confirms the join end to end). The suite is SCOREABLE:
the row is live on the harness, the teacher capture can ride it.

## The machinery's coverage after this landing

- **case-exact scorer**: LANDED (consumed by home_appliance; the
  same scorer Home appliances' full row and any future case-exact
  board row needs).
- **BRIGHT / ToolRet (nDCG@10)**: still open — their own new scorer
  (rank-based, not case-exact), its own claim (the triage's "one
  scorer buys both rows" note stands).
- **iSarcasm track B (the multilabel envelope)**: still open — the
  SuiteCase envelope already carries multi-question cases (this
  suite proves it); what track B needs is the multilabel OPTION
  shape (avg positive-class F1 over six labels), a scorer question,
  not an envelope question — recorded as the re-open beside the
  licence-clean source (iSarcasmEval is MIT; the T10 lane's probe
  covers it).
- **ACOS**: REFUSED at the licence (item 1) — the machinery gate the
  triage parked behind is moot; the licence gate is the binding one.

## Files

- `src/harness/metrics.rs` — `CaseExactMetrics` + `case_exact_metrics` + tests
- `src/harness/runner.rs` — the `case_exact` LaneResult field, `CASE_EXACT_SUITES`, `question_option_count`, `case_exact_of`, the modelless-lane wiring, the registry spec + the prepare population pin
- `src/harness/runner/lanes_wire.rs` — the shared laya assembler wiring
- `src/harness/suites.rs` — `build_home_appliance` + the four builder tests
- `scripts/fetch_datasets.sh` — `fetch_home_appliance` (the generator + pinned-kept-set lane)
- `.docs/02_protocols/dataset_manifest.md` — §26 + the licence-ledger rows (home_appliance ✓ MIT-this-repo; acos ⛔ REFUSED)
- `.issues/114_jdi_ladder_adapters_wave1.md` — the T11 landing + the lease fold (same commit family)
