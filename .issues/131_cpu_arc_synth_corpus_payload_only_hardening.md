# Issue 131: synth-corpus payload-only hardening (CPU-ARC §D STAGE-2 adoption)

**Status:** LANDED + CLOSED 2026-11-22 — reflex develop `7a0734b` (zed-amo-071num@amo; T1–T4 all landed in one commit). T1 scaffold hash: rows carry `scaffold_hash` (BLAKE3 16-hex over the normalized frame); writer self-check refuses a divergent row (count in the header `hardening` block); loader verifies every row carrying the field, legacy rows without it keep loading (seated artifacts unaffected). T2 degeneracy: the paper's <10-char floor on the WHOLE text (payloads are legitimately short — "nine am" is a real slot; the payload-read floor killed the lane's own shape and was corrected to the completion analog), CoT markers conservative (instruction-scaffold phrases only). T3 the closure: id tuple (suite/kind/variant) on every row + plan-time refusal of whole-text and PAYLOAD-WINDOW test echoes — always-on (contamination machinery is not flag-gated; the "feature-off byte-identical" phrasing reads as behavioral: on clean data the accepted set is unchanged, the artifact bytes gain the additive fields, version stays 2). T4 gates: 5 planted-violation arms + legacy tolerance; lib 728/0; clippy -D warnings ×3 postures; real-data `--synth-plan`: the closure refused **23,184** (massive_intent_en) and **43,999** (banking77) test-echo candidates at harvest with allocated yield unchanged (2048) — these were flowing to the echo gates downstream before. UNEXECUTED LEG: the full veto+corpus-ab V5 paired read on a fresh hardened artifact — no veto teacher on amo (openthai env is the Windows lane's; hosted mammoth is owner-gated network); the loader path corpus-ab consumes is covered by the round-trip/re-signed-tamper/legacy arms. Re-run trigger: the next real veto run (any box with a teacher) is the corpus-ab green leg.

## Why

The `--synth-corpus` lane (teacher-synthesized train-pool data, echo-gated) currently accepts free-form teacher variants. CPU-ARC's §D STAGE 2 ships a stricter, more leak-scannable augmentation shape — and the leak-scannability is the point: when the scaffold is a CONSTANT, a leak scan hashes the scaffold and scans only payloads against the test echo index (high precision, near-zero false positives). Four upgrades, all additive, feature-off byte-identical:

- [x] **T1 — payload-only structural identity:** the harness records the scaffold hash per row so the constraint is CHECKED, not assumed (a row whose scaffold hash diverges from the seeding record's is refused loud, counted in the artifact header). (The teacher-prompt half is inapplicable until a teacher-variant lane exists — this lane's candidates are deterministic transplants, already structurally identical to their frames; the hash makes the scaffold CONTRACT machine-checked, which is the leak-scannability the paper wants.)
- [x] **T2 — degeneracy filters:** refuse-and-count rows under the minimum length (paper: <10 chars, applied to the whole generated text — the completion analog) or with leaked chain-of-thought markers; no retry loop (our echo gates are the authority).
- [x] **T3 — id-tuple exclusion (the contamination closure):** every row carries the deterministic id tuple (suite, kind, variant + the existing src seed index); the lane refuses any candidate whose whole text equals a test row or whose PAYLOAD appears verbatim as a test-row token window — excluded AT HARVEST, not filtered later. Cal rows stay the lawful source (unchanged C1).
- [x] **T4 — gates:** the planted-violation arms per T1–T3 (bad scaffold hash / degenerate completion / test-row tuple must each refuse loud) landed as lib tests; `--corpus-ab` green leg environment-blocked on the veto teacher (see the Status line) — the loader surface it consumes is hermetically proven.

## Non-goals

No retry-with-reduced-counts loop (the paper's); no change to the V5 paired gate; no new datasets.

## Proof path

Existing: `scripts/fetch_datasets.sh` corpora + the echo-gate battery + `--corpus-ab` V5 paired read. T4's planted-violation arms are the falsification proof; a green corpus-ab with hardening on is the no-regression proof.

**Lease closed:** zed-amo-071num@amo, 1795364600, was expires 2026-11-25T12:00:00Z — closed by completion 2026-11-22 (reflex `7a0734b`)
