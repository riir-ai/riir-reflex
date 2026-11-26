# sprint!136 — bench_preflight's load gate samples at LAUNCH only: a mid-run load rise poisons the cell while the PROVENANCE line still reads the launch state (measured on the sprint!020 bars lane)

**Status:** OPEN — filed 2026-11-26 ~12:5x+07 (zed-31ff0055@m3), from the sprint!020 bars round-3 cell (seal-sprints `.issues/020…` — the delegated verdict round 3's amendment 3: "the round-3 cell hit load 6.86 mid-run against the 6.0 ceiling, which is only checked at launch… the gate certifies box state at launch only and can't see a rise during the run").
**Claimed:** zed-31ff0055@m3, 1795755000, expires 2026-11-28T12:00Z
(scope: the filing + the fix design; the fix lands when a lane takes it — the preflight script is this repo's.)

**Repos:** [reflex] (the script's home; consumed by every fleet lane quoting a PROVENANCE line)

## What was measured (three amo exclusive cells, 2026-11-26 07:1x-11:4xZ)

| cell | preflight at launch | mid-run reality | outcome |
|---|---|---|---|
| `74717fc4` | PASSED (load 3.01) | POISONED ×4 mid-window (Zed 39055 + rustc + cargo + a train engine, 07:19:51-07:21:07Z) | unusable — the ship rows visibly inflated |
| `24e26c08` | PASSED | POISONED ×1 (the same Zed, 07:26:13-07:26:29Z) + PROVENANCE would have read load 8.0 mid-run | unusable |
| `01a216c4` | PASSED (load read at start below ceiling) | **PROVENANCE load=6.86 — above the 6.0 ceiling** + **POISONED ×16** (a rustc build swarm + cargo + a train engine + Zed, 07:33:27→end) | labeled OUT-OF-SPEC post-hoc by the verdict — the gate never did; ⚠ the poison verdict rode the end MAIL, not the log body — a log-grep read the cell "poison-free" (the detection lesson: the poison state is orchestrator-side, never greppable from the cell's own log) |

The gate's design samples load ONCE at launch; a sibling session (or an owner app — amo's local Zed, pid 39055, was the poisoner in all three windows) starting mid-run raises the load after the gate has certified the box. The cell's numbers are visibly inflated (the ship-pose arms read ~5.4-5.6 ms vs ~4.3-4.5 in a quiet smoke), and nothing in the output names the breach — the record's PROVENANCE line actively asserts a box state that the run's own tail contradicts.

## The fix shape (design; the implementing lane re-derives)

- **Mid-run sampling:** the preflight (or the harness it fronts) re-reads load on a cadence through the measurement window (e.g., every 30 s), records min/max/mean beside the final PROVENANCE line, and FLAGS (`load_mid_run_max=X exceeds Y`) or REFUSES (a strict mode) when the ceiling is crossed.
- **The PROVENANCE line gains the mid-run half:** `load=2.12 load_mid_run_max=6.86` — a reader can then see at a glance whether the window held.
- **Poison witness integration:** on rigs running the orchestrator's poison watch, a poison event during a preflight-gated cell should surface IN the cell's record (the orchestrator already mails the lane — the cell's own log never learns of it).
- **The out-of-spec precedent:** the sprint!020 bars row labels its round-3 cell OUT-OF-SPEC post-hoc — the fix makes that label automatic instead of a verdict-lane manual act.

## Tasks

- [ ] The mid-run sampler + the extended PROVENANCE line (the design above; the cadence and the flag/refuse modes are the implementing lane's call).
- [ ] The poison-witness surfacing (orchestrator → cell record) if the orchestrator-side seam is reachable from this script.
- [ ] The sprint!020 bars lane's out-of-spec precedent cited as the motivating case.

## Close-out note

The sprint!020 lane held its pins to a THREE-CELL record (the quiet m3 smoke ≤ 0.08 ms + two amo loaded cells) precisely because no amo window held the ceiling — the load-corrected bound form (the verdict's amendment 1: the control-compression factor) is the compensating discipline this fix makes unnecessary.
