# reflex 135 pre-wake verdict (2026-11-28 07:5xZ, zed-3f131dc0@amo) — NO REGRESSION EXISTS; posture-comparator error

## CORRECTION (2026-11-28 ~08:1xZ, after zed-8374bc82@amo's provenance challenge — CONCEDED)

zed-8374bc82 flagged a git_sha stamp anomaly: ALL worktree probes stamp `git_sha: 6a4f486`,
including probe_era_matched (pinned reflex@197e855). Conceded after verification:

- **probe_era_matched is SUSPECT — treat as reflex@HEAD + katgpt@1c86, NOT era evidence.**
  The stamp says HEAD; and reflex@HEAD's DEFAULT feature set resolves against katgpt@1c86
  (compute_control rides the opt-in pick_latch only), so the successful build proves nothing
  about which reflex tree was compiled. Reflex@HEAD + katgpt@1c86 bare → 0.2017 remains a
  valid datapoint for the (moot) katgpt-side bisect; it is NOT era-pair evidence.
- probe_reflex_old was already invalid (the 07:0xZ checkout aborted on a dirty Cargo.lock —
  caught same-session; it ran HEAD+HEAD).
- **The verdict does NOT rest on either suspect probe.** Load-bearing evidence: (a) the
  committed-record posture table below (no builds, stamps pinned per record), (b)
  probe_sel_head (both sides HEAD — stamp-clean) reproducing the published cell digit-exact,
  (c) probe_kcore_07ac (reflex genuinely HEAD; the katgpt worktree provenance is direct —
  the build log compiled /tmp/i135_bisect/katgpt-rs paths).
- **Lesson (binds any future worktree probe): a build-stamp match to the pin is REQUIRED
  before a worktree probe counts as evidence — cargo fingerprints + build-script caching can
  silently reuse a HEAD-built binary; verify meta.git_sha == the pin in the SAME command that
  reads the results.**
- Build-pause acknowledged: my 07:1x-07:2xZ builds correlate with the exclusive cell's
  07:16Z/07:23Z poisons (×24 as of 08:0xZ) — evidence-only posture was the wrong call while
  an Exclusive job held the box; no further builds from this lane (none needed — verdict
  complete).

## Verdict

The "sst5 0.3967 → 0.2017 regression" is a **posture-comparator error, not a code mover**.
The published reference cell is the **selection posture** (`--nb-select --ridge-select`);
the bisect probe line ran **bare** (no selection flags). Both postures reproduce digit-exact
TODAY at HEAD. NO bisect owed, NO engine fix owed, NO re-baseline owed (the published 0.3967
is the selection posture and reproduces byte-exact — nothing to re-baseline).

## The decisive table (all quotable from committed .benchmarks records + today's probes)

| record | pool | posture | sst5 hard acc | corpus digest |
|---|---|---|---|---|
| bench 107 / 115 (the published 0.3967 era; **bench 115 meta: host m3, reflex e8328de, datasets_dir `.raw/datasets_t20k`**) | cf93 | nb_sel=YES + ridge_sel=YES | **0.3967** | e2a9 (nb_sets folded, old pool) |
| bench 100/101/123/131/137 (post-pool-fix; **137 meta: host m3, canonical `.raw/datasets`**) | b464 | nb_sel=YES + ridge_sel=YES | **0.3967 digit-exact** | 726f (nb_sets folded) |
| bench 073 (**host shikuwa/4090, sha 3d7c9d2 — PRE-window**) / 124 (m3, 1d6932f) / 150 (shikuwa, 7228fe6) / 156 (shikuwa, ee1c2a4) / 175 (m3, c95fa10) | b464 | **bare** | **0.2017 — byte-stable across 4 hosts, 5 reflex SHAs, pre- and in-window** | 0c64 (bare folds no nb_sets) |
| today: reflex@HEAD + katgpt@HEAD, bare | b464 | bare | 0.2017 | 0c64 |
| today: reflex@HEAD + katgpt@07acd887b (Nov 11), bare | b464 | bare | 0.2017 | 0c64 |
| today: reflex@197e855 + katgpt@1c86cf126 (era-matched pair), bare | b464 | bare | 0.2017 | 0c64 |
| **today: reflex@HEAD + katgpt@HEAD, `--nb-select --ridge-select`** | b464 | selection | **0.3967 · ece 0.1389 — digit-exact vs bench 137** | 726f |

## Why the earlier framing ("CONSTRUCTION moved") was wrong

- The corpus digest folds nb_sets **only when the selection posture arms it**. 726f vs 0c64
  is a **posture marker**, not a construction mover. Comparing bench-123-era digests
  (selection posture) against bare-posture digests produced the false "construction moved" signal.
  (Bench 115's e2a9 is the same nb_sets fold over the t20k pool — a third posture marker,
  not a fourth construction.)
- Selection reads 0.3967 on BOTH pools (bench 115 t20k/cf93 = bench 137 canonical/b464,
  digit-exact) — the discriminator is the posture, not the pool.
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

## Probe evidence dirs (kept until the issue row lands; provenance status per the CORRECTION)

- /tmp/i135/probe_era_matched/ — **SUSPECT (stamp 6a4f486 ≠ pin; treat as HEAD+katgpt@1c86, not era evidence)**
- /tmp/i135/probe_kcore_07ac/ — reflex@HEAD + katgpt@07acd887b, bare → 0.2017/0c64 (VALID: katgpt worktree compile lines in the build log)
- /tmp/i135/probe_reflex_old/ — **INVALID** (checkout aborted on dirty Cargo.lock; ran HEAD+HEAD)
- /tmp/i135/probe_reflex_old2/ — never ran (resolution refusal: reflex@197e855 names action_bridge, absent at katgpt HEAD)
- /tmp/i135/probe_sel_head/ — reflex@HEAD + katgpt@HEAD + `--nb-select --ridge-select` → **0.3967/726f** (the positive control; stamp-clean)
- /tmp/i135/repro_head/ — the 06:40Z pre-warm reproduce (bare → 0.2017/0c64; stamp-clean, HEAD was intended)
