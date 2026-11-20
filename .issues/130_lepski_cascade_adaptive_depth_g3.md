# Issue 130: Lepski cascade adaptive retrieval depth + abstain-on-cascade-failure — the G3 promotion cell

**Status:** open — routed from katgpt-rs Issue 1229 (primitives + G1/G4 + dash_attn arm landed upstream, katgpt-rs `e866932a1`); unclaimed, needs a quiet rig for the harness cell
**Claimed:** zed-m3-i130lepski@m3, 1795201690, expires 2026-11-22T19:08Z

Consumer of `katgpt_core::lepski_cascade` (`estimators` umbrella): `lepski_cascade`,
`prefix_means_into`, `tau_hat_into`, `sigma_hat_*`, `A_DEFAULT = 1` (serving constant — katgpt-rs
Issue 1229 G1 cell 2 PASS; `A_THEORY = 4.5` was FALSIFIED as a serving constant by cell 1).
Wire, never fork (katgpt-rs BOUNDARY: reflex consumes the primitive).

**Surface:** `Lz4FlexDrafter` scores option bytes against the routed domain's WHOLE corpus today.
Adaptive form: score against the top-k retrieved corpus docs (ONE retrieval at k_max), cascade
over the prefix grid, serve the largest agreeing k. Abstain when the cascade rejects at k₀
(fusable with `CorpusDistanceGate`, the global-threshold incumbent — this is the per-query
second signal, never a replacement).

**σ̂ choice (read katgpt-rs Issue 1229 before wiring):** a covariate-free ranked score vector
needs the bulk-MAD form (`sigma_hat_bulk_mad_f32`), not the NN-MAD — differencing ranked values
measures spacing and collapses τ̂. Pick per surface and write the derivation at the site.

- [ ] **Latency note (katgpt-rs `35c98d9dd`):** the dash_attn arm's G2 FAILED at n = 1024 (e2e 1.027 vs fixed-K_max; the bulk-MAD σ̂ is two O(n) selections, and nothing is saved when k̂ saturates). Price the σ̂ against the reflex p99 ≤ 1 ms budget; prefer a σ̂ over the top-k_max window when the corpus slice is large.
- [ ] Wire the cascade behind its own feature/flag (opt-in), report-only first.
- [ ] **Premise check BEFORE the G3 read (the named weak point, Research 681 §2.3):** does
      prefix-truncated corpus scoring degrade monotonically in k (bias grows, τ decays) the
      way kNN regression does? Measure on the cal slice; non-monotone ⇒ the premise is refuted
      on this surface ⇒ stop, record the negative, the consumer stays at attention/belief.
- [ ] **G3 (the promotion gate, katgpt-rs Issue 1229):** paired accuracy, cascade-adaptive
      depth vs incumbent fixed-depth scoring, on the harness suite slice (cal + test), paired
      LB95, ONE frozen read. Pre-register the slice + bar in this issue before running. Quiet
      rig only (`agent-mail rig run … --exclusive`). No promotion talk before it runs.
- [ ] Abstain arm: coverage/accuracy of abstain-on-cascade-failure fused with
      `CorpusDistanceGate` vs the gate alone. If it serves τ̂ as a calibrated interval, the
      conformal-naive floor (`ConformalIntervalCalibrator<SeasonalNaiveForecaster>`) is the
      comparison arm (katgpt-rs Report-the-Floor rule).
- [ ] Report the result back to katgpt-rs Issue 1229's lineage (Research 681 / HISTORY) with the
      commit hash.

## Pre-registration (written 2026-11-20T19:3xZ, BEFORE any run — the issue's own gate)

**Structural scale note (discovered at wiring, before the pre-read):** the engine builds ONE
DOMAIN PER LABEL; each domain's drafter corpus = that label's pool docs capped at
`corpus_cap_per_label` (40–64 docs — `per_label_corpus_docs`). The paper's grid constant
k₀ = ⌈8·ln(n/η)⌉ ≥ 8·ln n > n/2 at n ≤ 64 ⇒ `implementable_j` = 0 on EVERY domain ⇒ the
IN-DOMAIN grid cannot form at the served corpus scale (measured as P1 below, not assumed).
The only formable grid is at the SUITE-POOL level (the capped per-label doc sets flattened
label-major): the pool arm scores each option against the top-k retrieved pool docs — a
DIFFERENT estimator from the shipped routed-domain form, so the shipped form is carried as the
REFERENCE row, and the G3 PRIMARY pair is same-estimator single-variable: cascade-adaptive k̂
vs fixed-grid-max depth.

