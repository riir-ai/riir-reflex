# Phase-1 harness tables (CI-regenerated — Plan 603 T1.5)

- run: `4046f8f` on `m3` (2026-11-10T00:48:11Z) · profile release · laya feature true
- box state (Issue 021): start power=AC Power mode=high load=4.59 swap=532M · end power=AC Power mode=high load=1.92 swap=532M — latency QUOTABLE
- laya device posture: metal (LAYA_DEVICE or the build's macOS default)
- laya-python lane: off (pass --laya-python to add the reference lane)
- clm lane: off (pass --clm to add the comparison lane; .issues/027)
- gliner lane: off (pass --gliner to add the comparison lane; needs the gliner2 venv — .issues/029)
- bekko lane: off (pass --bekko to add the comparison lane; needs the bekko venv — Bench 103)
- agentjev lane: off (pass --agentjev to add the comparison lane; needs their jev_service on AGENTJEV_SERVE_URL — .issues/025)
- clef lane: off (pass --clef to add the comparison lane; the hosted posture needs the owner's Workers AI creds behind the loopback forwarder, or serve the open weights locally — plan 011 Phase A + .research/007)
- drex lane: off (pass --drex to add the comparison lane; needs their service on DREX_SERVE_URL, default http://127.0.0.1:8000 — Issue 073)
- d1 lane: off (pass --d1 to add the comparison lane; needs the reference server on D1_SERVE_URL, default http://127.0.0.1:8078 — Issue 078)
- pplx lane: off (pass --pplx to add the comparison lane; serve their autojev server on PPLX_SERVE_URL, default http://127.0.0.1:8793 — Issue 082)
- d3 lane: off (pass --d3 to add the comparison lane; serve their d3_server.py on D3_SERVE_URL, default http://127.0.0.1:8794 — riir-train 629 T2)
- startlux lane: off (pass --startlux to add the comparison lane; serve their startlux-decide engine on STARTLUX_SERVE_URL, default http://127.0.0.1:8090 — reflex Issue 095, riir-train 635 T3)
- mammoth lane: off (pass --mammoth to add the hosted comparison lane; run the loopback TLS forwarder to inference.baseten.co on MAMMOTH_SERVE_URL, default http://127.0.0.1:8793, with MAMMOTH_API_TOKEN set — rethink Issue 032 T10; T10.1: the key is an OWNER act; window ends 2026-10-27 23:59 PT)
- elephant lane: off (pass --elephant to add the self-hosted comparison lane; serve our scripts/elephant_server.py under the qev venv on ELEPHANT_SERVE_URL, default http://127.0.0.1:8096 — rethink Issue 032 T10.2, riir-train 640 T2(a) records the exact venv + weight pin)
- paw lane: off (pass --paw to add the comparison lane; PAW_API_KEY optional — .issues/033)
- paw-local lane: off (pass --paw-local to add the local-runtime twin; needs the programasweights venv + a hosted-lane cache — .issues/033)
- corpus cap posture: registry defaults
- label heads: OFF (head_scale 0 — the published baseline posture)
- count tables (issue 038): OFF (nb_scale 0 — the published baseline posture)
- option-conditioned tables (issue 038 T7b): 
- NBSVM ridge readout (issue 038 T7a): 
- corpus: one engine domain per label; corpus = train-split docs of that label, capped per label (registry), self-doc fallback for labels absent from the fetched train rows 
- calibration: sigmoid-gate fit on a train-tail calibration slice (same builder over the train rows); fused-gate thresholds fitted per suite at the cal-slice 30th percentile (the T1.6 arena posture rho=30%; the birth constants do not transfer — measured: 100% abstain on several suites at defaults); raw-vs-calibrated readout ECE compared per the G1 gate; no calibration claim when the calibrator never moved
- sampling: test sample = label-STRATIFIED round-robin over the whole test split (budget = the registry test cap; first-appearance label order, dataset order within each label, deterministic, no RNG); cal slice = the same law over the train rows; corpus pool = the train rows MINUS the cal front (excluded by construction, not position); budget-0 suites unchanged (identity split) — Issue 039 T2
- readout: shipped Dispatch law everywhere (Issue 039 T4 DEMOTED: the cal-side arming lever overfit the narrow suites' cal slice — emotion's test G1 regressed — and coincided with Dispatch on the wide suites it targeted; the wide-label G1 gap was closed by T2's stratified cal slice). The candidate table is recorded per suite (report-only); EngineConfig::readout stays the opt-in knob
- floor: conformal-naive floor = split-conformal recalibration of the raw readout confidence, c'(s) = (1 + #{cal s_i <= s}) / (n_cal + 1) — the exchangeability-valid baseline (G1 fails unless the calibrated ECE beats it)
- modal floor: input-keyed modal-answer floor = the cal-split (reference) modal gold over questions sharing the input key, pooled by option identity and mapped through each case's own option order (first-4 normalized input tokens × qid × option-count; ties → global prior → case order; arXiv:2610.17495's anti-mimicry census; LOO over eval only when no cal slice exists) — a lane at/below it carries no input-independent signal (issue 094)
- determinism: the bit-identity claim is the MODELLESS lane's (plan caveat 3); the laya lane gets the same observed repeat check and it is reported, not claimed
- divergence: massive option sampling: SplitMix64 (fixed seed), not CPython MT19937 — both lanes see byte-identical questions, which is the integrity that matters
- divergence: banking77: mteb/banking77 mirror (the reference's own bench_apps variant); PolyAI/banking77 is script-based and unservable
- divergence: engine context = state + prompt (wire criteria None) — the modelless serving path
- divergence: synthetic fixtures (semantic_defects): in-process fixtures with programmatic gold; the six Issue-004 harness families (2026-10-02) and code_fixtures (2026-10-11) were retired — owner calls — so the modelless lane no longer answers any self-authored suite but the one that reads above chance

## Worth-ab (issue 111 T3, matched-cost A/B — report-only)

The GOAT question, pre-registered in `.issues/111` BEFORE any run: at equal test-side escalation spend, does committee-informed arming beat delta-only arming on serving accuracy? Accuracy alone can never pass — every posture reports its accuracy/cost PAIR; the paired LB95 reads the flipped suites' escalated rows on ONE frozen test read.

### checkpoint `english` (cost-match tolerance 5%)

| posture | armed | cost (esc q) | esc rate | correct | accuracy |
|---|---|---|---|---|---|
| delta_only | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.00) | 8 | 3288 | 0.697 | 1997 | 0.4233 |
| combined(t=0.00) | 9 | 3350 | 0.710 | 2008 | 0.4256 |
| worth_only(t=0.01) | 8 | 3288 | 0.697 | 1997 | 0.4233 |
| combined(t=0.01) | 9 | 3350 | 0.710 | 2008 | 0.4256 |
| worth_only(t=0.02) | 8 | 3288 | 0.697 | 1997 | 0.4233 |
| combined(t=0.02) | 9 | 3350 | 0.710 | 2008 | 0.4256 |
| worth_only(t=0.03) | 8 | 3288 | 0.697 | 1997 | 0.4233 |
| combined(t=0.03) | 9 | 3350 | 0.710 | 2008 | 0.4256 |
| worth_only(t=0.04) | 8 | 3288 | 0.697 | 1997 | 0.4233 |
| combined(t=0.04) | 9 | 3350 | 0.710 | 2008 | 0.4256 |
| worth_only(t=0.05) | 7 | 1288 | 0.273 | 1870 | 0.3964 |
| combined(t=0.05) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.06) | 7 | 1288 | 0.273 | 1870 | 0.3964 |
| combined(t=0.06) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.07) | 7 | 1288 | 0.273 | 1870 | 0.3964 |
| combined(t=0.07) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.08) | 7 | 1288 | 0.273 | 1870 | 0.3964 |
| combined(t=0.08) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.09) | 7 | 1288 | 0.273 | 1870 | 0.3964 |
| combined(t=0.09) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.10) | 6 | 939 | 0.199 | 1819 | 0.3855 |
| combined(t=0.10) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.11) | 6 | 939 | 0.199 | 1819 | 0.3855 |
| combined(t=0.11) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.12) | 5 | 676 | 0.143 | 1789 | 0.3792 |
| combined(t=0.12) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.13) | 4 | 544 | 0.115 | 1738 | 0.3684 |
| combined(t=0.13) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.14) | 4 | 544 | 0.115 | 1738 | 0.3684 |
| combined(t=0.14) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.15) | 4 | 544 | 0.115 | 1738 | 0.3684 |
| combined(t=0.15) | 8 | 1350 | 0.286 | 1881 | 0.3987 |
| worth_only(t=0.16) | 4 | 544 | 0.115 | 1738 | 0.3684 |
| combined(t=0.16) | 8 | 1350 | 0.286 | 1881 | 0.3987 |

