# Bench 100 — clinc150 suite birth: the ladder wave-1 CLINC150 adapter, first paired read (issue 114 T2)

**Date:** 2026-11-09 · **Repo:** riir-reflex @ `73c93d2` · **Box:** m3 (M3 Max, 16 cores)
**Issue:** reflex `.issues/114` T2 (+ the reflex-side half of T6; the rethink-side fold is the separate 060 R1d/R2a booking)
**Posture:** `cargo run --release --features laya-riir-metal --bin harness -- --suites clinc150 --laya-max-questions 100 --out .benchmarks/100_clinc150_suite_birth`

## What landed

The CLINC150 suite (`clinc/clinc_oos` config `plus`): 151-way intent choice
(150 intents + `oos` at ClassLabel position 42), banking77's machinery as it
stands — `CLINC150_KEYS` (151 keys, mechanical from the FETCHED features),
`build_clinc150` with verify-at-port (the card's YAML differs from the served
features in TWO spellings — `no'`/`yes'` card typos vs the served `no`/`yes`;
the port check caught it live), the `train_row_label`/`train_docs` arms, the
151 dispatch arm, the expected-key pin, the 151-universe guard. Two additive
envelope root-fixes rode the lane: `wrap_rows` threads source features through
split envelopes; `load_rows` carries the first page's features block.

## The read (this record)

| lane | n | acc | ECE(maxp) | p50 |
|---|---|---|---|---|
| modelless (Dispatch) | 500 | **0.3520** | 0.3433 | 0.545 ms |
| laya[english] | 100 (capped probe) | **0.6500** | 0.2995 | 51.0 ms |

- Chance = 1/151 = **0.0066** — modelless sits 53× above random; laya 98×.
  Both seats answer the suite sensibly; nothing is stuck at chance or at a
  single class (the slice-integrity floors held: test 500
  `fnv1a64-317d15fe5a26d022` · cal 200 `fnv1a64-7dc66be93f3e9b96` · pool
  15,050 `fnv1a64-705b33da11fdd028` · OK).
- ⛔ **Latency NOT QUOTABLE** — the run's own box-state disclosure refused it
  (load 8.94/8.48 > 6 at start/end, powermode high, swap 532M; sibling cargo
  lanes on the box). The preflight agrees (`PROVENANCE: … load=6.29 …
  powermode=2(high)` → `✗ preflight REFUSED`). The ACCURACY columns are
  deterministic computation joins and ride with the digests above; the p50
  cells are provisional until a re-run at the quiet posture.
- The laya `n` columns reflect the 100-question cap (the standard probe
  posture) — not comparable row-wise against modelless n=500.

## Read-back (honest, no promotion claim)

This is a suite-BIRTH record, not a promotion: the suite enters the registry
`named_only` (a `--suites clinc150` opt-in, the ladder-suites posture), never
a default-run member. The modelless number is the zero-shot corpus lane on a
151-way task (banking77's 77-way reads 0.85+ trained; this suite has no
served cell yet). The teacher capture + ESC fit + any served-cell ambitions
are the rethink-side bookings (060 R1b/R1d) and stay owner-gated amo hours.

## Wave-1 state after T2

- T1 probes: DONE (all four sources, licences verified: CC-BY-3.0 / CC-BY-4.0
  / Apache-2.0; API-Bank honestly unprobed — rides T5).
- T2 clinc150: **LANDED** (this record).
- T3 When2Call: wire CONFIRMED 4-way fixed-choice; **BLOCKED on a lawful
  corpus source** (llm_judge = documented mcq subset [overlap measured
  11/100]; train splits unlabeled [the marker covers tool_call only]; mcq
  answers = test rows). Three re-open paths + the xlam-60k corpus-synthesis
  route recorded in the issue.
- T4 BFCL / T5 API-Bank: same corpus wall (BFCL verified single-file-per-
  category, no train split); dispositions recorded.
- T6: the reflex-side paired read is THIS record; the feed cells + rethink
  fold extension are the rethink-side booking (060 R1d/R2a), gated on the
  corpus decisions above.
