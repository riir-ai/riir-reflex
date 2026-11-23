# Bench 205 — issue 126 P3: the semantic_defects engine lever measured NEGATIVE; the A2 small-n parity clause lands as code

**Status:** RECORD — 2026-11-23 (zed-c499940c@amo, successor of the zed-a029893e@amo claim). Issue 126's one open half (the semantic_defects "engine lever", P3) is REFUTED by measurement and closed by delegated-owner verdict `060599b8` (round 1 AGREE-with-changes, round 2 confirmed). The A2 closing rule gains a small-n parity clause, COMPUTED by the floor lane on every row: **semantic_defects = `FLOOR-PARITY@102`** (never a PASS), ag_news = `PASS` (Bench 204's closure, unaffected).

## The pre-registered lever and why it fired

Issue 126's design, pre-recorded before the corpus growth: if the gap persisted, "(a) idf-weight the hashed-lexicon halves, OR (b) blend a BM25-shaped fallback vote for tiny corpora; GOAT-gated". The c80f8d28 re-sweep left engine **0.5686** (58/102) vs bm25 **0.5784** (59/102), so the lever fired. This record is its measurement.

## Measurement 1 — the lever's premise is false (the real binary)

The "hashed-lexicon halves" are the unigram (weight 1.0) and bigram (weight 0.5) hashes of a 256-bucket signed embedding (`src/embed.rs`); the engine consumes them ONLY through the centroid-cosine route term. With nb/ridge unarmed the engine IS that term (+ the near-constant drafter):

```
./target/release/harness --skip-laya --suites semantic_defects --bm25-floor --dump-items --out /tmp/i126p3/route_only
```

- **route-only forced accuracy 0.1961 (20/102)** — against the 6-way chance floor 0.1667 (17/102). The harness's own headline in that run (abstain-aware) is 0.1961 with 44/102 raw abstains.
- The bm25 floor in the same run reproduced the sweep exactly: **0.5784** (72 docs, 0/102 empty votes) — the instrument is valid.
- So the engine's whole 0.5686 is carried by the NB + ridge wide-vocabulary terms. Lever (a) would re-weight a channel that carries no class signal on this suite; it cannot recover anything. (`src/nb_scope.rs` already documents that the 256-bucket embed collides; the suite's eval text is authored 3-gram-fresh from the corpus.)

Run identity: binary at HEAD `33831c2`, run line `33831c2` on amo 2026-11-23T10:18:37Z, AC, load 3.18 — accuracy is deterministic; no latency claim is made here.

## Measurement 2 — doc-level voting in the same space is a wash (offline replica)

Substrate-first found the primitive already shipped: `katgpt_core::distance_abstain::CorpusDistanceGate::max_similarity` (zero-alloc nearest-exemplar cosine; every `DomainExpert` holds one over its own docs). A nearest-exemplar vote beside the centroid was probed BEFORE any engine code was written, with a float64 Python replica of `src/embed.rs` (throwaway, `/tmp/i126p3/probe.py`, deleted at teardown). Eval n=102:

