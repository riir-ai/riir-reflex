# Bench 197 — Issue 117 T3+T4: the frozen `--corpus-ab` V5 paired read (massive_intent_en)

The internalization gate over the Bench-196 sealed synth artifact: **V5 PASS** — synthesis lifts the modelless row. Per the issue's own law (keep on PASS, discard on fail, no retuning), the corpus is KEPT as a validated artifact; any serving-posture consequence (arming, rate band) stays the owner-gated follow-up, not this issue.

## Execution

- **Rig:** bmo Shared (quiet, load 1.3/30 — an accuracy read is load-invariant), job `01a1dd13-2fc5` (notify armed; the first submit `01a1dd12` was cancelled at once for a missing `--notify` and disclosed here — the resubmit is the record's run). Read wall **33.5 s**.
- **Posture:** reflex @ `1796ad6` (== the artifact's own build HEAD, verified in the SYNT v2 header), datasets `.raw/datasets_t20k`, extra-cap 128 (the harness default == the artifact's own per-label budget), cap 48/label, one frozen test read, forced-pick hard accuracy, no gate fit — the two arms differ ONLY in corpus content.
- **Read-time split-law asserts (the T1 wall's closing assert, run in-job before the read, fail-loud):** `captured_ids.txt` = 200 lines, all `^massive_intent_en:cal:[0-9]+$` namespaced (inverted-match count 0); artifact integrity = 2,049 lines (1 SYNT v2 header + 2,048 rows), magic/version/suite header line asserted; bytes unchanged from the T2 fold (the blake3 `f47ef555…a634` was three-way verified there; this job scp'd the same bytes). The corpus-ab lane reads the frozen TEST split only (its recorded read rule names it) while every captured id is cal-namespaced — disjointness holds structurally and is asserted at read time.
- Outputs preserved beside this record: `197_issue117_t3_corpus_ab/{CORPUS_AB.md,corpus_ab.json}`; bmo staging (`.raw/i117_t3*/`) deleted at fold (the `.raw/` law).

## T3 — the V5 verdict

- **Aliveness:** arm A reproduces the published 0.7800 exactly — the instrument is alive.
- **V5 gate: PASS** — gold 0.7800 → synth **0.8133** (+3.33 pp), paired LB95 **+0.0110 > 0**; 11 synth-only wins vs 1 gold-only loss (the one loss: `news_query` −1; the gains concentrate on the thin labels the weighting targeted — audio_volume_other +3, audio_volume_mute +2).
- Corpus: gold 11,314 docs → synth 13,362 docs (2,048 of 2,048 artifact rows in scope; zero dropped).
- Echo gates: abstention-entropy KL(gold‖synth) **0.0000** / reverse **0.0000** (PASS; the abstain-rate drift is the disclosed-by-design column, see T4); OOD word-dropout ladder gate-rung p=0.20 paired LB95 **+0.0130** → **transfer-ok** (p=0.10 +0.0134, p=0.30 −0.0005 — the gain survives moderate corruption, washes out at heavy); **no echo verdict fired**.
- V6 (the sub-ms law): gold p50 82 µs / p99 103 µs → synth p50 **81 µs** / p99 147 µs — the lane stays sub-ms after corpus growth.
- Determinism: true.

## T4 — the report (the same frozen read; no new run)

- **Accuracy movement:** +3.33 pp on the frozen read (0.7800 → 0.8133), LB95-bounded positive — the axes the ESC rate guard latches move the right way.
- **Cheap-only (abstention) rate:** 51.33% → **40.67%** (−10.7 pp) at KL 0.0000 — corpus growth raises answer confidence BY DESIGN (the Issue-064 first-real-reading law: a rate drop at KL 0.0000 is the system working, not a collapse; the column is disclosure, never a gate leg).
- **Serving posture:** UNCHANGED by this record — seating the synth corpus into the serving lane, and any ESC arming/rate-band consequence, is the owner-gated follow-up the issue itself names. Nothing wires here.

## Lineage

Issue 117 T1 (`84675ac` + Bench 195) → T2 (Bench 196, `e261fc2`) → T3+T4 (this record). The issue file is removed at close per the noise-reduction rule; the closed-lease row and the full task history live in git history + `HISTORY.md`.