**Matched** at t=0.09: worth cost 1288 vs delta 1350 (gap 4.6%, tolerance 5%) · acc 0.3964 vs 0.3987 · paired (worth − delta) -0.00233 LB95 -0.00377 (a=3 b=14, n=4718) → **NEGATIVE**

### checkpoint `multilingual` (cost-match tolerance 5%)

| posture | armed | cost (esc q) | esc rate | correct | accuracy |
|---|---|---|---|---|---|
| delta_only | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.00) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.00) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.01) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.01) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.02) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.02) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.03) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.03) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.04) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.04) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.05) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.05) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.06) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.06) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.07) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.07) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.08) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.08) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.09) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.09) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.10) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.10) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.11) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.11) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.12) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.12) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.13) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.13) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.14) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.14) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.15) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.15) | 0 | 0 | 0.000 | 588 | 0.2940 |
| worth_only(t=0.16) | 0 | 0 | 0.000 | 588 | 0.2940 |
| combined(t=0.16) | 0 | 0 | 0.000 | 588 | 0.2940 |

**Matched**: none — delta_only cost 0 (nothing armed), the sweep's worth postures are priced in the table above.

### checkpoint `typed` (cost-match tolerance 5%)

| posture | armed | cost (esc q) | esc rate | correct | accuracy |
|---|---|---|---|---|---|
| delta_only | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.00) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.00) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.01) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.01) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.02) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.02) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.03) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.03) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.04) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.04) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.05) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.05) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.06) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.06) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.07) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.07) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.08) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.08) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.09) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.09) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.10) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.10) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.11) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.11) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.12) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.12) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.13) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.13) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.14) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.14) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.15) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.15) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| worth_only(t=0.16) | 1 | 2000 | 1.000 | 1489 | 0.7445 |
| combined(t=0.16) | 1 | 2000 | 1.000 | 1489 | 0.7445 |

**Matched** at t=0.16: worth cost 2000 vs delta 2000 (gap 0.0%, tolerance 5%) · acc 0.7445 vs 0.7445 · paired (worth − delta) +0.00000 LB95 +0.00000 (a=0 b=0, n=2000) → **UNDECIDABLE_ZERO_STRADDLE**

