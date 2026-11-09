# Bench 178 — Issue 115 T2: the Div↔corpus-ab correlation cell

Issue 115 T2 (the pre-registered correlation cell): "does low-Div predict
low marginal corpus-ab gain?" across the existing synth pools — one
frozen read each (the pools' OWN recorded corpus-ab deltas; this probe
adds the Div scalar per pool and the Spearman rank correlation).
**Verdict: the YES-direction correlation does not reach the
pre-registered floor (Div↔delta = +0.41, Div↔lb95 = +0.48 vs the ≥ 0.7
floor in the YES direction) — T3 stays DEFERRED, exactly as the issue
gated it. The direction itself is informative: positive (higher Div →
higher gain) — the OPPOSITE of the paper's conjecturer-failure worry on
these pools: more spread never hurt, but the correlation is too weak to
promote a selection weight on.**

## Run posture

- Probe: `examples/synth_div_probe.rs` behind the opt-in `synth_div`
  feature forward (`katgpt-core/diffusion_diversity` — CONSUMED, never
  re-derived: `SimGraph::div_tau` is Plan 634's primitive, the
  substrate-first law). Report-only; no gate; exit codes untouched.
- Pools: the 7 synth pools with a recorded frozen corpus-ab read
  (`corpus_ab.json` per pool dir; deltas transcribed once, source dirs
  named in the probe). All 7 artifacts present and read (n = 974–2048
  synth rows each; every pool is massive_intent_en — disclosed
  limitation: the cell is ONE suite's pools, never a cross-suite claim).
- Graph: kNN-8 over the pool's embed vectors (the shared 256-bucket
  hashed-bag embedding, cosine edges clamped ≥ 0, symmetrized);
  μ = uniform; τ = 2 — one posture, disclosed (no τ/k sweep: the cell
  is a correlation screen, not a tuning pass).
- Box state: m3, AC power, load ≈ 5 (counts are contention-immune —
  the Div scalar is deterministic; no latency claim).

## The numbers

| pool | rows | Div(τ=2) | delta | lb95 |
|---|---|---|---|---|
| corpus_synth | 2048 | 1.9420 | +0.0333 | +0.0110 |
| corpus_synth_bekko | 1975 | 1.9514 | +0.0267 | +0.0062 |
| corpus_synth_bekko400m | 2048 | 1.9445 | +0.0300 | +0.0086 |
| corpus_synth_dens50 | 1859 | 1.9416 | +0.0200 | +0.0016 |
| corpus_synth_dens75 | 1980 | 1.9493 | +0.0233 | +0.0062 |
| corpus_synth_learn104_p50read | 974 | 1.9326 | +0.0033 | −0.0080 |
| corpus_synth_learn104_p75read | 1445 | 1.9382 | +0.0267 | +0.0062 |

Spearman (7 pools): **Div↔delta = +0.41 · Div↔lb95 = +0.48**.

## Reading

1. The Div spread across pools is NARROW (1.9326–1.9514, ~1% range) —
   the synth pools' graph-spread is nearly identical regardless of
   teacher/density recipe. The pools that differ MEANINGFULLY are the
   learn104 cuts (974/1445 rows vs ~2000), and the p50 cut's tiny Div
   goes with its collapsed gain (+0.0033/−0.0080) — the only real
   signal in the cell: **pool SIZE dominated spread; the recipe axis
   the paper's conjecturer failure lives on never separated here.**
2. The YES floor (|ρ| ≥ 0.7 toward low-Div-predicts-low-gain, i.e.
   NEGATIVE ρ) is not met — the observed sign is POSITIVE. T3's
   promotion gate fails on both sign and magnitude. `T3` stays
   `- [-]` DEFERRED per the issue's own wording; no re-run is owed
   unless a pool family with real Div separation appears (a pool at
   Div < 1.5, say — none exists today).
3. Honest caveats: n = 7 pools, one suite, one (k, τ) posture; a
   Spearman on 7 points is a screen, not an estimate — the failure
   verdict is safe BECAUSE the floor was pre-registered and the sign is
   wrong, not because 7 points are precise.

## Side record: the gate-chain bug this cell surfaced (katgpt-rs `c85ea1348`)

Enabling `diffusion_diversity` ALONE failed to compile katgpt-core
(E0433): Plan 634 T1.1 added the diffusion exports under the
`temp_loss_fingerprint`-only `pub mod diversity` gate. Fixed upstream
(`c85ea1348`, the mod gate is EITHER child feature; the temp rows keep
their narrower gates) — the feature-gate-audit class, found by a
consumer.
