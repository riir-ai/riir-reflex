# i122 T3 — ESC worthiness δ-target served-side cell (incumbent / δ-fit / inverted-δ)

**Status:** COMPLETE 2026-11-28T05:29Z — verdict **FAIL → NO GAIN, cell CLOSES** (pre-registered bars; the honest-death arm applies). The job: amo rig `01a2209c` (Shared lane, in-script Plan-337 guard waited out leg-A — `mtp_block_economics` exited 05:26Z, arms ran 05:27–05:29Z; the job's exit 1 is the adjudicator's pre-registered FAIL encoding, not an infrastructure failure). Issue: `.issues/122_esc_worthiness_delta_target_cell.md`. This record lands T5; T4's control finding and T6's routing note are below.

## Arms (Bench-195 posture verbatim: `--nb-select --oc-select --ridge-select --cascade --cascade-worthiness --cascade-worthiness-margin 0.05 --gate-fit-selection --gate-distance-only --gate-rho 0.60`, Metal, one frozen 300-case test read each)

| arm | n | escalated | think_only | cheap_only | precision | Wilson95 LCB | value (mean δ) | esc rate |
|---|---|---|---|---|---|---|---|---|
| incumbent | 300 | 173 | 18 | 35 | 0.1040 | 0.0668 | −0.0983 | 0.577 |
| delta (`--esc-delta-fit` Bench-195 triples) | 300 | 173 | 18 | 35 | 0.1040 | 0.0668 | −0.0983 | 0.577 |
| inverted (T4 control, `triples_inverted.jsonl`) | 300 | 173 | 18 | 35 | 0.1040 | 0.0668 | −0.0983 | 0.577 |

The three arms' served-side aggregates are **row-identical** — the δ target changed nothing on the served surface.

## Bars (pre-registered in the issue BEFORE the read)

- **(a) precision: FAIL** — δ-arm LCB 0.0668 vs incumbent point 0.1040; Newcombe mover CI [−0.0916, +0.0916] (unpaired — no row-level cross-arm pairing by the split law, DISCLOSED) straddles zero.
- **(b) value: PASS** — δ −0.0983 − incumbent −0.0983 = +0.0000 (bar ≥ −0.02) — vacuously, everything is identical.
- **(c) rate band: PASS** — 0.577 vs 0.577 (±0.10).
- **T4 control: DID NOT COLLAPSE** — see the mechanism row below (this is a *predictable* non-collapse, recorded as the cell's own mechanism evidence, not a scorer defect).

## Why every arm is identical (the T2 mechanism, now proven served-side)

T2's fit-side finding ("the percentile posture cuts the same quantile regardless of the label") IS the served-side result's cause, now demonstrated end-to-end: `gate-fit-calibrated` fits a **score-percentile threshold**, and the score axis (cheap-lane confidence/distance features) carries no δ information for it to re-rank — the δ relabeling can only move which quantile *counts as* the boundary, never move the boundary off the same score quantile. Three independent fits (gold labels, δ>0, δ<0) landed the same cut, so the three arms serve byte-identical escalations (173/300, the same rows). **The cell's negative is a POSTURE finding, deeper than "the δ target has no signal": at ρ=0.60 with `--gate-distance-only`, the fit machinery cannot express a target-driven cut at all.** A future δ lane would need a threshold that moves off the percentile posture (e.g., a direct δ-regression head on the escalation features) — a different mechanism, out of this cell's scope by its own §Refusals (same features, same fit machinery, only the target changes).

## T5a ceiling row (Bench-195 fired-set 2×2, n = 117 — cite 117, never 200)

| ceiling | served think | cheap alone |
|---|---|---|
| 85.5% oracle (100/117) | 70.1% (82/117) | 55.6% (65/117) |

Headroom 15.4 pts = exactly the 18 cheap-only rows. The 15.4-pt headroom stands untouched by this cell — the δ target reached none of it.

## Disjointness + slice identity (asserted before the read)

- Cross-arm slice identity: identical (`fnv1a64-328f111f90d7a0c1`, the Bench-195 test slice — all three runs' `cases_digest` match).
- Fit/test split: the fit corpora are the Bench-195 **cal** capture (200 ids, `195_issue117_esc_capture_massive_run`); the served read is each run's **test** half (300 cases) — disjoint by the split law's clause-3 construction.
- All three arms' capture triples carry sha256-prefix `938cb428e61fb645` — byte-identical to each other and to the Bench-195 fit triples (the capture lane reproduces exactly; the delta/inverted arms relabel the same rows).

## Box-state provenance (per the house bench law; correctness reads — routing aggregates, never latency claims)

PROVENANCE: host=amo, profile=release, rustc 1.99.1; per-arm results.json `meta.box_state` (start→end):
- incumbent @ `32cc6fe`, 05:27:56Z — load 5.69→3.92, AC Power, ane=ok
- delta @ `32cc6fe`, 05:28:44Z — load 3.69→3.27, AC Power, ane=ok
- inverted @ `32cc6fe`, 05:29:33Z — load 3.27→3.44, AC Power, ane=ok
Rig job `01a2209c` waited out the leg-A economics cell in-script (guard poll, 05:24→05:26Z) before building — the Plan-337 contamination rule held; all three arms ran inside the opened quiet window.

## T6 — routing note on close (FAIL branch)

The ESC comparison family records the negative: the δ-target relabeling of the existing capture produces **no served-side change at the incumbent posture**, and the posture finding above (percentile-fit cannot express a target-driven cut) is the load-bearing lesson for any future re-open. NO serving-posture change (out of scope by the issue's own §Refusals), NO corpus-lane edit, NO capture-lane widening. Re-open trigger: a mechanism that can move the cut off the score-percentile posture (a direct δ head over escalation features), filed as its own issue with fresh pre-registered bars — never a re-tune of ρ/margin on this posture.

**Session:** zed-46faf6c8@amo, 1795843500 (T3 booking `32cc6fe` + this record's landing; the arms ran as rig job `01a2209c`, adjudicator `/tmp/i122delta_run/i122_t3_adjudicate.py`, draft + per-arm results.json committed beside this file).