## typed_decisions — 400 cases / 2000 questions

**slice-integrity: test 400 (fnv1a64-287d5f73a11932cd) · cal 100 (fnv1a64-0b6e8426558b0d1e) · pool 1100 (fnv1a64-a1679de56e9278b1) · OK**

**corpus cap:** 48 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.3005 · max_prob 0.1050 · inv_entropy 0.3005

**modal floor (issue 094):** acc `0.4640` over 2000 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (80 keys, 40 fallbacks, reference-modal share `0.2280`)

- ⛔ `modelless · modelless` acc `0.2940` ≤ floor — no input-independent signal on this suite
- ⛔ `laya-riir · english` acc `0.3575` ≤ floor — no input-independent signal on this suite
- ⛔ `laya-riir · multilingual` acc `0.3490` ≤ floor — no input-independent signal on this suite

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 2000 | 0.2940 | 0.2038 | 0.0860 | 0.6888 | 1.2269 | 0.5614 | 0.4670 | 0.0140 | 1.00/1.00 | 0.0000 | 0.760 ms | 1.691 ms (5) | ✓ | s 0.3103 / d 0.9122 (n 100) |
| laya-riir · english | 2000 | 0.3575 | 0.3262 | 0.2113 | 0.7684 | 1.3593 | 0.5627 | 0.4290 | 0.1918 | — / — | — | 244.0 ms | 372.0 ms (5) | ✓ | — |
| laya-riir · multilingual | 2000 | 0.3490 | 0.3499 | 0.3228 | 0.9057 | 1.9437 | 0.5548 | 0.4040 | 0.2567 | — / — | — | 170.0 ms | 480.0 ms (5) | ✓ | — |
| laya-riir · typed | 2000 | 0.7445 | 0.7349 | 0.1924 | 0.4099 | 0.7185 | 0.1263 | 0.8590 | 0.3953 | — / — | — | 188.0 ms | 323.0 ms (5) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 2000 | 0.0% | 0.2940 | 0.2940 | +0.0000 | — | — | 2000 (disarmed by worthiness) |
| cascade · multilingual | 2000 | 0.0% | 0.2940 | 0.2940 | +0.0000 | — | — | 2000 (disarmed by worthiness) |
| cascade · typed | 2000 | 100.0% | 0.7445 | 0.2940 | +0.4505 | 1489 / 2000 (0.7445) | 588 / 2000 (0.2940) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | DISARMED | 500 | 0.3540 | 0.3080 | +0.0460 | — | +0.1600 | — |
| cascade · multilingual | DISARMED | 500 | 0.3400 | 0.3080 | +0.0320 | — | +0.1600 | — |
| cascade · typed | armed | 500 | 0.8060 | 0.3080 | +0.4980 | — | +0.1600 | — |


**Collective worth** (issue 111, cal-slice probe): agreement-on-error = among rows where BOTH lanes err, the share emitting the SAME wrong pick (arXiv:2506.07962; definition carried in every results.json row). The committee verdict is the size-aware empirical read — at the 2-lane gate shape the committee IS the deployed cascade pick, measured on the cal rows against the modelless floor; the union bound is a ceiling only. Bars pre-registered in `.issues/111` BEFORE the run.

| escalator | rows | both err | agree on err | AoE | both-wrong | κ | committee acc | Wilson LCB | floor | Δ | verdict | ceiling |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cascade · english | 500 | 229 | 92 | 0.402 | 0.458 | -0.083 | 0.3540 | 0.3133 | 0.3080 | +0.0460 | worth_committee | 0.542 |
| cascade · multilingual | 500 | 233 | 63 | 0.270 | 0.466 | -0.023 | 0.3400 | 0.2998 | 0.3080 | +0.0320 | undecidable | 0.534 |
| cascade · typed | 500 | 68 | 33 | 0.485 | 0.136 | +0.051 | 0.8060 | 0.7691 | 0.3080 | +0.4980 | worth_committee | 0.864 |


**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 2000 | 0.2940 | 0.2038 | 0.2925 | 0.0021 | ✓ |
| laya-riir · english | 2000 | 0.3575 | 0.3262 | 0.2925 | 0.0919 | ✓ |
| laya-riir · multilingual | 2000 | 0.3490 | 0.3499 | 0.2925 | 0.0799 | ✓ |
| laya-riir · typed | 2000 | 0.7445 | 0.7349 | 0.2925 | 0.6389 | ✓ |


**G1 (modelless readout ECE):** raw 0.2860 · calibrated 0.0140 · conformal-naive floor 0.3873 → **PASS** (beats both the uncalibrated output AND the floor)

**typed-decisions extras (modelless):** soft_acc 0.3172 · brier_soft 0.2424 · score MAE 0.7148 · within_1 0.7362

