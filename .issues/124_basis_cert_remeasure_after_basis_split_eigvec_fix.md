# Issue 124: re-measure basis_cert (Bench 138/139) after the basis_split eigenvector-layout fix

**Status:** OPEN — filed 2026-11-19 by katgpt-rs-57@m3. Code half LANDED (this issue's commit); the re-measure is unclaimed. Routed: any reflex lane (heavy run → amo/bmo via `agent-mail rig run`, never m3).
**Claimed:** katgpt-rs-57@m3, 1795097632, expires 2026-11-21T14:13Z

## What happened

- katgpt-rs `9cae0610f`: `basis_split::split` returned the ROWS of the eigenvector matrix as its variant basis instead of the eigenvectors (solvers store eigenvector j as column j). A rotated planted rank-2 block captured 0.39/2.0 (d=8) and 0.20/2.0 (d=24). Eigenvalues, `k`, gap ratios and verdicts were always right.
- This repo's `src/harness/basis_cert.rs` T3 raw-direction map read `build_zca_whitener`'s eigenvectors as `sig_eigvecs[j*d + i]` (rows) — the same transpose: it applied Vᵀ Λ^{1/2} V instead of V Λ^{1/2} Vᵀ. Fixed in this issue's commit (`[i*d + j]`); `basis_cert` lib tests 10/0.

## What is void

- Bench 138: `holdout leakage`, `agreement residual`, `rename residual` columns (basis-derived). `verdict`/`k`/`gap` stand.
- Bench 139: every T3 row — the perturbation directions were neither the variant eigenvectors nor their correct raw-space map.

## Tasks

- [ ] Re-run the Bench 138/139 command (RECORD.md has it verbatim) on a quiet rig against katgpt-rs ≥ `9cae0610f`; write the new tables as a NEW bench number citing 138/139 (never overwrite a record), and mark 138/139's basis-derived columns void with a pointer.
- [ ] Re-read Issue 086's conclusions that cite those columns; correct any verdict that moves.
