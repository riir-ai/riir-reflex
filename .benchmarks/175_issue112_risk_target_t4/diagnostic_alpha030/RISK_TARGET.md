## Issue 112 T4 — risk-target certificate report (report-only)

alpha 0.3 · delta 0.05 (pinned v1) · realized-error interval two-sided exact CP at delta/2 per tail · breach = LOWER end > alpha · score-axis certificate (the fused gate's served set also folds the distance axis) · group unit = the cal case (template families would count once — none in these slices today)

| suite | fit | cutoff | cal n/accepted/errors | floor | test accepted/errors | err rate | 95% CP CI | breach |
|---|---|---|---|---|---|---|---|---|
| ag_news | not certified | — | 200/—/— | 9 | — | — | — | — |
| emotion | not certified | — | 200/—/— | 9 | — | — | — | — |
| massive_intent_en | certified | 0.834 | 200/16/1 | 9 | 41/3 | 0.0732 | [0.0154, 0.1992] | no |
| banking77 | not certified | — | 200/—/— | 9 | — | — | — | — |

**Breach budget (T4a):** breach budget clean — 0 breaches on 1 certified suite(s) (pre-registered line: >=2 LOWER-end breaches on <=10 suites = posture wrong)
**c3 distribution-shift row (report-only):** certified cutoff 0.834 (massive_intent_en cal) applied to banking77 test — accepted 2 (errors 1), rate 0.5000, CI [0.0126, 0.9874] — the guarantee's exchangeability premise is broken here by construction; recorded, never gated.
