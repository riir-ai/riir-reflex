# reflex 135 pre-wake verdict (2026-11-28 07:5xZ, zed-3f131dc0@amo) — NO REGRESSION EXISTS; posture-comparator error

## Verdict

The "sst5 0.3967 → 0.2017 regression" is a **posture-comparator error, not a code mover**.
The published reference cell is the **selection posture** (`--nb-select --ridge-select`);
the bisect probe line ran **bare** (no selection flags). Both postures reproduce digit-exact
TODAY at HEAD. NO bisect owed, NO engine fix owed, NO re-baseline owed (the published 0.3967
is the selection posture and reproduces byte-exact — nothing to re-baseline).

## The decisive table (all quotable from committed .benchmarks records + today's probes)

| record | pool | posture | sst5 hard acc | corpus digest |
|---|---|---|---|---|
| bench 107 / 115 (the published 0.3967 era) | cf93 | nb_sel=YES + ridge_sel=YES | **0.3967** | e2a9 (nb_sets folded, old pool) |
| bench 100/101/123/131/137 (post-pool-fix) | b464 | nb_sel=YES + ridge_sel=YES | **0.3967 digit-exact** | 726f (nb_sets folded) |
| bench 073 (pre-window!) / 124 / 150 / 155 / 156 / 175 | b464 | **bare** | **0.2017 — byte-stable since 073** | 0c64 (bare folds no nb_sets) |
| today: reflex@HEAD + katgpt@HEAD, bare | b464 | bare | 0.2017 | 0c64 |
| today: reflex@HEAD + katgpt@07acd887b (Nov 11), bare | b464 | bare | 0.2017 | 0c64 |
| today: reflex@197e855 + katgpt@1c86cf126 (era-matched pair), bare | b464 | bare | 0.2017 | 0c64 |
| **today: reflex@HEAD + katgpt@HEAD, `--nb-select --ridge-select`** | b464 | selection | **0.3967 · ece 0.1389 — digit-exact vs bench 137** | 726f |

## Why the earlier framing ("CONSTRUCTION moved") was wrong

- The corpus digest folds nb_sets **only when the selection posture arms it**. 726f vs 0c64
  is a **posture marker**, not a construction mover. Comparing bench-123-era digests
  (selection posture) against bare-posture digests produced the false "construction moved" signal.
- The bare corpus digest 0c64 is IDENTICAL at reflex@197e855 (Oct 9) and HEAD (Nov 28) —
  construction is byte-stable across the entire alleged window.
- Slice digests (test 67fe / cal d210 / pool b464) match bench 123 (Oct 4) exactly — the
  datasets are byte-stable since the Oct-1 pool refresh (bench 100/101 "pool_fix": cf93→b464).

## The two-repo entanglement facts (for the record; no longer load-bearing)

- reflex@HEAD needs katgpt-core ≥ 07acd887b (compute_control, Nov 11 Tranche 2a umbrellas).
- reflex@197e855 needs katgpt-core ≤ 8c3349bfb (action_bridge, baked/removed Nov 10).
- Cross-pair probes therefore cannot straddle the bake boundary; both mixed pairs were
  attempted and both refused to resolve (the scaffolding at /tmp/i135_bisect/ documents it).

## At-the-wake checklist (replaces the old bisect runbook)

1. Claim reflex 135 after 09:00Z per the standing discipline (fetch + claim_lease_check;
   holder zed-ee5ceb7b@m3 expiring 09:00Z; yield if renewed).
2. Append the closure to the issue: verdict NO-REGRESSION (posture-comparator error), cite
   this file + the probe dirs (/tmp/i135/probe_*), close the bisect lane (no re-baseline —
   the published 0.3967 is the selection posture and reproduces digit-exact; if a BARE sst5
   board row is wanted, it is a NEW posture row (0.2017), never a regression fix).
3. Fold the two 08:55Z/09:05Z scheduled wakes (they are self-mail; let them fire and no-op
   against this record, or cancel: `agent-mail scheduled` under zed-d4108612).
4. Teardown: `git worktree remove /tmp/i135_bisect/{reflex,katgpt-rs}` + prune + rm -rf
   /tmp/i135_bisect; /tmp/i135 kept only until the issue row lands (probes are evidence),
   then removed with the lane.
5. Check Arm C v3 (rig job e7fecae3) per the old runbook — if it ended, its post-job
   checklist (riir-rethink .issues/067) still takes priority.

## Probe evidence dirs (kept until the issue row lands)

- /tmp/i135/probe_era_matched/ — reflex@197e855 + katgpt@1c86cf126 (era pair), bare → 0.2017/0c64
- /tmp/i135/probe_kcore_07ac/ — reflex@HEAD + katgpt@07acd887b, bare → 0.2017/0c64
- /tmp/i135/probe_reflex_old2/ — the invalid first attempt (Cargo.lock checkout abort; rerun as era pair)
- /tmp/i135/probe_sel_head/ — reflex@HEAD + katgpt@HEAD + `--nb-select --ridge-select` → **0.3967/726f** (the positive control)
- /tmp/i135/repro_head/ — the 06:40Z pre-warm reproduce (bare → 0.2017/0c64)
