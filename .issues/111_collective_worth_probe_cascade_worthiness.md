# Issue 111: Collective-worth probe — cross-lane error correlation as a cascade-worthiness arm

**Status:** In progress — T1/T2/T3 claimed (m3-i111, epoch 1794269687, expires 2026-11-12T00:14:47Z); T3 pre-registration committed before any run (below); T4/T5 ride the run record. Fusion idea (source: dcog manifesto 2026-10-22 + VINCO post 2026-11-05, distributed.inc; internal distill 2026-11-06, private sibling repo). PoC scoped below, report-only first. Named NEGATIVE precedent to respect: reflex Issue 107 / Bench 173 (routing-state probe found no signal beyond the routed argmax). Design revised at verdict round 1: size-aware committee verdict + matched-cost gate.

**Claimed:** m3-i111, 1794269687, expires 2026-11-12T00:14:47Z

## Problem

`--cascade-worthiness` (Issue 042 L3, reflex `harness/cascade.rs::WorthinessInput`) arms/disarms per-suite escalation from a **head-to-head delta**: laya acc − modelless acc on the cal slice, margin 0.16 (Bench 063/066), plus the LCB support leg (Issue 046 L4). That signal cannot see **error correlation**: two lanes that are individually close but wrong on *different* rows combine into something that beats both; two lanes with a big delta that err on the *same rows the same way* do not (dcog's claim, grounded in arXiv:2506.07962 — models agree 60% of the time when both err; theory: Wood, JMLR 2023; classical: query-by-committee). The stack measures per-instance disagreement (`mc_ensemble`/`perturbation_ensemble` U_pair — within-lane input perturbation; `velocity_field_disagreement` — field-ensemble members) and per-suite delta — but nothing measures **cross-lane error correlation as a suite-level pre-flight**. Concept class is published; the internal composition is the claim. Gain-tier only if it beats the delta gate at matched cost.

**Gate shape constraint (round 1):** `WorthinessInput` is a **2-lane** gate (laya escalator + modelless). At n=2 majority voting is undefined (every disagreement is a tie) and the classical n′ = n/(1+(n−1)ρ̂) correction predicts nothing usable — so the verdict below is size-aware and EMPIRICAL, never formula-driven.

## Task

- [ ] **T1** — Probe implementation, report-only (new opt-in flag, default off, byte-identical posture): on the suite's cal slice, for every lane pair with answers on the same rows, compute the **agreement-on-error rate** — among rows where both lanes err, the share where they emit the SAME wrong pick (the arXiv:2506.07962 convention; computable because our lanes emit discrete picks). Secondary diagnostics: both-wrong rate and Cohen's κ over picks. The definition used is disclosed in every output row. Emit a `collective_worth` block into the cascade row (the `WorthinessVerdict` serde pattern — additive, skipped when off).
- [ ] **T2** — Size-aware committee verdict, EMPIRICAL first:
  - **n ≥ 3 lanes on the same cal rows:** committee = majority pick, accuracy measured DIRECTLY on the cal rows (all lanes' answers already exist there) + Wilson LCB. ρ̂ and the classical n′ = n/(1+(n−1)ρ̂) correction are labeled HEURISTICS, reported only as predictions for committee sizes NOT yet run — never the verdict.
  - **n = 2 (today's gate shape):** no majority exists — the committee is the TIE-BROKEN pick: when the lanes disagree, take the calibrated gate's confidence, first-index rule (the determinism law). The verdict statistic is THAT committee pick's accuracy measured on the cal rows + Wilson LCB. The complementarity/union bound P(at least one lane right) is reported as a CEILING only — it assumes the right lane is always picked on disagreement, so it is unreachable by a deployed gate and never a verdict input (fixed at verdict round 2).
  - verdict = `worth_committee` / `not_worth` / `undecidable` (probe rows < floor ⇒ undecidable, never folded — the esc_voi discipline).
- [ ] **T3** — Matched-cost gate (the GOAT bar, pre-registered BEFORE running): a committee runs both lanes on EVERY row while the delta gate spends only on abstained rows — an accuracy-alone comparison favors the committee by construction, so accuracy alone can NEVER pass this gate. Primary axis: the escalation/spend budget (the Bench 066 rate-lever shape) — at equal compute, does cross-lane-informed arming beat delta-only arming on serving accuracy? Every posture reports its accuracy/cost PAIR. Promote path: combined ≥ delta-only **at matched cost**, paired LB95 > 0 on a frozen test read across suites where the lanes have full rows. Otherwise the result is recorded NEGATIVE in the issue and the flag stays opt-in forever. Both directions falsifiable; a second negative probe (after Issue 107) is a legitimate close.
- [ ] **T4** — Disclosures: probe rows ≤ cal slice only (never test); hosted lanes behind their spend ceilings; box-state + counts printed beside any number; stationary-surface note (this is a LEVEL probe; no mutated state, no dynamics claims).
- **T5** — Consumers beyond the harness (report-only recording, no wiring in this issue): rethink ESC worthiness (same delta-shape gate today) and a possible rulewright consumer (does multi-domain rule fan-out beat the single best domain on this repo?) — each adopts only if T3 passes.

### T3 pre-registration — committed BEFORE any run (2026-11-10, m3-i111)

**Postures** (per checkpoint, over the suites of one frozen run):
- `delta_only` (the shipped gate): arm iff the worthiness verdict arms — delta ≥ margin (0.16 default) ∨ the LCB leg when on; unprobed/thin-support suites stay armed (the gate's own named behavior).
- `worth_only(t)`: arm iff the T2 committee verdict reads worth at threshold t — `improvement ≥ t` AND Wilson-95-supported (`committee_wilson_lcb > floor_acc`). t swept {0.00, 0.01, …, 0.16}.
- `combined(t)` (diagnostic only): arm iff delta-arms ∨ worth(t)-arms — dominates cost by construction, never the matched-cost comparison.

**Cost axis:** total escalated questions on the FROZEN TEST read — Σ (covered abstains over armed suites), from the hypothetical-armed counters (disarmed suites priced too, so a posture's cost is its rule's, not the run's arming accident).

**Match:** nearest `worth_only(t*)` to `delta_only`'s cost; the match is valid iff the relative cost gap ≤ 5% (actual gap disclosed). No valid match ⇒ `UNDECIDABLE_COST_MATCH` (nearest pair reported, never folded into a verdict).

**Promote bar (the GOAT question):** at the matched point, per-question paired (worth − delta) over the WHOLE test read, McNemar-composed from the flipped suites' escalated rows: `paired_lb95 > 0` AND `worth_accuracy ≥ delta_accuracy` ⇒ `WORTH_BEATS_DELTA` — evidence for the T5 consumers only; any real promotion still needs its own GOAT gate + delegated-owner verdict, and the flags stay opt-in either way (this issue's own promote-path text). `a + b = 0` (no disagreement rows on flipped suites) ⇒ `UNDECIDABLE_ZERO_STRADDLE`. Else `NEGATIVE` — recorded in this issue, the flags stay opt-in forever, T5 consumers do not adopt.

**T2 verdict pins (committed before the run):** `undecidable` iff probe rows < 16 (`PROBE_MIN_QUESTIONS`) or (improvement > 0 ∧ `committee_wilson_lcb ≤ floor_acc`) — the esc_voi zero-straddle discipline, never folded; `worth_committee` iff improvement > 0 ∧ `committee_wilson_lcb > floor_acc`; `not_worth` iff improvement ≤ 0. n=2 committee pick = the deployed cascade pick (gate hold → the modelless pick; gate abstain → the escalator's answer where served, else the modelless forced pick) — the calibrated gate's hold/abstain IS the tie-break; first-index determinism: the gate is a pinned threshold comparison, no unordered tie exists. The union bound P(≥1 lane right) is a CEILING only. n≥3: majority pick, tie → first-index (min pick index — the determinism law); ρ̂ and n′ = n/(1+(n−1)ρ̂) reported for sizes n+1..n+2 as HEURISTICS, never the verdict. (The harness has exactly 2 lanes today — the n≥3 branch is exercised by unit tests only.)

**Disclosures (T4):** T1/T2 read the CAL slice only; the A/B reads the frozen test records ONCE (one read; both postures recomputed in-process from the same per-question records — the arming rule is the only difference); box state rides results.json meta (accuracy reads are load-insensitive; NO latency claim is made from this run); stationary LEVEL probe — no dynamics claims; suites = the default run set (named-only suites excluded).

## References

- dcog sources (public): [our imperative duty to exist](https://distributed.inc/research/our-imperative-duty-to-exist/), [solving 1v1 catan](https://distributed.inc/research/solving-1v1-catan/)
- reflex `src/harness/cascade.rs` (WorthinessInput/WorthinessVerdict), `src/bin/harness.rs` (`--cascade-worthiness*` flags); Bench 063/066 (margin + rate levers)
- reflex `src/mc_ensemble.rs` + katgpt-core `perturbation_ensemble.rs` (the UQ provenance family; this issue adds the missing cross-lane axis at suite level)
- arXiv:2506.07962 (Correlated Errors — the agreement-on-error convention), Wood JMLR 2023 (diversity theory), arXiv:2603.24676 via katgpt-rs Research 542 (crowd scaling laws)
- NEGATIVE precedent: reflex Issue 107 / Bench 173 / `.research/014_Library_of_Layers_Routing_State_Probe_Pin.md`
