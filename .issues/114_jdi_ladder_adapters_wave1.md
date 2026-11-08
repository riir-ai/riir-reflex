# Issue 114 — JDI ladder adapters, wave 1: the decision-shaped tools three (BFCL / When2Call / API-Bank) + CLINC150 (retrieval)

**Status:** OPEN — plan-half of rethink Issue 060's R1a/R2a; adapters unbulleted, one suite at a time
**Claimed:** zed-464454b5 (JDI ESC serve capture monitoring), 1794138600, expires 2026-11-10T00:00:00Z

Rethink Issue 060 (the beat-the-JDI ladder) orders the areas most-use × quick-win first:
**tools (18.3% fold weight, ZERO coverage today)**, then retrieval (20.0%, one served
suite). This issue carries the reflex-side half: real harness adapters for the
decision-shaped suites, so the teacher capture + ESC fit + arming wave (rethink Issue
059's machinery) can ride them exactly like banking77 rides today. **Boundary law
rethink 060 R1a: adapters are FILED here and built here — rethink never edits reflex.**

## The board rows we are adapting (snapshot 66d25721, edition decision-index-0.3)

| board row | chance | gold weight | what it asks | wire verdict (R2b law, triaged here) |
|---|---|---|---|---|
| BFCL | 0.2592 | ×1.2 | given a query + tool schemas, pick the function to call (or irrelevance-detect) | **scoreable-partial**: the pick-a-tool tier is choice-shaped; the arg-filling tiers are NOT choice-shaped — out of scope for wave 1, never forced |
| When2Call | 0.25 | ×1.0 | given a query + tool set: call a tool (which one?) or answer directly | **scoreable**: this IS the abstention-first shape — the wire's own contract (choice with an abstain option) |
| API-Bank | 0.0189 | ×1.2 | API-call selection in a dialogue | **scoreable-partial**: the which-API tier is choice-shaped; multi-turn state carry is out of scope wave 1 |
| CLINC150 | 0.006 | ×1.2 | 150-intent (+ OOS) classification | **scoreable**: banking77's nearest neighbour — same intent lane, one more suite on existing machinery |

Home appliances (chance 0) and ToolRet (0.1341) stay TRIAGE-LATER per rethink 060 R1
(the shapes are weakest fits; the decision-shaped three carry the rung's points).

## The adapter recipe (exactly what banking77 got — no new machinery invented)

1. **`scripts/fetch_datasets.sh` stanza** — the `want_suite`/`fetch_suite`/
   `probe_license`/`dedupe_train` pattern per suite; parquet-backed mirrors preferred
   (the PolyAI/banking77 script-dataset 404 lesson — probe /rows BEFORE committing the
   source; a script-based dataset needs its mteb-class mirror named in the stanza
   comment). Candidate sources to probe: BFCL `gorilla/cs-bench-...` — no: the BFCL
   test set lives in the gorilla repo's `berkeley-function-call-leaderboard` data dir
   (JSON, NOT a parquet HF dataset — the fetcher may need a direct-JSON lane or an
   HF mirror; probe first, record the verdict, never invent a transform silently).
   When2Call: `nvidia/when2call`-family (probe /rows). API-Bank: ` Backbone-Models/...`
   mirrors exist; probe. CLINC150: `clinc_oos` is script-based on the Hub (the
   banking77 404 shape) — the parquet-backed mirror is `FaizaanAliza/clinc_oos`-class;
   probe /splits + /rows first, record the chosen mirror + licence in the manifest.
2. **`.docs/02_protocols/dataset_manifest.md`** — every fetched file's blake3 digest +
   a VERIFIED licence row; `license.json` probes ride the fetch (the drift tripwire).
   Licence verdicts recorded BEFORE any number is published; fail-closed.
3. **`src/harness/suites.rs` builders** — one `build_<suite>` fn per suite, keyed on
   the mirror's real columns (banking77's `label_text` precedent); `Suite::name` is
   the OUR_SUITES spelling (`bfcl_jdi`? — NO: name them what the board names them
   minus fold-mapping noise: `bfcl`, `when2call`, `api_bank`, `clinc150`. The
   `_jdi` suffix exists because the four rethink panel suites are re-captures of
   board rows under fresh digests; the ladder suites are FIRST-CLASS suites — plain
   names, the fold maps them).
4. **`src/harness/runner.rs` SuiteSpec rows** — test/cal/corpus caps per suite shape:
   BFCL pick-tier test rows cluster by API domain — apply the sampling law
   (hash-ordered sampling if rows cluster, the 060 gate); When2Call test_cap ~its
   real split; CLINC150 test_cap 500 first-row convention (banking77 parity),
   cal_cap 200.
5. **Option-key universes**: tools suites carry LARGE option sets (BFCL 100+ APIs,
   CLINC150 150+1 intents) — reuse banking77's derived-universe rule (labels = sorted
   unique label keys, gold = key position); the seat's corpus_cap_per_label bounds
   the train side.
6. **The served-cells path (already live)** — rethink's `build_jdi_data.py`
   `served_composed` tier + rethink `scripts/jdi_aggregate.py` `OUR_SUITES` extend by
   ONE row per new suite (suite → gid, metric, label); the ESC fit instrument
   (`esc_mammoth_items_fit`) extends per the 087 pattern. That half is RETHINK-side
   (060 R1d) — reflex only ships the harness suite + the feed cells.

## Gates (bind every suite, in order)

- **Licence first, fail-closed** — a suite whose source licence does not read clean
  is refused, noted in this issue, never "annotated in" later.
- **grammar-not-answers** — no test rows in any training pool; a future distill corpus
  for the tools rung builds from TRAIN splits only, echo-gated + leak-scanned before
  any number.
- **Sampling law** — clustered test rows → hash-ordered sampling, disclosed in the
  record.
- **B5 law** — the JDI public read and the our-suite paired read ship together.
- **No silent transforms** — a source needing a normalization (BFCL's JSON lane)
  records the transform in the stanza comment + the manifest row; the digest pins
  the OUTPUT.

## Disposition

- [ ] T1 — fetch probes for all four sources: /splits + /rows + licence per dataset;
      verdicts recorded here (mirror + config + split per suite), sources that 404
      named honestly.
- [ ] T2 — CLINC150 first (the 060 verdict: it reuses banking77's machinery as it
      stands): stanza + builder + SuiteSpec + manifest row + a smoke `harness`
      run; the teacher capture for it rides the NEXT amo trip (rethink 059 lineage,
      owner-gated amo hours — book it with the R1b tools capture, one booking).
- [ ] T3 — When2Call second (the abstention-first shape is the wire's own; highest
      product-value of the three tools suites).
- [ ] T4 — BFCL pick-tier third (the arg-filling tiers REFUSED wave 1 — recorded as
      the R2b-class wire verdict, never forced).
- [ ] T5 — API-Bank which-API tier (multi-turn refused wave 1).
- [ ] T6 — per-suite paired read (modelless + laya on the new suites, the standard
      harness posture) + the results.json cells land in the feed; rethink's fold
      extends (060 R1d/R2a) — separate booking, gate, and record on the rethink side.
