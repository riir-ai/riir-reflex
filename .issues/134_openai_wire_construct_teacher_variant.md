# Issue 134 — OpenAI-wire `construct_teacher` variant (the SenseNova local-teacher arm)

**Status:** LANDED 2026-11-26 (`801ffca6`) — the lane is shipped: `src/lanes/sensenova.rs` (the repo's first chat-completions wire; the parser written against the 068 probe's OBSERVED shapes) + `TeacherForward::Sensenova` + the `--sensenova` comparison lane + the record row. Gates: 15 lane tests, lib 761/761, clippy -D at all three postures, rustfmt clean. The construct_teacher distill arm is the FOLLOW-UP (the distill-seam variant naming this lane; opened only when the B5 paired read is scheduled — the comparison lane is the B5 rig itself). **B5 COMPLETED 2026-11-28 (~10:19 paired / 11:24 completion runs, amo zed-6addfb44): verdict UNDECIDABLE-BY-ABSENCE — the no-CoT dial posture answers OFF-CONTRACT under the harness's refuses-loud law (banking77 died at case 3, massive at 187: prose answers don't fit the verbatim-key grammar); trained posture unservable (300 s+ reasoning, the 2184-token derivation); candidate NOT elected, mammoth-1 keeps the served seat, the ESC think-leg candidate cell stays CLOSED. Full record + re-open triggers: rethink `.issues/068` (committed there `7e3dc1b`/`f4c06b4`/`4b90dfb`). Evidence dirs committed by the sweep lane (zed-99556257@amo, `6a4f4861`) — read logs folded from /tmp into the paired dir. This issue's own follow-up landed with the cell: the lane's masked-timeout fix (`7d3803c`) + the module doc's reasoning-class note.**
**Claimed:** zed-a73720e4@m3, 1795782750, expires 2026-11-29T12:30Z

**Endpoint facts of record (the parser's authority, from the 068 probe):** `http://127.0.0.1:<port>/v1/chat/completions`, OpenAI-compatible llama-server wire; `content` carries ONLY the final answer — the chain-of-thought lands in `message.reasoning_content` (read BOTH or you see empty content on reasoning-token exhaustion); the model id echoes the load path (set it explicitly); finish reasons are standard (`stop`/`length`).

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