| signal | forced acc |
|---|---|
| centroid-argmax (the route term's own argmax) | 0.2843 |
| nearest-exemplar-argmax (`max_similarity`) | 0.2843 |
| top-3-mean exemplar | 0.2647 |
| best centroid + λ·exemplar blend (λ ∈ {0.25…4}) | 0.3137 |
| the real engine (nb-select + ridge-select) | 0.5686 |

Rows where centroid-argmax and exemplar-argmax disagree: 68 — gold gained 17, gold lost 17. Exactly a wash. (The replica is an ORDERING read, validated in direction by the binary's own 0.1961; it is not a GOAT read.)

## Measurement 3 — the instrument cannot resolve the gap

Paired read at the sweep posture: `b = 11` (bm25-right/engine-wrong), `c = 10` (engine-right/bm25-wrong), both-right 48, both-wrong 33.

- exact two-sided McNemar **p = 1.000**;
- Δ(engine − bm25) = **−0.98 pp**, Wald 95% CI **[−9.8, +7.8] pp**;
- the cal front (the only slice a knob may be selected on, the harness no-cheat law) is **18 cases** (per class 3, 5, 3, 2, 3, 2) — one case = 5.6 pp.

Lever (b) — a wide-vocabulary BM25-shaped vote strong enough to move this suite — is the floor itself: "engine ≥ bm25" would then hold by construction and the A2 row would stop measuring anything (the issue-121 premise). Refused as circular.

## The verdict (delegated owner, session `060599b8`, 2026-11-23 — recorded per the 2026-11-11 gating law; no credential touch)

**AGREE with Option A, three changes.** (1) `p ≥ 0.05` alone is absence of evidence — add a point-deficit cap and restrict the clause to small suites; (2) the verdict is a separate label, `FLOOR-PARITY@n` — it retires the CORPUS-DEFECT finding and never counts as a PASS; (3) the parity verdict expires on any change. The reviewer's gate-test vectors are the unit tests below.

Sign-convention note: the verdict's example row printed Δ = −0.98pp (engine − bm25) beside the interval [−7.8, +9.8] (bm25 − engine, this record's Measurement 3). The clause's formula is authoritative and symmetric: on (engine − bm25) the interval is **[−9.8, +7.8] pp** — what the code computes.

## The clause, as landed (`src/bm25_floor.rs` — the floor's ONE home)

> A2 small-n parity clause: a suite with `engine < bm25` on a fresh two-run byte-identical record closes as **FLOOR-PARITY@n** (not PASS) iff ALL hold: (i) `n_eval < 400`; (ii) exact two-sided McNemar p ≥ 0.05, the conditional binomial on the discordant pairs (`b = bm25-right/engine-wrong`, `c = engine-right/bm25-wrong`), `p = min(1, 2·P(X ≤ min(b, c)))`, `X ~ Bin(b+c, ½)`, and 1 when `b + c = 0`; (iii) the point deficit `(b − c)/n ≤ 2.0 pp`. Otherwise the CORPUS-DEFECT finding stands. At `n_eval ≥ 400` strict `engine ≥ bm25` applies unchanged.

- `floor_parity(engine_rows, floor_rows)` joins the two lanes' forced rows (the same `(gold, probs)` shape and question order) into `FloorParity` — `n`, `b`, `c`, both-right, both-wrong, `mcnemar_p`, `delta_pp`, `ci95_pp` (Wald `±1.96·√(b + c − (b − c)²/n)/n`) and the `FloorVerdict`. A mis-joined read (length or gold mismatch) REFUSES loud; picks follow the first-index argmax of `hard_metrics` (`metrics::argmax_first`, now `pub(crate)` — one tie law).
- `mcnemar_exact_two_sided` sums the conditional binomial in log space (`ln C` recurrence + log-add) — `C(300, 150)` ≈ 1e88 overflows nothing.
- `Bm25FloorReport.parity: Option<FloorParity>` — additive wire, skip-when-None (absent on a typed n/a row); the floor row in `TABLES.md` carries an **`A2 verdict:`** line with every disclosure the clause requires. No exit-code change.
- Gate tests (`bm25_floor::tests`, 9): the closed case (b=11, c=10, n=102 → FLOOR-PARITY@102, p=1.000, table reconciles to 58/59); `b = c = 0` (p = 1, CI `[0, 0]`, NaN-free); (9, 1) → p ≈ 0.0215 CORPUS-DEFECT; (3, 0) → p = 0.25 but the 2.94 pp deficit fires the cap on its own → CORPUS-DEFECT; (11, 10) at n = 400 and n = 409 → CORPUS-DEFECT (the clause does not apply); engine above the floor → PASS whatever p; the mis-join refusals; the exact tail at 300 discordant pairs; the label strings + the three constants pinned.

## The landing record (two runs, the code's own verdict lines)

```
cargo run --release --bin harness -- --skip-laya --suites ag_news,semantic_defects \
  --bm25-floor --nb-select --ridge-select --dump-items --out <dir>   # ×2
```

Binary built from the landing tree (base `c4c09c1` + this change; isolated `CARGO_TARGET_DIR`), amo, AC, load 3.79→3.64 / 3.46→3.24, 2026-11-23T10:34Z. Accuracy is deterministic — no latency claim is made.

| suite | A2 verdict | engine | bm25 | n | b | c | both-right | both-wrong | p | Δ (engine−bm25) | 95% CI |
|---|---|---|---|---|---|---|---|---|---|---|---|
| ag_news | **PASS** | 0.9200 | 0.9050 | 400 | 13 | 19 | 349 | 19 | 0.377 | +1.50 pp | [−1.3, +4.3] pp |
| semantic_defects | **FLOOR-PARITY@102** | 0.5686 | 0.5784 | 102 | 11 | 10 | 48 | 33 | 1.000 | −0.98 pp | [−9.8, +7.8] pp |

Two-run identity: both item dumps byte-identical across the runs (ag_news `4a5276fa…`, semantic_defects `87b21bbd…`); the parity blocks byte-identical (`e5823bb2…` both runs); the runs differ only in timestamp, the box-state line and p99 wall-clock (0.198 vs 0.199 ms; 0.039 vs 0.036 ms). ag_news's engine and floor numbers reproduce Bench 204 exactly.

Honest disclosures: the offline replica read eval rows for the hashed-space lever family for DIAGNOSIS only — it selected no posture and nothing is promoted off it; the route-only harness run is likewise diagnostic. semantic_defects is not on the instinct serving board and the serve posture keeps nb/ridge at scale 0, so no served lane changes.

## Re-open triggers (recorded in issue 126; any one fires a re-adjudication)

- the eval set grows to ≥ 400 rows, or the cal front to ≥ 60 cases (≥ 10 per class) — either makes knob selection and the strict A2 rule meaningful;
- a later two-run sweep breaks the clause (deficit > 2.0 pp at n = 102, i.e. ≥ 3 rows; or p < 0.05 with the engine behind);
- semantic_defects gains a served consumer (instinct board, or any serve posture with nb/ridge scale > 0), or a new tiny-corpus consumer appears (≤ ~20 docs per class);
- any change to the engine's scoring for route, NB or ridge.

## What this bench is NOT

- No engine change. The route term, NB, ridge and the embedder are untouched.
- Not a PASS for semantic_defects and not a GOAT claim: FLOOR-PARITY retires a finding; it does not assert the engine beats the floor there.
- The verdict's optional follow-up — a route term that self-gates on corpus separability (leave-one-out centroid accuracy over the corpus docs alone) — is filed as its own issue after a substrate-first check; it does not re-open 126 unless a trigger above fires.

## Provenance

- Verdict: delegated-owner, session `060599b8` (round 1 AGREE with three changes; round 2 confirmed + the gate-test vectors) — recorded in issue 126.
- The gap's origin: Bench 200 (standing sweep) → Bench 202 (per-suite audit) → the c80f8d28 re-sweep → Bench 204 (ag_news closed).
- Gates at the landing: `cargo test --lib bm25_floor` 20/20 (9 new); `cargo clippy --all-targets -- -D warnings` clean; per-file rustfmt clean.

## ADDENDUM — issue 121's closing standing sweep (2026-11-23, two runs, the clause live on every row)

The record issue 121 waited for: Bench 200's six suites **plus `typed_decisions` re-included** (its issue-125 panic class is now a typed n/a row). Same lane as Bench 200, from the landed binary:

```
harness --skip-laya --suites ag_news,emotion,xnli_en,massive_intent_en,banking77,semantic_defects,typed_decisions \
  --bm25-floor --nb-select --ridge-select --out <dir>   # ×2
```

Run line `4af3db8` on amo, 2026-11-23T10:41Z. ⛔ Latency NOT QUOTABLE (the harness's own box-state gate: a sibling job held the load at 16.67 > 6) — accuracy is deterministic and no latency number is cited. Two-run identity: the verdict table below is byte-identical across the runs (`fb571bc2…`, over suite · verdict · n · b · c · both-right · both-wrong · p · Δ · CI · both lanes' accuracies).

| suite | A2 verdict | engine | bm25 | n | b | c | both-right | both-wrong | McNemar p | Δ (engine−bm25) | 95% CI |
|---|---|---|---|---|---|---|---|---|---|---|---|
| ag_news | **PASS** | 0.9200 | 0.9050 | 400 | 13 | 19 | 349 | 19 | 0.377 | +1.50 pp | [−1.3, +4.3] pp |
| emotion | **PASS** | 0.8850 | 0.6000 | 400 | 8 | 122 | 232 | 38 | <0.001 | +28.50 pp | [+23.7, +33.3] pp |
| xnli_en | **PASS** | 0.5233 | 0.3867 | 300 | 44 | 85 | 72 | 99 | <0.001 | +13.67 pp | [+6.4, +20.9] pp |
| banking77 | **PASS** | 0.8420 | 0.8080 | 500 | 24 | 41 | 380 | 55 | 0.046 | +3.40 pp | [+0.3, +6.5] pp |
| semantic_defects | **FLOOR-PARITY@102** | 0.5686 | 0.5784 | 102 | 11 | 10 | 48 | 33 | 1.000 | −0.98 pp | [−9.8, +7.8] pp |
| typed_decisions | n/a — typed row (issue 125) | 0.2940 | — | | | | | | | | |
| massive_intent_en | n/a — 20-of-59 per-question option space | 0.7800 | — | | | | | | | | |

**No CORPUS-DEFECT finding remains on the standing sweep.** Both typed n/a rows went through the new code path without a parity field and without a panic (the n/a path carries `parity: None` by construction — the 5 `A2 verdict:` lines are exactly the 5 real floor rows). banking77 is the closest strict PASS (p = 0.046 on the engine's side, CI [+0.3, +6.5] pp) — a strict pass, not a parity one: at n = 500 the clause does not apply and was not needed.

**Against Bench 200 (read with one coverage change):** the engine reproduces it exactly on emotion (0.8850), xnli_en (0.5233), massive_intent_en (0.7800) and banking77 (0.8420), and the floor on emotion (0.6000), xnli_en (0.3867) and banking77 (0.8080); ag_news (0.8825/0.8875 → 0.9200/0.9050) and semantic_defects (0.4412/0.4706 → 0.5686/0.5784) moved for the recorded reasons (the Bench 204 overlay, applied to BOTH lanes; the issue-126 corpus growth). **`massive_intent_en` is a coverage change, not a reproduction:** Bench 200 carried a floor row for it (0.4667, n = 300) because it ran BEFORE the issue-125 label-vote contract gate (`ee33ce5`, 2026-11-20); the gate now types the suite n/a — its questions are 20-option subsets of a 59-label universe, outside the floor's contract (the pre-gate row was scored by the degenerate path the gate exists to refuse, so it is superseded, not comparable). **The standing floor therefore covers five suites**; `massive_intent_en` and `typed_decisions` are typed n/a rows with no A2 verdict — a disclosed gap in the floor's coverage, not a pass.
