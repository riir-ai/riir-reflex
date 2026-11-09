# Issue 115: Synth-Corpus Diversity Measurement Arm (Diffusion-Diversity Div)

**Date:** 2026-11-09
**Status:** CLOSED 2026-11-10 — T1+T2 landed, the correlation gate FAILED (Bench 178: Spearman +0.41/+0.48 vs the ≥0.7 YES floor; positive sign, ~1% Div spread across pools — pool size dominated, the recipe axis never separated). T3 stays deferred, its gate failed; the issue's own wording closes it ("only a YES promotes T3" — a NO closes). Re-open only when a pool family with real Div separation exists. Record: `.benchmarks/178_issue115_synth_div_correlation.md`. Origin: katgpt-rs Research 660 ([arXiv:2606.01861](https://arxiv.org/abs/2606.01861)) — distillation of the self-play curriculum-diversity theory. Consumer plan: katgpt-rs Plan 634 (primitive `diffusion_diversity`); sibling consumers: riir-ai Plan 646, riir-train Issue 680, riir-refine Issue 222.
**Origin:** katgpt-rs Research 660 ([arXiv:2606.01861](https://arxiv.org/abs/2606.01861)) — distillation of the self-play curriculum-diversity theory. Consumer plan: katgpt-rs Plan 634 (primitive `diffusion_diversity`); sibling consumers: riir-ai Plan 646, riir-train Issue 680, riir-refine Issue 222.

## Problem

The teacher-synthesized corpus lanes (`--synth-corpus`, the grammar-not-answers train-pool construction) are gated for **leakage** (echo gates) but not for **spread**: a synthesized train pool can pass every echo gate while clustering tightly around the seed questions' semantic region — the paper's exact conjecturer failure (artificially complex, cluster-local curricula; Minimo's longer-not-deeper analog). No shipped reflex surface measures distribution-level corpus spread.

## Signal-diff vs shipped cousins (why this is not covered)

- Echo gates (`harness/echo_gates`) detect test-row leakage/echo — contamination, not spread.
- Corpus-distance/cap levers score per-item retrieval distance — per-item, not distribution-level effective spread under multi-hop graph structure.
- instinct `corpus_centroid` (simhash fold) is an admission-time centroid gate, cross-repo — no diffusion, no per-pool scalar.

`Div_P^τ(μ) = 1/‖μᵀP^τ‖²_{ℓ2(1/π)}` (τ-step random-walk diffusion on the corpus kNN-embed graph, inverse-stationary weighting) is a closed-form per-pool scalar no existing gate computes.

## Task

- [x] **T1** DONE (2026-11-10, Bench 178): the measurement lane is `examples/synth_div_probe.rs` behind the opt-in `synth_div` feature forward (`katgpt-core/diffusion_diversity` — CONSUMED `SimGraph::div_tau`, never re-derived) — NOT a harness arm: the pools live as `.raw/corpus_synth*/` artifacts with their own recorded corpus_ab.json reads, so the correlation cell runs against the artifacts directly; report-only, exit codes untouched, no corpus/gate changes. En route: fixed the katgpt-rs gate-chain bug the forward surfaced (`c85ea1348` — the diversity mod gate is EITHER child feature).
- [x] **T2** DONE (2026-11-10, [Bench 178](../.benchmarks/178_issue115_synth_div_correlation.md)): the pre-registered question answered NO — Spearman Div↔delta = +0.41, Div↔lb95 = +0.48 over the 7 pools with frozen reads (the floor was |ρ| ≥ 0.7 in the YES/NEGATIVE direction); the observed sign is POSITIVE (more spread → more gain, the opposite of the conjecturer worry) but the Div spread itself is ~1% across pools (1.9326–1.9514) — pool SIZE dominated; the recipe axis never separated. T3 stays DEFERRED (its gate failed on sign AND magnitude); re-run only if a pool family with real Div separation appears.
- [ ] **T3** `- [-]` DEFERRED (promotion-gated): a diversity-aware pool-selection weight in `--synth-corpus`. **Gate FAILED (Bench 178): the YES floor did not clear** — never wires without a NEW T2 pass whose pool family actually separates on Div.

## Gates

- T1 feature-off byte-identity (the arm is opt-in; default runs unchanged — green-zero rule, `required-features` row in the same commit).
- No softmax anywhere; sigmoid-compatible weighting only if T3 ever arms.
