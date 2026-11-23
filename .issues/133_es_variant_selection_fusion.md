# Issue 133: ES-derived runtime variant selection + confidence-as-reward loop (fusion ideas, novelty TBD)

**Status:** Open — fusion idea filed from research; novelty NOT claimed; promotion blocked on the owed sweep
**Date:** 2026-11-23
**Source:** [riir-train/.research/501_ES_At_Scale_Distill.md](../../riir-train/.research/501_ES_At_Scale_Distill.md) distilling [arXiv:2509.24372](https://arxiv.org/abs/2509.24372) (ES at Scale, ICML 2026). Downgrade target of the research session's C3+C5 candidates — filed here as ideas because the prior-art risk was not swept to the §4 standard.
**Secondary to:** riir-train Plan 512 (the training track is the primary filing this paper earned).

---

## The two ideas

**A. Distribution-not-point variant selection (C3).** The paper's mechanistic claim: ES optimizes a solution *distribution*, which is harder to reward-hack than a point-optimized single solution. Runtime reframe: serve from k seeded variants of the incumbent lane (`VariantSlot { seed, delta, posterior: Beta }`), select per window by `beta_lcb_order`, abstain-to-incumbent on ties; the deployment IS the distribution. Candidate surfaces: routing/gate/corpus-weight params (NOT base weights), rule selection over perturbed corpus snapshots (riir-refine side), arena seating (riir-instinct side). The paper's no-KL-needed result maps onto our fused abstention (a typed answer the loop cannot overfit past).

**B. Confidence-as-reward runtime self-tuning loop (C5).** The authors' own future-work note: ES needs only one scalar per candidate → unsupervised adaptation from internal confidence (they name semantic entropy/density). Modelless realization: k variant slots scored by runtime-internal fitness — pick-histogram entropy / margin-weighted agreement across the k variants on a probe window (closed-form, free from the ensemble itself), layered with `SigmoidGateCalibrator` margin + `CorpusDistanceGate` density; `step_noise` gates WHEN tuning is legal (stable windows only); z-score per window; `beta_lcb_order` pick; paired non-inferiority before any install; `adoption_gate` demote/readmit after; pre-registered confidence-outcome correlation floor as the falsifier (demote-on-fail, never assumed).

## Why an issue, not a plan

1. **Prior-art sweep OWED (blocking task below).** The evolutionary-config-optimization class is named in the paper's own related work (BBTv2, EvoPrompt, GA prompt search) and adjacent lanes (GA-over-configs, population-based prompt optimization). If that class covers "population-of-config-variants scored at runtime, LCB-selected," the novelty claim dies or narrows.
2. **Stationarity constraint (the ESC lesson).** The ESC escalation gate is EXACT/stationary — a value-slope/dynamics signal is unproducible on a pure function of inputs. Idea A/B's selection loop only has dynamics where mutated state exists (the ESC rate-guard latch qualifies; a static serve path does not). Any wiring must pass the mutated-state test (§ step 4a discipline) before a promotion claim.
3. **Secondary** — Plan 512 (training track) is this paper's primary filing; ideas A/B must not cannibalize its cells.

## Tasks

- [ ] **T1 (BLOCKING)** Prior-art sweep: BBTv2, EvoPrompt, GA-prompt-search, GA-over-configs, population-based runtime prompt/model selection (2024–2026), plus internal grep across `.research/` for ensemble-variant selection. Output: a sweep note appended here with the pinned one-sentence claim ("distribution-of-variant selection for <surface>, consuming <signal>, distinguished from <closest> by <delta>") — or the idea dies here.
- [ ] **T2** Mutated-state analysis: name the state variable each candidate surface mutates in response to evidence (rate-guard latch exists on rethink ESC; reflex serve paths need a named latch/window before any dynamics signal arms).
- [ ] **T3** Only after T1+T2: pick ONE surface (candidate: reflex routing/gate weights — small, cheap, high-value) and file a plan with pre-registered bars (paired LB95 vs incumbent on a frozen read + the shift-shadow-read robustness axis from the Bench-089 discipline + escalation-rate band intact).
- [ ] **T4** Proof-path requirement: no promotion claim without a runnable PoC (fixture + falsification arm + pre-registered gate) per the ungoat discipline.
