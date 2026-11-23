# Issue 133: ES-derived runtime variant selection + confidence-as-reward loop (fusion ideas, novelty TBD)

**Status:** Closed — negative (T1 sweep 2026-11-24: the covering class is published; T2 finds no legal surface; re-open condition recorded in the sweep note)
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

- [x] **T1 (BLOCKING)** Prior-art sweep: DONE 2026-11-24 — verdict NEGATIVE (note below); the idea dies here under this task's own standard.
- [x] **T2** Mutated-state analysis: DONE 2026-11-24 (table below) — no surface satisfies the three conditions the loop needs.
- [-] **T3** NOT TRIGGERED — precondition (a live idea after T1+T2) unmet.
- [-] **T4** NOT TRIGGERED — lapses with T3.

---

## T1 sweep note (2026-11-24, zed-133t1@amo) — VERDICT: the ideas DIE on this evidence

### Pinned external claims (abstract-verified this session)

| Prior art | Pin | One-sentence claim (verified against the abstract) |
|---|---|---|
| BBTv2 | arXiv:2205.11200 (EMNLP 2022) | Gradient-free divide-and-conquer optimization of layer prompts for few-shot tuning — tuning-time; deploys ONE optimized prompt. |
| EvoPrompt | arXiv:2309.08532 (ICLR 2024) | Population of prompts iterated by EA operators scored against a DEVELOPMENT SET — offline loop; deploys a single point; no serving-time selection. |
| PBT | arXiv:1711.09846 | Population of models + hyperparameters jointly optimized at TRAINING time by an external fitness metric, discovering hyperparameter schedules — training-time; fitness is labeled/external. |
| Adaptive Model Selection | arXiv:1905.08874 | Thompson-sampling meta-decision routes production requests (airline pricing) to a POOL of models by online performance — **the covering class**: population-of-config-variants scored at runtime, posterior-selected, in production. |
| TS rising bandits | arXiv:2505.12092 (2025) | Regret analysis of Thompson-sampling online MODEL SELECTION — the class is active literature through 2025. |
| Semantic entropy / self-consistency | arXiv:2302.09664 (Nature 2024) / arXiv:2203.11171 | Agreement-across-samples → entropy/cluster → flag-or-abstain — the agreement SIGNAL family is published (already adjudicated COVERED in reflex Research 017 §2). |

### Internal substrate census (the engine already ships most of the shape)

- `mc_ensemble` (reflex Plan 008 / Bench 092, opt-in): **a serve-time variant population already exists on this engine** — N seeded perturbation passes → answer histogram → LCB ranking + U-stats rejection. Its pre-registered null FIRED as an abstention channel (≈0 marginal lift over the fused gate at matched coverage); default-off behind `RIIR_REFLEX_NO_MC_ENSEMBLE`.
- `beta_lcb_order`/`_into` — katgpt-core `hint_regret/memory.rs`: the selection statistic idea A names is existing substrate, consumed by `hint_regret::triage`, which already fuses semantic-entropy + corpus-distance signals (Research 609).
- `pick_latch` (katgpt-core issue 987): the held-index latch idea B's loop would need — substrate exists, reflex wiring unwired.
- `adoption_gate` (katgpt-rs Plan 633; consumers riir-refine + instinct swap gate) + the rethink ESC rate-guard latch (N=200 rolling demote): demote/readmit + mutated-state machinery both exist.

### Verdict

**Idea A** pinned claim as filed — "distribution-of-variant selection for serving config slots, consuming per-window Beta-LCB posteriors over seeded variants, distinguished from EvoPrompt/BBTv2 by serving-time persistence and from Thompson model selection by LCB + abstain-to-incumbent" — FAILS the novelty bar: the covering class (Thompson-sampling online model selection) already covers population-of-config-variants scored at runtime with posterior selection; LCB-vs-Thompson and the abstain-to-incumbent tie rule are conservatism choices INSIDE that class; and the nearest internal analogue (`mc_ensemble`) already measured ≈null for the neighbouring use on this engine. Residue is a COMPOSITION (persistent config slots vs per-request perturbations; ES-at-Scale's reward-hack framing), not a new mechanism class — the exact Research-017 standard ("a composition, not a new mechanism class"). GOAT-gated composition if ever wanted; honest expectation null; NOT filed.

**Idea B** pinned claim — "confidence-as-reward self-tuning for the same slots, consuming internal agreement entropy/margins across k variants, distinguished from semantic-entropy detection by the tuning-loop use" — DIES on three legs: (1) the signal family is published as a detection channel (semantic entropy; R017 adjudication); (2) every candidate tuning surface is label-measurable today — harness suites carry gold labels, refine's rule selection reads deterministic compile-gate outcomes, instinct's swap gate reads suite accuracy — and the standing finding (R429 Self-Rewarding) is that deterministic labels dominate noisy self-judgment wherever they exist, so confidence-as-reward is DOMINATED exactly where tuning is measurable; (3) the label-free surface its own falsifier needs does not exist (T2 below), and the nearest internal prior for agreement signals is the mc_ensemble null. **Re-open condition (recorded): a serving surface with NO gold labels AND a legal mutated latch AND evidence the cross-variant agreement signal correlates with outcomes there.**

## T2 — Mutated-state analysis (same session; kills the remaining wiring path)

| Candidate surface (as filed) | Mutated state today | Legal dynamics? | Outcome labels? | Verdict |
|---|---|---|---|---|
| reflex routing/gate/corpus weights | NONE — `DecisionEngine` is deterministic and stateless per request (`solve_sample_into`; no config field mutates on evidence) | No — any loop must FIRST add a latch (pick_latch substrate unwired) | YES — harness suites carry gold | Dead: dominated by label-as-reward; nearest prior null |
| rethink ESC escalation arm | YES — the rate-guard latch (N=200 rolling) is the family's one legal mutated state | Yes | Drift/rate signal already consumed | Not a delta: the guard already adapts from its own signal; no evidence agreement-entropy beats it |
| riir-refine rule selection | YES — traj_store Elo/BetaPosterior ratings mutate on fix outcomes | Yes | YES — compile gates are deterministic oracles | Dead: R429's finding applies at full strength |
| instinct arena seating | Epoch-gated LaneSlot swaps | Yes (adoption_gate-gated) | YES — suite accuracy | Dead: dominated |

**T2 conclusion: no surface satisfies (no gold labels) ∧ (legal mutated state) ∧ (plausible agreement signal) — the three conditions idea B requires. T3 is NOT triggered** (its precondition — a live idea after T1+T2 — unmet), and T4 lapses with it. The issue closes negative under its own T1 standard ("or the idea dies here").

**Lease closed:** zed-133t1@amo, 1795454364, expires 2026-11-26T17:19Z
