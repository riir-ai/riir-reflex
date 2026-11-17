# Issue 121: Lexical floor lane + per-row flip census — the WSP paper's two memory-quality instruments (report-only)

> **Status:** Open — claimed (this lane executes the full one-session scope: A1 + A-pre + B1/B2; A2 only if the floor survives A-pre)
> **Source:** katgpt-rs [Research 679](../katgpt-rs/.research/679_WSP_No_Transformers_Associative_Memory.md) — Ok & Lee, "Weight-Space Programming Doesn't Need Transformers" (alphaXiv `2611.weight-space-programming-associative-memory`, 2026-11-17): (a) most written Transformers score BELOW plain BM25 over the writer's own text while sparse-table models score above — the empirical signature of dense-key interference; (b) deleting one written row flips 0.5–1.8 answers (sparse) vs 4.8+ (dense) — interference as a measured, banded, per-row number.
> **Claimed:** zed-amo-i121floor@amo, 1794936000, expires 2026-11-19T17:30:00+00:00
> (scope: the full session effort block below; datasets present at `.raw/datasets/`, harness bin cached; the landing amends this row to `**Lease closed:**` with the bench/record paths)

## Why

The harness compares the modelless engine against NB, ridge, GLiNER, CLM, laya and hosted lanes — but has no STANDALONE lexical baseline: the NB/ridge scorers (`src/nb_ridge.rs`, `src/nb_scope.rs`) are the engine's own hashed-lexicon halves, not lanes, so nothing asserts the corpus-is-the-model engine beats plain keyword search over its own corpus. The paper makes that floor the admission test for any authored/dense memory (scoring under BM25-over-author-text = discarding the author's signal). And nothing measures per-row interference on the arena's corpora: which rows are dead weight (zero flips), which carry outsized blast radius (many flips), and whether the corpus sits in the paper's sparse band (≈ ≤2 flips/row) or the dense band (≈ 4.8+). The census doubles as the deletion cross-check for decision receipts, for free.

## Item A — the BM25 floor lane

- [ ] **A1 — the BM25 scorer** (home beside `src/nb_ridge.rs`/`src/nb_scope.rs` — the LOCAL scorers' home; NOT `src/lanes/`, which holds the remote/hosted HTTP lanes — wired through the harness the way the local scorer selections are, `--nb-select`-class, never as a remote lane): std-only lexical scorer over the suite's own corpus rows — whitespace tokenization + idf + the BM25 k1/b form; retrieval → top-k train rows → label vote (the paper's `bm25/bm25.py` is the reference shape: score each option against the indexed text). NO new dependency (reflex stays one foreign code-level dep — katgpt-core).
- [ ] **A-pre — the redundancy cell (runs on the A1 scorer, BEFORE any floor row ships):** run BM25 beside the standalone hashed-lexicon scorers (`nb_ridge`/`nb_scope` alone, no corpus routing) on one sweep; redundancy = BM25 adds no suite-ranking or floor signal beyond them (rank correlation across suites + paired per-suite deltas). ⚠ The modelless-engine-vs-NB/ridge margins CANNOT establish this — the engine CONTAINS those scorers, so that comparison is engine-vs-its-own-parts. Record the cell either way; A3 adjudicates.
- [ ] **A2 — the floor row in the tables (only if A-pre shows BM25 carries distinct signal):** every suite row carries `floor: bm25` beside the modelless engine row. REPORT-ONLY — no exit-code change on the standard sweep. A suite where the modelless engine < bm25 is a CORPUS-DEFECT finding to file, never an auto-fix.
- [ ] **A3 — pre-registered kill bar:** the lane's value = a violation found OR a standing floor adopted. Close NEGATIVE iff: A-pre shows full redundancy (BM25 dominated by the standalone lexicon scorers on every suite) AND no engine-vs-BM25 violation exists across two independent runs. A single violation = the finding; if BM25 carries distinct signal, the scorer stays as the cheap standing floor row (bench_preflight posture).

## Item B — the per-row flip census

- [ ] **B1 — `--row-flip-census` probe** (exclusive early-exit mode, the null/rubric/routing-state shape in `src/bin/harness.rs`): for each corpus row (cap `--row-flip-max-rows`, default 512), delete the row, replay the frozen test read ONCE per deletion, count answer flips. Report per-row flip counts, the zero-flip dead-row count, and mean flips/row against the paper's bands (sparse-table class ≈ ≤2; dense class ≈ 4.8+). Outputs `<out>/row_flip_census.json` + `ROW_FLIP_CENSUS.md` (the probe family's output convention).
- [ ] **B2 — determinism canary:** delete one row twice → identical flip sets; a mismatch refuses the run (the probe family's identity-red precedent — a nondeterministic replay would fake interference).
- [ ] **B3 — receipts cross-check (free byproduct, report-only):** for any decision attribution that names corpus rows, the census mechanically verifies deletion-sensitivity — a cited row that flips nothing on the census read is a false citation; report the count, change no receipt format.

## Non-goals

- No promotion path, no default-on flip, no corpus auto-pruning — the census is a READ instrument; pruning stays with `--active-set`/cal-select-class decisions and their own gates.
- NOT the fixer's necessity lane: riir-refine `--necessity-rule` already prices per-rule retirement on the FIXER (per-decision mode). This is the ARENA's standing whole-corpus form. A refine-side census over RULE corpora would be its own issue citing this one, not a scope creep here.
- The corpus-cap tension stays recorded (Research 679 §1): the paper's top-1-row dominance is per-OPTION scoring-time evidence; reflex's corpus-cap negatives are corpus-SIZE index-time evidence. Do not cite one against the other.

## Effort

One session: A1 + A-pre + B1/B2 (A2 only if the floor survives A-pre). If A-pre shows full redundancy AND no violation, item A closes NEGATIVE with the cell recorded; item B ships independently either way. Suite set = the standing harness families; datasets per the standing fetch script.
