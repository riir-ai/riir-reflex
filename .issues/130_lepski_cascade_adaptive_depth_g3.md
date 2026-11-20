# Issue 130: Lepski cascade adaptive retrieval depth + abstain-on-cascade-failure — the G3 promotion cell

**Status:** CLOSED NEGATIVE (2026-11-20, bench 203) — the premise is refuted on the reflex surface; no G3 read, no promotion. Record: `.benchmarks/203_lepski_depth_g3/` + `HISTORY.md`.
**Lease closed:** zed-m3-i130lepski@m3, 1795201690, expires 2026-11-22T19:08Z — closed by the same lane.

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

- [x] **Latency note (katgpt-rs `35c98d9dd`):** ANSWERED on this surface, shape-only (the bench
      preflight refused the window — powermode=2 high — so the figures are NOT publishable
      numbers, disclosed in BENCH.md): σ̂ p50 1–2.5 µs / p99 ≤ 3.5 µs (two O(n) selections over
      ≤512 elements) and the cascade fold ~0.04–0.2 µs are negligible against the 1 ms budget;
      the adaptive scoring PASS itself (options × grid-max per-doc compressions) reads 0.4–17 ms
      — 4–50× the routed-whole pass, far over budget — corroborating the upstream G2 cost
      finding. The σ̂ was computed over the top-grid-max window per the note.
- [x] Wire the cascade behind its own feature/flag (opt-in), report-only first. LANDED: feature
      `lepski_depth` (implies `modelless` + `katgpt-core/estimators`), the `--lepski-depth`
      exclusive harness mode, `src/harness/runner/lepski_depth.rs` — harness-only (G-ISO), the
      serve path untouched; the flag refuses exit 2 on a build without the feature.
- [x] **Premise check BEFORE the G3 read (the named weak point, Research 681 §2.3):** RUN —
      **REFUTED**, on three independent legs (bench 203): (1) no depth headroom — P-A failed
      8/8 suites, the depth curves are flat-to-worse vs the routed-whole reference at EVERY
      grid depth; (2) the noise scale does not exist — σ̂ = 0 on 75–100% of questions (the
      per-doc compressed-length deltas are near-constant step functions per option, not noisy
      continuous measurements; with τ̂ = 0 the cascade abstains on 68–97% of cal questions,
      first-step); (3) structurally — the in-domain grid cannot form at the served scale at all
      (k₀ ≥ 8·ln n > n/2 at the ≤64-doc per-label corpora: `implementable_j` = 0 on 160/160
      domains). ⇒ the consumer stays at attention/belief (riir-ai Issue 1239), per the
      pre-named fallback. The gates decided before any test read — exactly the pre-registered
      verdict path.
- [x] **G3 (the promotion gate, katgpt-rs Issue 1229):** NEVER READ — the premise gates
      gated out all eight suites (the pre-registered behavior; a skipped test read is a
      verdict, not a gap). Slice + bars were pre-registered in this issue before the run
      (commit `26fcfba9`). No promotion.
- [x] Abstain arm: cal-side data only (test gated out) — the cascade-first-step abstain fires
      on 68–97% of cal questions, driven by the σ̂ = 0 degeneracy, not by genuine instability:
      useless as an abstention signal on this surface. No conformal arm (τ̂ is not served as an
      interval here).
- [x] Report the result back to katgpt-rs Issue 1229's lineage (Research 681 / HISTORY) with the
      commit hash. LANDED: katgpt-rs Research 681 status + HISTORY Issue-1229 row carry the
      reflex consumer outcome with the landing hash.

Ref: katgpt-rs `.research/681_Locally_Adaptive_kNN_Neighbourhood_Selection.md`, katgpt-rs Issue 1229.
