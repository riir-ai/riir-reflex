# Bench 208 — i135 root cause: the armed site posture reproduces the published sst5 cell digit-exact at today's HEAD

- 2026-11-28T07:37Z, amo, release, binary at reflex c497e52 (develop tip), isolated CARGO_TARGET_DIR=/tmp/i135-armed-tgt
- Command (the FULL site posture per bench 137 RECORD.md): `REFLEX_BENCH_HOST=amo ./harness --skip-laya --nb-select --oc-select --ridge-select --drafter-fix per_byte --suites sst5 --out <this dir>`
- PROVENANCE: AC power, load ~5.1 at launch (post-exclusive window; amo 0 managed jobs), box_state in results.json; accuracies/digests are load-insensitive (the claim is identity, not latency — latency NOT quotable here, never published from this cell)

## The cell (vs the published reflex-site/data/bench.json sst5 modelless row, source_run 197e855 era)

| | this run | published |
|---|---|---|
| sst5 hard acc | **0.39666666666666667** | 0.39666666666666667 — digit-exact |
| corpus_digest | **fnv1a64-726f717634ca24fb** | fnv1a64-726f717634ca24fb — byte-exact |
| cases_digest | fnv1a64-ad64a6cb0039e916 | fnv1a64-ad64a6cb0039e916 |
| nb selection | scale 16.0, observed-laplace | scale 16.0, observed-laplace (bench 137) |
| transductive acc | 0.39666666666666667 | — |

## What this closes

Issue 135's premise ("the served arm regressed to ~chance") is REFUTED: the 0.2017 readings
were the BARE posture (no selection flags; `nb_posture: OFF` in every bad run's meta), compared
against a published cell recorded at the ARMED posture. Full chain + the bare-vs-armed table +
the contributor bug (the misleading meta string) live in `.plans/014_i135_posture_verdict.md`
(and the mirrored updates in this lane's resolution commit). NO engine fix, NO re-baseline.
