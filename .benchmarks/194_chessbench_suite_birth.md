# Bench 194 — chessbench suite birth: the knowledge rung's seventh adapter, the tie law carried (issue 114 R4a queue #7 — the queue's last scoreable)

**Date:** 2026-11-13 · **Repo:** riir-reflex (the fetch lane + builder + scorer seam + registry + manifest land in the same commit as this record; the digests below pin the run) · **Box:** amo (M5 Ultra, 30 cores)
**Issue:** reflex `.issues/114` R4a queue #7 (the knowledge nine triage's seventh adapter — the queue's successor to GSM8K/BBH/SATA/CLadder/CRUXEval/GPQA; rethink 060 R4a's reflex-side half; after this the queue is EMPTY of scoreable-uncontingent adapters — blocked: MuSR truncation-as-shaped, HLE owner gate)
**Posture:** `LAYA_DEVICE=metal ./target/release/harness --suites chessbench --laya-max-questions 0 --out .benchmarks/194_chessbench_suite_birth` (release, laya-riir-metal; corpus-off — the modelless lane skips loud)
**PROVENANCE:** ✓ preflight PASSED at launch (power=AC powermode=0(auto) load 3.22; the run's own box-state stamps read start load **3.04 / end 3.09** — **latency QUOTABLE**).

## What landed

1. **The licence read PASSES at the source pin** — the repo README's
   License section at the pinned sha `90ae0e6` (read verbatim
   2026-11-13): "All software is licensed under the Apache License,
   Version 2.0 (Apache 2.0)"; "Some portions of the dataset are in the
   public domain by a Creative Commons CC0 license from lichess.org.
   The remainder of the dataset is licensed under Creative Commons
   Attribution 4.0 (CC-BY)." — **DATA CC-BY 4.0** (the lichess portion
   CC0), **CODE Apache-2.0**, the split named on the manifest row (the
   mteb-mirror lesson). The bag is data — CC-BY-4.0 governs it.
   Attribution: Ruoss et al., "Amortized Planning with Large-Scale
   Transformers: A Case Study on Chess", NeurIPS 2024 /
   arXiv:2402.04494 (DeepMind; Stockfish 16 provided the action
   values).
