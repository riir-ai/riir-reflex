# Issue 130: Lepski cascade adaptive retrieval depth + abstain-on-cascade-failure — the G3 promotion cell

**Status:** open — routed from katgpt-rs Issue 1229 (primitives + G1/G4 + dash_attn arm landed upstream, katgpt-rs `e866932a1`); unclaimed, needs a quiet rig for the harness cell

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

Ref: katgpt-rs `.research/681_Locally_Adaptive_kNN_Neighbourhood_Selection.md`, katgpt-rs Issue 1229.
