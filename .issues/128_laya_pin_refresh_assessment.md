# riir-reflex 128 — laya reference pin refresh assessment (0.3.5 → upstream 0.4.35)

**Status:** CLOSED — 2026-11-20 (zed-6ef0ba37@m3, same session as the claim). Verdict: **PIN HELD** — T1 UNCHANGED (all 3 checkpoint LFS oids + all 12 small-file BLAKE3s byte-match; HF tree API + downloads re-verified), T2 all deltas PIN-SIDE-IRRELEVANT (option_layout parallel = the named WATCH trigger), T3 laya-onnx second oracle DECLINED with re-open trigger, T4 outcome row appended to `laya_reference_pin.md` (§"Pin-refresh assessment 2026-11-20") + Research 425 citation added beside 576. **Assessment-only engagement: read the pin, re-verify HF, read the upstream deltas, produce a verdict — no port changes, no gate edits in this issue.**

**Lease closed:** zed-6ef0ba37@m3, 1795173442, expires 2026-11-22T11:17:22Z

## Why now

The laya reference pin (`.docs/02_protocols/laya_reference_pin.md`) pins the code repo at
release 0.3.5 (`573e5b62`, 2026-09-21). Upstream is now 0.4.35 (tip `416dd9ed`, 2026-11-20,
2942 commits past the pin — Research 425's full delta list). Post-pin upstream deltas that
touch the **port contract** the G5 gate replays:

- ONNXAgent: `predict_long` windowing / tournament / shortlist — the serving-shape behavior
  around the forward the port mirrors.
- Heterogeneous `Router.predict_batch` — the batch/serve path shape.
- Per-bucket temperature calibration (qtype × option-band) — post-forward scoring, exactly the
  layer the decision head's probabilities feed.
- `option_order` rotation + the parallel `option_layout` weight-side layout — the option-key
  mapping the port's labels consume.
- vegaml, the Rust laya-onnx runtime (crates.io, differential goldens — a candidate SECOND
  ORACLE beside the G5 frozen captures), TileLang fast path.

None of these invalidate the G5 gate by themselves — the gate replays FROZEN captures against
the pinned checkpoint — but two claims are now stale-or-unverifiable as written:

1. **Checkpoint drift is UNVERIFIED.** The pin doc's SHA-256s were verified 2026-09-22 via HF
   LFS oid. Whether the hub repo's checkpoint files moved since is unknown. The pin doc's
   SHA-256s are the arbiter — re-check HF (tree API, the pin doc's own recipe) BEFORE claiming
   drift either way. An unmoved checkpoint means the port contract is intact regardless of the
   2942 code commits; a moved checkpoint is a loud refresh trigger.
2. **"The upstream research note is Research 576"** — the delta list (Research 425) is a
   second source the pin doc does not cite.

## The assessment (all read-only, one session, ~an hour)

- [x] T1 — HF re-verify: **UNCHANGED.** Three checkpoint LFS oids byte-match the pins (full sha256 equality, sizes exact: 842,609,210 / 643,835,514 / 842,609,220 B); 12 small files downloaded + b3sum-rehashed, all byte-match (typed tokenizer still git-oid-identical to english, `2f4d8583`). Verdict rows in the pin doc. No refresh trigger — this issue closes with "no drift".
- [x] T2 — Contract delta adjudication: **all six deltas PIN-SIDE-IRRELEVANT** (default held on T1=UNCHANGED; the two sensitive rows verified concretely — calibration tables ship inside `rl_agent_config.json` files T1 proved byte-unchanged, and `option_layout: "parallel"` is weight-side and would only fire WITH a re-packed checkpoint → recorded as the WATCH trigger). Table in the pin doc.
- [x] T3 — laya-onnx second-oracle: **DECLINED** with reasons (no ONNX artifacts in the hub tree — export unpriced; substrate-side dep home; no forward-refactor pressure; BOUNDARY row owed before any adoption). Re-open trigger recorded in the pin doc: next riir-infer-laya forward-semantics refactor or a G5 failure needing an independent witness.
- [x] T4 — Verdict + pin-doc update: **DONE** — outcome row appended (`§Pin-refresh assessment 2026-11-20 (reflex 128 …)`), pin HELD with the re-verified date + the delta-adjudication summary; Research 425 cited beside 576 in the source-notes line.

## Routing + provenance

- Full delta list + dedup + candidate rules: riir-refine `.research/425_laya_upstream_redistill.md`;
  the staged mining batch (separate engagement, its own lease): riir-refine `.plans/390`
  (Batch 374, unclaimed at filing).
- Filed from zed-m3-laya@m3's advisory mail (1795172210) — routing answer: YES, warranted; the
  assessment rides reflex (the pin doc's home), the mining batch rides riir-refine.
