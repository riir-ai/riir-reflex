# Bench 188 — gsm8k suite birth: the knowledge rung's first suite, the two-track board-v3 shape (issue 114 R4a queue head)

**Date:** 2026-11-13 · **Repo:** riir-reflex (the fetch stanza + builder + registry + manifest land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` R4a (the knowledge nine triage's first adapter — the queue head; rethink 060 R4a's reflex-side half)
**Posture:** `LAYA_DEVICE=metal ./target/release/harness --suites gsm8k --laya-max-questions 0 --out .benchmarks/188_gsm8k_suite_birth` (release, laya-riir-metal; corpus-optional — the modelless lane skips loud)
**PROVENANCE:** ✓ preflight PASSED at launch (power=AC powermode=0(auto) load 2.03 swap 4954.06M; the run's own box-state stamps read start load **3.85 / end 2.90** — **latency QUOTABLE**).

## What landed

1. **The licence probe PASSES at both hub surfaces** — `openai/gsm8k`
   carries the hub tag `license:mit` AND `cardData.license: ['mit']`
   (probed at fetch 2026-11-13; the fetch script's licence tripwire
   guards refetches). The datasets-server /rows lane serves the suite
   (config `main` split `test`, 1,319 rows = the board row 30's
   population exactly; 14 pages, 0 failed_requests).
2. **`gsm8k`** (suite §28 of the manifest): **1,319 cases / 2,638
   questions** — TWO choice tracks per problem (the board's v3 rebuild
   shape): a 10-option question (keys `1`..`10`, chance 0.1) and a
   4-option question (keys `1`..`4`, chance 0.25) over the SAME
   problem; pooled chance 0.175 = the board's per-track average (the
   board counts 2,638 requests on 1,319 problems).
3. **The option sets are BUILT AT LOAD** — the port of the board's own
   published builder (`decision_index/suite/build/gsm8k_v3.py`, METHOD
   `answer-groups-v1` @ clone sha `f5a6e4fd85fe`): golds sorted into
   groups of K=10 DISTINCT values (each 10-option question's
   distractors are nine OTHER problems' golds — no lucky
   near-duplicates; the group laws fixture-tested: distinct-value
   groups incl. gold, the band partition's run law, the dominating-value
   loud refusal), the 4-option track takes a shifted 4-wide window of
   the group's sorted ring, option order is a per-question seeded
   shuffle. **RE-SEATED on the house primitives**: the board ties with
   sha256 and shuffles with Python's MT19937; this port folds the house
   FNV constant (the Issue-088 multiplier) and shuffles by key-sort —
   the option sets are METHOD-identical but NOT byte-identical to the
   board's rows (the disclosure; the board's SEED 20261003 kept for
   provenance). Determinism: the whole option walk is a pure function
   of (envelope, SEED) — no RNG state; the builder's determinism test
   replays it.
4. **CORPUS-OPTIONAL, laya-only** (the finentity/isarcasm_b posture):
   the options are case-specific NUMBERS and the per-question shuffle
   makes any position prior zero by construction — the modelless lane
   carries no lawful signal (it skipped loud, as declared). NO train
   split is fetched. (The R4a triage row's initial "corpus-ON" reading
   is corrected by this landing: the numeric option universe disqualifies
   the modelless drafter regardless of the train pool.)

## The birth read

| lane · model | n | acc | ECE(maxp) | p50 | p99 (support) | det |
|---|---|---|---|---|---|---|
| laya-riir · english | 2638 | **0.1706** | 0.1582 | 17.0 ms | 24.0 ms (14) | ✓ |

- **modal floor (issue 094): acc 0.1785** over 2,638 questions (modal
  census pooled by option identity; 2,562 keys, 2,504 fallbacks,
  reference-modal share 0.1808) — **laya 0.1706 ≤ floor: no
  input-independent signal on this suite**.
- ⛔ **The honest birth verdict is the FLOOR LINE, and the floor sits at
  chance.** The pooled chance is 0.175; the modal floor 0.1785 ≈
  chance — the suite is chance-hard BY CONSTRUCTION (distinct-value
  distractors are the board's anti-lucky-guess design), and a
  zero-shot encoder picking numeric answers with no working reads
  chance. Skill ≈ 0 at birth; the row is SCOREABLE and the
  infrastructure (fetch → build → lane) is the deliverable, the same
  posture the contractnli/finentity births disclosed. The
  teacher-capture side rides R4b (owner-gated amo booking; the
  quant-posture spot-check law applies per family).
- The pooled read is the board-comparable number (the board's own
  chance 0.175 is the pooled per-track average); no per-track harness
  split exists (the pooled `hard` readout is the runner's unit) — a
  per-qid track breakdown is a future readout need, recorded here, not
  built speculatively.

## Scope notes

- The board's v3 builder ALSO depends on its own internal rows file
  (the 0.2-era gsm8k rows) for the row scaffolding — irrelevant to our
  adapter (our wire is the reflex envelope; only the gold values, the
  grouping method, and the option-count tracks carry over).
- The 512-token truncation question does NOT arise: GSM8K problem
  states are short (the longest fetched question text is ~1,700
  chars ≈ well under the encoder budget; the board frames them whole).
- The licence-table row + §28 landed in the same commit; the fetch
  stanza's licence tripwire (`probe_license gsm8k openai/gsm8k`)
  guards refetches.
