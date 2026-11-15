# Ctx probe — additive context-transform agreement (issue 119)

- date 2026-11-15T17:59:16Z · sha a215e78 · host m3
- datasets /Users/kat/git/riir-reflex/.raw/datasets
- K primary 4 of the 8-transform set × ≤ 40 case(s)/suite
- suite set: the registered modelless-eligible dataset suites with n >= 100 at run time (the Bench-092 gate-6 rule); every exclusion named; --suites overrides (disclosed)
- seat: the strict-majority core over the K-primary prefix (first-appearance tie order); empty core = the seat abstains; unanimous disclosed beside; the K sweep (2/8) is a PREFIX of one 8-transform decision set — zero extra decides
- fused: the fused gate IS the identity transform's decide (the production input): answered+pick or abstained, read from the engine slot verdicts
- verdict: G2a passes iff BOTH pooled paired LB95s > 0 (semantic-placebo on the both-answer pairing; semantic-fused at matched coverage); G2b the same on its pair, scored on the fused-answered set with the extra abstention scoring 0; LB95 = mean - 1.6449·SE, pooled with suite fixed effects; G3 (the null path) fires when G2a fails BOTH bounds — the negative is scoped to this run table, THIS engine (bag-of-hashed-words additive features), the tested families
- route: by-name + index-anchored (issue 079)

**G1 identity control: HOLDS**

## Family agreement census (K=4 primary; the sweep rungs beside)

| suite | family | n | unanimous | majority | no-core | no-core@K2 | no-core@K8 |
|---|---|---|---|---|---|---|---|
| typed_decisions | semantic | 60 | 60 | 0 | 0 | 0.000 | 0.000 |
| typed_decisions | placebo | 60 | 60 | 0 | 0 | 0.000 | 0.000 |
| ag_news | semantic | 40 | 33 | 6 | 1 | 0.025 | 0.025 |
| ag_news | placebo | 40 | 33 | 1 | 6 | 0.175 | 0.050 |
| emotion | semantic | 40 | 25 | 14 | 1 | 0.325 | 0.100 |
| emotion | placebo | 40 | 21 | 9 | 10 | 0.375 | 0.025 |
| xnli_en | semantic | 38 | 36 | 1 | 1 | 0.026 | 0.026 |
| xnli_en | placebo | 38 | 36 | 1 | 1 | 0.026 | 0.000 |
| massive_intent_en | semantic | 38 | 24 | 10 | 4 | 0.158 | 0.132 |
| massive_intent_en | placebo | 38 | 22 | 10 | 6 | 0.211 | 0.105 |
| banking77 | semantic | 39 | 14 | 14 | 11 | 0.282 | 0.308 |
| banking77 | placebo | 39 | 18 | 14 | 7 | 0.359 | 0.077 |
| semantic_defects | semantic | 34 | 28 | 4 | 2 | 0.059 | 0.059 |
| semantic_defects | placebo | 34 | 22 | 7 | 5 | 0.206 | 0.088 |

## Paired cells (per suite, percentage points)

| suite | cell | n | mean | SE | LB95 |
|---|---|---|---|---|---|
| typed_decisions | g2a_sem_plac | 60 | 0.00 | 0.00 | 0.00 |
| typed_decisions | g2a_sem_fused | 0 | — | — | — |
| typed_decisions | g2b_channel_sem | 0 | — | — | — |
| typed_decisions | g2b_channel_plac | 0 | — | — | — |
| ag_news | g2a_sem_plac | 33 | -3.03 | 3.03 | -8.01 |
| ag_news | g2a_sem_fused | 0 | — | — | — |
| ag_news | g2b_channel_sem | 0 | — | — | — |
| ag_news | g2b_channel_plac | 0 | — | — | — |
| emotion | g2a_sem_plac | 30 | 3.33 | 5.84 | -6.27 |
| emotion | g2a_sem_fused | 0 | — | — | — |
| emotion | g2b_channel_sem | 0 | — | — | — |
| emotion | g2b_channel_plac | 0 | — | — | — |
| xnli_en | g2a_sem_plac | 36 | 0.00 | 0.00 | 0.00 |
| xnli_en | g2a_sem_fused | 0 | — | — | — |
| xnli_en | g2b_channel_sem | 0 | — | — | — |
| xnli_en | g2b_channel_plac | 0 | — | — | — |
| massive_intent_en | g2a_sem_plac | 30 | -3.33 | 5.84 | -12.94 |
| massive_intent_en | g2a_sem_fused | 0 | — | — | — |
| massive_intent_en | g2b_channel_sem | 0 | — | — | — |
| massive_intent_en | g2b_channel_plac | 0 | — | — | — |
| banking77 | g2a_sem_plac | 24 | -4.17 | 4.17 | -11.02 |
| banking77 | g2a_sem_fused | 0 | — | — | — |
| banking77 | g2b_channel_sem | 0 | — | — | — |
| banking77 | g2b_channel_plac | 0 | — | — | — |
| semantic_defects | g2a_sem_plac | 28 | 0.00 | 0.00 | 0.00 |
| semantic_defects | g2a_sem_fused | 13 | 0.00 | 0.00 | 0.00 |
| semantic_defects | g2b_channel_sem | 13 | 0.00 | 0.00 | 0.00 |
| semantic_defects | g2b_channel_plac | 13 | -7.69 | 7.69 | -20.35 |

## Pooled verdict (suite fixed effects)

| cell | suites | cases | mean | SE | LB95 |
|---|---|---|---|---|---|
| g2a_sem_plac | 7 | 241 | -1.03 | 1.17 | -2.95 |
| g2a_sem_fused | 1 | 13 | 0.00 | 0.00 | 0.00 |
| g2b_channel_sem | 1 | 13 | 0.00 | 0.00 | 0.00 |
| g2b_channel_plac | 1 | 13 | -7.69 | 7.69 | -20.35 |

**G2a:** NEGATIVE (G3) — both pooled bounds <= 0: the additive-transform class is bounded on this engine, this run table, these families

**G2b:** NEGATIVE — the empty-core channel shows no placebo-separated lift

**G3 (null path): FIRED — the scoped negative is the record's verdict**