| model | type | n | acc | ECE(maxp) | mean conf |
|---|---|---|---|---|---|
| modelless | choice | 600 | 0.1400 | 0.1043 | 0.2443 |
| modelless | noul | 600 | 0.5650 | 0.0568 | 0.5082 |
| modelless | score | 800 | 0.2062 | 0.0941 | 0.3004 |
| laya-riir·english | choice | 600 | 0.2883 | 0.1681 | 0.4527 |
| laya-riir·english | noul | 600 | 0.4733 | 0.2808 | 0.7542 |
| laya-riir·english | score | 800 | 0.3225 | 0.2065 | 0.5168 |
| laya-riir·multilingual | choice | 600 | 0.2950 | 0.3623 | 0.6559 |
| laya-riir·multilingual | noul | 600 | 0.4867 | 0.4084 | 0.8919 |
| laya-riir·multilingual | score | 800 | 0.2863 | 0.2313 | 0.5110 |
| laya-riir·typed | choice | 600 | 0.7333 | 0.2548 | 0.4785 |
| laya-riir·typed | noul | 600 | 0.7850 | 0.1294 | 0.6624 |
| laya-riir·typed | score | 800 | 0.7225 | 0.1987 | 0.5246 |
**typed-decisions extras (laya-riir·english):** soft_acc 0.3346 · brier_soft 0.3348 · score MAE 0.6937 · within_1 0.7525
**typed-decisions extras (laya-riir·multilingual):** soft_acc 0.3271 · brier_soft 0.4737 · score MAE 0.7604 · within_1 0.7000
**typed-decisions extras (laya-riir·typed):** soft_acc 0.4668 · brier_soft 0.0677 · score MAE 0.2424 · within_1 0.9950

## ag_news — 400 cases / 400 questions

**slice-integrity: test 400 (fnv1a64-238133933bb8d6fc) · cal 200 (fnv1a64-96c58e69e7c6048e) · pool 19800 (fnv1a64-996876e41929dc33) · OK**

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4845 · max_prob 0.2262 · inv_entropy 0.4845

