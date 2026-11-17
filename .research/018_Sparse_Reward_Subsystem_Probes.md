# Research 018 (riir-reflex): Sparse Reward Subsystem — TD-Supervised Probes, and What They Buy the Decision Family

> **Source:** [Sparse Reward Subsystem in Large Language Models](https://arxiv.org/abs/2602.00986) — Xu, Yuksekgonul, Zou (Stanford zou-group), NeurIPS 2026. arXiv:2602.00986 (v1 2026-02-01, v2 2026-05-11). Official code: [zou-group/reward_subsystem](https://github.com/zou-group/reward_subsystem) (MIT, release `e46df04c2d25927b26b614faa158020e4dfe4698`; reproduces the two pruning-curve figures only).
> **Date:** 2026-11-17
> **Status:** Active — verdict **Gain** (files: reflex Issue 122 — the ESC δ-target worthiness cell; katgpt-rs Issue 1228 — the reward-census instruments; Plan 356 recipe rows ROUTED, not edited — the plan is amo-claimed, issue 695 arc live)
> **Related Research:** katgpt-rs [509](../../katgpt-rs/.research/509_Rewarding_Progress_PAV_Prover_Advantage.md) (PAV process verifiers — the δ-object's canonical in-stack distill, CLOSED, `prover_selection` default-on) · riir-ai [380](../../riir-ai/.research/380_tmnf_c_mb_dopamine_value_circuit.md) (TMNF-C dopamine circuit — the runtime-learning cousin; its primitive landed as katgpt-core `mb_value.rs`, Issue 767) · katgpt-rs [267](../../katgpt-rs/.research/267_Future_Probe_Controlled_Generation_Detection_vs_Prediction_Features.md) (detection vs prediction features — the paper's s0 read is prediction-side)
> **Related Issues:** reflex [117](https://github.com/riir-ai/riir-reflex) (esc_capture lane, Bench 195) · reflex **122** (filed here) · katgpt-rs **1228** (filed here) · riir-train Plan 356 + issue 695 (label-degeneracy ladder, amo)
> **Classification:** Public — reflex harness comparison-lane content only; no serving-arm posture (trained-probe PRODUCT serving would be the private family's own act, out of scope here).

## TL;DR

RL-trained LLMs carry a **sparse reward subsystem**: <1% of hidden neurons ("value neurons") predict the success probability of the current state — readable at the QUESTION position, before generation — and a second sparse set ("dopamine neurons") encodes step-level TD errors δt = γ·V̂(s_t) − V̂(s_{t−1}) estimated by K=16 Monte-Carlo rollouts at paragraph boundaries. Both are read out by tiny trained probes; the TD supervision (vs final-reward-only) localizes more reasoning-critical neurons; value neurons transfer across datasets/models and zeroing them craters accuracy (intervention specificity). Applications: pre-generation confidence (beats full-state linear probes) and a neuron-level process reward model for K=4 beam search.

**Distilled for the decision family (what we take, modelless-first):**
1. **The δ-object at our escalation boundary** — supervise the ESC worthiness gate on outcome-delta targets (δ = think−cheap) from the already-captured `esc_capture` triples, instead of cheap-lane correctness/uncertainty. Same fit machinery, different target shape: predicted escalation VALUE, not cheap-lane doubt. → reflex Issue 122.
2. **The census instruments, made closed-form** — TD-delta estimation + MC significance gate, hypergeometric overlap null, L1 pruning-curve knee census, difference-of-means direction readout: all gradient-free compositions of textbook statistics. → katgpt-rs Issue 1228.
3. **Recipe rows for the trained-scorer arm** (riir-train Plan 356's PAV edit-scorer is the SAME object class the paper builds): TD targets, surprise filtering |δ|>0.3, sparse probe inputs. Routed to the plan owner — the plan is amo-claimed (issue 695 arc live 2026-11-16), not edited here.

## 1. Paper core (mechanism exactly)

- **Value probe**: 2-layer ReLU MLP (hidden→1024→1) on ONE layer's hidden states; TD(0) loss, δt = γV(s_t) − V(s_{t−1}), terminal δ = r(s_T) − V(s_T). Read at s0 (question position) → AUC vs final correctness. Sparsity via L1 pruning of probe input dims (L1 norm of first-layer incoming weights): AUC flat past p≈0.99 → <1% of neurons are "value neurons". Layers 2–4 (early) carry the signal; at the final response position AUC > 0.8 (Appendix I).
- **Interventions**: zeroing top-1% value neurons craters MATH500 accuracy (75.2% baseline) vs Random / Magnitude / Wanda / NTP-probe baselines; NTP-probe overlap 0.7% (random 0.5%) — reward-specific, not salience. Zeroing value neurons in early layers breaks dopamine neurons' TD-predictability downstream (functional coherence, Appendix H).
- **Transferability**: IoU of value-neuron sets across datasets (GSM8K/MATH500/ARC) > hypergeometric random baseline, rising as pruning → 1; transfers across RLVR models sharing a base (SimpleRL-Zoo vs PPO-Zero).
- **Dopamine probe**: at paragraph boundaries, K=16 rollouts estimate V̂(s_t); δt = γV̂(s_t) − V̂(s_{t−1}); features = all-layer hidden states, z-scored per response, averaged per paragraph (Eq. 12–14); MLP (hidden→32→1) regresses δ on paragraphs with |δ| > 0.3 only; Spearman vs pruning ratio flat → sparse "dopamine neurons". Activations peak on unexpected progress, trough at the exact error token (Fig. 3, 9).
- **Applications**: value neurons at s0 = confidence predictor (beats LCD full-state linear probe, verbal confidence, first-token logprob, question length — Table 3); dopamine probe as PRM: K=4 candidates per boundary, argmax probe score — beats greedy / random / implicit PRM (logπ_RL − logπ_base) on MATH500 (Table 4).
- **Motivation**: soft-optimal coupling log π*(a|s) = (Q*−V*)/τ; Bayes-optimal binary prediction reduces to a scalar LLR T(h) = log p(h|R=1)/p(h|R=0); superposition hypothesis predicts sparse dedicated neurons for frequently-useful features.

## 2. Published prior art (searched 2026-11-17, 12 queries)

| Claim | Verdict | Nearest |
|---|---|---|
| Sparse reward/value/TD-error neuron subsets in LLMs | **NOT FOUND** — the paper's clean novelty | H-Neurons arXiv:2512.01797 (sparse hallucination neurons, <0.1%, causal ablation — same METHOD pattern, different target); EMNLP 2025 ambiguity neurons (sparse, question-level) |
| Confidence before generation from hidden states | FOUND (full-state): LCD arXiv:2509.10625, lookahead probing arXiv:2504.05419, Kadavath arXiv:2207.05221 | sparse-subset attribution NOT found |
| Probe on internal activations as PRM | PARTIAL: SWIFT arXiv:2505.12225 (small learned head on hidden states → best-of-N reward); PRIME arXiv:2502.01456 (logit-based implicit PRM) | sparse/TD/neuron-level PRM NOT found |
| TD-error / RPE probing in transformer hidden states | **NOT FOUND** | — |

Position: the paper transfers the H-Neurons-style sparse-probing method from hallucination to reward/value/TD in RLVR models — the transfer and the TD-supervision half are new; every ingredient class exists separately. **Our verdict does not rest on the paper's literature novelty** — we consume its objective and measurement machinery onto shipped surfaces.

## 3. Internal cousin census (signal-diffs, §3.6 discipline)

| Cousin | What signal it consumes | The diff |
|---|---|---|
| `prover_selection` (katgpt-core, Research 509, default-on) | outcome-grounded D/Al complementarity stats at the COMPONENT level (rank provers/drafters) | component-level ranking vs state-level δ estimation; no TD-delta estimator, no overlap null, no pruning-census ships |
| `mb_value` (katgpt-core, Issue 767 LANDED) | RPE as a learning MODULATOR — bounded three-factor plasticity, LEARNS values online | mb_value learns from δ; this paper's machinery ESTIMATES/LOCALIZES δ from a frozen substrate and asks which coordinates carry it. Complementary: a `td_delta` estimator is mb_value's signal-source sibling, not a competitor |
| reflex `gate_fit_calibrated` + cascade worthiness (Issue 038/056, Bench 066) | cheap-lane correctness/uncertainty, fit on cal splits | target shape: correctness-supervised vs escalation-value-supervised. Same fit machinery — the cell swaps the TARGET |
| rethink ESC VoI readout (private, Issue 047, closed 2026-10-31) | value-per-escalation measured AFTER both legs run (receipts-only observer) | measurement vs prediction: the VoI observes δ; the cell predicts δ pre-escalation from cheap-leg features. (Serving-arm posture stays private-side; not this note's scope.) |
| katgpt-rs 267 future probe | detection vs prediction feature classes | the paper's s0 read is the prediction class; 267's vocabulary slots it cleanly, no conflict |
| Research 509 Fusion C first-pit kernel | temporal changepoint on logged Q̂ | first-pit LOCALIZES failure post-hoc; the δ-target cell predicts failure value pre-escalation. Adjacent, compose |

## 4. Path 0 inventory (every row terminated)

| # | Component | Coverage | Extraction | Terminal state |
|---|---|---|---|---|
| 1 | TD-delta estimator + MC significance gate (\|δ\| > z_α·σ̂/√K) + K-budget formula | none ships | closed-form given rollouts | **katgpt-rs Issue 1228 I1** (API carries the general Bellman form δt = r_t + γ·V̂(t) − V̂(t−1); the paper's sparse-terminal-reward special case r_t = 0 is the default instance — a per-step-reward consumer must not get a silently wrong δ) |
| 2 | Hypergeometric set-overlap null (the paper's IoU baseline, Eq. App C) | **partial — the shared Lanczos log-gamma kernel ships** (`crate::special_fn::ln_gamma`, Plan 597 T1.1; `best_belief.rs` ln B(a,b) and `bmr.rs` both consume it) | closed form ON the existing kernel — no new log-space combinatorics | **Issue 1228 I2 — reuse `special_fn::ln_gamma`** |
| 3 | L1 pruning-curve knee census + typed NO-KNEE | **partial — elbow/knee logic exists** in `log_frontier.rs` and `subspace_phase_gate.rs` (found by the post-review grep; the first sweep missed the "elbow" vocabulary) | closed form; reuse/extend the existing elbow detectors, never a parallel knee implementation | **Issue 1228 I3 — check both sites first, then extend** |
| 4 | Difference-of-means direction readout (closed-form probe challenger) | **YES (partial) — `construct_probe_via_mean_difference` SHIPS** in `katgpt-pruners/src/fpcg_modelless.rs:101` (planted-direction recovery + empty-input refusal, tested) | the delta is small: sigmoid scoring readout + the typed no-signal refusal on top of the shipped constructor | **Issue 1228 I4 — consume the shipped function, extend beside it** |
| 5 | ESC worthiness supervised on δ = think−cheap | gate-fit surfaces ship; δ-target supervision absent | fit on captured triples (data exists: Bench 195 capture lane) | **reflex Issue 122** |
| 6 | Plan 356 recipe rows (TD targets, surprise filter, sparse inputs) | Plan 356 IS the same object (PAV edit-scorer); recipes not incorporated | needs GD (trained scorer) → Path 0.5 → plan exists | **Amendment routed to the plan owner** (amo, issue 695 arc live 2026-11-16); not edited under the live claim — reassess at A5/GOAT. Honest note: the surprise filter is NOT a fix for 695's label-degeneracy (a saturated corpus has δ ∈ {−1,0,+1}; the missing mid-band is exactly what the filter discards; 695's fix is the temperature arm) |
| 7 | s0-confidence readout on encoder hidden states (Table 3 analog) | reflex abstention is feature-based (no hidden states); laya-lane per-layer state exposure UNVERIFIED | I4 makes a probe-free arm possible | **Recorded candidate, not filed** — blocked on a hidden-state capture seam in the encoder lane; any future calibrated-probability claim owes the conformal-naive floor (Report-the-Floor) |
| 8 | Per-step δ-credit over fix trajectories (rulewright) | Elo/Beta = per-rule aggregate outcome counters (context-free); campaigns mostly single-edit | closed-form given campaign logs | **Audited discard** — the documented corpus gap (katgpt-rs-clippy Issue 039: 98.5% candidates unseen; selection work deferred on densification) is exactly the missing substrate; multi-edit campaigns are rare; revisit on store densification |
| 9 | MCTS/cgsp value-probe amortization (game runtime) | dd_tree BestQ ships; mb_value owns runtime RPE learning | rollouts free in search, probe needs GD | **Audited discard** — the paper's mechanism presupposes transformer hidden states; NPC cognition has none (sigmoid drives + belief kernels are the substrate); the transferable half (RPE-driven value learning) is already owned by `mb_value` |
| 10 | PRM beam-search application (K=4 argmax) | no K-candidate consumer lane ships (serving lanes are single-pick; specdec marginal per its lane record) | selection = closed-form argmax | **Audited discard** — no consumer; revisit only if a best-of-K lane appears |

## 5. Fusion

**Fusion A — the δ-object at the escalation boundary (reflex Issue 122).** The paper's dopamine probe predicts "will the next step improve expected reward" from internal state BEFORE the step. Our ESC wrapper escalates cheap→think on cheap-lane doubt; the escalation boundary has the same shape and the corpus already exists AND IS MEASURED: the Bench 195 capture holds **200 triples (all cal-split), `think_pick` present on all 200 (117 gate-fired; the capture records both legs even where the gate did not fire — 83 non-fired rows), with δ = think−cheap splitting +1: 52 / 0: 122 / −1: 26** (coordinator-measured from `triples.jsonl` this session — the corpus is NOT label-degenerate: 39% of rows carry a non-zero δ). Supervise the worthiness fit on δ and measure paired escalation precision/value. What none of the cousins alone gives: the gate currently predicts "cheap lane unsure" — which is not "escalation will help" (a hard question stays hard for the think leg); δ-supervision predicts the difference. Weakest point, named: whether the CHEAP-LANE FEATURES carry enough signal to predict δ is exactly what the cell must measure — the fit consumes the same features as the incumbent, so the null hypothesis (features carry correctness signal, not escalation-value signal) is live; that IS the cell's falsification arm.

**Fusion B — reward census instruments (katgpt-rs Issue 1228).** The paper's measurement layer is closed-form algebra over (state, outcome) captures — and half of it ALREADY SHIPS: the Lanczos log-gamma kernel (`special_fn::ln_gamma`), elbow detectors (`log_frontier`, `subspace_phase_gate`), and the mean-difference probe constructor (`katgpt-pruners::fpcg_modelless::construct_probe_via_mean_difference`). The kit COMPOSES these with the genuinely-missing pieces (TD-delta estimator, hypergeometric null, typed no-signal refusals) — reuse over re-derivation, one implementation per concept.

**Fusion C — Plan 356 recipe rows.** The paper builds (for math reasoning) what Plan 356 builds (for code fixes): an amortized per-state success/advantage scorer. Three recipe deltas transfer when A5 opens: TD-shaped targets over the fix trajectory, |δ| surprise filtering on the regression set, sparse input selection via the I3 census. Routed, not edited.

## 6. Novelty gate (honest, for OUR claims)

| Q | Claim A (ESC δ-target cell) | Claim B (census instruments) |
|---|---|---|
| Q1 no prior art | Internal: grep-verified absent. External: our fusion construct; not separately searched as a headline (the paper's objective is the anchor) | The MATH is textbook (no novelty claim); the census-kit composition is utility, not novelty |
| Q2 new behavior class | No — target-shape change on existing fit machinery | No — measurement kit |
| Q3 selling point | Conditional on the cell's bars passing | No (infrastructure) |
| Q4 force multiplier | YES — esc_capture + esc_voi + PAV δ-object + gate-fit machinery | YES — mb_value, ESC cell, future capture lanes |

Not all-4-YES on either → **Gain**, issue-tier filings. No "candidate" wording anywhere.

## 7. Verdict + routing

**Gain.** The paper ships (code, MIT, NeurIPS 2026) and carries actionable items onto three surfaces; none rises to GOAT yet (the cell is unproven, the instruments are consumers-of-future-captures), none is PASS (documented surfaces move).

| Artifact | Destination | Status |
|---|---|---|
| This note | `riir-reflex/.research/018_Sparse_Reward_Subsystem_Probes.md` | ✅ |
| ESC δ-target worthiness cell | `riir-reflex/.issues/122_esc_worthiness_delta_target_cell.md` | ✅ |
| Reward census instruments (I1–I4) | `katgpt-rs/.issues/1228_reward_census_instruments.md` | ✅ |
| Plan 356 recipe rows | routed to plan owner (amo) via this note; plan NOT edited under the live claim | ✅ |

MOAT: reflex = decision-serving comparison arena ✓ (the cell strengthens the escalation comparison half); katgpt-rs = base primitives ✓ (generic math, no game/chain semantics); riir-train = recorded recipes, routing only.

## 8. What does NOT transfer (say it once)

1. **TD on classifiers** — single-step trajectories collapse δt to r − V(s0): plain residual regression. All TD-shaped items are scoped to multi-step lanes (fix campaigns, escalation boundaries with two legs, generation).
2. **Paragraph boundaries** — classification has none; boundaries must be redefined per consumer (escalation boundary, fix step, draft step).
3. **RLVR effect magnitudes** — the paper's models are RLVR-trained reasoners; our served encoders are classifiers. Expect replication in kind, not size.
4. **The scientific claim itself** — we make no claim about OUR models carrying value/dopamine neurons; that would require the census (Issue 1228) + a capture lane and is out of this verdict's scope.

## 9. Pre-flight record (honest)

Five-step pre-flight: (1) product-set scope from the in-context AGENTS.md set (the R380 precedent — the in-context set was the scope authority); riir-train pipeline doc read (outline; LoRA/GRPO/DPO + draft-model pipelines confirmed, no probe/PRM lane). (2) `.research/` folders derived by ls (22 on disk incl. worktrees). (3) src-tree vocab via targeted greps: `esc_capture.rs` / `esc_voi` / `gate_fit` / `prover_selection.rs` / `mb_value.rs` all verified live this session; `mb_value` discovery (Issue 767 landed) came out of the coverage grep. (4) Prior-art: 12 web searches via subagent (table §2); internal-first grep across 13 `.research/` folders (dopamine/TD/probe vocab) BEFORE web — hits read, cousins tabled §3. (5) Training-code grep: `mb_value.rs`, `height_gate.rs` (RPE mentions), GRPO/LoRA lanes per AGENTS — the three-track check ran; no repo characterized modelless-only. **Second-pass greps (surfaced by the verdict review, folded into rows 2–4):** my first sweep searched "knee" but not "elbow" — `log_frontier.rs`/`subspace_phase_gate.rs` carry elbow logic; `construct_probe_via_mean_difference` and the shared `special_fn::ln_gamma` kernel were found the same way. The rows were corrected to reuse-not-rebuild; the lesson (grep BOTH spellings of a shape word) is recorded here. Live-state: Plan 356 claim verified live (issue 695 arc, 2026-11-16 prep landed by amo — the reason no edit was made); reflex Issue 121 / katgpt-rs 1227 highwaters read at allocation; dual_allocation_gate clean both repos at allocation time. Bench 195 corpus measured THIS session (200 triples, δ split 52/122/26 — numbers in §5 are coordinator-measured, not reviewer-transcribed). Panel: 2 advocates + 1 prior-art searcher in one spawn round; advocate code facts spot-checked by coordinator grep (all four named surfaces verified). NO clone of the source repo (verdict built from paper HTML v2 + repo README; no byte-quote claims from source files made). Advocates' outputs are arguments, not evidence — every load-bearing claim above carries a coordinator-verified fact or is marked advocate-reported.

## Cross-references

- katgpt-rs `.research/509` + `.issues/692` (PAV/`prover_selection` — the δ-object's modelless home, GOAT PASS Bench 684)
- riir-ai `.research/380` + katgpt-core `mb_value.rs` (dopamine plasticity — the learning-side sibling)
- reflex Bench 195 / Issue 117 (`esc_capture` — the corpus this cell consumes)
- riir-rethink Issue 047 (ESC VoI — the receipts-side observer; private)
- riir-train Plan 356 + issue 695 (the trained-scorer arm; amo-claimed)
- katgpt-rs `.research/267` (detection vs prediction features)
