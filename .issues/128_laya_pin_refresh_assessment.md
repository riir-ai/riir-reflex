# riir-reflex 128 — laya reference pin refresh assessment (0.3.5 → upstream 0.4.35)

**Status:** OPEN — filed 2026-11-20 (zed-dfbadc62@m3, routing the laya 0.4.35 re-distill's recommendation — riir-refine Research 425, commits 599593de/faa96f7d; advisory mail from zed-m3-laya@m3, session 1795172210). **Assessment-only engagement: read the pin, re-verify HF, read the upstream deltas, produce a verdict — no port changes, no gate edits in this issue.**

**Claimed:** zed-6ef0ba37@m3, 1795173442, expires 2026-11-22T11:17:22Z

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

- [ ] T1 — HF re-verify: the three checkpoint SHA-256s + the small-file BLAKE3s, exactly per
      the pin doc's verification recipe. Verdict rows: UNCHANGED (pin holds; this issue closes
      with "no drift") / MOVED (a loud refresh trigger — the G5 gate's golden captures
      re-baseline becomes its own issue). Cite the LFS oids read.
- [ ] T2 — Contract delta adjudication from Research 425's list: for each post-pin delta that
      touches serving shape (ONNXAgent windowing/tournament, Router.predict_batch, temperature
      calibration, option_order rotation), decide PIN-SIDE-IRRELEVANT (the frozen captures
      replay 0.3.5 forward semantics — code-side serving changes never touch them) or
      CONTRACT-RELEVANT (e.g. a checkpoint schema/option-layout change WITH a moved
      checkpoint). The default is PIN-SIDE-IRRELEVANT while T1 reads UNCHANGED.
- [ ] T3 — The laya-onnx Rust runtime as a second oracle: assess (read-only) whether its
      differential goldens could sit beside the G5 frozen captures as an independent parity
      witness. Verdict: candidate issue filed / declined with the reason. crates.io dep —
      BOUNDARY.md check owed before any adoption (this assessment only records the candidate).
- [ ] T4 — Verdict + pin-doc update: append the assessment's outcome row to
      `laya_reference_pin.md` (pin refreshed / pin held with the re-verified date + the
      delta-adjudication summary). The one write this issue makes.

## Routing + provenance

- Full delta list + dedup + candidate rules: riir-refine `.research/425_laya_upstream_redistill.md`;
  the staged mining batch (separate engagement, its own lease): riir-refine `.plans/390`
  (Batch 374, unclaimed at filing).
- Filed from zed-m3-laya@m3's advisory mail (1795172210) — routing answer: YES, warranted; the
  assessment rides reflex (the pin doc's home), the mining batch rides riir-refine.
