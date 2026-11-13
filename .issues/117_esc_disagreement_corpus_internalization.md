# Issue 117: ESC-disagreement corpus internalization (the VoI outcome join)

**Status:** Open — unclaimed
**Source:** arXiv:2611.13410 SteerDistill via [riir-rethink/.research/025](../riir-rethink/.research/025_SteerDistill_Closed_Loop_Teacher_Steering.md)
**Related:** rethink `src/esc_voi.rs` (RECEIPTS-ONLY blind until an outcome join lands — its own doc says so); reflex `harness/runner/{synth,echo_gates,corpus_ab}.rs`

## The gap

`esc_voi.rs` computes `P(think ≠ cheap ∧ think better | escalated)` but its value legs are blind by construction — decstat records no outcomes. On bench suites we hold gold, so the join is one capture run: replay an ESC-armed suite slice through the escalating backend and record the **(cheap pick, think pick, gold)** triple per case. That single run (a) unblinds the VoI legs and (b) yields verified disagreement rows — the SteerDistill "behavioral divergence" set for our family.

## The actionable cell

- [ ] **T1** Capture harness arm: `--esc-capture --out <dir>` over one ESC-armed suite slice. **SPLIT LAW (blocking):** mining reads ONLY the cal/train side of the suite split; the T3 frozen test read must be DISJOINT from every mined case id (assert disjointness in the capture output, fail loud on overlap). Initial mining source = **massive_intent_en ONLY** (Bench-089 shadow read SHADOW-PASS; condition (b) — the composed cal→test shift on the original distribution — remains unmeasured, so captures disclose that posture). banking77 carries the Bench-089 shadow-MIXED contamination-consistent caveat — its rows are forbidden until a separate caveat-cleared lane; if ever enabled they are marked, reported separately, and never pooled into the T2 corpus. Emits `triples.jsonl` (case id, split tag, cheap pick, think pick, gold, gate flag) + divergence stats → the voi report gets its outcome join.
- [ ] **T2** Verified-divergence rows → teacher-synthesized **train-pool** corpus candidates via the standing synth lane. ⛔ grammar-not-answers law binds the whole lane: never ingest test rows in any proportion (a teacher-made row aimed at a test-set error case is STILL a leak even when no test bytes are copied — the split law in T1 is the wall); corpus is built AROUND the frozen test; echo gate + leak scan on every pass; ONE frozen read.
- [ ] **T3** Internalization gate: `--corpus-ab` V5 paired read (synth-augmented corpus vs seated corpus, paired LB95, one frozen read) — the read's case set asserted disjoint from the T1 mined ids. Keep on PASS, discard on fail — no retuning.
- [ ] **T4** Report: cheap-only rate and accuracy movement on the frozen read (the same axes the ESC rate guard latches). Any serving-posture consequence (arming, rate band) is an owner-gated follow-up, not this issue.

## Refusals / non-goals

- No live-traffic mining (decstat carries no outcomes; bench capture is the disclosed posture).
- No corpus edit outside the synth lane's echo-gated pass.
- Success = the capture cell runs and the paired read lands a verdict; corpus improvement is the gate's call, not a promise.
