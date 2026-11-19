# Bench 199 — basis_cert re-measure after the basis_split eigenvector-layout fix (Issue 124)

**Date:** 2026-11-19T14:47Z · **Host:** amo (rig job `3c0ab5fa`, Shared lane) · **Supersedes the basis-derived columns of:** Bench 138 + Bench 139
**Lattice (sibling worktrees at origin/develop):** katgpt-rs `cc214e431` (contains the fix `9cae0610f`, asserted in-job), riir-reflex `92cc9f4` (contains this repo's `sig_eigvecs` fix `33ddc0b`), riir-infer `91ac03dc`, riir-reflexer `549f3a9`.
**Command:** identical to Bench 138/139 (`--perm-probe --basis-split --skip-laya --suites typed_decisions,banking77,ag_news,emotion --perm-max-cases 60`), release, `basis_cert`. Deterministic; not a latency claim, so no preflight bar applies. Artifacts: `PERM_PROBE.md` + `perm_probe.json` beside this file.

## Why

katgpt-rs `9cae0610f`: `basis_split::split` returned the ROWS of the eigenvector matrix as the variant basis, not the eigenvectors. Separately, this repo's T3 raw-direction map read `build_zca_whitener`'s eigenvectors transposed (fixed `33ddc0b`). Eigenvalue-derived quantities (`verdict`, `k`) were unaffected by the transpose.

## T1 — presentation-subspace certificate (Bench 138 → now)

| suite | verdict | k | gap 138 → now | holdout leakage 138 → now | agreement residual 138 → now | rename residual 138 → now |
|---|---|---|---|---|---|---|
| typed_decisions | sufficient_variation | 202 | 4.86 → 68.19 | 0.1920 → **0.0851** | 0.4327 → **0.2365** | 0.1857 → **0.1459** |
| ag_news | sufficient_variation | 111 | 1879.38 → 3880.56 | 0.5417 → **0.0016** | 0.7186 → **0.0304** | 0.6117 → **0.4685** |
| emotion | sufficient_variation | 124 | 380.26 → 1268.62 | 0.5358 → **0.0003** | 0.7307 → **0.0105** | 0.4475 → **0.4635** |
| banking77 | sufficient_variation | 255 | 18.27 → 145.76 | 0.0040 → **0.0000** | 0.0536 → **0.0003** | 0.0010 → **0.0001** |

- **The headline moves:** with the true eigenvectors, the fitted presentation null space generalizes almost perfectly on ag_news, emotion and banking77 (held-out leakage ≤ 0.0016). Bench 138's 0.54 leakage on ag_news and emotion was the transpose, not the embedding. typed_decisions keeps a real residual (0.085).
- **Gap changes are NOT from this fix** (gaps are eigenvalue-only). Bench 138 ran at reflex `e3b8d72` against a katgpt-rs predating the Issue 931 whitener repairs (Bench 139 records the 931-fixed pin). Those repairs change the whitened spectrum. Verdicts and `k` are unchanged.
- Rename residuals stay substantive on ag_news and emotion (~0.47): renamed renderings genuinely leave the fitted null space there. That is a finding about the embed, not the instrument.

## T3 — Thm-3 perturbation certificate (Bench 139 → now)

Identical at displayed precision to Bench 139: typed_decisions `vacuous_no_response`; ag_news, emotion and banking77 `pass` with 0 gate flips; gate max Δp 5.96e-8 / 2.98e-8 / 3.73e-9; median ‖Δ‖ unchanged. The verdicts survive the corrected directions. At the `fp_1x` rung the scorer is insensitive to any norm-ULP direction, so the direction fix does not show at this precision. Bench 139's verdict stands; its direction provenance is superseded by this record.

## Perm probe

All four modelless lanes PASS at 0.00 pt median swing (unchanged from 138).