**Constants (pre-registered):** η = 0.05 · A = `A_DEFAULT` (1.0) · σ̂ =
`sigma_hat_bulk_mad_f32` over the k₀-winner's ranked response vector, top-grid-max window
(the latency note's large-slice form; derivation at the site: covariate-free ranked score
vector ⇒ bulk-MAD, never NN-MAD) · n in τ̂ = the ranked population size (pool size) ·
grid = `geometric_grid_into(implementable_k0(n, η), k_max)` · k_max = min(n, 512) ·
retrieval = descending dot(embed(ctx), doc row) via the shipped `Embedder` (L2-normalized;
stable sort, ties keep pool order) · per-doc score = a per-doc `Lz4FlexDrafter`'s
compressed-length delta `score(ctx, cand)` (the wire-faithful form) · prefix means via
`prefix_means_into` per option · winner per depth = FIRST-INDEX argmax · ctx = state + '\n' +
instructions (the wire question carries no criteria — mirrors `fresh_question`) · cand =
option bytes (choice = criteria keys in map order; score = level strings; noul = [yes, no],
label pick = usize::from(s_yes > s_no)) · reference row = the shipped form's drafter term
(routed domain by first-index argmax dot vs label centroid; whole-corpus drafter =
`docs.join("\n")`, `DrafterFix::Off` raw delta) — forced-pick accuracy everywhere (no
sigmoid, no gates; one variable at a time).

**Slice:** premise on CAL ONLY — typed_decisions (100), prompt_injections (100), sst5 (200),
emotion (200), ag_news (200), xnli_en (200), massive_intent_en (200), banking77 (200) — every
corpus-ON dataset suite. G3 test read ONLY for suites passing BOTH premise gates: banking77
(500), massive_intent_en (300), ag_news (400), emotion (400), sst5 (600), xnli_en (300),
prompt_injections (116), typed_decisions (400). ONE invocation = ONE frozen read (cal gates
decide whether test is touched; a skipped test read is a verdict, not a re-run); a test
re-run happens only on an INFRA failure and is disclosed loud.

**Premise gates (decided on cal, before any test read):**
- **P1 (structural, in-domain):** the fraction of domains with `implementable_j ≥ 2` at the
  constants above — measured, reported; expected 0 (the in-domain arm is inert; reported as
  the structural finding, not a gate).
- **P-A (headroom, per suite):** max over grid k of cal forced-pick acc(k) ≥ cal
  acc(reference routed-whole-corpus) + 1.0 pp.
- **P-B (selector sanity, per suite):** cal acc(cascade) ≥ cal acc(best fixed grid depth) −
  1.0 pp AND the cascade abstain rate (rejected_at == Some(1)) ≤ 50%.
- **G3 bar (the gated read, suites passing P-A ∧ P-B):** paired LB95 of (cascade −
  fixed-grid-max) per-case correctness > 0 on the ONE frozen test read. The reference pair
  (cascade − routed-whole) is REPORTED, never gated. No promotion talk before it runs.

**Abstain arm (report-only, cal+test of the G3 suites):** fused rule = gate-abstain OR
cascade-first-step-reject; the gate = routed-domain max-cos < the cal 30th-percentile
threshold (the shipped gate's cal-fitted convention); report coverage + accuracy of
(gate alone) vs (gate ∨ cascade). No conformal arm unless τ̂ is served as an interval —
it is not, on this surface.

**Latency (report-only):** per-suite p50/p99 per-question scoring-pass cost + the σ̂ call
cost + the cascade fold cost, priced against the p99 ≤ 1 ms decision budget; σ̂ over the
top-grid-max window per the latency note. Box state disclosed beside every timing row
(m3, release binary, load at run time).

**Determinism:** sequential folds, first-index ties, no RNG, one process. G-ISO: harness-only;
the serve path is untouched; the lane ships behind the opt-in `lepski_depth` feature + the
`--lepski-depth` harness flag (the flag refuses exit 2 on a build without the feature — the
`--corpus-gap` refusal class). The default build is byte-identical by construction.

Ref: katgpt-rs `.research/681_Locally_Adaptive_kNN_Neighbourhood_Selection.md`, katgpt-rs Issue 1229.
