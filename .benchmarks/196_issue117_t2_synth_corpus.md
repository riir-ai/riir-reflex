# Bench 196 — Issue 117 T2: the teacher-synthesized train-pool corpus (massive_intent_en, openthai teacher)

**Date:** 2026-11-15
**Lane:** reflex issue 117 T2 (riir-train plan 426 T5 synth lane lineage; grammar-not-answers law binds — built AROUND the frozen test, never from it)
**Claim:** zed-amo-hk-633@amo (T2 only; see the issue's claim row)

## What ran

The sealed synth-corpus cell over the **massive_intent_en** suite with the **openthai** teacher (openthai-systemone @ `5d04bcca`, uvicorn :8000, the `thai_rerun.sh` staging pattern): 212,912 candidates → veto 1,475 + dedup 4,598 + cal-side dedup 27/pool 65 → **2,048 accepted** (the ≤2048 / ≤128-per-label budget), veto forwards 3,523, p50 latency 355 ms, 1,387 s wall. Weighting = per-intent E0 rumor fraction (bag view, n<4, own-tables; unmeasured labels take the mean) — the T1 capture's 68 disagreement rows were the steering census; the mined ids stay OUT of the corpus by the split law (T1's wall; the T3 `--corpus-ab` disjointness read asserts it against `captured_ids.txt`).

## Artifact (sealed)

`.benchmarks/196_issue117_t2_synth_corpus/`:
- `massive_intent_en_synth.jsonl` — 2,049 lines = 1 SYNT v2 header + 2,048 rows
- `massive_intent_en_synth.jsonl.blake3` — `f47ef5552043d9e4fe3c321a08fad5874d6d942d193f728f07676f19a594a634` (verified amo-side post-copy: b3sum == sidecar == the bmo report's digest, three-way)
- `synth_report.json` — meta (git_sha `1796ad6`, **host: bmo**, teacher_provenance `openthai:openthai-systemone`, weighting, dedup + veto census), per-suite block, the label-bucket table
- `synth_plan.txt` — the report-only allocation preview (writes nothing; the pre-flight)
- `SYNTH.md` — the cell's own machine-generated summary

## Box + execution disclosure (the honesty row)

- The cell was ALLOCATED on amo exclusive, moved to **bmo quiet** by the orchestrator (queue contention: the 633 RTFx re-runs own amo's exclusive lane tonight), resubmitted as `01a1dced-c838` after the first attempt died at the teacher staging (`uv` not on the runner PATH — the script's `>/dev/null 2>&1` swallowed it; fixed: uv by absolute path + stderr visible, venv pre-staged by hand, then re-run clean).
- Build HEAD: bmo reflex `1796ad6` (== amo HEAD at submit — verified in the artifact header; the plan-era `7ce647d` pin is superseded by this disclosure).
- Poison events: ONE — `mediaanalysisd` (a macOS system daemon) at 02:04:11Z, mid-synthesis. Class ruling: the synth cell's outputs are a sealed artifact + counts (never latency), and generation is load-invariant → the artifact STANDS; the poison is recorded here per the honesty law.
- scratch cleanup: `.build_i117_synth.sh` + `/tmp/i117_disagreement_ids.txt` deleted at fold (the artifact + this record are the only keepers).

## Gates

- echo gate / leak scan: the split law held — mined ids ≠ corpus ids (the harness's own veto/dedup pass ran; the T3 disjointness read is the standing assert).
- teacher veto: the REQUIRED veto teacher ran (veto_rejected 1,475 rows refused); density gate OFF (the shipped veto-only posture; T2's deliverable is the plain V5-consumable artifact).
- budget: 2,048/2,048 accepted, per-label ≤128 honored (the bucket table in synth_report.json).

## T3 handoff

The frozen `--corpus-ab` read consumes THIS artifact with `captured_ids.txt` disjointness asserted (the T1 split law's closing assert). T3 stays unclaimed.
