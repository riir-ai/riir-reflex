# Issue 119 — the context-invariance probe (`--ctx-probe`): additive input-transform agreement, measured once with a placebo control

**Status:** OPEN — unclaimed; routed: the next reflex idle lane (boss re-fire backstop). Report-only harness lane; ANY serving wiring exists only behind G4 passing.

**Ref:** [`.research/017_Against_All_Likelihood_Invariance_Serving.md`](../.research/017_Against_All_Likelihood_Invariance_Serving.md) (arXiv:2611.09385 "Against All Likelihood", Lopez-Paz 2026)

## Why (the gap, named — and the honest mechanism)

- **Plan 008 / Bench 092** measured the nearest perturb-and-aggregate family on THIS engine — `mc_ensemble` (Bernoulli bucket-mask embedding noise → N deterministic passes → U-stats rejection + LCB ranking) — and the **null path FIRED** (gate 6, dataset suites n ≥ 100): ≈0 marginal lift over the fused gate at matched coverage; the feature stays opt-in.
- The paper's sketch is the context variant: transform the input's context (perspective/framing), re-answer under each, extract the invariant core, **abstain when no core survives**. Unmeasured here.
- **The honest mechanism caveat (verdict round 1):** on this engine, "semantic context transform" has no semantic channel to ride — `src/embed.rs` embeds ASCII-lowercased words + word bigrams as FNV-1a-hashed buckets (sign trick, L2-normalized). A persona prefix is therefore a **constant additive bucket set plus one boundary bigram** — the additive mirror of `mc_ensemble`'s bucket mask, not a perspective shift. The probe measures exactly what it can: whether *any additive input transform* produces an agreement-structure abstention signal with lift over the fused gate — and a **placebo arm** separates "additive hash noise" from anything invariance-specific.
- The fused abstain (`noul` = calibration score + `CorpusDistanceGate`) carries **no agreement-structure channel**.
- **Ranked prior is negative**: the Jagged Judges paper (arXiv:2608.12645, distilled in katgpt-rs Research 499) measured positional invariance the WEAKEST instability predictor on LLM judges (0.37, vs repeat 0.42, jury margin 0.59) — an external prior of a neighbouring kind (LLM judges, positional perturbation); Bench 802's lesson stands: agreement labels measure "will I change my mind", never "am I right".

## What (report-only; one harness pass)

- [ ] **T1** — `src/harness/context_transform.rs` (pure, ungated, like `permutation.rs`): TWO transform families, both from fixed name-stable tables, both additive-by-construction on this engine:
  - **semantic-shape family**: persona prefixes ("From the perspective of a <discipline>: "), framing suffixes, neutral formatting variants;
  - **placebo family**: token-count-matched nonsense/random-word prefixes from a fixed table (matched token count so the additive bucket mass is comparable).
  - `transform[0]` of each family = identity (byte-identical passthrough). No RNG; if any ordering shuffle is needed, seeded Sattolo (the label-perm convention).
- [ ] **T2** — `--ctx-probe` exclusive early-exit mode in `src/bin/harness.rs` following the perm-probe shape (`--ctx-k`, `--ctx-max-cases`): re-decide every transformed case through the SAME production `ChoiceOracle` seam (permutation.rs:408) — never a private decide path; collect per-case answer distributions across transforms, per family.
- [ ] **T3** — report block: **the PRIMARY CELL is pre-registered — K=4, semantic-shape family, the Bench 092 gate-6 suite set (dataset suites with n ≥ 100 at run time, exclusions named), pooled verdict = mean per-suite paired Δ-accuracy with pooled SE; every other cell (K sweep 2/8, per-suite tables, placebo tables) is descriptive.** Full report: (a) agreement histogram per family (all-K-agree / majority / no-majority shares); (b) empty-core rate; (c) **G2a cell**: mode-seat alone at matched coverage vs the fused gate (placebo-adjusted); (d) **G2b cell**: fused + empty-core channel vs fused alone at matched coverage; (e) placebo-arm twins of both cells; (f) vacuous-core disclosure across K; (g) `ctx_probe.json` + markdown like the perm-probe's records.
- [ ] **T4** — gates, pre-registered:
  - **G1 (identity, hard):** `--ctx-k 1` reproduces the default lane byte-identically (the perm-probe's control discipline, minus the order-flip red — additive input transforms have NO invariance guarantee, so flips at K>1 are expected data, never a control red).
  - **G2a (PRIMARY — extraction-vs-replacement):** mode-seat alone vs the fused gate at matched coverage, **placebo-adjusted by statistic, not point estimate** — the verdict is the paired LB95 of (semantic − placebo) per-case Δ-accuracy on the primary cell > 0, AND the paired LB95 of (semantic-mode-seat − fused-gate) at matched coverage > 0. A win the placebo shares is additive hash noise, not invariance. This is THE verdict arm.
  - **G2b (SECONDARY — the third-channel claim):** fused + empty-core vs fused alone at matched coverage, same statistic rule — paired LB95 of (semantic − placebo) on the channel-armed Δ > 0, then paired LB95 of the channel lift > 0. The note's "third abstention channel" claim lives or dies HERE, not in G2a.
  - **G3 (the NULL PATH — the Plan 008 verdict shape):** if the primary cell's paired LB95 ≤ 0 → record the NEGATIVE in a bench record and close this issue. Scope of the close: **this run table, on THIS engine (a bag-of-hashed-words additive-feature engine), for the tested families** — not the class. Named reopen trigger: an engine with order- or context-sensitive features (position-aware n-grams, attention, structure beyond one bigram), where transforms stop being additive.
  - **G4 (the promotion path):** G2a passing (placebo-adjusted LB95 > 0) → a promotion plan behind opt-in feature `ctx_seat` (never default without GOAT: G1 correctness / G2 latency at the serve budget / G3 no-regression on all suites / G4 alloc-free at the K=4 posture). G2b passing alone feeds the fused-gate extension plan instead.
- [ ] **T5** — bench record under `.benchmarks/` (either verdict) with `bench_preflight.sh` PROVENANCE quoted; post-AGREE this issue's close rides the record, never a silent removal.

## Constraints

- **Report-only**: no wire change, no serve-path change, no new feature flag in this issue (the lane is ungated like perm-probe).
- **Deterministic**: fixed transform tables, no hosted calls, no GPU; one pass ≈ the perm-probe's cost (K× µs-scale decisions × 2 families).
- **Datasets required**: SKIP loud when `.raw/datasets` is absent — never a fabricated number.
- **Named-out per Research 017 §3.4** (not in scope here): trace-level invariance (hosted lanes only), cross-model environments (ESC remote leg), ESC escalation composition (`EscalateSpec`), the refine intake variant (named future consumer, unfiled until the transform family proves signal).

## Honest expectation

The prior says G3 fires — and on an additive-feature engine the placebo arm plausibly ties the semantic arm, which would mean the probe measured hash noise wearing a persona. That is a fine outcome: one harness pass either finds placebo-separated lift (G4 opens) or bounds the additive-transform class on this engine with a measured number (G3 closes, scoped). Neither is a seat.
