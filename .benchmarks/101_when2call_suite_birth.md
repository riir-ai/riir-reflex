# Bench 101 — when2call suite birth: the corpus-optional posture + the first laya-only read (issue 114 T3)

**Date:** 2026-11-09 · **Repo:** riir-reflex @ `36c1d09` (suite code lands after this sha — the digest below pins the run) · **Box:** m3 (M3 Max, 16 cores)
**Issue:** reflex `.issues/114` T3 (the reflex-side half; the rethink-side fold stays the 060 R1d/R2a booking)
**Posture:** `cargo run --release --features laya-riir-metal --bin harness -- --suites when2call --laya-max-questions 0 --out .benchmarks/101_when2call_suite_birth`

## What landed

The When2Call suite (`nvidia/When2Call`, config `test`, split `mcq`) — the
JDI ladder's tools-rung row, the abstention-first shape. Three pieces:

1. **The corpus-optional suite posture** (the runner change, path (b) of the
   issue's re-open list — the only self-serve path; (a) teacher spend and
   (d) xlam synthesis are owner/lane-gated, (c) demotion makes nothing):
   `modelless_lane: false` on a DATASET suite now means corpus-optional —
   `prepare` skips the train/cal/pool machinery entirely (the slice audit is
   vacuous without a pool), the modelless lane is skipped loud (an absence
   line, never a silent zero), and only the LLM lanes answer. The no-LLM-lane
   posture refuses loud (the declared-inability law — verified: the
   modelless-only build prints the SKIPPED line and lists one absence).
   `load_suite_envelope` refuses corpus-optional suites (nothing to store).
   Byte-identical for every existing suite (the branch sits after the
   pins, keyed on the flag every `modelless_lane: true` suite sets).
2. **The fetch stanza** — `test/mcq` fetched WHOLE (3,652 rows, 37 pages;
   stored AS `test`, the mmlupro as-split precedent reversed); NO train
   split, ever (the three measured refusals: llm_judge = documented mcq
   subset, overlap 11/100; train splits unlabeled beyond the tool_call
   marker; the answers objects are test rows). Licence **CC BY 4.0**,
   commercial-use declared by the card — the strongest grant language in
   the manifest ledger. Manifest §19 + the licence row landed.
3. **The builder** — TASK-FAITHFUL rendering (the official lm-eval MCQ
   form): each row's four choices are the row's OWN response candidates
   (the `answers` object's values, keyed by the fixed category names in the
   object's uniform insertion order), gold = `correct_answer`'s position in
   that order (the official `get_choices_and_index` law). Fixed option-KEY
   universe (`WHEN2CALL_KEYS`), per-row criteria VALUES; verify-at-port
   pins the answers-key order PER ROW (the gold is a POSITION — a reorder
   would silently rebind every row; the refusal test ships). State = query
   + tool-schema strings AS STORED (258 rows carry an empty tools list —
   the benchmark's own no-tools tier, kept verbatim). Whole-split census:
   cannot_answer 1,295 · tool_call 1,295 · request_for_info 1,062 ·
   direct 0 (never gold; measured). Guards: population pin 3,652, gold
   coverage (the three scored classes), expected-key pin.

## The read (this record) — laya-only, full population

| lane | n | acc | macro-F1 | ECE(maxp) | p50 |
|---|---|---|---|---|---|
| modelless | — | **SKIPPED (corpus-optional)** | — | — | — |
| laya[english] | **3,652 (all)** | **0.2757** | 0.2281 | 0.1281 | 80.0 ms |

- cases_digest `fnv1a64-6e63e50408f92b4f`; no pool/cal slices exist (the
  corpus-optional posture — `slices: None`, disclosed in results.json's
  modal-floor line as the leave-one-out degraded form).
- **The honest floors:** majority-class share = 1295/3652 = **0.3546** (the
  proper single-arm floor on this population); the official MCQ chance
  framing (uniform pick over 4) = 0.25. laya[english] reads **0.2757 —
  BELOW the majority floor** (above uniform chance). An honest NEGATIVE for
  the encoder on the response-pick rendering at the shipped posture.
- ⛔ **Latency NOT QUOTABLE** — the run's own box-state disclosure: load
  7.11 > 6 at start, powermode high, swap 532M (sibling lanes on the box).
  The p50 cell is provisional until a quiet-posture re-run (the preflight
  PASSED only after the run: load 3.44). The ACCURACY columns are
  deterministic computation joins and ride with the digest above.

## Read-back (honest, no promotion claim)

Suite-BIRTH record, not a promotion: the suite enters the registry
`named_only` (`--suites when2call` opt-in, the ladder-suites posture).
The below-majority laya read is itself information — the encoder's
response-discrimination over long JSON tool_call candidates vs short
direct candidates is weak at the shipped encoding posture; whether that is
a rendering artifact (state truncation over multi-KB tool schemas,
response-length bias in the option encoding) or a capability ceiling is
the ESC lane's diagnosis question, and the category-classification
rendering (fixed criteria instead of response candidates) is the obvious
comparison arm for it. The teacher capture + ESC fit + any served-cell
ambition stay the rethink-side bookings (060 R1b/R1d, owner-gated amo
hours); the corpus re-open paths (a)/(d) stay recorded in the issue.

## Wave-1 state after T3

- T1 probes: DONE (all four sources, licences verified).
- T2 clinc150: LANDED (Bench 100).
- T3 when2call: **LANDED** (this record) — the suite + the corpus-optional
  posture; the modelless corpus remains the recorded owner/lane decision.
- T4 BFCL pick-tier / T5 API-Bank: the corpus-optional posture now UNBLOCKS
  the same build for both (single-file-per-category sources, no train
  splits — the same posture applies); their adapter builds ride a follow-up
  lane (BFCL needs the direct-JSON fetch lane — the gorilla mirror is not
  datasets-server-backed).
- T6: the reflex-side paired read is THIS record; the feed cells + the
  rethink fold (060 R1d/R2a) stay the rethink-side booking.
