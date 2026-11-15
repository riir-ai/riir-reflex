# Research 017: Against All Likelihood — Invariance as a Serving Signal (Position-Paper Distill)

> **Source:** [arXiv:2611.09385](https://arxiv.org/abs/2611.09385) "Against All Likelihood" — David Lopez-Paz, 2026-11-06, CC-BY 4.0
> **Date:** 2026-11-15
> **Status:** DISTILLED — one POC issue filed (reflex [Issue 119](../.issues/119_context_invariance_probe.md), report-only probe with a pre-registered null path); any serving wiring gated on its verdict. **Verdict record (§5 ping-pong):** round 1 REVISE (claude reviewer, 2026-11-15, session 8a40f252) — the placebo arm, the fused-third-channel arm, the primary-cell pin, the G3 rescope, and the honest-family/external-prior rewordings below are all applied; round 2 AGREE (same session) with two riders — the placebo-comparison pinned as a paired-LB95 statistic in G2a/G2b, and this TL;DR's stale attribution line fixed; both folded at commit.
> **Related Research:** katgpt-rs [.research/499](../../katgpt-rs/.research/499_Jagged_Judges_Epistemic_Stability.md) (Jagged Judges — the margin/invariance head-to-head)
> **Related Benchmarks (internal, load-bearing):** reflex Bench 092 / Plan 008 (`mc_ensemble` null verdict), Bench 128 (perm spread), Bench 147 (null probe), Bench 174 (gate stability); rethink Bench 089 (ESC paraphrase shadow read)
> **Classification:** Public

---

## TL;DR

Lopez-Paz argues likelihood is neither necessary nor sufficient for truth; truth should be judged by **invariance across admissible transformations** of the context (Nozick / Wigner / structural realism), and sketches a test-time probe: generate perspective environments, answer under each, **extract the invariant core**, and **abstain when the proposals share no core**. For our stack the probe CLASS is published prior art (self-consistency family, semantic entropy) and the prior evidence is negative on both legs — but unequally: Plan 008's `mc_ensemble` (bucket-mask perturb-and-aggregate rejection on THIS engine) hit its pre-registered null (≈0 marginal lift over the fused gate at matched coverage) — an INTERNAL measurement; the Jagged Judges figures (positional invariance 0.37 weakest, vs repeat 0.42, jury margin 0.59) are that paper's OWN measurements on LLM judges, distilled in Research 499 — an EXTERNAL prior of a neighbouring kind. And on this engine the two families CONVERGE: `embed.rs` embeds words + bigrams as FNV-1a-hashed buckets, so a persona prefix is a constant additive bucket set + one boundary bigram — the additive mirror of the bucket mask, not a perspective shift. The one honest question left: does ANY additive input transform produce an agreement-structure abstention signal with placebo-separated lift over the fused gate? Issue 119 measures it once (placebo arm + fused-third-channel arm + pinned primary cell), with a null expected and the close scoped to the run table + this engine.

**Distilled for the serving family (modelless, inference-time):** an "invariance seat" on `decision_wire` — score the decision under K deterministic context transforms, serve the plurality-stable answer, abstain (`noul`) when no core survives — is *not* a new mechanism class for us; it is a second perturbation family on a harness we already built (Plan 008), for a signal class an external LLM-judge study (Jagged Judges, distilled in Research 499) ranked last among instability predictors. The paper's durable gifts here are corroborations, not primitives (see §4): its softmax/one-hot open problem is answered by our sigmoid-only law; its "perplexity can't tell right from wrong" citation instantiates our lossy-surface promotion rule; its continual-learning-as-invariance principle restates our intake laws.

---

## 1. Paper Core Findings

Position paper; no benchmarks, one sketch.

1. **Likelihood ≠ truth, both directions.** Low-probability events happen; lies repeated a million times become probable. Cites Bottou & Schölkopf 2025 ("fiction machines"), Kalai et al. 2026 (accuracy-incentivized hallucination), Veličković et al. 2026 (likelihood-based model selection can prefer the less accurate model OOD — see the corroboration line in §4).
2. **Truth = invariance.** Nozick: objective facts are invariant under admissible transformations; Wigner: invariance principles correlate correlations; structural realism (Poincaré → Cassirer → Worrall): what survives theory change is the structure/relations, not the objects.
3. **The admissible-transform set is not given a priori** — Nozick's bootstrap: facts and transformations co-evolve; Woodward relativizes invariance ("invariant under WHICH changes"); van Fraassen's Symmetry Requirement: isolate relevant parameters ⟺ define the symmetries.
4. **IRM recap (§5).** Invariant Risk Minimization (Arjovsky et al. 2019, the author's own) formalizes it as `Y ⊥ E | φ(X)`; empirically IRM ≈ ERM in practice (Rosenfeld/Gulrajani critiques, cited honestly). The author's pivot: **LLM outputs change the calculus** — a classifier emits a few bits; a language model emits a sequence whose reasoning trace exposes whether it relied on invariant or spurious mechanisms.
5. **The §6 test-time sketch** (the actionable part):
   - receive q → generate environment prompts `{e_i}` ("what perspectives could change the answer?") — the model proposes its own transformations ("reverse prompting");
   - answer under each: `a_i ~ P(·|e_i, q)`;
   - extract the invariant answer: `a ~ P(·|inv, {a_i})` ("what is invariant across the following answers?");
   - **abstain when the proposals share no common core** — "when the proposals share no common core, the model should abstain rather than hallucinate";
   - (optional) fine-tune on (q, a) — continual learning as invariance.
6. **Explicit deltas from self-consistency:** SC samples repeatedly under a FIXED context and majority-votes; this varies the CONTEXT itself and keeps only what survives. Intersecting too many environments risks a vacuous core (Nozick's breadth-vs-depth tradeoff). Contexts of one model are not independent sources — corpus-shared errors need different models, retrieval, or experimentation (model councils).

---

## 2. Prior Art (published — the class is covered; searched before any claim)

| Claim | Closest published prior art | Verdict |
|---|---|---|
| Sample under perturbation → aggregate → decide | Self-consistency (Wang et al., [2203.11171]); Universal SC (Chen et al., [2311.17311]) | COVERED — the paper cites both and claims only the context-variation + invariance framing |
| Consistency → hallucination detection / abstention | SelfCheckGPT (Manakul et al., [2305.13388]); **semantic entropy** (Farquhar et al., Nature 2024, [2302.09664] — sample → meaning-cluster → entropy → flag; the Nature paper's abstract framing is detection-and-refusal) | COVERED — "abstain when inconsistent" is a published serving pattern |
| Consistency vote share as selective classification | "Self-Consistency Vote Share as a Selective-Classification Signal" (MDPI Information, 2026); Calib-Abstain (2026, token uncertainty + multi-path consistency); "When LLMs Agree, Are They Right?" (2026 audit — agreement ≠ correctness) — ⚠ **snippet-level verified only** (titles/venues from search results, not full-text reads) | COVERED at class level — including the audit-level caution that agreement does not certify correctness |
| Environments proposed by the model itself | Multi-persona self-collaboration (Wang et al. 2024); model councils (Perplexity 2026); XRM (Pezeshki et al. 2024, environment discovery) | COVERED |
| Invariance as the truth criterion (the philosophy) | Nozick/Wigner/Poincaré lineage — the paper's own contribution is the ML framing | The paper's honest novelty; nothing for us to claim on top |

Bottom line: the sketch's class is published prior art. Our Q1 claim must be scoped to the *instantiation delta* on our engine, not the technique.

---

## 3. Distillation

### 3.1 Path 0 component table

| Paper component | Coverage (ships?) | Extraction (modelless?) |
|---|---|---|
| Context-varied invariance probe | **Partial — once internal, once external, both negative.** INTERNAL: Plan 008 `mc_ensemble` perturbs the hashed-feature embedding (Bernoulli bucket mask) and aggregates N deterministic passes → U-stats rejection + LCB ranking; **null path FIRED** (≈0 marginal lift over the fused gate at matched coverage; stays opt-in). `--perm-probe` varies option ORDER — as a harness INVARIANT (control reds refuse), not a serving signal. EXTERNAL: the Jagged Judges figures (positional invariance 0.37 weakest of three predictors) are LLM-judge measurements distilled in Research 499, not our engine. | YES — the additive input-transform family (persona/framing prefixes; on this engine, additive — see §3.2) + placebo arm → Issue 119 |
| Abstain when no invariant core | Partial — fused abstain ships two signals (calibration score + `CorpusDistanceGate`); no agreement-structure channel | YES — same probe measures the empty-core channel at matched coverage (the G2b arm, placebo-adjusted) |
| Invariant-core EXTRACTION (serve the surviving answer) | **No analog** — Plan 008's histograms feed rejection/LCB, never serve a mode pick | YES — plurality-stable pick under the transforms; the G2a arm (THE verdict) |
| Trace-level invariance (where reasoning diverges) | No analog in modelless lanes (no CoT); hosted comparison lanes could expose traces | Deferred — hosted-lane territory (ESC think legs), not filed |
| Optional fine-tune on (q, a) | Training-side, **no recipe** (no optimizer/loss/schedule/rank — nothing actionable for riir-train) | n/a — noted, nothing filed |
| Continual learning = invariance across user groups | Principle-level cousins: refine `adoption_gate` (outcome windows), `calibrated_dispatch` (evidence groups), grammar-not-answers echo gates + leak scan | YES as a principle; no new filing (cousins already gate admission; the delta — vary context AT admission time — named below as a future consumer, unfiled) |

Three-track note: first-pass classification is decision-serving (the abstract carries no optimizer/loss/RL framing; IRM is a 7-year-old recap with documented empirical failures, cited by the paper itself). The fine-tune row above is the only training contact and carries no recipe — the adversarial panel has nothing to extract and is skipped with this reason recorded.

### 3.2 Signal-diff on the closest shipped cousins (§3.6 discipline — one read each)

| Shipped cousin | Core mechanism | Signal it consumes | Paper component's signal | Diff |
|---|---|---|---|---|
| `mc_ensemble` (reflex Plan 008 / Bench 092, feature `mc_ensemble`) | Bernoulli bucket-mask perturbation of the hashed-feature embedding → N deterministic passes → answer histogram → U-stats rejection + LCB ranking | **embedding-level nuisance noise** (destroys random feature buckets) | **input-text context transforms** (persona/framing prefixes) | **the families CONVERGE on this engine** — `src/embed.rs` hashes lowercase words + bigrams into buckets, so a prefix transform is a constant additive bucket set + one boundary bigram: the additive mirror of the bucket mask, not a semantic channel. The probe therefore carries a PLACEBO arm (token-count-matched nonsense prefixes) — the semantic arm must beat the placebo, or the measured signal is hash noise wearing a persona. Same harness, same matched-coverage verdict discipline |
| Research 499 margin (`verdict_margin`) | `|Σ votes|/K` across K judges at baseline → flip-forecast | **cross-agent** agreement at one moment, consumed as a *forecast* | **cross-context** invariance of ONE judge, consumed as *extraction + abstention* | different axis (agent vs context) and different use (forecast vs serve); 499's figures are the Jagged Judges paper's own LLM-judge measurements — positional invariance specifically, the WEAKEST of the three (0.37) — an external prior of a neighbouring kind, not an internal engine measurement |
| `--perm-probe` (Bench 128) | K option orderings; median swing ≤ 2 pt is an engine INVARIANT; control reds refuse | option-ORDER invariance as a correctness gate | state-CONTEXT invariance as a serving signal | measurement artifact discipline vs serving capability; unlike option order, state transforms have NO invariance guarantee (any token change moves hashes) — small flip rates are expected, so the read is distributional, never red-on-flip |
| Bench 089 (rethink, ESC paraphrase shadow read) | paraphrase transforms judge whether armed ESC GAINS survive — eval-time robustness read | paraphrase as evaluation instrument | paraphrase-like transforms as per-decision serving signal | eval-side precedent; serving-side extraction + abstention is the delta |
| Bench 802 (katgpt-rs, commitment-gap rig) | self-consistency label `[z_t == z_commit]` measures "will I change my mind", not "am I right" — wrong label in contamination regimes | agreement as a label | agreement as a truth certificate | standing caution — the probe must score against gold on a frozen read, never sell agreement as correctness (the "When LLMs Agree" audit agrees) |
| Fused abstain (`noul` = cal score + `CorpusDistanceGate`) | two-signal thresholded abstention | per-decision confidence + corpus distance | empty invariant core | no agreement-structure channel exists — the shaped gap the probe measures |

**Granularity check (one level up):** Plan 008's histogram consumers are rejection/LCB only — nothing serves a mode pick; Research 499's margin feeds Plan 545's conviction-gated pressure (game side), not the decision wire; the perm-probe's spread feeds no serving decision. The extraction + third-channel gap is real and unshipped.

### 3.3 The distilled primitive — the invariance seat (spec, gated on Issue 119's verdict)

For a `decision_wire` question q with options O:

1. **K deterministic semantic transforms** of the input state from a fixed, name-stable table (persona prefixes — "From the perspective of a <discipline>:"; framing suffixes; neutral formatting variants). Modelless: string ops, no RNG (seeded Sattolo only if ordering is shuffled — the label-perm convention), no hosted calls.
2. **Re-decide through the production path** — the perm-probe's `ChoiceOracle` seam; never a private decide path.
3. **Invariant core** = the plurality-stable answer set; **empty/small core** = no majority among K → abstention-eligible.
4. **Two measured arms, both placebo-adjusted by statistic** (verdict round 1 + round-2 rider): G2a = mode-seat alone vs the fused gate at matched coverage (THE verdict — extraction-vs-replacement); G2b = fused + empty-core vs fused alone (the third-channel claim's own arm); placebo adjustment = paired LB95 of (semantic − placebo) > 0, never a point estimate. Serve only if G2a passes; fuse the channel only if G2b does.

### 3.4 Latent + consumer reframes (the two-sided standing check)

- **Decision-serving loop (reflex/instinct/rethink):** a third abstention channel with a different failure signature than cal+distance (agreement structure vs point confidence), plus a new arena axis (agreement histograms per suite beside the accuracy tables). Escalation composition (core-disagreement → ESC think leg) is instinct-side vocabulary (`EscalateSpec`) — named, not filed.
- **Rulewright loop (refine):** the paper's continual-learning principle ("decide what new data is worth learning from by invariance, not recency") restates our intake posture — `adoption_gate` admits on outcome windows, `calibrated_dispatch` on per-(domain×shape) census, echo gates refuse test-shaped rows. The un-shipped delta: none of them VARY the context at admission time (all read historical windows). A mined rule could be re-evaluated on paraphrase-mutated spans before admission. Named as a future consumer of the same probe machinery — unfiled until the reflex probe proves the transform family carries signal at all.

### 3.5 Fusion

**Paper × Plan 008 × Research 499:** the probe is Plan 008's perturb-and-aggregate harness with the perturbation family swapped (bucket-mask → input-text transforms — additive twins on this hashed-bag engine, hence the placebo arm) and the readout extended (rejection stats → core extraction + matched-coverage accuracy vs the fused gate, on two named arms). Research 499 supplies the ranked external prior (positional invariance weakest of three predictor classes on LLM judges) — the pre-registered expectation is NEGATIVE, and the probe's honest job is to bound the additive-transform class on THIS engine at µs-scale cost, not to ship a seat. If the semantic arm separates from the placebo AND the fused gate, THAT is the finding worth a promotion plan; if the placebo ties it, the probe measured hash noise and G3 closes the class for additive-feature engines, scoped.

### 3.6 Honest caveats

- **Single-corpus invariance ≠ truth.** The engine's corpus is one source; the probe certifies *decision stability under input transforms*, not factuality — the paper itself says single-model contexts cannot expose corpus-shared errors (that needs model councils / retrieval / experimentation — ESC remote-leg territory for us). The probe's abstention claim is exactly as strong as Bench 802's caution allows: agreement is evidence of stability, never of correctness; all accuracy reads are against gold on a frozen read.
- **On this engine, context transforms are additive.** The hashed-bag embedder gives a perspective prefix no semantics beyond constant bucket mass — the placebo arm exists because a "positive" without it would be uninterpretable (verdict round 1's central correction). Any future engine with order- or context-sensitive features is the named reopen trigger for the class.
- **The 0.37 prior is a neighbour's number.** Jagged Judges measured POSITIONAL invariance on LLM judges; our probe measures input-transform agreement on a corpus engine. The prior transfers as direction (negative), never as magnitude.
- **Deterministic engine + prefix transforms may barely move decisions** — the null is the expected outcome; the probe is cheap precisely so the class can be closed honestly rather than argued about.
- **K too large → vacuous core** (Nozick's breadth-vs-depth tradeoff): K is a dial with a sweep (2/4/8) in the probe; no serving posture pins K before the sweep.
- Modelless lanes have no reasoning traces — the paper's trace-level invariance (inspect WHERE proposals diverge) has no modelless instantiation; deferred to hosted comparison lanes.

---

## 4. Verdict

**Tiers: Gain (thin).**

- **Not Super-GOAT:** Q1 fails — the class is published (self-consistency family, semantic entropy, consistency-abstention selective classification) and the nearest family member is internally MEASURED (Plan 008 null). Our delta is a second perturbation family + a readout extension on an existing harness — a composition, not a new mechanism class.
- **Not GOAT:** no proven gain; the proof path is one report-only probe with a pre-registered null. GOAT-grade claims wait for G4.
- **Not Pass:** the input-transform family + invariant-core extraction are genuinely unmeasured on the serving path, the fused abstain demonstrably lacks an agreement-structure channel, and the reverse-grep found the shaped gap (Plan 008 closed ONE family's verdict, pre-registered as a matched-coverage comparison — the family swap is its own falsifiable, now with a placebo arm because the families converge additively on this engine). Cheap enough that Pass would be the lazy verdict.
- **One-line:** the paper's sketch, instantiated on our engine, is Plan 008's experiment with an additive input-transform family (semantic-shaped and placebo twins), an extraction readout, and a fused-third-channel arm — Issue 119 runs it once with the null path pre-registered and the close scoped to the run table + engine; corroborations ride free.

**MOAT gate (riir-reflex):** fits the decision-serving moat exactly (first-class abstention + honest comparison arenas); public-safe (public paper, public modelless engine, no rethink-class content); the escalation-composition naming stays instinct-side vocabulary.

**Actionable filed:** reflex [Issue 119](../.issues/119_context_invariance_probe.md) — `--ctx-probe`, report-only, gates G1–G4 pre-registered with the null path.

**Corroborations (no filing, cited for the record):**

> **Corroboration (synthesis):** Veličković et al. [arXiv:2601.22950 "Perplexity cannot always tell right from wrong"] — likelihood-based model selection preferring the less accurate model OOD is the published form of our **lossy-surface promotion rule** (riir-infer serving posture: quantization gates on per-family conditional retention, never aggregate perplexity — "aggregates can be flat while family behaviour flips"). PASS — the rule already ships; this paper is its citation.

> **Corroboration (synthesis):** Lopez-Paz footnote 2 (the open problem: "find a variant of the softmax function that does not require infinite inputs to model one-hot vectors") — our stack answered this by construction: **sigmoid, never softmax**; `SigmoidGateCalibrator` + hesitant answers as a first-class answer shape; overconfidence-as-hallucination-cause (Kalai et al., cited by the paper) is the failure mode the sigmoid law structurally avoids. PASS — the law already ships (katgpt-rs constraint 2).

> **Corroboration (synthesis):** continual learning framed as invariance across user groups (never time slices) restates the intake laws — refine `adoption_gate` + `calibrated_dispatch` (evidence-gated admission) and the grammar-not-answers echo gates (never ingest test-shaped rows). PASS — the principles already ship.
