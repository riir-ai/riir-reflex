# Bench 175 — Issue 112 T4: the `--gate-risk-target` report-only run (one frozen read, α=0.10, δ=0.05)

**Date:** 2026-11-08 · **Box:** m3 (Apple M3 Max, 16 cores) · **Landing:** reflex `c95fa10` (the lever) + this record · **Claim:** zed-m3-i112-ltt (Issue 112)

## Box state

`scripts/bench_preflight.sh` REFUSED for latency work: `PROVENANCE: power=AC Power load=11.64 swap=540.44M canary=skipped powermode=2(high)` — a sibling session was compiling on the box. **No latency number is claimed here.** The read is an accuracy/certificate read: deterministic computation over fixed data, load-insensitive by construction, and the run's own repeat-run bit-identity check passed on every suite (`determinism_ok=true`, n=10 repeat pairs each — 8/8 suites). The latency columns in `results.json` are the loaded box's and are not the claim.

## The frozen read (pre-registered in `.issues/112` BEFORE the run)

Command: `harness --gate-risk-target 0.10 --skip-laya --suites ag_news,banking77,emotion,massive_intent_en,prompt_injections,sst5,typed_decisions,xnli_en` — the 8 board suites (≤ 10, the pre-registered budget window), modelless lane only (the certificate is a modelless-gate surface), default posture (`gate_fit_calibrated` on — the certificate reads the calibrated scale; the fit and the realization share one observation surface and one per-case convention: last slot per case, noul wire flip).

| suite | fit | cal n (floor 29) | test | breach |
|---|---|---|---|---|
| typed_decisions | not certified | 100 | — | — |
| ag_news | not certified | 200 | — | — |
| emotion | not certified | 200 | — | — |
| sst5 | not certified | 200 | — | — |
| prompt_injections | not certified | 100 | — | — |
| xnli_en | not certified | 200 | — | — |
| massive_intent_en | not certified | 200 | — | — |
| banking77 | not certified | 200 | — | — |

**0 certified, 0 breaches — breach budget clean; c3 skipped loud (no certified suite with a following suite's test rows).** Artifacts: `175_issue112_risk_target_t4/{risk_target.json,RISK_TARGET.md,results.json,TABLES.md}`.

## The read against the pre-registered gates

- **(a) CERTIFICATE:** vacuous-clean — zero coverage means zero suite gate rows and zero breach opportunities. The budget line (≥2 LOWER-end breaches on ≤10 suites) did not fire. The posture is NOT wrong by this gate.
- **(c1)/(c2) FALSIFICATION:** not re-run here — they live as unit-level arms on the engine (landed with T1/T2: `shuffled_labels_certify_nothing_below_the_base_rate`, `planted_violation_does_not_certify`), passing at this commit.
- **(c3) DISTRIBUTION-SHIFT:** skipped in the frozen read (no coverage); fired for real in the diagnostic probe below — recorded, never gated.

**T6 verdict: the run MATCHES the pre-registered expectation.** The issue's own words: "strict α certifies ~nothing at our cal-slice sizes — the loud 'none certified' is the EXPECTED outcome at our group counts, never a failure." The expected-outcome branch is what landed; the issue does NOT close negative.

## Why nothing certifies at α=0.10 (the mechanism, not a defect)

Certification requires the strictest ADMISSIBLE candidate (the first grid point where accepted n ≥ floor = 29) to pass CP at level δ. With ONE accepted error the bound is cp_upper(1, 29, 0.05) ≈ 0.169 > 0.10 — the scan stops there (the T1 fixed-sequence stop law, by design: "a slice dirty there certifies nothing even when a looser large-n cutoff would pass"). A zero-error 29-front passes only by the bare margin 1 − 0.05^(1/29) ≈ 0.0981 ≤ 0.10. So α=0.10 certification demands a PERFECT top-29 cal front — and sigmoid calibration compresses/saturates the score scale (banking77's calibrated readout saturates; ag_news's top-confidence band carries errors at these slice sizes), so every suite's strictest admissible front was dirty. This is the honest disclosure the issue bought: the certificate refuses loudly at sample sizes that cannot support it.

## Diagnostic probe (NOT the frozen read — a wiring sanity arm, α=0.30, 4 suites)

`harness --gate-risk-target 0.30 --skip-laya --suites ag_news,emotion,massive_intent_en,banking77` → `diagnostic_alpha030/`:

- **massive_intent_en CERTIFIED** — cutoff 0.834, cal 200/16 accepted/1 error (CP ≤ 0.30), floor 9. Test realization: 41 accepted / 3 errors → rate 0.0732, two-sided exact CP CI [0.0154, 0.1992], **LOWER 0.0154 ≤ 0.30 → no breach** — the certificate held on real data end-to-end (fit → realization → interval → breach read).
- The other 3 suites still refuse at α=0.30 (their strictest admissible fronts are dirty even at floor 9) — the fixed-sequence stop law again, expected.
- **c3 fired for real:** massive_intent_en's certified cutoff applied to banking77's test rows — 2 accepted, 1 error, rate 0.50, CI [0.0126, 0.9874] — the exchangeability-broken row reads exactly as it should (small accepted set, wide interval, no gate).

The probe proves the wiring: the refusal arms (c1/c2 unit-level) and the positive arm (real certification + holding interval) are both demonstrated. Not-certified is a property of the data at the target, never a silent pass.

## Byte-identity (lever-off law)

Pre-change binary (`target/release/harness`, built Nov 8 09:11 — predating BOTH `e3dd8dd` (T1/T2/T3b) and `c95fa10` (T4)) vs the T4 binary, ag_news lever-off: `results.json` identical after stripping timing fields, differing ONLY in the meta rows the old build's laya feature carried (`laya_device`/`laya_feature` — a build-posture disclosure, not a lane output). Headline metrics byte-identical: acc 0.4050, ece 0.1465, and the fitted thresholds/recommendation unchanged. The default path is byte-identical across all three Issue-112 landings. (The ag_news 0.4050 headline is this harness's default posture on the canonical datasets — the board's 0.8975 is the arena's tuned seat posture in riir-instinct, a different surface; the pre-change binary agrees with the new one, which is the claim this section makes.)

## Verdicts

- T4 lever + run + gates: **PASS** (the expected-vacuous kind — the loud none-certified disclosure, exactly what T6 pre-registered).
- T5 wire law: **PASS** (lever-off serializes no `risk_target` key — pinned by `lever_off_emits_no_rows_and_the_wire_stays_clean`; the byte-identity A/B confirms no default-run emission).
- T6 expectations: **RECORDED — no negative branch fired.**
- G4: n/a (fit-time only; the record path allocates behind the opt-in lever).
- Remaining: serving wiring is owner-gated (the issue's own law); the T3b retirement rides the katgpt-rs Issue 989 export.
