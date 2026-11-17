# Issue 122 — ESC worthiness gate supervised on outcome-delta (δ) targets

**Status:** OPEN — unclaimed. Routing: the ESC comparison family's lane (the Bench 195 esc-capture lane's owner, or any ESC-family lane); production serving-side adoption is NOT in scope here.
**Source:** riir-reflex `.research/018_Sparse_Reward_Subsystem_Probes.md` (arXiv:2602.00986, sparse reward subsystem) — the paper's dopamine objective (predict step-level improvement δ before the step) applied at our escalation boundary.
**Date filed:** 2026-11-17

## The cell (one sentence)

Fit the ESC worthiness gate on **δ = think−cheap outcome-delta targets** from captured `(cheap, think, gold)` triples — same features, same fit machinery as the incumbent `gate_fit_calibrated` posture, only the TARGET changes (predicted escalation value, not cheap-lane doubt) — and adjudicate paired escalation precision/value on one frozen test read.

Why: the incumbent gate predicts "cheap lane unsure", which is not "escalation will help" (a hard question stays hard for the think leg). The esc corpus already records both legs' correctness, so the δ target is a relabeling of existing captures, not new collection.

## Tasks

- [ ] **T1 — corpus: the Bench 195 capture IS the corpus (floor already met).** The capture lane pins `ESC_CAPTURE_SUITES = ["massive_intent_en"]` (esc_capture.rs:34; other suites REFUSE — banking77 is banned by the split law), so a multi-suite floor is structurally impossible without a new caveat-cleared lane — do NOT widen the lane for this cell. The floor: **≥ 200 triples from massive_intent_en**; the Bench 195 capture already holds exactly 200 (`195_issue117_esc_capture_massive_run/capture/triples.jsonl`, all cal-split, measured δ composition +1: 52 / 0: 122 / −1: 26 — non-degenerate). A fresh capture run is needed ONLY if the frozen read (T3) needs a disjoint test capture.
- [ ] **T2 — the δ-fit arm, binary target.** The incumbent gate is a binary sigmoid gate, so the PRIMARY target is binary: label = `δ > 0` (escalation helped) vs `δ ≤ 0` — a straight relabeling of the captured triples (δ = think_pick==gold − cheap_pick==gold ∈ {−1,0,+1}; the tie class folds into the negative side, mirroring "escalate only when help is expected"). A REGRESSION form (raw δ) may ride as a REPORT-ONLY secondary arm if the fit machinery allows, never the adjudicated posture. Features identical to the incumbent fit. Keep the incumbent correctness-fit arm trained on the SAME rows as the paired control.
- [ ] **T3 — paired adjudication, fit/test SPLIT.** The 200 cal rows are the FIT set (cross-fit or nested split INSIDE them — the fit never sees the test rows). The adjudication read is a SEPARATE frozen test capture (one fresh `--esc-capture` run on the same posture), verified disjoint against the fit corpus's `captured_ids.txt` before scoring — the capture lane may never mine the test split. Bars (pre-registered here): (a) escalation precision (escalate ∧ think-better) Wilson 95% LCB ≥ the incumbent arm's by a paired margin — report the delta with CI, and (b) value-per-escalation (mean δ over escalations) non-inferior at the SAME escalation rate ± the rate band; (c) the escalation rate must stay inside the lane's band (the rate contract is untouched by this cell).
- [ ] **T4 — falsification arm.** Inverted-δ fit (labels negated) must collapse below the incumbent on axis (a). Additionally the HONEST-DEATH arm: if the δ-fit and the incumbent are statistically indistinguishable on the frozen read, record NO GAIN and close the cell — a null result is a verdict, not a failure to promote.
- [ ] **T5 — bench record + verdict.** Land `.benchmarks/NNN_esc_delta_target_cell/RESULTS.md` (per-arm rows, box-state PROVENANCE line per the house bench law). If the cell claims CALIBRATED PROBABILITIES (not just precision/value), the conformal-naive floor (`ConformalIntervalCalibrator<SeasonalNaiveForecaster>`) is a mandatory comparison arm — prefer the precision/value framing to stay out of UQ scope.
- [ ] **T6 — routing note on close.** If PASS: state where the result goes (the serving-side family reads comparison-lane results through its own gates; do not edit serving posture from this repo).

## Refusals / scope

- No serving-posture edits from this repo; this cell is harness-comparison only.
- No new capture lanes if the Bench 195 lane still runs — reuse it.
- If the corpus is label-degenerate in the δ sense (e.g. escalations almost never flip the outcome), that IS the honest no-signal verdict (T4), not a reason to re-sweep thresholds until something moves.

Unclaimed — no lease line yet. Whoever picks this up appends the `**Claimed:** <session>, <epoch>, expires <ISO-UTC>` line under the Status line per the house rules.
