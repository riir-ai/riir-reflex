# Bench 203 — Lepski cascade adaptive depth: the i130 frozen read (NEGATIVE)

**Verdict: PREMISE REFUTED on the reflex surface — no G3 read, no promotion.** The
pre-registered gates (`.issues/130`, written before the run) decided it: P-A failed on
8/8 suites (no grid depth beats the routed-whole reference anywhere — the depth curves
are flat-to-worse), P-B failed on 7/8 (the cascade abstains on 68–97% of cal questions,
driven by a σ̂ = 0 rate of 0.75–1.00). The test slice was never read — that is the
pre-registered verdict path, not a gap.

Artifacts: `lepski_depth.json` (full rows) · `LEPSKI_DEPTH.md` (the rendered read).
Lane: `--lepski-depth` behind the opt-in `lepski_depth` feature (harness-only, G-ISO —
the serve path untouched). One invocation = one frozen read (2026-11-20T19:5xZ,
`26fcfba9` + the lane commit, release binary, ~7 s wall).

## Box state (the latency rows' disclosure)

`scripts/bench_preflight.sh` **REFUSED this window** — `powermode=2(high)`, load 6.3
transient. **No latency figure below is a publishable number**; they are carried for
shape only. The QUALITY read is deterministic (forced-pick accuracy, sequential folds,
first-index ties) — load/power do not enter it, so the verdict stands independent of
the refusal.

- σ̂ (bulk-MAD, two O(n) selections over ≤512 elements): p50 1.0–2.5 µs, p99 ≤ 3.5 µs
  across suites — the katgpt-rs `35c98d9dd` latency question is answered on this
  surface: the σ̂ is negligible against the 1 ms decision budget (needs a clean-window
  confirmation to QUOTE, but the order of magnitude is unambiguous).
- The cascade fold: p50 0.04–0.2 µs — negligible.
- The adaptive scoring pass (options × grid-max per-doc compressions): p50 0.4–17 ms —
  4–50× the routed-whole pass, far over the budget — moot given the quality negative,
  but it corroborates the upstream G2 cost finding at reflex scale.

## The mechanism (evidence-supported reading)

1. **P1 — the in-domain grid is structurally inert at the served scale (160/160
   domains):** the engine is one domain per label; each domain's drafter corpus is
   ≤ 64 docs, and k₀ = ⌈8·ln(n/η)⌉ ≥ 8·ln n > n/2 there — `implementable_j` = 0 on
   every domain. The adaptive-depth arm cannot even form a two-point grid in-domain.
2. **No depth headroom (P-A, 8/8 FAIL):** the pool-level per-doc prefix-mean estimator
   is dominated by the shipped whole-corpus form at EVERY depth — accuracy(k) is
   flat-to-worse across the grid on all eight suites. There is no bias/variance
   tradeoff for the cascade to adapt over: the whole corpus is simply the better
   estimator at every k.
3. **The noise scale does not exist (σ̂ = 0 on 75–100% of questions):** the per-doc
   compressed-length deltas of the leading option are near-constant across the ranked
   window — the response variable is a step function per option, not a noisy continuous
   measurement. With σ̂ = 0 ⇒ τ̂ = 0 ⇒ any drift rejects at the first step (the 68–97%
   abstain rates; those same rejections prove the estimate series genuinely moves — an
   exactly-zero difference would PASS under `≤`). The cascade's premise (agreement as a
   smoothness detector over noisy responses) has no material to work on here.

Consistent with the upstream fleet: dash_attn G2 FAIL (cost at n = 1024), riir-refine
Issue 254 NO-GAIN (watch row). The remaining open consumer is riir-ai Issue 1239 (the
DEC-field agreement fusion — an attention/belief surface, untouched by this negative;
Research 681 §2.3's pre-named fallback is exactly where the consumer stays).
