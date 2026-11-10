# Bench 179 — Issue 111 collective-worth probe: T1/T2 landed, T3 matched-cost A/B NEGATIVE

**Date:** 2026-11-10 (run 2026-11-10T00:48:11Z) · **Commit:** `4046f8f` (the probe + evaluator) · **Host:** m3 (M3 Max, 16 cores, 64 GiB) · **Posture:** release, `laya-riir-metal`, LAYA_DEVICE=metal, datasets `.raw/datasets_t20k` (frozen), 9 suites / 4718 questions, 3 checkpoints (english primary, multilingual + typed armed-0) · **Box state** (harness meta, start): AC Power, power_mode=high, load_1m 4.59, swap 532 MB — **accuracy reads only; NO latency claim is made from this run** (the T4 disclosure; accuracy is load-insensitive).

**Pre-registration:** committed BEFORE the run (`eaccd6c`) in `.issues/111` — postures, cost axis (test-side covered abstains from the arming-blind counters), 5% match tolerance, the paired-LB95 promote bar, and the T2 verdict pins. One frozen test read; both postures recomputed in-process from the same per-question records.

## Posture

`--cascade --cascade-worthiness --collective-worth --worth-ab` (all opt-in, default-off, byte-identical off; margin 0.16 shipped). The T1 pair block + T2 committee verdict ride every worthiness verdict (cal slice only); the T3 evaluator reads the per-suite hypothetical-armed counters (covered abstains regardless of the run's arming) + the committee worth-arm data.

## T1 — agreement-on-error (cal slice, english checkpoint)

| suite | rows | both err | agree on err | AoE | both-wrong | κ |
|---|---|---|---|---|---|---|
| typed_decisions | 500 | 229 | 92 | 0.402 | 0.458 | −0.083 |
| ag_news | 200 | 3 | 1 | 0.333 | 0.015 | +0.314 |
| emotion | 200 | 94 | 23 | 0.245 | 0.470 | −0.037 |
| sst5 | 200 | 93 | 22 | 0.237 | 0.465 | −0.013 |
| prompt_injections | 100 | 22 | 22 | 1.000 | 0.220 | 0.000 |
| xnli_en | 200 | 4 | 3 | 0.750 | 0.020 | −0.012 |
| massive_intent_en | 200 | 44 | 5 | 0.114 | 0.220 | +0.340 |
| banking77 | 200 | 78 | 11 | 0.141 | 0.390 | +0.258 |
| semantic_defects | 18 | 9 | 1 | 0.111 | 0.500 | 0.000 |

Errors are substantially SHARED (both-wrong 13.5–47% of common rows; prompt_injections AoE 1.000 — every shared error is the same wrong pick), κ near or below zero on the big-probe suites — the lanes agree often because both are right, and where both err they often err alike. The dcog complementarity premise does NOT describe this stack at the 2-lane shape: the lanes are not diverse error-wise. (typed_decisions' cal slice is the outlier in SIZE, not in shape.)

## T2 — committee verdict (n=2 = the deployed cascade pick, measured on the cal rows)

8 of 9 suites read `worth_committee` (improvement +0.046…+0.498, Wilson-supported), semantic_defects reads `not_worth` (improvement +0.000, floor = committee exactly). multilingual-checkpoint rows read undecidable (zero-straddling: typed_decisions improvement +0.032, LCB 0.2998 < floor 0.3080). The n≥3 majority path is landed + unit-tested (first-index ties, ρ̂/n′ heuristics, decorrelated-lane worth case) but unreachable in this harness (2 lanes).

## T3 — the matched-cost A/B (the GOAT question): **NEGATIVE**

english checkpoint (the only one with armed suites; multilingual + typed checkpoints arm nothing under ANY posture — delta_cost 0, honest no-match):

| posture | armed | cost | esc rate | accuracy |
|---|---|---|---|---|
| delta_only (shipped) | 8 | 1350 | 0.286 | **0.3987** |
| worth_only(t=0.00) | 8 | 3288 | 0.697 | 0.4233 |
| worth_only(t=0.09) ← matched | 7 | 1288 | 0.273 | 0.3964 |
| worth_only(t=0.16) | 4 | 544 | 0.115 | 0.3684 |

- **Match** at t=0.09: cost 1288 vs 1350 (gap 4.6% ≤ 5% tolerance) — worth(0.09) is the delta set minus semantic_defects.
- **Paired (worth − delta)** over the whole frozen read: mean −0.00233, **LB95 −0.00377 < 0** (a=3, b=14, n=4718) — and worth accuracy 0.3964 < delta 0.3987. Both pre-registered promote legs FAIL → **NEGATIVE**.
- The flipped suite tells the mechanism: semantic_defects' escalated rows read McNemar [14, 3] — escalation GAINS +11 there on the test read — but the committee's cal read says not_worth (improvement exactly 0 on 18 cal rows). The committee verdict is the WORSE arming rule at matched cost.
- Accuracy alone never had a chance to pass, by the issue's own design: worth_only(0.00) reaches 0.4233 but at 2.44× the spend (3288 vs 1350). At matched cost the committee-informed rule loses.

## Secondary observation (diagnostic, NOT a promotion claim)

The committee verdict flags **typed_decisions** as worth (+0.046, Wilson-supported at n=500 cal rows) where the shipped delta margin (0.16) disarms it — and arming typed on the test read gains **+0.0269 (≈ +127 questions at +2000 spend)**: worth(0.00–0.04) 0.4233 vs worth(0.05+) 0.3964. The typed checkpoint's cal-slice delta (+0.046 on the abstained probe set) badly understates what typed escalation buys (test-side McNemar [478, 351] = +127 on 2000 covered abstains). That is a cal-slice representativeness observation about the DELTA gate's probe set (Issue 042 lever territory — the margin's own re-open trigger law applies), not a collective-worth finding: the committee read typed worth for the right reason but the matched-cost comparison still fails.

## Verdict

- **T3 NEGATIVE** per the pre-registered bars → the flags stay opt-in forever; T5 consumers (rethink ESC worthiness, rulewright domain fan-out) do NOT adopt. Second consecutive negative probe (after Issue 107's routing-state probe) — a legitimate close per the issue's own text.
- T1/T2 stay landed as report-only instruments (the blocks ride `--collective-worth`; `--worth-ab` reruns the A/B against any future third lane — the n≥3 majority path is the drop-in).
- Re-open trigger (the issue's own): a committee with genuinely decorrelated errors — a THIRD lane whose errors are anti-correlated with both existing lanes (T5's rulewright multi-domain shape is the plausible candidate). The n≥3 path is unit-proven; only the lane is missing.

Artifacts: `results.json`, `TABLES.md`, `run.log` in this directory. Instrument landed `4046f8f`; pre-registration `eaccd6c`; `.benchmarks/.highwater` 178 → 179 (and 176 → 178 repaired in `eaccd6c` — the 177/178 holders landed unrecorded in `333b64a`/`6cdae2f`).
