# Issue 127 — re-check the laya probes that read `EncodedQuestion.hidden` (its semantics changed upstream)

**Status:** IN PROGRESS — filed 2026-11-20 (katgpt-rs-93@amo) from riir-infer Issue 1239.
**Claimed:** katgpt-rs-93@amo, 1795112000, expires 2026-11-22T00:00:00Z

riir-infer Issue 1239 found that `RiirAgent::encode_question` returned `hidden` AFTER the head ran in place, i.e. the head's post-type-emb, post-two-layer residual, not the documented encoder output (max |Δ| 1281.6–1311.8 on english/typed). The fix (riir-infer, same day) makes `hidden` the encoder output, pinned bit-exact on CPU + Metal (`tests/encode_question_hidden_contract.rs`).

Two probes here mean-pool or read `enc.hidden` as "encoder states":

- `examples/pair_free_probe.rs` (Issue 105; riir-train Issue 663 T2)
- `examples/rho_suite_probe.rs` (Issue 106; katgpt-rs Issue 978 T5)

## Tasks

- [ ] **T1** — re-run both probes at the fixed riir-infer revision. Say in each issue's record whether its numbers or verdict tier move. Numbers taken before the fix describe the head's residual, not the encoder.
