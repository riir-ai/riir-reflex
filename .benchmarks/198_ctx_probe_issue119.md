# Bench 198 — the context-invariance probe's pre-registered cell (issue 119)

**Status:** RECORD — G3 FIRED (the null path, as the issue's honest expectation predicted). Issue 119 closed scoped.
**Lane:** `harness --ctx-probe` (report-only; T1–T4 landed this commit — `src/harness/context_transform.rs` + `src/harness/runner/ctx_probe.rs` + the bin mode).
**PROVENANCE:** `power=AC Power load=2.68 swap=0.00M canary=skipped powermode=2(high)` — preflight PASSED (m3, AC, High Power mode disclosed).
**Run:** 2026-11-15T17:59:16Z · sha `a215e78` · host m3 · datasets `.raw/datasets` (canonical re-baselined seat).

## Posture

- 7 suites measured (the registered modelless-eligible set at n ≥ 100: typed_decisions, ag_news, emotion, xnli_en, massive_intent_en, banking77, semantic_defects); zero exclusions; ≤ 40 stride-sampled choice slots/suite; K primary 4 of the one 8-transform decision set (the K=2/8 sweep is a prefix — zero extra decides).
- Every arm answers through the production decide path (`engine_request` → `decide_with` → `answer_probs_pick`, the perm probe's exact oracle body + the fused-slot read); the fitted plain posture is the perm probe's own (`fit_posture_inner` + `build_engine_with`, every selection lever pinned off, oc/nb arming refused loud).
- **G1 identity control: HOLDS** in every suite — the identity transform reproduces the raw decide byte-identically (probs bits + pick + the fused-gate abstain flag), both families.
- The fused gate is the identity decide's slot verdict (the production input). On this seat the score-gate abstains heavily on several suites (ag_news 8/8 smoke, typed_decisions 60/60 in-run — cause `score_gate`, mean confidence ≈ chance on ag_news 0.2585): the matched-coverage cells name n=0 where the pairing is empty, never a fabricated number.

## The pre-registered verdict (K=4, semantic family, pooled with suite fixed effects)

| cell | suites | cases | mean pp | SE pp | LB95 pp |
|---|---|---|---|---|---|
| g2a_sem_plac (both seats answer) | 7 | 241 | −1.03 | 1.17 | **−2.95** |
| g2a_sem_fused (matched coverage) | 1 | 13 | 0.00 | 0.00 | 0.00 |
| g2b_channel_sem | 1 | 13 | 0.00 | 0.00 | 0.00 |
| g2b_channel_plac | 1 | 13 | −7.69 | 7.69 | −20.35 |

- **G2a NEGATIVE** — both bounds ≤ 0. **G2b NEGATIVE** — the placebo channel arm is worse (−7.69 mean: the placebo family's empty cores remove answered-correct cases the semantic family keeps — noise abstention, not invariance signal).
- **G3 FIRED** — the null path. **Scope of the close (the issue's own law): this run table, on THIS engine (a bag-of-hashed-words additive-feature engine), for the tested families — not the class.** Named re-open trigger (rides the record): an engine with order- or context-sensitive features (position-aware n-grams, attention, structure beyond one bigram), where transforms stop being additive.
- No promotion (G4 does not open); no fused-gate extension plan.

## The mechanism read (why the null is the expected shape)

The census is the paper's confound made visible: semantic and placebo agreement structures are near-identical per suite (ag_news semantic 33/6/1 unanimous/majority/no-core vs placebo 33/1/6; typed_decisions 60/60 unanimous in BOTH — the engine's picks are essentially invariant to ANY additive transform, persona or noise). A persona prefix is a constant additive bucket set — the same mass as word-count-matched nonsense — so "agreement under persona transforms" measures additive hash noise wearing a persona. Exactly the verdict round 1 caveat and the issue's honest expectation: the probe measured hash noise wearing a persona; the class is bounded here, not refuted everywhere.

## Artifacts

- `198_ctx_probe_issue119/ctx_probe.json` — the full record (meta rules quoted verbatim from the pre-registration; per-case rows, families, cells, pooled verdict).
- `198_ctx_probe_issue119/CTX_PROBE.md` — the run's own markdown (census + cells + pooled verdict).
- Machinery: `src/harness/context_transform.rs` (8 tests) + `src/harness/runner/ctx_probe.rs` (2 tests) — 693 lib tests green; clippy clean.
