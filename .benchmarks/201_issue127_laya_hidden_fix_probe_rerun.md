# Bench 201 — Issue 127: the laya probes re-run after riir-infer 1239 (`EncodedQuestion.hidden` = the true encoder output)

**Status:** RECORDED 2026-11-20 (katgpt-rs-93@amo). Shared rig job `6cdf8e7c` on amo, Metal. PRE = riir-infer `14654abe` (the commit before the fix), in a worktree layout. POST = riir-infer `595ce32b` (fix `954b27be`). Same box, same riir-reflex `9fa8a90`. These are accuracy and geometry reads, so no latency is quoted. banking77 has no `.raw/datasets_t20k` data on amo, so `pair_free_probe` covers massive_intent_en + ag_news only. `rho_suite_probe` reads banking77 from its own fixture path.

## pair_free_probe (Issue 105 → riir-train Bench 691), `--n 1000`

| suite | arm | verdict | lift | transfer top1 | align cos | nb-hit | FOSCTTM raw / paired | post-Proc cos |
|---|---|---|---:|---:|---:|---:|---|---:|
| massive_intent_en | PRE | low | 0.600 | 0.045 | 0.834 | 0.784 | 0.519 / 0.494 | 0.0038 |
| massive_intent_en | POST | **low** | 0.480 | 0.036 | 0.844 | 0.879 | 0.486 / 0.500 | −0.0010 |
| ag_news | PRE | low | 0.865 | 0.231 | 0.850 | 0.821 | 0.522 / 0.489 | 0.0048 |
| ag_news | POST | **low** | 0.783 | 0.209 | 0.874 | 0.939 | 0.528 / 0.490 | 0.0037 |

PRE reproduces Bench 691 to its printed precision (massive lift 0.60 / nb-hit 0.784; ag_news 0.87 / 0.821), so the delta is the fix alone. **The verdict tier is unchanged (`low` on both).** On true encoder states, neighborhood structure is stronger (nb-hit +0.10 / +0.12) and transfer lift is weaker. Bench 691's GL(k)/nuisance mechanism reading stands.

## rho_suite_probe (Issue 106 → katgpt-rs Issue 978 T5)

The lane tally is unchanged (0 correct / 0 wrong refusals / 6 permitted → FAIL, no saving), and so is every gate decision. One adjudication changes:

| suite, arm | PRE held-out post | POST held-out post | floor | bound |
|---|---:|---:|---:|---|
| banking77 k=256 | 468.40 | **449.14** | 467.41 | HELD → **VIOLATED** (3.9% below floor; band 2%) |
| banking77 k=64 | 114.73 | 106.22 | 102.05 | HELD → HELD |
| emotion k=64 | 126.57 | 125.73 | 123.82 | HELD → HELD |
| ag_news k=64 | 123.37 | 122.80 | 125.21 | HELD → HELD (within the 2% band) |
| the k=256 arms of emotion / ag_news | | | | NOT VALID (degenerate fit) both sides |

Laya ρ moves with the fix (banking77 k=256 0.0932 → 0.0993, k=64 0.2318 → 0.2893). The held-out gain grows to 12.7% / 17.1% of identity on banking77.

**Consequence:** katgpt-rs Issue 978's round-2 record ("the held-out protocol shows the bound HELD in all 4 estimable arms") was measured on the head's residual. On the true encoder output, **3 of 4 hold and banking77 k=256 is VIOLATED** beyond the estimator band. Routed as katgpt-rs Issue 1236 (`fca33de8c`, unclaimed) for re-adjudication.

Logs: `201_issue127_laya_hidden_fix_probe_rerun_logs/{pre,post}_{pair_*,rho}.log`.
