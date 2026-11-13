# Issue 117: ESC-disagreement corpus internalization (the VoI outcome join)

**Status:** Open — unclaimed
**Source:** arXiv:2611.13410 SteerDistill via [riir-rethink/.research/025](../riir-rethink/.research/025_SteerDistill_Closed_Loop_Teacher_Steering.md)
**Related:** rethink `src/esc_voi.rs` (RECEIPTS-ONLY blind until an outcome join lands — its own doc says so); reflex `harness/runner/{synth,echo_gates,corpus_ab}.rs`

## The gap

`esc_voi.rs` computes `P(think ≠ cheap ∧ think better | escalated)` but its value legs are blind by construction — decstat records no outcomes. On bench suites we hold gold, so the join is one capture run: replay an ESC-armed suite slice through the escalating backend and record the **(cheap pick, think pick, gold)** triple per case. That single run (a) unblinds the VoI legs and (b) yields verified disagreement rows — the SteerDistill "behavioral divergence" set for our family.

## The actionable cell

- [x] **T1 — LANDED 2026-11-13 (zed-02f4db55@amo, reflex `84675ac`):** the `--esc-capture <dir>` arm (needs `--cascade --cascade-worthiness`) rides the worthiness probe's cal records — `esc_capture::capture` writes `triples.jsonl` (id, split=cal, cheap, think|null when unserved, gold, gate_fired), `captured_ids.txt` (the T3 disjointness input), and the divergence stats; `WorthinessInput::probe` widened `pub(crate)` so the capture mirrors the lane's arming byte-for-byte. SPLIT LAW enforced fail-loud (mining allowlist = massive_intent_en only, banking77 refused; gate-shape drift refused; every captured id asserted absent from the suite's test ids). Tests 3 new; lib 651/0; clippy clean. **The capture RUN stays deferred** (needs a serve lane — the 4090 is t679-held; run on the next quiet box).
- [ ] **T1** Capture harness arm: `--esc-capture --out <dir>` over one ESC-armed suite slice. **SPLIT LAW (blocking):** mining reads ONLY the cal/train side of the suite split; the T3 frozen test read must be DISJOINT from every mined case id (assert disjointness in the capture output, fail loud on overlap). Initial mining source = **massive_intent_en ONLY** (Bench-089 shadow read SHADOW-PASS; condition (b) — the composed cal→test shift on the original distribution — remains unmeasured, so captures disclose that posture). banking77 carries the Bench-089 shadow-MIXED contamination-consistent caveat — its rows are forbidden until a separate caveat-cleared lane; if ever enabled they are marked, reported separately, and never pooled into the T2 corpus. Emits `triples.jsonl` (case id, split tag, cheap pick, think pick, gold, gate flag) + divergence stats → the voi report gets its outcome join.
- [ ] **T2** Verified-divergence rows → teacher-synthesized **train-pool** corpus candidates via the standing synth lane. ⛔ grammar-not-answers law binds the whole lane: never ingest test rows in any proportion (a teacher-made row aimed at a test-set error case is STILL a leak even when no test bytes are copied — the split law in T1 is the wall); corpus is built AROUND the frozen test; echo gate + leak scan on every pass; ONE frozen read.
- [ ] **T3** Internalization gate: `--corpus-ab` V5 paired read (synth-augmented corpus vs seated corpus, paired LB95, one frozen read) — the read's case set asserted disjoint from the T1 mined ids. Keep on PASS, discard on fail — no retuning.
- [ ] **T4** Report: cheap-only rate and accuracy movement on the frozen read (the same axes the ESC rate guard latches). Any serving-posture consequence (arming, rate band) is an owner-gated follow-up, not this issue.

## Refusals / non-goals

- No live-traffic mining (decstat carries no outcomes; bench capture is the disclosed posture).
- No corpus edit outside the synth lane's echo-gated pass.
- Success = the capture cell runs and the paired read lands a verdict; corpus improvement is the gate's call, not a promise.

**Re-route (2026-11-13, zed-4430bf70@4090):** the 4090 wake (zed-41) did not pick this up — inbox accepted the work order but the agent stayed idle, no receipt (suspected non-draining inbox). The lane is re-offered to the amo rig (compile headroom; the ESC family lanes live there). T1 remains first: the --esc-capture arm + split-law disjointness asserts (CPU work), capture run deferred until a quiet GPU box. (The original 4090 routing claim — epoch 1794580716, was expires 2026-11-13T13:38:36+00:00 — expired unfulfilled; its claim line removed 2026-11-13 zed-a833e55b@amo per the one-claim-line fold rule, its substance lives in this paragraph. That line was also format-normalized first: a milliseconds epoch + a Z-suffixed ISO that disagreed by 3h, normalized to the later reading — the conservative direction for a foreign claim.)

**Lease closed:** zed-02f4db55@amo claimed T1 at epoch 1794580734 (expiry 16:38:54Z) per the re-route; landed `84675ac` the same session (the claim line itself was folded by the 24f72cb sweep mid-flight — this note restores the receipt). T2/T3 + the capture RUN remain open.
