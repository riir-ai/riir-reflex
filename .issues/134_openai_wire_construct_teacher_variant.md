# Issue 134 — OpenAI-wire `construct_teacher` variant (the SenseNova local-teacher arm)

**Status:** OPEN — routed: riir-rethink Issue 068's local-teacher cell owns the claim
(`zed-a73720e4@m3`, exp 2026-11-26T15:01Z); this issue is the reflex-side arm that cell
needs. Claim HERE only once the serving-posture probe (amo, in flight) reports a real
endpoint — the arm's parser must be written against the observed response shape, never a
guessed one.

**Why this issue exists (the premise check, 2026-11-24):** rethink Issue 068 serves
SenseNova-Loop-9B (qwen3.5-family looped transformer, Apache-2.0) behind an
OpenAI-compatible endpoint. The issue's own precondition — "the adapter's model-field-only
law verified first, `lanes/mammoth.rs`" — was run and REFUTED reuse-by-model-field:
`src/lanes/mammoth.rs` reuses the pplx lane's SystemOne wire (`build_body_for_model` /
`map_answers`, posted to `/v1/systemone`); the SystemOne dialect is not OpenAI
chat-completions, so no env change can point the existing adapter at the endpoint. The
lg4b cell's zero-reflex-code precedent does not transfer — that vendor bundled a
`/v1/systemone` server (`simple-jev/hf_server.py`); SenseNova has none.

**The arm (bounded):** one new `construct_teacher` variant (the factory lives in
`src/harness/runner/distill.rs`; consumers `synth.rs` / `ensemble.rs`):

- **Request** — translate `build_body_for_model`'s per-suite question render into the
  chat-completions `messages` form. The SystemOne body's options/abstain contract is
  carried as the prompt text; the option ORDER and the noul presentation law are the
  request's responsibility (the rethink plan 014b presentation-spec facts are the
  authority — a presentation mismatch between fit and deciding read is the 0075 class).
- **Response** — parse `choices[0].message.content` through `map_answers`' strict shapes
  (the pplx parser's own loud errors, never a positional guess). Expect endpoint quirks
  (reasoning-prefixed content, tool-call fields) — the probe report names them; the
  parser strips typed prefixes it can name and refuses the rest.
- **Env** — the mammoth lane's contract carried verbatim under the new arm's own prefix
  (`SERVE_URL` / `TOKEN` / `FORWARDER_AUTH` / `MODEL` / `DEVICE` / `TIMEOUT_MS` /
  `SMOKE_MAX_CASES` + `ALLOW_UNCAPPED`): the no-creds gate refuses at lane construction
  and the spend ceiling rides for free.
- **Handshake** — one minimal `noul` probe proves the wire AND the key before any read
  (the clef law; a 200 + a well-formed answers object is the proof).
- **MEASUREMENT-ONLY** — the mammoth lane's T10.2 law inherited verbatim: no capture
  artifact, no RIDT arm, no raw-body persistence beyond the determinism rerun's
  in-memory compare.
- **Tests** — a fixture replay gate (a recorded probe response as the golden) + the
  no-creds refusal + the uncapped refusal, in the lane's own `required-features`-free
  posture.

**Blocked on:** the serving-posture probe's report (endpoint URL + wire shape + quirks +
weights digest). Until it lands this issue stays unclaimed — filing is the route, not the
start.
