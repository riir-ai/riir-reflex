# Issue 115: Synth-Corpus Diversity Measurement Arm (Diffusion-Diversity Div)

**Date:** 2026-11-09
**Status:** Open
**Origin:** katgpt-rs Research 660 ([arXiv:2606.01861](https://arxiv.org/abs/2606.01861)) — distillation of the self-play curriculum-diversity theory. Consumer plan: katgpt-rs Plan 634 (primitive `diffusion_diversity`); sibling consumers: riir-ai Plan 646, riir-train Issue 680, riir-refine Issue 222.

## Problem

The teacher-synthesized corpus lanes (`--synth-corpus`, the grammar-not-answers train-pool construction) are gated for **leakage** (echo gates) but not for **spread**: a synthesized train pool can pass every echo gate while clustering tightly around the seed questions' semantic region — the paper's exact conjecturer failure (artificially complex, cluster-local curricula; Minimo's longer-not-deeper analog). No shipped reflex surface measures distribution-level corpus spread.

## Signal-diff vs shipped cousins (why this is not covered)

- Echo gates (`harness/echo_gates`) detect test-row leakage/echo — contamination, not spread.
- Corpus-distance/cap levers score per-item retrieval distance — per-item, not distribution-level effective spread under multi-hop graph structure.
- instinct `corpus_centroid` (simhash fold) is an admission-time centroid gate, cross-repo — no diffusion, no per-pool scalar.

`Div_P^τ(μ) = 1/‖μᵀP^τ‖²_{ℓ2(1/π)}` (τ-step random-walk diffusion on the corpus kNN-embed graph, inverse-stationary weighting) is a closed-form per-pool scalar no existing gate computes.

## Task

- [ ] **T1** After katgpt-rs Plan 634 Phase 1 lands: add a REPORT-ONLY `--synth-div` measurement arm to the harness — build the kNN similarity graph over the synthesized pool's embed vectors, emit `Div_P^τ` per pool beside the echo-gate output; no corpus changes, no gate changes, exit codes unchanged.
- [ ] **T2** Correlation cell: across existing synth pools, record Div vs `--corpus-ab` accuracy deltas (one frozen read each); pre-register the question "does low-Div predict low marginal corpus-ab gain?" — only a YES with a Spearman floor promotes T3.
- [ ] **T3** `- [-]` DEFERRED (promotion-gated): a diversity-aware pool-selection weight in `--synth-corpus`. Never wires without T2's measured correlation and the Plan 634 GOAT pass.

## Gates

- T1 feature-off byte-identity (the arm is opt-in; default runs unchanged — green-zero rule, `required-features` row in the same commit).
- No softmax anywhere; sigmoid-compatible weighting only if T3 ever arms.
