# Issue 116 — T1 consumer intake: selection hysteresis latch (`pick_latch`) — the `pick_domain` seat

**Status:** OPEN — intake issue, no wiring yet; consumer half of katgpt-rs Issue 987's selection-hysteresis lead (T0 landed `c9bdb533b`, 2026-11-09: `katgpt-core/src/pick_latch.rs`, opt-in feature `pick_latch`, catalog entry 165, 15 gate-mechanics fixtures green). Claim via the lease law before implementing (`git fetch origin && py ../katgpt-rs/scripts/claim_lease_check.py --item 116`, append the `**Claimed:**` line under this Status, push — 48 h expiry, the push is the arbiter). Twin intake: riir-instinct Issue 032 (the A7 LRU eviction seat).
**Claimed:** zed-m3-idle3 (mac m3, sprint#003-followup housekeeping lane), 1794261826, expires 2026-11-12T0537Z

## Producer (landed; zero new deps; byte-identical when off)

`PickLatch::decide` + `LatchDecision`: the gate is `exact_sigmoid((challenger − held)/temperature) ≥ threshold` — the committed-value-path form, so the additive margin is exact (`temperature·lnit(threshold)`; ln 3 ≈ 1.099 score units at the defaults). State = the held index ONLY (stationary surface). Argmax = the `pick_domain` comparator (`float_order::cmp_for_max` + first-index tie), so a cold latch seats exactly what bare `pick_domain` would. Abstain/empty ⇒ reuse; NaN holds; clamped construction, no panic path. Sigmoid only, never softmax (house law).

## What this repo would consume

The `pick_domain` corpus-routing seat: hold the last domain pick unless the challenger's score beats the held pick by the sigmoid margin — a near-tie re-routes today on every evaluation; the latch makes routing steady under score noise while keeping cold-boot and clear-win behavior identical. Knobs ride envs per the env-switch law (opt-in feature first, PROMOTE-ONLY literals until G3). The per-suite cal fits stay the producer of the scores — the latch only steadies the pick, never rescales a score, and the fused-gate/confidence readouts are untouched (their thresholds do not transfer, per the harness laws; the latch adds no threshold of its own beyond the pinned margin).

## Bars (pre-registered before any wiring)

- **G1 (purity):** a seeded synthetic score stream replays through the reflex seam bit-identical to the producer fixture verdicts — same inputs, same picks (the producer's always-from-base/stationary pins carry over).
- **G2 (no regression):** feature-off byte-identity — every harness suite's routing decisions + decision receipts identical to today; decision receipts (blake3 canonical decision) unchanged on the on-path too when the latch holds.
- **G3 (the GOAT):** gate-on vs gate-off on the real harness stream — domain flip-rate reduction vs bare argmax (the 987 wording) WITHOUT suite-accuracy loss (paired LB95 ≥ 0 per suite) and no G2-bar latency delta (the modelless p99 ≤ 1 ms lane unchanged). Judge-noise handled honestly; `decline` (keep the held pick) is the default answer forever; no promotion talk before G3; may fail honestly — a verdict, not a bug.

## Tasks

- [x] T0 claim + seam read (locate the `pick_domain` dispatch seat in `src/`; name the config home for temperature/threshold; verify the producer fixture surface imports consumer-side) — seat = `engine.rs` solve path's `route_pick` (was the bare `pick_domain` call); config home = `EngineConfig.route_latch_gate` (+ the harness flag / the serve env for arming); producer import = `katgpt_core::pick_latch::{PickLatch, LatchDecision}`; score fold = `router_scores` (the `routing_state_probe::router_scores_of` pinned-fold shape, pub as the G1 parity surface)
- [x] T1 wiring behind opt-in `pick_latch` (forward the katgpt-core feature, never fork — the consumption precedent; `--no-default-features` stays the tested flag-OFF posture) — landed `1540a07`: feature forward, the seat twins, the config knob, engine-owned latch state, reset + route_probe surfaces, the harness `--pick-latch` family (main lane ONLY — the 7 measurement probes + the selection engines stay latch-off, loud refusal without the feature), the serve env family (promote-only literals, both-or-none gate values, knob-does-nothing refusals)
- [x] T2 G1 + G2 gates — `tests/pick_latch_gates.rs` 7/7 armed: G1 fold parity (512-case seeded battery incl. exact ties vs `pick_domain`) + cold-seat byte-identity; G2 unarmed byte-identity + reset-restore; gate-mechanics replay through the real fold (margin = ln(3) ≈ 1.099 at the defaults, NaN-hold, abstain-reuse, first-index ties); seat-level hysteresis (armed flips ≤ bare flips over a 96-row near-tie stream). clippy clean ×3 postures; lib 563/563 both postures; engine_gates 9/9 armed; `--no-default-features` clean
- [ ] T3 G3 GOAT on the real harness stream; honest negative allowed → close with the verdict either way — PENDING the box: `bench_332_karc_goat` holds m3 at ~199% CPU (the Plan-337 posture; the release build + harness cell defer). Cell design when it fires: the modelless suites, paired `--pick-latch` on/off runs (paired LB95 ≥ 0 per suite) + the flip-rate read through `route_probe` over the same state texts + the G2-bar latency lane unchanged. The lease stays live until the cell lands.

## Refs

- katgpt-rs Issue 987 (selection hysteresis row; T0 `c9bdb533b`) · katgpt-rs Research 657 §A1 (Temporal MoE analogy — OURS, not a paper claim)
- riir-refine Issue 220 (the twin intake shape: adoption-gate consumer wiring)
- Reflex Bench 128 (the option-permutation spread probe — the measured permutation-noise class the latch absorbs)