**modal floor (issue 094):** acc `0.2625` over 400 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (195 keys, 398 fallbacks, reference-modal share `0.2500`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.4050 | 0.3750 | 0.1465 | 0.7436 | 1.3735 | 0.5223 | 0.4750 | 0.0813 | 1.00/0.39 | 0.3951 | 0.143 ms | 0.262 ms (5) | ✓ | s 0.4397 / d 0.4105 (n 200) |
| laya-riir · english | 400 | 0.9500 | 0.9439 | 0.0316 | 0.0808 | 0.1637 | 0.0067 | 1.0000 | 0.1412 | — / — | — | 32.0 ms | 62.0 ms (5) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 400 | 39.2% | 0.6050 | 0.4050 | +0.2000 | 146 / 157 (0.9299) | 66 / 157 (0.4204) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 60 | 0.9833 | 0.4000 | +0.5833 | — | +0.1600 | — |


**Collective worth** (issue 111, cal-slice probe): agreement-on-error = among rows where BOTH lanes err, the share emitting the SAME wrong pick (arXiv:2506.07962; definition carried in every results.json row). The committee verdict is the size-aware empirical read — at the 2-lane gate shape the committee IS the deployed cascade pick, measured on the cal rows against the modelless floor; the union bound is a ceiling only. Bars pre-registered in `.issues/111` BEFORE the run.

| escalator | rows | both err | agree on err | AoE | both-wrong | κ | committee acc | Wilson LCB | floor | Δ | verdict | ceiling |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cascade · english | 200 | 3 | 1 | 0.333 | 0.015 | +0.314 | 0.6600 | 0.5919 | 0.4850 | +0.1750 | worth_committee | 0.985 |


**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 400 | 0.4050 | 0.3750 | 0.3075 | 0.1408 | ✓ |
| laya-riir · english | 400 | 0.9500 | 0.9439 | 0.3075 | 0.9278 | ✓ |


**G1 (modelless readout ECE):** raw 0.4046 · calibrated 0.0813 · conformal-naive floor 0.2282 → **PASS** (beats both the uncalibrated output AND the floor)

## emotion — 400 cases / 400 questions

**slice-integrity: test 400 (fnv1a64-36f3eb1067bed96a) · cal 200 (fnv1a64-f2ec48ed7070889c) · pool 15800 (fnv1a64-a05205a8dc7160cb) · OK**

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.1449 · max_prob 0.0245 · inv_entropy 0.1449

**modal floor (issue 094):** acc `0.3125` over 400 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (162 keys, 267 fallbacks, reference-modal share `0.1700`)

- ⛔ `modelless · modelless` acc `0.2750` ≤ floor — no input-independent signal on this suite

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 400 | 0.2750 | 0.1345 | 0.1053 | 0.8323 | 1.7886 | 0.7259 | 0.3000 | 0.1297 | 1.00/0.33 | 0.2910 | 0.110 ms | 0.130 ms (5) | ✓ | s 0.1453 / d 0.4265 (n 200) |
| laya-riir · english | 400 | 0.5925 | 0.4709 | 0.3088 | 0.6963 | 2.1840 | 0.2657 | 0.7150 | 0.2571 | — / — | — | 22.0 ms | 51.0 ms (5) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 400 | 33.0% | 0.4025 | 0.2750 | +0.1275 | 83 / 132 (0.6288) | 32 / 132 (0.2424) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 60 | 0.5167 | 0.1167 | +0.4000 | — | +0.1600 | — |


**Collective worth** (issue 111, cal-slice probe): agreement-on-error = among rows where BOTH lanes err, the share emitting the SAME wrong pick (arXiv:2506.07962; definition carried in every results.json row). The committee verdict is the size-aware empirical read — at the 2-lane gate shape the committee IS the deployed cascade pick, measured on the cal rows against the modelless floor; the union bound is a ceiling only. Bars pre-registered in `.issues/111` BEFORE the run.

| escalator | rows | both err | agree on err | AoE | both-wrong | κ | committee acc | Wilson LCB | floor | Δ | verdict | ceiling |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cascade · english | 200 | 94 | 23 | 0.245 | 0.470 | -0.037 | 0.2650 | 0.2087 | 0.1450 | +0.1200 | worth_committee | 0.530 |


**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 400 | 0.2750 | 0.1345 | 0.2975 | 0.0000 | ✓ |
| laya-riir · english | 400 | 0.5925 | 0.4709 | 0.2975 | 0.4199 | ✓ |


**G1 (modelless readout ECE):** raw 0.2749 · calibrated 0.1297 · conformal-naive floor 0.2982 → **PASS** (beats both the uncalibrated output AND the floor)

## sst5 — 600 cases / 600 questions

**slice-integrity: test 600 (fnv1a64-67fe9666fa0c3a6f) · cal 200 (fnv1a64-d210e7c6707c3d40) · pool 8344 (fnv1a64-cf933108b9825392) · VIOLATIONS**

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.2442 · max_prob 0.0340 · inv_entropy 0.2442

**modal floor (issue 094):** acc `0.1383` over 600 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (188 keys, 578 fallbacks, reference-modal share `0.2000`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 600 | 0.2017 | 0.1890 | 0.0087 | 0.7986 | 1.6059 | 0.8159 | 0.1833 | 0.0658 | 1.00/0.58 | 0.2072 | 0.091 ms | 0.112 ms (7) | ✓ | s 0.2049 / d 0.4166 (n 200) |
| laya-riir · english | 600 | 0.3717 | 0.3292 | 0.2478 | 0.8172 | 1.7291 | 0.5279 | 0.4567 | 0.0811 | — / — | — | 23.0 ms | 34.0 ms (7) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 600 | 58.2% | 0.2867 | 0.2017 | +0.0850 | 120 / 349 (0.3438) | 69 / 349 (0.1977) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 104 | 0.4038 | 0.2308 | +0.1731 | — | +0.1600 | — |


**Collective worth** (issue 111, cal-slice probe): agreement-on-error = among rows where BOTH lanes err, the share emitting the SAME wrong pick (arXiv:2506.07962; definition carried in every results.json row). The committee verdict is the size-aware empirical read — at the 2-lane gate shape the committee IS the deployed cascade pick, measured on the cal rows against the modelless floor; the union bound is a ceiling only. Bars pre-registered in `.issues/111` BEFORE the run.

| escalator | rows | both err | agree on err | AoE | both-wrong | κ | committee acc | Wilson LCB | floor | Δ | verdict | ceiling |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cascade · english | 200 | 93 | 22 | 0.237 | 0.465 | -0.013 | 0.3350 | 0.2732 | 0.2450 | +0.0900 | worth_committee | 0.535 |


**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 600 | 0.2017 | 0.1890 | 0.2733 | 0.0000 | ✓ |
| laya-riir · english | 600 | 0.3717 | 0.3292 | 0.2733 | 0.1353 | ✓ |


**G1 (modelless readout ECE):** raw 0.2009 · calibrated 0.0658 · conformal-naive floor 0.3343 → **PASS** (beats both the uncalibrated output AND the floor)

**sst5 score metrics (modelless):** MAE 1.1781 · within_1 0.4433

## prompt_injections — 116 cases / 116 questions

**slice-integrity: test 116 (fnv1a64-a6a375f0273e0bef) · cal 100 (fnv1a64-a45e6561b2a98ff1) · pool 446 (fnv1a64-8b32b961bbd47195) · OK**

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.4999 · max_prob 0.0055 · inv_entropy 0.4999

**modal floor (issue 094):** acc `0.5345` over 116 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (84 keys, 107 fallbacks, reference-modal share `0.5000`)

- ⛔ `modelless · modelless` acc `0.4828` ≤ floor — no input-independent signal on this suite

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 116 | 0.4828 | 0.3256 | 0.0228 | 0.5005 | 0.6937 | 0.3980 | 0.5690 | 0.0172 | 1.00/1.00 | 0.0000 | 0.068 ms | 0.089 ms (2) | ✓ | s 0.5100 / d 0.4619 (n 100) |
| laya-riir · english | 116 | 0.6983 | 0.6751 | 0.2620 | 0.5259 | 3.1497 | 0.1073 | 0.9655 | 0.2620 | — / — | — | 22.0 ms | 64.0 ms (2) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 116 | 100.0% | 0.6983 | 0.4828 | +0.2155 | 81 / 116 (0.6983) | 56 / 116 (0.4828) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 100 | 0.7700 | 0.5000 | +0.2700 | — | +0.1600 | — |


**Collective worth** (issue 111, cal-slice probe): agreement-on-error = among rows where BOTH lanes err, the share emitting the SAME wrong pick (arXiv:2506.07962; definition carried in every results.json row). The committee verdict is the size-aware empirical read — at the 2-lane gate shape the committee IS the deployed cascade pick, measured on the cal rows against the modelless floor; the union bound is a ceiling only. Bars pre-registered in `.issues/111` BEFORE the run.

| escalator | rows | both err | agree on err | AoE | both-wrong | κ | committee acc | Wilson LCB | floor | Δ | verdict | ceiling |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cascade · english | 100 | 22 | 22 | 1.000 | 0.220 | +0.000 | 0.7700 | 0.6785 | 0.5000 | +0.2700 | worth_committee | 0.780 |


**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 116 | 0.4828 | 0.3256 | 0.5172 | 0.0000 | ✓ |
| laya-riir · english | 116 | 0.6983 | 0.6751 | 0.5172 | 0.3750 | ✓ |


**G1 (modelless readout ECE):** raw 0.4827 · calibrated 0.0172 · conformal-naive floor 0.2491 → **PASS** (beats both the uncalibrated output AND the floor)

## xnli_en — 300 cases / 300 questions

**slice-integrity: test 300 (fnv1a64-614262484399affa) · cal 200 (fnv1a64-b8126a40673cc7c6) · pool 19800 (fnv1a64-f3f8f200eba61985) · OK**

**corpus cap:** 64 (registry)

**readout (report-only):** best-on-cal `max_prob` NOT armed — the margin pick overfits cal (Bench 052 demotion); cal in-sample calibrated ECE: dispatch 0.3196 · max_prob 0.0238 · inv_entropy 0.3196

**modal floor (issue 094):** acc `0.3333` over 300 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (198 keys, 288 fallbacks, reference-modal share `0.3350`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.3400 | 0.2052 | 0.0055 | 0.6676 | 1.1000 | 0.6937 | 0.3467 | 0.0335 | 1.00/0.56 | 0.3383 | 0.084 ms | 0.147 ms (4) | ✓ | s 0.2999 / d 0.5110 (n 200) |
| laya-riir · english | 300 | 0.8600 | 0.8612 | 0.0685 | 0.2204 | 0.3987 | 0.0381 | 0.9800 | 0.1044 | — / — | — | 24.0 ms | 32.0 ms (4) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 300 | 55.7% | 0.6333 | 0.3400 | +0.2933 | 145 / 167 (0.8683) | 57 / 167 (0.3413) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 110 | 0.9182 | 0.3455 | +0.5727 | — | +0.1600 | — |


**Collective worth** (issue 111, cal-slice probe): agreement-on-error = among rows where BOTH lanes err, the share emitting the SAME wrong pick (arXiv:2506.07962; definition carried in every results.json row). The committee verdict is the size-aware empirical read — at the 2-lane gate shape the committee IS the deployed cascade pick, measured on the cal rows against the modelless floor; the union bound is a ceiling only. Bars pre-registered in `.issues/111` BEFORE the run.

| escalator | rows | both err | agree on err | AoE | both-wrong | κ | committee acc | Wilson LCB | floor | Δ | verdict | ceiling |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cascade · english | 200 | 4 | 3 | 0.750 | 0.020 | -0.012 | 0.6350 | 0.5663 | 0.3200 | +0.3150 | worth_committee | 0.980 |


**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 300 | 0.3400 | 0.2052 | 0.3333 | 0.0100 | ✓ |
| laya-riir · english | 300 | 0.8600 | 0.8612 | 0.3333 | 0.7900 | ✓ |


**G1 (modelless readout ECE):** raw 0.3395 · calibrated 0.0335 · conformal-naive floor 0.3137 → **PASS** (beats both the uncalibrated output AND the floor)

## massive_intent_en — 300 cases / 300 questions

**slice-integrity: test 300 (fnv1a64-328f111f90d7a0c1) · cal 200 (fnv1a64-74b76416cdd836d5) · pool 11314 (fnv1a64-fb5a4f0d9147dc0d) · OK**

**corpus cap:** 48 (registry)

**modal floor (issue 094):** acc `0.1100` over 300 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (175 keys, 265 fallbacks, reference-modal share `0.0200`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 300 | 0.6100 | 0.6035 | 0.5454 | 0.9258 | 2.7756 | 0.2278 | 0.7667 | 0.0614 | 1.00/0.35 | 0.6122 | 0.102 ms | 0.122 ms (4) | ✓ | s 0.2372 / d 0.5259 (n 200) |
| laya-riir · english | 300 | 0.6933 | 0.6936 | 0.2483 | 0.5405 | 3.5500 | 0.1444 | 0.9133 | 0.2441 | — / — | — | 34.0 ms | 42.0 ms (4) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 300 | 34.7% | 0.6333 | 0.6100 | +0.0233 | 70 / 104 (0.6731) | 63 / 104 (0.6058) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 99 | 0.7576 | 0.3737 | +0.3838 | — | +0.1600 | — |


**Collective worth** (issue 111, cal-slice probe): agreement-on-error = among rows where BOTH lanes err, the share emitting the SAME wrong pick (arXiv:2506.07962; definition carried in every results.json row). The committee verdict is the size-aware empirical read — at the 2-lane gate shape the committee IS the deployed cascade pick, measured on the cal rows against the modelless floor; the union bound is a ceiling only. Bars pre-registered in `.issues/111` BEFORE the run.

| escalator | rows | both err | agree on err | AoE | both-wrong | κ | committee acc | Wilson LCB | floor | Δ | verdict | ceiling |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cascade · english | 200 | 44 | 5 | 0.114 | 0.220 | +0.340 | 0.6200 | 0.5511 | 0.4300 | +0.1900 | worth_committee | 0.780 |


**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 300 | 0.6100 | 0.6035 | 0.0867 | 0.5730 | ✓ |
| laya-riir · english | 300 | 0.6933 | 0.6936 | 0.0867 | 0.6642 | ✓ |


**G1 (modelless readout ECE):** raw 0.5454 · calibrated 0.0614 · conformal-naive floor 0.1249 → **PASS** (beats both the uncalibrated output AND the floor)

## banking77 — 500 cases / 500 questions

**slice-integrity: test 500 (fnv1a64-cb0b5cbcb0cdfcce) · cal 200 (fnv1a64-8f14f68a2e55799f) · pool 9793 (fnv1a64-14a38d138e218d1b) · OK**

**corpus cap:** 40 (registry)

**modal floor (issue 094):** acc `0.0460` over 500 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (148 keys, 311 fallbacks, reference-modal share `0.0150`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 500 | 0.3940 | 0.3526 | 0.3780 | 0.9822 | 4.1742 | 0.5090 | 0.4840 | 0.0278 | 1.00/0.53 | 0.5485 | 0.348 ms | 0.649 ms (6) | ✓ | s 0.3177 / d 0.5549 (n 200) |
| laya-riir · english | 500 | 0.4220 | 0.3765 | 0.3813 | 0.9327 | 6.1219 | 0.4066 | 0.6000 | 0.3940 | — / — | — | 51.0 ms | 59.0 ms (6) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 500 | 52.6% | 0.4540 | 0.3940 | +0.0600 | 97 / 263 (0.3688) | 67 / 263 (0.2548) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 109 | 0.4220 | 0.2110 | +0.2110 | — | +0.1600 | — |


**Collective worth** (issue 111, cal-slice probe): agreement-on-error = among rows where BOTH lanes err, the share emitting the SAME wrong pick (arXiv:2506.07962; definition carried in every results.json row). The committee verdict is the size-aware empirical read — at the 2-lane gate shape the committee IS the deployed cascade pick, measured on the cal rows against the modelless floor; the union bound is a ceiling only. Bars pre-registered in `.issues/111` BEFORE the run.

| escalator | rows | both err | agree on err | AoE | both-wrong | κ | committee acc | Wilson LCB | floor | Δ | verdict | ceiling |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cascade · english | 200 | 78 | 11 | 0.141 | 0.390 | +0.258 | 0.5100 | 0.4412 | 0.3950 | +0.1150 | worth_committee | 0.610 |


**JDI crosswalk (reference-only):** JDI-comparable crosswalk, NOT a JDI board row — different corpus, protocol caps, hardware; board membership requires their full frozen suite. Chance = majority gold-class share of the slice ACTUALLY EVALUATED (plan 011 B3 — the board's pinned chance values stay reference columns); skill = (acc − chance)/(1 − chance) clipped [0,1]. Coverage per row: answered = n (all), unsupported 0, errors 0 — the harness law: a lane refuses-and-fails-loud, it never silently skips a case. The harness families carry no JDI columns (Issue 059 law); cascade compositions are excluded (composed postures, not lanes).

| lane · model | n | acc | macro F1 | chance | skill | det |
|---|---|---|---|---|---|---|
| modelless · modelless | 500 | 0.3940 | 0.3526 | 0.0140 | 0.3854 | ✓ |
| laya-riir · english | 500 | 0.4220 | 0.3765 | 0.0140 | 0.4138 | ✓ |


**G1 (modelless readout ECE):** raw 0.3780 · calibrated 0.0278 · conformal-naive floor 0.1942 → **PASS** (beats both the uncalibrated output AND the floor)

## semantic_defects — 102 cases / 102 questions

**corpus cap:** 18446744073709551615 (registry (selection n/a: self-corpora))

**modal floor (issue 094):** acc `0.1667` over 102 questions — modal census pooled by option identity; key = first-4 normalized input tokens × qid × option-count; ties → global prior → the case's own option order (18 keys, 102 fallbacks, reference-modal share `0.2778`)

| lane · model | n | acc | macro F1 | ECE(maxp) | Brier | NLL | AURC | acc@50 | readout-ECE | abst(raw/cal) | sel-acc(cal) | p50 | p99 (support) | det | gate-fit (ρ=.30) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| modelless | 102 | 0.1863 | 0.1798 | 0.0287 | 0.8338 | 1.7937 | 0.8220 | 0.1961 | 0.1843 | 0.61/0.61 | 0.2500 | 0.023 ms | 0.029 ms (2) | ✓ | s 0.0010 / d 0.5209 (n 18) |
| laya-riir · english | 102 | 0.3431 | 0.3682 | 0.2746 | 0.8278 | 1.6137 | 0.6479 | 0.3725 | 0.1687 | — / — | — | 34.0 ms | 64.0 ms (2) | ✓ | — |

**Reflex · cascade** (issue 038 T4′): the modelless lane answers every question; the calibrated fused gate's abstains escalate to the named checkpoint. The escalation rate IS the latency claim — a deployed cascade pays the escalator only on that fraction.

| escalator | n | esc rate | cascade acc | modelless forced | Δ | laya on escalated | modelless on same set | un-escalated |
|---|---|---|---|---|---|---|---|---|
| cascade · english | 102 | 60.8% | 0.2941 | 0.1863 | +0.1078 | 20 / 62 (0.3226) | 9 / 62 (0.1452) | 0 |


**Cascade worthiness** (issue 042 lever 3, cal-slice probe): the escalation stays armed only where the escalator reads ≥ the forced modelless picks on the cal questions the calibrated gate abstained on — or, when the issue-046 LCB leg is on, where the probe delta's one-sided-95% lower bound clears the floor (support-aware arming; conservative on paired data). No cal records / thin support stays armed with the named reason — the probe never invents a disarm it cannot measure.

| escalator | verdict | probe n | laya on probe | modelless on probe | Δ | probe LCB | min Δ | note |
|---|---|---|---|---|---|---|---|---|
| cascade · english | armed | 10 | 0.2000 | 0.2000 | +0.0000 | — | +0.1600 | thin support: 10 probe questions < 16 — armed, pre-lever behavior holds |


**Collective worth** (issue 111, cal-slice probe): agreement-on-error = among rows where BOTH lanes err, the share emitting the SAME wrong pick (arXiv:2506.07962; definition carried in every results.json row). The committee verdict is the size-aware empirical read — at the 2-lane gate shape the committee IS the deployed cascade pick, measured on the cal rows against the modelless floor; the union bound is a ceiling only. Bars pre-registered in `.issues/111` BEFORE the run.

| escalator | rows | both err | agree on err | AoE | both-wrong | κ | committee acc | Wilson LCB | floor | Δ | verdict | ceiling |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| cascade · english | 18 | 9 | 1 | 0.111 | 0.500 | +0.000 | 0.2778 | 0.1250 | 0.2778 | +0.0000 | not_worth | 0.500 |


**G1 (modelless readout ECE):** raw 0.1843 · calibrated 0.1843 · conformal-naive floor 0.3617 → **NO CLAIM** (the calibrator never fitted (cal window below the occupancy floor) — nothing to pass or fail)

## Landscape — published specialist rows (NOT measured by this harness)

External "System One" typed-decision models on the same 400-case /
2000-question Typed Decisions official test split, quoted AS PUBLISHED
(issue 025; pin `malevrigns/agent-jev` @ `a965ca8f`, Apache-2.0). Their
protocol differs from ours — the footnotes are part of the row; no number
here is comparable without them.

| lane · model | source | acc | bool·noul / choice / score | p50 case |
|---|---|---|---|---|
| AgentJev-0.6B (598M, Qwen3-0.6B backbone) | published (their run) | **0.7925** | 88.83 / 75.33 / 75.00 | ~60–70 ms, their cuda box |
| reflex · agentjev (their service, gold-label) | **MEASURED** (bench 039, 4090) | **0.7715** | — | 88 ms (loopback HTTP) |
| Laya (published checkpoint, 421M ModernBERT) | their table — card-copied, not re-scored | 0.7700 | — | 41.53 ms (their box) |
| TypeSafe Jev 1.13.0 | their table — zero-shot generalist | 0.727 | — | — |
| reflex · laya-riir·typed | MEASURED — the typed_decisions table above | 0.7445 (baseline `aa37823`) | 78.50 / 73.33 / 72.25 | 1312 ms (m3 metal, that baseline) |
| reflex · modelless | MEASURED — the typed_decisions table above | 0.3190 (baseline `aa37823`) | 53.17 / 18.67 / 25.87 | 0.472 ms |

Footnotes: (1) their accuracy is agreement with the public TEACHER argmax;
ours is gold-label under the standard harness protocol — different
references of truth. (1b) the MEASURED agentjev row (bench 039, Issue 025
amendment 4) closes that gap for AgentJev: **0.7715 gold-label** on this
harness's split (teacher-argmax 0.7925 → gold −2.1pt) — the published
ranking SURVIVES the protocol change (+2.7pt over our measured laya-typed
0.7445; their teacher-protocol gap was +2.25). (2) their run held out 120 dev + 120 cal cases and
selected the step-600 checkpoint on dev soft-CE before opening test; ours
fits no per-benchmark head. (3) their wide-load figure (shared-prefix
298.91 ms vs unshared 609.65 ms at 66 paths / 33,547 tokens, backbone
token-ops 33,547 → 2,551 = 92.4% reduction, max prob delta 5.08e-4) is
their box and their load — not re-measured here. (4) on SHORT inputs their
own table reads Laya faster (41.53 ms vs ~60–70 ms p50/case); AgentJev's
latency win is wide candidate loads only. (5) the bool·noul column maps
their boolean primitive to our noul primitive — the closest analogue, not
a wire match; neither of their lanes carries an abstention primitive (the
wire's first-class abstention is ours alone).

## Landscape — fast-decisions (vendor suite; NOT measured by this harness)

fastino's `fast-decisions` suite — 17 English operational-decision
domains (commerce support intent/topic, ticket routing, product feedback,
banking intent, document type, review sentiment, assistant handoff,
email triage, clinic request, travel request, news topic, paper field,
sports recap, restaurant review, benefits request, screen tags), 300
held-out test examples per domain, exact-match accuracy, the same text
and candidate labels for every model. Quoted AS PUBLISHED (issue 029).

| model | avg exact-match |
|---|---|
| GLiNER2.5-Decide (340M, DeBERTa-v3-large) | **60.2%** |
| GLiNER2.5-Decide-1B (their dataset card: "GLiNER2 XL (1B)") | 59.6% |
| JevK5 | 57.6% |
| GLiNER2.5-multi-Decide (287M) | 56.7% |
| SemIf (Qwen3.5-4B) | 56.4% |
| GLiFormer large-v1 | 49.0% |
| Laya Router | 46.6% |

Footnotes: (1) the scored split is private — their public repo carries
only the 100/domain development split with an explicit "do not report a
score computed on the files in this repo" note, so no row here can be
re-scored outside fastino. (2) their metric is per-head exact match
(single-label string equality; multi-label heads compared as sets),
averaged over the 17 domains. (3) their "Laya Router" row names no
checkpoint variant — neither the base router nor the typed specialist;
our measured laya-riir cells are the port checkpoints under OUR protocol,
so the 60.2-vs-46.6 gap is their-suite/their-checkpoint/their-protocol.
(4) our measured comparison lives in the 15-suite tables (bench 037,
`.benchmarks/037_gliner_lane_4090`): the DIRECTION is confirmed on
decision-style suites — gliner beats the laya base checkpoint 9/15
(banking77 +20.8pt, massive_intent +7.3pt, all five harness families) —
and honestly refuted on classic NLU (ag_news −24.8pt, xnli_en −38.3pt;
their card's own "not a general-purpose model" framing). The
typed_decisions headline stays laya's: the `typed` specialist 0.7445 vs
gliner 0.5280.

