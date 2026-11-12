# Bench 182 — finentity suite birth: the language rung's second suite, the wave-3 residue's first passing probe (issue 114 T8)

**Date:** 2026-11-12 · **Repo:** riir-reflex (the harness/bin deltas land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` T8 (the wave-3 residue's first adapter; rethink 060 R3a — the T8 licence ordering resolved: FinEntity's probe passed first)
**Posture:** `LAYA_DEVICE=metal ./target/release/harness --suites finentity --laya-max-questions 0 --out .benchmarks/182_finentity_suite_birth` (release, laya-riir-metal; corpus-optional → LLM lanes only)
**PROVENANCE:** ⛔ preflight REFUSED (load 7.59–8.89 > MAX_LOAD 6, swap 4.17G — Zed + sibling sessions on the box; the run's own TABLES.md carries `load 7.01/8.89 > 6`). **Accuracy is the deliverable; the p50/p99s are posture-disclosed, never quotable baselines** (the Bench-180/181 convention verbatim).

## What landed

The wave-3 residue's first suite — the entity-given sentiment decision
(board id 39, B_mechanical; the T8 ordering note resolved: ContractNLI
licence CONFLICTED → UNDECIDED fail-closed earlier the same day, and of
FinEntity / VAST / NLI4CT the FinEntity probe passed first):

1. **The licence probe PASSED at two authoritative surfaces** — the
   authors' own readme at the board's pinned GitHub sha
   (`yixuantt/FinEntity@3b6cedc`) states verbatim "FinEntity is licensed
   under ODC-BY", and the authors' OWN HF dataloader (`yixuantt/
   FinEntity`, linked from that readme) tags `license:odc-by`. The
   mirror-tag-alone lesson never bites: the card IS the authors'. The
   fetched file is the authors' HF `FinEntity.json`, verified
   BYTE-IDENTICAL to the GitHub pinned `data/FinEntity.json` (710,430
   bytes; sha256 `3208667de69383120b0380aebeaabe360669eda72269c33e1c8b09d63df55463`), so the
   board's own source pin is honoured through an
   identical-bytes fetch. ODC-BY = attribution, no NC/ShareAlike — the
   StartLux DECLARED-NC precedent does not bite. The board's suite.md
   licence cell ("not stated in the repository") predates the readme's
   licence section.
   **VAST REFUSED the same probe pass** (the fail-closed law): the
   upstream `emilyallaway/zero-shot-stance` carries NO licence at any
   surface (no LICENSE file at the pinned sha e7c4775 — API listing
   checked; the README silent) — noted in issue 114, re-openable only
   on an author statement. NLI4CT's probe rides its own build start
   (the T5 pattern — never burn a verdict that can go stale).
2. **finentity** (suite §23 of the manifest): **2,129 cases** — one
   case per annotated entity SPAN across the 979 upstream documents
   (2,131 annotations − 2 quarantined conflict rows = 2,129 = the
   board's own `fields` count, the reconciliation that pins the
   shape). State = `{text, entity, start, end}` (the span PROVIDED —
   the board's own explainer); options = the three sentiment keys
   (sorted unique upstream labels, `FINENTITY_KEYS` exact-membership
   guard); gold = the span's label. **The quarantine is the board's
   own disclosed rule**: within doc 299 the same exact (start, end)
   span ("S&P 500") carries Neutral AND Positive — an unresolved
   annotation conflict; BOTH rows drop (never a guessed pick). The
   builder's census asserts (2,131 seen / exactly one conflicting
   span / 2,129 built) are the drift guard; the prepare-side
   gold-coverage pin requires all three classes present.
3. **The metric frame is span-level macro-F1** — the frame the
   board's own chance arithmetic pins: the catalog's Monte-Carlo
   random-chance 0.3201 is reproduced analytically over the span label
   distribution (**0.32016** over the built population, Δ 3e-5 = MC
   noise), while per-DOCUMENT frames do NOT reproduce it
   (first-annotation 0.3178, plurality 0.3158 — >10× the noise). The
   catalog's `requests=979` (one per document) is the board's own
   internal bookkeeping, disclosed in the manifest row, never
   inherited.
4. **CORPUS-OPTIONAL** (the T3 decision tree): evaluation-only
   unsplit source (the board's own disclosure — "not an invented
   official test split"; the authors' HF dataloader carries the same
   979 documents as one `train`-NAMED split — a convention, not a
   training pool), so the spans and their labels ARE the test content
   (grammar-not-answers). `modelless_lane: false`; the modelless lane
   skipped loud; only the laya lane answers. No new harness machinery
   (the wave-3 law holds: fetch stanza + builder + registry row).
   One fetch-lane repair rode the lane: `probe_license` now decodes
   `%2F` for the hub API (the first two-segment dataset id through the
   probe — the api endpoint wants the literal slash, 400 on the
   encoded form, verified live before the fix).

## The read (this record) — corpus-optional, laya lane, full population

| suite | lane | n | acc | **macro-F1 (the board metric)** | board chance | board skill | acc-frame chance/skill | ECE(maxp) | modal floor | p50 |
|---|---|---|---|---|---|---|---|---|---|---|
| finentity | laya[english] | 2129 | 0.7412 | **0.7433** | 0.3201 (random-choice macro-F1, MC) | **+0.423** | 0.5303 / +0.4490 | 0.0644 | 0.8027 ⛔ below | 11.0 ms |

- cases_digest: `fnv1a64-1f60e8fadf821bdb`; the fetched file blake3
  `e98766acd57d15a135905564d2917b545ba066a703de7769d23f32fe4e682cbe`
  (the manifest §23 row); determinism ✓ (the observed 10-case repeat,
  reported not claimed — the corpus-optional posture has no modelless
  claim).
- **G1 n/a** — corpus-optional carries no cal slice, so the calibration
  gates do not fire (no calibration claim is made; raw ECE 0.0644 is
  the readout disclosure).
- ⛔ **The honest birth disclosure: BELOW the modal floor.** acc
  0.7412 ≤ the input-keyed modal-answer floor 0.8027 (issue 094's
  anti-mimicry census: 904 keys, 342 fallbacks, reference-modal share
  0.5303) — the laya lane carries NO input-independent signal on this
  suite at the corpus-less zero-shot posture. The lane reads FAR above
  the random-choice chance (macro-F1 0.7433 vs 0.3201, +0.423 board
  skill) — real per-span discrimination, but below what answering the
  modal class per input key would score. The birth records the floor;
  the corpus/teacher re-open paths (the rethink-side booking, 060
  R3a/R1b, owner-gated) are where a better read comes from. The
  suite's VALUE lands regardless: the board row is now SCOREABLE by
  the stack, the teacher capture can ride it, and every future distill
  gets the paired our-suite read the B5 law requires.
- ⛔ **Latency NOT QUOTABLE** — the PROVENANCE line above: preflight
  REFUSED (load 7.59–8.89, swap 4.17G). The p50 is the run's own
  posture-disclosed reading, carried for continuity only.

## Gates

- 600 lib tests green (the 4 new finentity builder arms: the
  state-faithful span rendering + the FINENTITY_KEYS order, the
  conflicting-span quarantine dropping BOTH rows, the same-label
  duplicate retention, and the drifted-label-universe refusal);
  clippy `-D warnings` clean at the lib posture.
- The population/coverage pins: builder census asserts (2,131 / 1
  conflicting span / 2,129 built) + the runner's FINENTITY_KEYS
  exact-membership guard + the three-class gold-coverage pin in
  `prepare`.
- Licence rows landed BEFORE any published number (the manifest §23 +
  the licence-ledger row: ODC-BY at both authoritative surfaces; the
  probe signal `license:odc-by` re-asserted by the fetch stanza).
- grammar-not-answers: no train pool exists or is fetched; the corpus
  re-open is the rethink-side synthesis route (unchanged). No test
  rows in any pool — the suite answers on its eval surface only.
- Fetch self-test unaffected (the probe_license `%2F` decode rides the
  existing lane; the self-test's scenarios cover the fetch/probe
  machinery, not a live hub name).

## Recording convention

The birth read is the FIRST read at the corpus-optional zero-shot
posture — never a board row until the rethink-side fold books it (060
R1d/R3a's capture side, owner-gated amo hours, unchanged). The wave-3
remaining suites (ContractNLI UNDECIDED pending an upstream-authoritative
licence read; VAST REFUSED no-licence; NLI4CT unprobed — probe rides its
build start; ACOS + iSarcasmEval behind wave-2's scorers) file one at a
time per the issue's law.
