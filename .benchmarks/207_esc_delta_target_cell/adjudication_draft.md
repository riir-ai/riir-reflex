# i122 T3 served-side adjudication draft (auto-generated; numbers measured)

## Slice identity + disjointness (pre-read assertions)
  cross-arm slice identity: identical — OK

## Per-arm served-side aggregates (TEST half, cascade · english)

| arm | n | escalated | think_only | cheap_only | precision | Wilson95 LCB | value (mean δ) | esc rate |
|---|---|---|---|---|---|---|---|---|
| incumbent | 300 | 173 | 18 | 35 | 0.1040 | 0.0668 | -0.0983 | 0.577 |
| delta | 300 | 173 | 18 | 35 | 0.1040 | 0.0668 | -0.0983 | 0.577 |
| inverted | 300 | 173 | 18 | 35 | 0.1040 | 0.0668 | -0.0983 | 0.577 |

## Bars (pre-registered in the issue BEFORE the read)

- (a) precision: δ-arm LCB 0.0668 vs incumbent point 0.1040 → FAIL; Newcombe MOVER δ CI [-0.0916, +0.0916] (unpaired — no row-level cross-arm pairing by the split law, DISCLOSED) → straddles/negative
- (b) value: δ -0.0983 − incumbent -0.0983 = +0.0000 (bar ≥ −0.02) → PASS
- (c) rate band: δ 0.577 vs incumbent 0.577 (±0.10) → PASS
- T4 control: inverted precision 0.1040 < incumbent 0.1040 → DID NOT COLLAPSE (control suspect)

## T5a ceiling row (Bench-195 fired-set 2×2, n = 117 — cite 117, never 200)

| ceiling | served think | cheap alone |
|---|---|---|
| 85.5% oracle (100/117) | 70.1% (82/117) | 55.6% (65/117) |

Headroom 15.4 pts = exactly the 18 cheap-only rows. Any δ-fit gain adjudicates INSIDE this band.

## VERDICT (mechanical, pre-registered): FAIL
- bar (a) failed — the δ target produced no precision gain over the incumbent gate
- T4 control did not collapse — inspect before trusting any PASS
- the honest-death arm applies: δ-fit indistinguishable from incumbent ⇒ NO GAIN, cell closes

## Box-state provenance

Each arm's results.json RunMeta carries box_state (start/end). Quote the three arms' PROVENANCE lines in the landed RESULTS.md; the runs are correctness reads (routing aggregates), not latency claims.
