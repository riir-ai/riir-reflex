# Bench 176 — Issue 116 pick_latch G3 cell: NEGATIVE (the hold costs accuracy; the churn is signal)

Issue 116 T3 (the G3 GOAT cell) — gate-on vs gate-off on the real harness
stream, per the issue's pre-registration: domain flip-rate reduction
WITHOUT suite-accuracy loss (paired LB95 ≥ 0 per suite) and no G2-bar
latency delta. **Verdict: FAIL — the accuracy bar breaks on the suites
where routing matters (banking77 −8.0pp at the producer defaults), the
flip-reduction the latch buys is purchased directly with that accuracy,
and the flip-rate premise itself fails structurally at this seat.**
Honest negative — a verdict, not a bug (the issue's own escape hatch).
The feature stays opt-in, never promoted.

## Run posture

- reflex `3254602` (develop) — the census instrument landing this cell rode.
- Binary: `target/release/harness` built `--features pick_latch` (one
  binary, both census postures; `--pick-latch` arms the latch at the
  producer defaults temperature 1.0 / threshold 0.75, margin
  `1.0·lnit(0.75) = ln(3) ≈ 1.099` score units).
- Suites: the board's 8 modelless-lane suites — ag_news, banking77,
  emotion, massive_intent_en, prompt_injections, sst5, xnli_en,
  typed_decisions. Default posture (`--skip-laya`,
  gate-fit-calibrated on, registry corpus caps), canonical
  `.raw/datasets` pool; slice-integrity OK on every suite both arms.
- Box state: `PROVENANCE: power=AC Power load=5.27 swap=532.44M
  canary=skipped powermode=2(high)` (m3, preflight PASSED, load 5.3 <
  6.0 ceiling). Three sibling seal-viewer lanes ran concurrently during
  the A/B arms — **the accuracy/flip census cells are
  contention-immune** (deterministic decisions, no timing claim rides
  them); the LATENCY claim below was measured separately by the
  decision_set_goat bench under the PASSED preflight posture.

## The census (the instrument: `--route-census`, landed 3254602)

| suite | n_q (+10 det reruns) | bare changes | armed changes (t=1.0) | armed disagreements |
|---|---|---|---|---|
| typed_decisions | 2050 | 1547 | **0** | 1500 |
| ag_news | 410 | 284 | **0** | 275 |
| emotion | 410 | 162 | **0** | 108 |
| sst5 | 610 | 330 | **0** | 216 |
| prompt_injections | 126 | 38 | **0** | 31 |
| xnli_en | 310 | 89 | **0** | 148 |
| massive_intent_en | 310 | 156 | **0** | 100 |
| banking77 | 510 | 494 | **0** | 500 |

The armed seat's `changes = 0` on EVERY suite: it seats on the first
question and holds forever. Two independent causes, both measured:

1. **Structural — the producer defaults are mis-scaled for this
   consumer's score geometry.** `router_scores` is the raw dot of the
   L2-normalized query embedding with the L2-normalized unit centroids
   (cosines ∈ [-1, 1]); the challenger−held delta cannot practically
   reach the ln(3) ≈ 1.099 margin. katgpt-rs Issue 987's premise — the
   gate rides "the same score units" the argmax compares — does not
   transfer to this seat: the pick_domain fold is cosine-space, the
   producer fixtures were calibrated on unbounded synthetic scores.
2. **Empirical — the harness stream's route churn is SIGNAL, not
   noise.** The bare argmax changes route 284–494 times per stream
   because each eval question genuinely belongs to a different domain
   (iid-ish dataset order, up to 77 domains). The latch cannot
   distinguish signal-churn from noise-churn, so a margin that holds
   through churn holds through the signal.

## The accuracy cost (the G3 bar: paired LB95 ≥ 0 per suite — FAILS)

| suite | acc bare | acc armed (t=1.0) | Δ |
|---|---|---|---|
| typed_decisions | 0.2800 | 0.2800 | 0 |
| ag_news | 0.4050 | 0.4050 | 0 |
| emotion | 0.2750 | 0.2725 | −0.25pp |
| sst5 | 0.2017 | 0.1950 | −0.67pp |
| prompt_injections | 0.4828 | 0.4828 | 0 |
| xnli_en | 0.3400 | 0.3400 | 0 |
| massive_intent_en | 0.6100 | 0.6000 | −1.0pp |
| banking77 | 0.3940 | 0.3140 | **−8.0pp** |

banking77 breaks the ≥ 0 bar decisively (77 domains, adjacent eval rows
route to different intents — the hold routes most questions to the
FIRST question's intent). massive_intent_en −1.0pp breaks it too.
RESULTS: `results_bare.json` / `results_armed.json` beside this record.

## The temperature sweep — the tradeoff curve completes the negative

`--pick-latch-temp` at threshold 0.75 over banking77 / ag_news / sst5
(margin = temp·ln 3):

| temp (margin) | banking77 chg/dis/acc | ag_news chg/dis/acc | sst5 chg/dis/acc |
|---|---|---|---|
| bare | 494 / 0 / 0.3940 | 284 / 0 / 0.4050 | 330 / 0 / 0.2017 |
| 1.0 (≈1.099) | 0 / 500 / 0.3140 | 0 / 275 / 0.4050 | 0 / 216 / 0.1950 |
| 0.25 (≈0.275) | 129 / 369 / 0.3500 | 0 / 275 / 0.4050 | 0 / 216 / 0.1950 |
| 0.10 (≈0.110) | 428 / 70 / 0.3860 | 14 / 252 / 0.4050 | 0 / 216 / 0.1950 |
| 0.05 (≈0.055) | 483 / 11 / 0.3940 | 84 / 185 / 0.4050 | 39 / 209 / 0.1967 |

Monotone and damning: **every hold that actually holds costs accuracy
proportional to its disagreements** (banking77: −8.0pp at 500
disagreements, −4.4pp at 369, −0.8pp at 70, 0.0 at 11), and the
no-cost points buy no flip-reduction. The best cell on the board
(sst5 +0.15pp at t=0.05 with 209 disagreements) is inside one
question's granularity (1/600) — noise, and it still held through 209
signal-churn routes. There is no margin on this seat that reduces
flips without proportionally reducing accuracy: the churn it suppresses
is the routing being right.

## Latency lane (G2-bar unchanged — the one leg that passed)

`cargo bench --bench decision_set_goat --features pick_latch` (unarmed
latch = the shipped hot path; PASSED preflight posture above):
**G2 PASS — p99 69 µs ≤ 1000 µs · G4 PASS — core alloc-free, canary
armed.** The armed path's in-run per-suite p50s moved bare
0.073–0.372 ms → armed 0.075–0.377 ms (one extra cosine fold + gate
eval per decision) — same order, irrelevant: the posture never ships.

## Verdict + re-open conditions

**G3 FAIL. No promotion. `pick_latch` stays opt-in forever as a
measured-negative feature** (the `gw_corpus` posture — kept for
replication, the census + flags remain the instrument). The fold/G1/G2
craft stays landed and correct; the negative is the CELL's, not the
code's.

Re-open only on a NEW stream shape, not a re-tune: a serving stream
where adjacency correlates with domain (one user session marching one
corpus — the shape the 987 premise actually describes). The harness
stream cannot express it; a serving-lane capture could. A temperature
re-sweep alone is closed by the table above (monotone tradeoff, no
free point).

## Housekeeping recorded here

- `.benchmarks/.highwater` repaired 101 → 176: it was stale two writers
  deep (Bench 174/175 landed by `968c906`/earlier without the bump;
  `617_openthai_exl3_4090` is a plan-named raw home, not an allocation
  — its own header says so). Allocation-time gate:
  `dual_allocation_gate riir-reflex: 0 twin, 0 independent, 0
  counter-only colliding number(s)`.
- The route-census instrument (`--route-census`, report-only) is the
  cell's legacy: the seat's flip/disagreement counters stay available
  for any future routing measurement.
- The T2 hysteresis gate's stream was measured VACUOUS during this cell
  (0 ≤ 0 — the bare argmax never flipped on the original single-base
  stream): rebuilt on a two-domain tilt stream, non-vacuity asserted
  (landed `3254602`, same commit as the census).
