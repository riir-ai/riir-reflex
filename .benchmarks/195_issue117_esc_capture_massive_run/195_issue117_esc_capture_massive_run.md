# Bench 195 — Issue 117 T1 capture RUN: the ESC-armed cascade's cal-side outcome join (massive_intent_en @ ρ 0.60, amo)

**Status:** DONE 2026-11-14 (zed-m3-i117-esc-cap) — the capture ran, 200 triples landed, the split law held, T1 of reflex `.issues/117` is complete (code `84675ac` + this run). T2/T3/T4 stay open, now unblocked by real disagreement rows.

## What ran

The issue-117 T1 capture arm (`--esc-capture`, landed `84675ac`) over the ESC-armed cascade posture,
byte-faithful to rethink's `esc_knobs("massive_intent_en")` (the ESC_POSTURES massive row's worthiness
margin 0.05 included):

```
LAYA_DEVICE=metal ./target/release/harness \
  --datasets-dir .raw/datasets_t20k --suites massive_intent_en \
  --nb-select --oc-select --ridge-select \
  --cascade --cascade-worthiness --cascade-worthiness-margin 0.05 \
  --gate-fit-selection --gate-distance-only --gate-rho 0.60 \
  --esc-capture <out>/capture --out <out>/run
```

- commit `c300600` on `amo`, release profile, `laya-riir-metal`, 2026-11-14T14:34:27Z.
- The two code lanes this run needed and landed in the same arc: `a0f6e57` (`--gate-rho <F>` — the
  ESC-armed per-suite rhos were reachable only through the library knob; the CLI arm re-fits the
  fused gate at the armed ρ 0.60; `None` default byte-identical) and `c300600` (dataset cal ids
  carry the slice namespace `<suite>:cal:<pos>` — the capture's first REAL run caught what the
  synthetic fixtures masked: builder positional ids collide across splits, so the split-law guard
  refused every real dataset suite).
- PROVENANCE: power=AC Power load=16.23 swap=4196.62M canary=skipped powermode=0(auto).
  ⛔ **Latency cells are NOT QUOTABLE** — the sprint010 merge-gate job ran concurrently
  (harness-recorded box state: start load 17.24 → end 56.18, TABLES.md carries the disclosure).
  The capture's COUNTS and picks are load-independent (both lanes det ✓).

## The capture (cal slice only — the split law held)

`esc_capture_stats.json` (verbatim, cross-checked against `triples.jsonl` row-by-row):

```json
{
  "suite": "massive_intent_en",
  "cal_cases": 200,
  "cal_questions": 200,
  "gate_fired": 117,
  "gate_fired_served": 117,
  "disagreements": 68,
  "think_correct": 82,
  "cheap_correct": 65,
  "think_wins": 35,
  "cheap_wins": 18,
  "armed": true,
  "probe_n": 117,
  "probe_delta": 0.14529914529914523,
  "triples_written": 200
}
```

The fired-set 2×2 over the 117 escalated questions (derived from the triples, cross-checked):
both right 47 (40.2%) · **think-only 35 (29.9%)** · **cheap-only 18 (15.4%)** · both wrong 17
(14.5%); disagreements 68 = 35 think-wins + 18 cheap-wins + 15 both-wrong. The escalator served
exactly the gate-abstained rows (83 unserved rows carry `think_pick: null` and never fire);
all 200 ids are `:cal:`-namespaced, unique, and disjoint from the 300 test ids.

## The VoI join, unblinded (the bench-lane legs)

rethink `esc_voi`'s value legs are receipts-blind at serve; this capture is their bench-lane join
over the cal slice (definitions from `esc_voi.rs`):

| VoI leg | value |
|---|---|
| escalation rate (cal) | 117/200 = **58.5%** |
| precision `P(think ≠ cheap ∧ think better \| escalated)` | 35/117 = **0.2991** |
| value per escalation `E[think better − cheap better \| escalated]` | (35−18)/117 = **+0.1453** |
| blindness line | resolved at the bench lane (this record); serve-lane VoI stays receipts-only until rethink's outcome join lands |

Arming honesty: the worthiness probe read **+0.1453 (n 117)** — far above the ESC massive row's
m0.05 bar. This is a different posture than Bench-063's measured +0.0064 (that probe ran at the
shipped ρ 0.30 gate; the ρ 0.60 rate-targeted gate abstains on a different, harder cal subset
where laya's edge concentrates). The escalator here is the reflex harness cascade's
`laya-riir · english` checkpoint — **not** the production massive arm (rethink serves the remote
mammoth think leg for massive_intent_en); the capture measures the ESC family's LOCAL lane at the
armed gate posture, which is exactly what the harness instrument captures.

## The test-half disclosure (the T4 axis, read early)

The same run's test half (NOT mined — the run's own tables, `TABLES.md`) shows the worthiness
delta does not transfer: cascade 0.7233 vs forced modelless 0.7800 (**−0.0567**); on the 173
escalated test questions laya reads 119/173 = 0.6879 vs modelless forced 136/173 = 0.7861
(**−0.0982**). The ESC rate guard's latch axis would read this traffic as a cheap-only demotion.
So the local-lane ESC posture on massive: **arms on cal, loses on test** — the production choice
of the mammoth think leg for this suite is corroborated, and any T2/T3 corpus work must treat the
cal-side disagreement rows as its only lawful input (they are; the split law forbids the rest).

## Artifacts

| file | what |
|---|---|
| `capture/triples.jsonl` (200 rows) | the VoI join: id, split=cal, cheap_pick, think_pick (null when unserved), gold, gate_fired |
| `capture/captured_ids.txt` (200) | the T3 frozen read's disjointness input (`<suite>:cal:<pos>` namespace, `c300600`) |
| `capture/esc_capture_stats.json` | the divergence summary (verbatim above) |
| `run/TABLES.md` | the full harness tables (worthiness + cascade + slice digests) |
| `run/results.json` (21,567 B, sha256 `36ce3cee455ad099e2d168fffae715b40b2def3aaa7118ac966fed686ece4b4d`) | the run record |

Slice integrity (TABLES.md): test 300 (fnv1a64-328f111f90d7a0c1) · cal 200 (fnv1a64-74b76416cdd836d5) · pool 11314 (fnv1a64-fb5a4f0d9147dc0d) · OK.

## Next (issue 117)

- **T2** — the 68 verified disagreement rows (35 of them think-wins) feed the standing synth lane
  (teacher-synthesized train-pool candidates; ⛔ grammar-not-answers binds: echo gate + leak scan
  every pass, corpus built AROUND the frozen test, ONE frozen read).
- **T3** — `--corpus-ab` V5 paired read, the read's case set asserted disjoint from
  `captured_ids.txt` (the namespace makes that assert mechanical).
- **T4** — the report axes: the test-half numbers above are the first reading; the production
  consequence (arming, rate band) stays owner-gated per the issue's own non-goals.
