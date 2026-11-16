# Issue 120 — the train-carved gate cal under-predicts test escalation: audit the shift at BOTH carve sites and re-audit the three served suites

**Status:** OPEN — filed from rethink plan 018 (verdict session `7c0cb628` r2 AGREE, condition R4: fix the cause once, not per-suite).

## The defect (measured, rethink benches 101/102)

`prepare` carves the fused-gate cal slice from the **TRAIN split** (`stratified_split(&train_rows, …)` at `src/harness/runner.rs` ~L5652) while the eval rows are the validation/test split. Train rows sit systematically closer to the train-built corpus than eval rows do, so **every rate-targeted gate fit under-predicts the test escalation rate**, and the gap GROWS with ρ:

- hellaswag_jdi (rethink bench 092, train-cal): cal 62.5% → test 82.05% = **+19.6pp** at ρ 0.60 — the shift guard refused every arm above ρ 0.10.
- hellaswag_jdi with the val-held-out repair (rethink bench 102, `--cal-from-val`): cal 60.0% → test 67.2% = **+7.2pp** — the shift is ~one third of the defective posture at the same ρ. The residual 7.2pp is inside the 10pp guard.
- winogrande_jdi (rethink bench 091): shift +3.6pp at ρ 0.60 — the 2-option shape tolerates it; its whole validation split IS the test panel (`test_cap: 0`), so it has NO val pool and carries the pre-registered per-suite window instead.

## The scope

- [ ] **Site 1 — `runner.rs` prepare:** the cal front comes from `stratified_split(&train_rows, spec.name, spec.cal_cap)`. Where the eval split's pool has room OUTSIDE the test identity set (the hash-ordered JDI suites: `val_cal_carve`, landed `a7acacc`), the gate-fit cal should prefer same-distribution eval rows.
- [ ] **Site 2 — `gate_drift.rs` `load_suite_artefacts`:** same carve (`stratified_split(&train_rows, …)` for the cal slice) — the drift instrument inherits the same under-prediction bias.
- [ ] **Re-audit the three SERVED suites' recorded cal→test shifts under this lens** — their arming evidence was the train-cal rate transfer:
  - emotion_syn (armed shift recorded — the Bench 077 +21.8pt prior is the compose-axis sibling);
  - massive_intent_en (Bench 086: shift +0.2pp — likely fine, disclose);
  - banking77 (Bench 087: shift +3.8pp — likely fine, disclose).
- [ ] **Disclose the no-pool suites:** suites whose whole eval split is the test panel (`winogrande_jdi`, `test_cap 0`) — and any other `test_cap: 0` row — have no val pool; their records must say the train-cal carve is the only calibrated source and carry the measured shift as-is.

## The landed repair (context — the mechanism is general)

- `runner.rs` `val_cal_carve` (skip = test_cap, take = cal_cap, the same `jdi_row_identity_hash` order; returns sorted identity-hash sets of both sides; refuses loud on test_cap 0 / no room / overlap) + the `Seat::gate_fit_cases` override (thresholds fit on the override cases against the FULL train corpus; engine selections untouched — the minimal repair; landed with plan 018's pre-registration commit).
- `Seat::gate_fit_cases` keeps the engine (cap/head/nb/oc/ridge selections) byte-identical — ONLY the gate thresholds re-fit. Any broad re-seat (selections on val rows) is its own measured lane, never a silent rider.

## Reference

- rethink `.plans/018_jdi_panel_gap_closure.md` (the pre-registration + verdict lineage).
- rethink `.benchmarks/101_wino_rho060_reread/` (the re-arm) + `.benchmarks/102_hswag_valcal_reread/` (the repair measurement + NO-ARM negative).
- Reflex changes landed in the plan-018 family: `val_cal_carve` + visibility widens (`a7acacc`), `gate_fit_cases_override` (this issue's landing commit).
