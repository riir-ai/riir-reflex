# Issue 135 — modelless sst5 regression: the served arm reads ~chance (0.2017 vs the published 0.3967); root-cause before any re-baseline

**Status:** OPEN — filed 2026-11-26 (zed-ee5ceb7b@m3) out of the i132 A3-republish data event's pre-registered stop condition (delegated-owner verdict, session `9dc5c21b`, 2 rounds AGREE).
**Claimed:** zed-ee5ceb7b@m3, 1764130000, expires 2026-11-28T09:00:00Z

## The finding (all cells measured 2026-11-26)

The published `reflex-site/data/bench.json` sst5 modelless cell (source_run reflex `197e855`, 2026-10-10) reads **hard acc 0.39667**, corpus_digest `fnv1a64-726f717634ca24fb`. The same lane re-run today (release `c82b011f`, quiet-box QUOTABLE, `PROVENANCE: power=AC load 4.47 powermode=2(high) ane=ok`, `.benchmarks/i132_m3_refresh/`) reads **0.20166666666666666 — chance level on a 5-class suite (1/5 = 0.2)**, digest `fnv1a64-0c648062c19b8c01`. The served arm's confidence-filtered quality halved with it: calibrated selective 0.4057 @ 53.2% abstain (published) → 0.2072 @ 58.2% (fresh). The raw lane abstains 100% in both eras (the served answers are the calibrated arm's), so the regression is in what the arm RANKS, not in the gate.

- [ ] Root cause: the two-repo-pinned bisect (reflex `197e855`-era + katgpt-rs Oct-10-era — measured: a bare `197e855` worktree cannot build against the live sibling; reflex that age requires katgpt-core feature `action_bridge`, absent from the current substrate — the repos moved together, so the window is BOTH repos' commits Oct-10 → Nov-25)
- [ ] Quick label-alignment screen (run BEFORE the bisect): done — inconclusive-negative. Fresh confusion is skewed (pred `neutral` 190/600) but NOT single-class-collapsed; `within_1` 0.4433 (MAE 1.178) is above the label-shuffle signature (~0.28 adjacency-chance) but degraded; amo and m3 agree digit-exact on every derived stat (121/600, same digest, same within_1) — construction/engine-determined, seating-irrelevant.
- [ ] If the root cause is a bug → FIX THE ENGINE; the published 0.3967 posture is the reference. **A fleet modelless re-baseline (the Issue-023 T5 "both hosts move together" event) is ILLEGAL until the root cause is known** — re-baselining onto a broken arm would launder the regression into the served table (the round-2 verdict's change 1).
- [ ] If the old construction is proven WRONG (a deliberate, GOAT-recorded improvement) → then the re-baseline event: all modelless-carrying hosts × all suites re-run together at the new posture, lane_sources refreshed, the site's served rows re-pinned in one publish.
- [ ] Update the stale-cell disclosure: after the i132 laya-only publish (site commit of record), the sst5 modelless cell on the page is NOT reproducible at current HEAD — the finding number is cited in the publish commit body + the i132 bench record; the page template has no per-cell annotation slot (follow-up here if one is wanted).
- [ ] Two-repo bisect mechanics when picked up: worktrees at `197e855` + the katgpt-rs sha live on 2026-10-10 (approximate by date from katgpt-rs log), probe = `cargo run --release --bin harness -- --skip-laya --suites sst5 --out <tmp>` (sub-second lane), test = acc == 0.39666666666666666 (good) vs 0.20166666666666666 (bad); the corpus_digest discriminator (726f…24fb vs 0c64…8c01) splits construction-vs-decisions at the first divergent step.

## Scope evidence (why the laya lanes are NOT implicated)

- laya:english acc **0.37166666666666665 reproduced digit-exact in three independent runs on two machines and two corpus seatings**: published m3 (source_run `8ca8770`, 2026-10-02), amo canonical (`c82b011`, 2026-11-25, latency NOT quotable — evidence only), m3 t20k (`c82b011f`, 2026-11-26, QUOTABLE p50 14.0/p99 17.0 ms — fresh timing recorded here, NOT published; the LANE_CARRY law keeps the incumbent 18.0 ms on the page).
- The i126 gold overlay binds ag_news only (`OVERLAY_SUITE`) — not a suspect.
- The corpus DATASET bytes are unchanged: `.raw/datasets_t20k` mtime 2026-09-26, untouched through both eras; the digest moved anyway → the CONSTRUCTION (or decision) code moved.

## Shipped-path severity check (round-2 verdict change 4)

**NOT a product regression today.** The deployed reflex.riir.ai is the static Tier-0 site (wrangler.toml: "the arena site is static"); the /decide walk-through captures are from the 0.2.4 release binary dated 2026-10-03, not a live seat; POST /decide on the domain 405s; no `RIIR_REFLEX_CORPUS` launchd/service/deploy wiring exists on this box. The game heads are vessel artifacts (separate engine). Any FUTURE serve deployment would inherit the regression — re-check this section before the next hosted-serving plan ships.

## Provenance

- Stop condition: round-2 delegated verdict (session `9dc5c21b`, both rounds AGREE; the round-1 approved shape — full same-host update rendering both lanes' ordinal rows — is REFUSED by the measured drift; the sanctioned shape is the lane-scoped `PUBLISH_BENCH_LANES=laya` publish per reflex `.issues/033`).
- Fresh run: `.benchmarks/i132_m3_refresh/` (watcher `quiet_box_timing_rerun.sh`, QUOTABLE both stamps; git_sha c82b011f).
- amo run 1: `amo:~/git/riir-reflex/.benchmarks/i132_ordinal_refresh/results.json` (canonical-seated, `latency_quotable: false` at BOTH stamps — load >6 — evidence only, never compared against published modelless cells).
- The i132 event record lands as the next `.benchmarks/` number (the laya-only publish + this finding's ref).
