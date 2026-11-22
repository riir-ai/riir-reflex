# Issue 131: synth-corpus payload-only hardening (CPU-ARC §D STAGE-2 adoption)

**Status:** Open — filed from [`.research/019_CPU_ARC_Reasoning_Cache_Serving.md`](../.research/019_CPU_ARC_Reasoning_Cache_Serving.md) (arXiv:2611.09891 §D STAGE 2). Route: reflex harness lane; self-contained and unblocked; gate = the lane's own echo gates + corpus-ab V5.

## Why

The `--synth-corpus` lane (teacher-synthesized train-pool data, echo-gated) currently accepts free-form teacher variants. CPU-ARC's §D STAGE 2 ships a stricter, more leak-scannable augmentation shape — and the leak-scannability is the point: when the scaffold is a CONSTANT, a leak scan hashes the scaffold and scans only payloads against the test echo index (high precision, near-zero false positives). Four upgrades, all additive, feature-off byte-identical:

- [ ] **T1 — payload-only structural identity:** the teacher prompt constrains variants to STRUCTURALLY IDENTICAL requests differing only in payload (the concrete question/data), preserving the instructional scaffold verbatim; the harness records the scaffold hash per row so the constraint is CHECKED, not assumed (a row whose scaffold hash diverges from the seeding record's is refused loud, counted in the run summary).
- [ ] **T2 — degeneracy filters:** refuse-and-count rows with completions under a minimum length (paper: <10 chars) or with leaked chain-of-thought markers; mirrors the paper's regex-extract + retry posture, minus the retry loop (our echo gates are the authority — a dead lane stays dead).
- [ ] **T3 — id-tuple exclusion (the contamination closure):** every harvested/synthesized row carries a deterministic id tuple (source, suite-id, row-kind, variant-index); the lane refuses any tuple whose source is a TEST-row id — payload-only variants of eval rows are contamination-by-synthesis (the Bench-089 class) and are excluded AT HARVEST, not filtered later. Cal rows are the lawful source.
- [ ] **T4 — gates:** extend the echo-gate battery with a planted-violation arm per T1–T3 (bad scaffold hash / degenerate completion / test-row tuple must each refuse loud); `--corpus-ab` runs green with the hardening on; flag-off posture byte-identical to today's lane.

## Non-goals

No retry-with-reduced-counts loop (the paper's); no change to the V5 paired gate; no new datasets.

## Proof path

Existing: `scripts/fetch_datasets.sh` corpora + the echo-gate battery + `--corpus-ab` V5 paired read. T4's planted-violation arms are the falsification proof; a green corpus-ab with hardening on is the no-regression proof.

**Claimed:** zed-amo-071num@amo, 1795364600, expires 2026-11-25T12:00:00Z