2. **The fetch lane** (`fetch_chessbench` + the `_chessbench_rows`
   transform, stanza #32): the action-value TEST bag from the
   searchless_chess GCS bucket — the FILE BYTES pinned by sha256 (the
   board's own pin):
   `5f73aac8f60e31734cdbf276ba3fca8d5ba5cb6171ba600e31af4f36327986b0`,
   **verified at fetch** (a moved source refuses loud). The bag never
   outlives the envelope (deleted under `.raw/` after conversion).
   Envelope page test-000 blake3
   `39f5b52b731b0913a1fc2f2e1b250d072ecbe70e409612b50c3a78cef916e6ef`.
   **python-chess rides the `.raw/venvs/chess` uv venv** (created on
   demand by the lane; untracked; the PYTHON oracle lane only — never a
   cargo dep; the triage's own scoping). The 1.1 TB train split is
   NEVER fetched (corpus-off by construction).
3. **THE RECIPE IS THE BOARD'S, BYTE-EXACT** (the transform ports
   `adapters_scored.py::chessbench` @ clone sha `f5a6e4fd85fe`
   verbatim): the bagz reader (mmap + the trailing little-endian `<Q`
   offsets table) + the LEB128-string/big-endian-f64 record decode
   VERBATIM; per-fen value maps (duplicates assert equal, counted — 0
   in this fetch); the eligibility excludes (incomplete legal-move
   coverage — 0 here; non-decision or >255 moves — 728 here); the
   sha256(fen:move) option order (byte-exact, deterministic — no
   shuffle seat anywhere); the freeze CASE_CAP selection (5,000 of
   61,833 eligible by sha256("20260919:31:<sha256(fen)>") lowest —
   the proven cladder key format, catalog id 31). Transform census:
   **1,838,218 records → 62,561 positions → 61,833 eligible (728
   excluded: 0 coverage / 728 non-decision; 0 duplicate records)**.
   The trap-drop: the per-move reference VALUES (the answer material)
   never reach the envelope — only the accepted NAMES survive (the
   SATA/GPQA class).
4. **THE TIE LAW IS THE SUITE METRIC** (the triage's named carry, the
   landing's core): the board's scorer accepts ALL tied-best moves
   (`chess_score`: `best_move = choice ∈ accepted`,
   `tie_tolerance: 0.0`; their `expected` carries ONE representative).
   **5.78% of the frozen positions carry tied-best moves** (mean
   accepted 1.53, max 71 — one position where 71 of its moves are
   equally best). The accepted set rides the case's GOLD-SIDE `soft`
   vector (1.0 per accepted move, 0.0 otherwise) — NEVER the
   model-visible state, which would leak the answer; `gold.idx` stays
   the board's `expected` (accepted[0]) for the generic walks and the
   modal floor. `CHESS_MOVE_SUITES` arms `chess_move_metrics` — the
   new scorer walks (gold, probs, soft) per question with the encoding
   laws asserted loud (0.0/1.0 only; ≥1 accepted; the representative
   accepted — the board's own assert shape), and BOTH lane assemblies
   carry it via the one-home seam `chess_move_of` (the CASE_EXACT/
   MULTILABEL law). The tie-accepting accuracy is never folded into
   `hard`.
5. **`chessbench`** (suite §34 of the manifest): **5,000 cases /
   5,000 questions** — option keys = the UCI move strings, criteria
   values = the SAN (the board's own both-notations criteria dicts, so
   the engine renders "e2e4: e4"); k 2..81 (mean 29.5, median 32; 74
   positions k>50 — the s1mb variable-breadth posture, `coverage_audit:
   false`); state = {fen, board, side_to_move} (the board's own
   shape). Census pinned two-layer (builder 5,000 + prepare 5,000).
   **Corpus-OFF**: eval-only (grammar-not-answers).

## The birth read (laya[english], the only lane)

| axis | value |
|---|---|
| **chess-move acc (the tie law — the board's primary metric)** | **0.0842** (n 5,000) |
| hard acc (the generic representative-only walk) | 0.0594 |
| chance | pooled mean 1/k **0.0595** (the run's chess_move.chance; the majority-gold share 0.0656) |
| chance-corrected skill | **+0.025 over pooled chance** (SE ≈ 0.003 at n 5,000 — significant) |
| modal floor (issue 094) | 0.0792 (4,887 keys — a real per-key census; representative-keyed, so the tie-aware readout's +0.005 margin is conservative) |
| tie census | tie_share 0.0578 · mean_accepted 1.5258 (the suite metric's own report-only axes) |
| ECE(maxp) | 0.1214 (mean confidence 0.181 — underconfident on an 8.4%-acc population) |
| determinism | ok=true, n=10 reruns clean |
| latency | p50 20 ms / p99 23 ms — QUOTABLE posture (109 s wall for 5,000 positions, mean 29.5 options each) |
| slice digest | fnv1a64-0ac235dc9636bfa1 (5,000) |

**The read:** ABOVE pooled chance (+0.025, significant at n 5,000)
AND above the modal floor (0.0842 > 0.0792) — real, small,
input-dependent signal, the BBH birth class. **The tie law is
materially load-bearing**: the generic representative-only walk reads
0.0594 (BELOW the majority floor 0.0656 — a false-floor posture), the
tie-aware suite readout 0.0842 (above both chance and the modal
floor) — the board's scorer semantics move the read from below-floor
to above-floor, which is exactly why the triage carried it. Far below
the searchless_chess paper's own transformer rows (their 270M model
learns Stockfish action values to ~grandmaster policy strength) — a
421M ModernBERT encoder picking UCI tokens at 8.4% over a 5.9% chance
is the honest modelless-encoder floor; the gap is the teacher-capture
lane's work (R4b, owner-gated).

## Scope notes

- The option-count breadth is the suite's own shape (ALL legal moves
  presented, k 2..81): the laya encoding budget truncates option text
  tokens per-option (the build_sequence budget law) — the UCI prefix
  survives; the SAN tail may truncate on the widest boards. Disclosed
  as the variable-breadth posture (the BFCL/s1mb class), never a
  blocker.
- The 512-token truncation question (the MuSR blocker): a chess
  position serializes small (FEN + ASCII board + side) — the head
  budget goes to the options, not the state. No truncation disclosure
  needed beyond the option-text note above.
- The 1.1 TB train split stays unfetched forever at this lane (the
  corpus-off posture is constructive — there is no train side to
  protect against).
- The queue after this: EMPTY of scoreable-uncontingent R4a adapters —
  MuSR (truncation-as-shaped) and HLE (owner gate) stay blocked. The
  R4a reflex half is DONE.
