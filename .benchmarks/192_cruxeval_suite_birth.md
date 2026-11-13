# Bench 192 — cruxeval suite birth: the knowledge rung's fifth adapter, the board's archived-distractor recipe ported byte-exact (issue 114 R4a queue #5)

**Date:** 2026-11-13 · **Repo:** riir-reflex (the fetch lane + builder + registry + manifest land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` R4a queue #5 (the knowledge nine triage's fifth adapter — the queue's successor to GSM8K/BBH/SATA/CLadder; rethink 060 R4a's reflex-side half)
**Posture:** `LAYA_DEVICE=metal /tmp/sata-dev/release/harness --suites cruxeval --laya-max-questions 0 --out .benchmarks/192_cruxeval_suite_birth` (release, laya-riir-metal; corpus-off — the modelless lane skips loud)
**PROVENANCE:** ✓ preflight PASSED at launch (power=AC powermode=0(auto) load 3.93; the run's own box-state stamps read start load **3.86 / end 3.66** — **latency QUOTABLE**).

## What landed

1. **The licence probe PASSES at two GitHub surfaces** — the repo's
   LICENSE at the pinned tree sha `190faf16d175b5847b0af05d937872b1fb395942`
   is the full MIT text ("Copyright (c) 2023 Meta"), and the GitHub API
   independently reports `spdx_id: MIT` (both read 2026-11-13; the
   hub-side probe does not apply — GitHub repo, not a hub dataset).
2. **The fetch lane** (`fetch_cruxeval` + the `_cruxeval_rows`
   transform, stanza #30): THREE files at the board's pinned tree sha
   via the GitHub raw lane — the 800-item source jsonl + the ARCHIVED
   codellama-7b sample generations + the scored evaluation flags (the
   model-generated WRONG outputs are the distractors — no teacher
   run needed, the triage's "in-repo distractors" resolved literal).
   Source digests: cruxeval.jsonl sha256 `8368b810…c2e0c96`,
   generations.json `e0a70dc1…f74ee412`, scored json
   `2b3095f6…d6ae2`. Envelope page test-000 blake3
   `c4e47bb3021bea5f7b078e657026f41206545dddcc8593a536b22714d5f4bc05`.
3. **THE RECIPE IS THE BOARD'S, BYTE-EXACT** (the transform ports
   `adapters_selection.py::crux` @ clone sha `f5a6e4fd85fe`
   verbatim — and everything it needs is python-native, so unlike
   gsm8k/sata there is NO re-seat anywhere): per source id (sorted),
   candidates = [gold] + the generations whose evaluation flag is
   False, value-equality dedup (python `==` semantics — 1 == True
   == 1.0, the repr-string tie key), max 10 candidates, the MT19937
   shuffle (`random.Random(f"20260917:CRUX:{sid}")`), choices = repr
   strings, the unique-gold assert, and the <2-distinct-candidates
   exclusion COUNTED. **570 kept / 230 excluded = the board's pin
   exactly.** Because the whole recipe runs in the python3 transform,
   our OPTION ORDER is the board's own MT19937 order — the rows are
   DIRECTLY board-comparable (the cleanest comparability of the five
   knowledge adapters: no re-seated anything).
4. **`cruxeval`** (suite §32 of the manifest): **570 cases / 570
   questions** — ONE choice question per case; candidates 2..10
   (avg 3.2; k-dist {2:256, 3:139, 4:84, 5:36, 6:31, 7:10, 8:8, 9:1,
   10:5}); positional option keys ("1".."k") with the repr strings as
   the criteria values (the gsm8k shape); the gold index rides the
   transform's precomputed `correct` (the unique index equal to the
   source output, asserted at the transform). Census pinned two-layer
   (builder 570 + prepare 570). **Corpus-OFF at the census**: the code
   snippets are eval-only and the distractors are archived model
   generations — no train split (grammar-not-answers);
   `modelless_lane: false`, laya-only.

## The birth read (laya[english], the only lane)

| axis | value |
|---|---|
| **accuracy (the board metric)** | **0.3860** (n 570) |
| chance | pooled mean 1/k 0.3697 · majority-class 0.3667 (the run's jdi_chance) |
| chance-corrected skill | +0.019 (over pooled chance) / +0.030 (over majority) |
| modal floor (issue 094) | 0.3684 (203 keys, 145 fallbacks, LOO over eval) |
| ECE(maxp) | 0.2227 (mean confidence 0.608) |
| determinism | ok=true, n=10 reruns clean |
| latency | p50 12 ms / p99 18 ms (support 6) — QUOTABLE posture |
| slice digest | fnv1a64-bab70891f83eb9e7 (570) |

**The read:** ABOVE pooled chance (+0.019) and the modal floor
(0.3860 > 0.3684) — real, small, input-dependent signal on the
output-prediction track, the same class as BBH's birth (+0.030).
Static execution reasoning with no working: the encoder reads the
snippet + input and picks among Python-literal candidates at better
than random but nowhere near the board leaders (CRUXEval board rows
run 0.5–0.8) — the gap is the teacher-capture lane's work (R4b,
owner-gated). The truncation question the triage flagged: inputs are
TINY (max 128 chars, mean 18; code max 278) — a full-context read,
no truncation disclosure needed.

## Scope notes

- The 512-token truncation question (the MuSR blocker): CLOSED for
  this suite at the birth census — the shortest contexts of the
  knowledge rung.
- The licence-table row + §32 landed in the same commit; the fetch
  stanza carries the GitHub-surface licence note.
- The board's CRUXEval track is output prediction only (the input-
  prediction track is the same 800 snippets the other direction —
  the board's row is output-choice; recorded as the named re-open if
  the board ever adds it, never built unilaterally).
